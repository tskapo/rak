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

/// Walk `root` recursively using an explicit stack (avoids recursion depth issues
/// on deep directory trees). Upserts every entry into the cache and emits a
/// `indexer:progress` event after each directory is processed.
async fn walk_and_index(pool: SqlitePool, root: String, app_handle: AppHandle) {
    let mut stack = vec![root];
    let mut files_indexed: u64 = 0;

    while let Some(dir_path) = stack.pop() {
        let mut reader = match tokio::fs::read_dir(&dir_path).await {
            Ok(r) => r,
            Err(_) => continue, // skip unreadable dirs (permissions, disconnected share, etc.)
        };

        while let Ok(Some(entry)) = reader.next_entry().await {
            let path = entry.path();
            let path_str = path.to_string_lossy().into_owned();
            let name = entry.file_name().to_string_lossy().into_owned();

            let Ok(metadata) = entry.metadata().await else {
                continue;
            };

            let is_dir = metadata.is_dir();
            let size = if is_dir { None } else { Some(metadata.len() as i64) };
            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);

            let mime_type = if is_dir { None } else { detect_mime(&path).await };

            let file_entry = cache::FileEntry {
                path: path_str.clone(),
                name,
                is_dir,
                size,
                modified_at,
                mime_type,
            };

            if let Err(e) = cache::upsert_file(&pool, &file_entry, now_unix()).await {
                eprintln!("[indexer] upsert error for {path_str}: {e}");
            } else {
                files_indexed += 1;
            }

            if is_dir {
                stack.push(path_str);
            }
        }

        let _ = app_handle.emit(
            "indexer:progress",
            IndexProgress {
                path: dir_path,
                files_indexed,
            },
        );
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
