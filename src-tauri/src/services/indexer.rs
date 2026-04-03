// IndexerService : scanne un dossier de façon asynchrone (tokio) et alimente le cache SQLite.
// Appelé au démarrage de l'app et sur demande explicite (ex : dossier non encore indexé).
// Émet des événements Tauri pour notifier le frontend de la progression.

use serde::Serialize;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::services::cache;

// ── Public types ──────────────────────────────────────────────────────────────

/// Payload emitted as the `indexer:progress` Tauri event after each directory is scanned.
/// The frontend can listen to this event to show a progress indicator.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    /// The directory that was just scanned.
    pub path: String,

    /// Running total of entries indexed so far in this scan session.
    pub files_indexed: u64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Spawn a non-blocking background task that recursively indexes `root_path`.
///
/// Returns immediately. The scan runs on the Tokio thread pool and writes
/// results into the SQLite cache via `cache::upsert_file`.
pub fn scan_dir(pool: SqlitePool, root_path: String, app_handle: AppHandle) {
    // tauri::async_runtime::spawn (not tokio::spawn) so this can be called
    // from Tauri's sync setup closure without a running reactor.
    tauri::async_runtime::spawn(async move {
        walk_and_index(pool, root_path, app_handle).await;
    });
}

/// Determine the best path to scan at application startup.
///
/// Priority:
/// 1. The most recently visited accessible path (from `recents` table).
/// 2. The user's home directory (`$HOME` / `USERPROFILE`).
/// 3. The filesystem root `/` as a last resort.
pub async fn resolve_start_path(pool: &SqlitePool) -> String {
    if let Ok(recents) = cache::get_recents(pool, 1).await {
        if let Some(recent) = recents.into_iter().next() {
            if tokio::fs::metadata(&recent.path).await.is_ok() {
                return recent.path;
            }
        }
    }

    home_dir()
}

// ── Private scan logic ────────────────────────────────────────────────────────

