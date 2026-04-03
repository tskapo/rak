// Commandes Tauri liées au système de fichiers.
// list_dir : liste le contenu d'un dossier depuis le cache SQLite (ou lance un scan si vide/périmé).
// get_metadata : retourne les métadonnées d'une entrée (cache en priorité, stat live en fallback).
// read_file_chunk : lit un extrait d'un fichier en bytes bruts (pour preview texte/binaire).
// rescan_dir : déclenche immédiatement un re-scan d'un dossier en arrière-plan.

use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::services::{cache, indexer};
use crate::AppState;

/// Cache entries older than this threshold trigger a background re-scan.
const STALE_SECS: i64 = 5 * 60;

// ── Response types ────────────────────────────────────────────────────────────

/// Returned by `list_dir` — includes entries and the cache timestamp so the
/// frontend can display "last indexed X minutes ago".
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirResult {
    pub entries:    Vec<cache::FileEntry>,
    /// Unix timestamp (seconds) of the most recent index run for this directory.
    /// Null when the directory has never been indexed.
    pub indexed_at: Option<i64>,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// List the direct children of `path` from the SQLite cache.
///
/// Returns immediately with whatever is cached. If the cache is empty or
/// older than 5 minutes, a background re-scan is triggered automatically;
/// the frontend will receive `indexer:progress` events as new entries arrive.
#[tauri::command]
pub async fn list_dir(
    path:       String,
    state:      State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<DirResult, String> {
    let pool = &state.db;

    let entries    = cache::list_dir(pool, &path).await.map_err(|e| e.to_string())?;
    let indexed_at = cache::get_dir_indexed_at(pool, &path).await.map_err(|e| e.to_string())?;

    let stale = indexed_at
        .map(|ts| now_unix() - ts > STALE_SECS)
        .unwrap_or(true);

    if stale {
        indexer::scan_dir(pool.clone(), path, app_handle);
    }

    Ok(DirResult { entries, indexed_at })
}

/// Return metadata for a single file or directory.
///
/// Tries the SQLite cache first; falls back to a live filesystem stat if the
/// entry is not cached (e.g. a freshly created file not yet indexed).
#[tauri::command]
pub async fn get_metadata(
    path:  String,
    state: State<'_, AppState>,
) -> Result<cache::FileEntry, String> {
    let pool = &state.db;

    // Fast path: cached entry
    if let Some(entry) = cache::get_file(pool, &path).await.map_err(|e| e.to_string())? {
        return Ok(entry);
    }

    // Fallback: live stat (covers files not yet indexed)
    live_stat(&path).await
}

/// Read up to `size` bytes from `path` starting at `offset`.
///
/// Returns raw bytes as a `Vec<u8>`. The frontend receives this as a
/// `number[]` and can convert it with `new Uint8Array(chunk)`.
#[tauri::command]
pub async fn read_file_chunk(
    path:   String,
    offset: u64,
    size:   u64,
) -> Result<Vec<u8>, String> {
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| format!("cannot open {path}: {e}"))?;

    file.seek(std::io::SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("seek error: {e}"))?;

    let mut buf = vec![0u8; size as usize];
    let n = file.read(&mut buf).await.map_err(|e| format!("read error: {e}"))?;
    buf.truncate(n);

    Ok(buf)
}

/// Trigger an immediate background re-scan of `path`, regardless of cache age.
///
/// The frontend should call this when the user clicks the ↺ rescan button.
/// Progress is reported via `indexer:progress` events.
#[tauri::command]
pub async fn rescan_dir(
    path:       String,
    state:      State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    indexer::scan_dir(state.db.clone(), path, app_handle);
    Ok(())
}

/// Return the mounted volumes visible to the user.
///
/// On macOS this reads `/Volumes`; on other platforms it returns an empty list.
/// Each entry has `is_dir = true` and `size = None`.
#[tauri::command]
pub async fn list_volumes() -> Result<Vec<cache::FileEntry>, String> {
    #[cfg(target_os = "macos")]
    {
        list_volumes_macos().await
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "macos")]
async fn list_volumes_macos() -> Result<Vec<cache::FileEntry>, String> {
    use std::time::UNIX_EPOCH;

    let mut entries = Vec::new();
    let mut rd = tokio::fs::read_dir("/Volumes")
        .await
        .map_err(|e| e.to_string())?;

    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let modified_at = tokio::fs::metadata(&path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        entries.push(cache::FileEntry {
            path:       path.to_string_lossy().into_owned(),
            name,
            is_dir:     true,
            size:       None,
            modified_at,
            mime_type:  None,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a `FileEntry` directly from filesystem metadata (no cache involved).
async fn live_stat(path: &str) -> Result<cache::FileEntry, String> {
    use tokio::io::AsyncReadExt;

    let std_path = std::path::Path::new(path);
    let name = std_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("cannot stat {path}: {e}"))?;

    let is_dir      = metadata.is_dir();
    let size        = if is_dir { None } else { Some(metadata.len() as i64) };
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    let mime_type = if is_dir {
        None
    } else {
        let mut file = tokio::fs::File::open(path).await.ok();
        if let Some(ref mut f) = file {
            let mut buf = [0u8; 36];
            let n = f.read(&mut buf).await.unwrap_or(0);
            infer::get(&buf[..n]).map(|t| t.mime_type().to_string())
        } else {
            None
        }
    };

    Ok(cache::FileEntry {
        path: path.to_string(),
        name,
        is_dir,
        size,
        modified_at,
        mime_type,
    })
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