/// Index the direct children of `root` only (non-recursive).
///
/// Pass 1 — metadata only (no MIME reads): upserts every direct child and emits
/// `indexer:progress` every `EMIT_INTERVAL` files so the frontend can refresh
/// incrementally. Sub-directories are recorded in the cache but not descended into;
/// they will be scanned on demand when the user navigates into them.
///
/// Pass 2 — MIME enrichment: runs as a separate background task after the main
/// scan, reading 36 bytes per file. Because MIME is preserved across upserts
/// (`COALESCE`), already-detected types are never overwritten with NULL.
async fn walk_and_index(pool: SqlitePool, root: String, app_handle: AppHandle) {
    const EMIT_INTERVAL: u64 = 50;

    let mut reader = match tokio::fs::read_dir(&root).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[indexer] cannot read {root}: {e}");
            // Emit so the frontend clears the "indexing…" spinner even on failure.
            let _ = app_handle.emit("indexer:progress", IndexProgress { path: root, files_indexed: 0 });
            return;
        }
    };

    let mut files_indexed = 0u64;
    let mut last_emit     = 0u64;
    let mut mime_queue    = Vec::<String>::new(); // file paths for pass-2 MIME enrichment
    let indexed_at        = now_unix();

    while let Ok(Some(entry)) = reader.next_entry().await {
        let path     = entry.path();
        let path_str = path.to_string_lossy().into_owned();
        let name     = entry.file_name().to_string_lossy().into_owned();

        let Ok(metadata) = entry.metadata().await else {
            continue;
        };

        let is_dir      = metadata.is_dir();
        let size        = if is_dir { None } else { Some(metadata.len() as i64) };
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        // Pass 1: MIME from extension (zero I/O). Files with unknown extensions
        // are queued for pass-2 magic-byte detection.
        let mime_type = if is_dir { None } else { mime_from_ext(&path) };
        let needs_magic = !is_dir && mime_type.is_none();

        let file_entry = cache::FileEntry {
            path: path_str.clone(),
            name,
            is_dir,
            size,
            modified_at,
            mime_type,
        };

        if let Err(e) = cache::upsert_file(&pool, &file_entry, indexed_at).await {
            eprintln!("[indexer] upsert error for {path_str}: {e}");
        } else {
            files_indexed += 1;

            // Only queue files whose extension is unknown for magic-byte detection.
            if needs_magic {
                mime_queue.push(path_str);
            }

            // Emit every EMIT_INTERVAL files so the frontend refreshes incrementally.
            if files_indexed - last_emit >= EMIT_INTERVAL {
                let _ = app_handle.emit(
                    "indexer:progress",
                    IndexProgress { path: root.clone(), files_indexed },
                );
                last_emit = files_indexed;
            }
        }
    }

    // Final emit for this directory.
    let _ = app_handle.emit(
        "indexer:progress",
        IndexProgress { path: root.clone(), files_indexed },
    );

    // ── Pass 2: MIME enrichment (background) ──────────────────────────────
    // Runs after the full tree is indexed so the user sees entries immediately.
    // Reads 36 bytes per file — tolerable on NAS since it is truly background.
    let pool2       = pool.clone();
    let app_handle2 = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for path_str in mime_queue {
            let path = std::path::Path::new(&path_str);
            if let Some(mime) = detect_mime(path).await {
                if let Err(e) = cache::update_mime(&pool2, &path_str, &mime).await {
                    eprintln!("[indexer] update_mime error for {path_str}: {e}");
                }
            }
        }
        // Final progress event so the frontend can pick up MIME-enriched entries.
        let _ = app_handle2.emit(
            "indexer:progress",
            IndexProgress { path: String::new(), files_indexed },
        );
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Guess MIME type from the file extension — no I/O, instant.
///
/// Covers the vast majority of files found on NAS shares (photos, videos,
/// audio, documents). Returns `None` for unknown or missing extensions so
/// the caller can fall back to magic-byte detection.
fn mime_from_ext(path: &std::path::Path) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    let mime = match ext.as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png"          => "image/png",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "heic" | "heif"=> "image/heic",
        "avif"         => "image/avif",
        "tiff" | "tif" => "image/tiff",
        "bmp"          => "image/bmp",
        "svg"          => "image/svg+xml",
        "ico"          => "image/x-icon",
        "raw" | "cr2" | "nef" | "arw" | "orf" | "rw2" => "image/x-raw",
        // Video
        "mp4" | "m4v"  => "video/mp4",
        "mov"          => "video/quicktime",
        "avi"          => "video/x-msvideo",
        "mkv"          => "video/x-matroska",
        "webm"         => "video/webm",
        "flv"          => "video/x-flv",
        "wmv"          => "video/x-ms-wmv",
        "m2ts"         => "video/mp2t",
        // Audio
        "mp3"          => "audio/mpeg",
        "m4a" | "aac"  => "audio/aac",
        "flac"         => "audio/flac",
        "wav"          => "audio/wav",
        "ogg" | "oga"  => "audio/ogg",
        "opus"         => "audio/opus",
        "wma"          => "audio/x-ms-wma",
        "aiff" | "aif" => "audio/aiff",
        // Documents
        "pdf"          => "application/pdf",
        "txt"          => "text/plain",
        "md"           => "text/markdown",
        "html" | "htm" => "text/html",
        "css"          => "text/css",
        "js" | "mjs"   => "text/javascript",
        "ts"           => "text/typescript",
        "json"         => "application/json",
        "xml"          => "application/xml",
        "csv"          => "text/csv",
        "zip"          => "application/zip",
        "gz" | "tgz"   => "application/gzip",
        "tar"          => "application/x-tar",
        "7z"           => "application/x-7z-compressed",
        "rar"          => "application/x-rar-compressed",
        "doc" | "docx" => "application/msword",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "ppt" | "pptx" => "application/vnd.ms-powerpoint",
        _              => return None,
    };
    Some(mime.to_string())
}

/// Detect MIME type by reading the first 36 bytes of the file.
///
/// Uses the `infer` crate (magic-byte detection). Returns `None` if the file
/// cannot be opened, is empty, or has an unknown type.
async fn detect_mime(path: &std::path::Path) -> Option<String> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(path).await.ok()?;
    let mut buf = [0u8; 36];
    let n = file.read(&mut buf).await.ok()?;

    infer::get(&buf[..n]).map(|t| t.mime_type().to_string())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Return the user's home directory as a String.
/// Checks `$HOME` (macOS/Linux) then `USERPROFILE` (Windows), falls back to `/`.
fn home_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
}
