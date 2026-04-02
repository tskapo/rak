// WatcherService : surveille le système de fichiers via la crate `notify`.
// Invalide les entrées du cache SQLite lorsqu'un fichier est créé, modifié ou supprimé.
// Déclenche un re-scan ciblé du dossier concerné via IndexerService.

use notify::{
    event::{ModifyKind, RenameMode},
    recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::Serialize;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::services::{cache, indexer};

// ── Public types ──────────────────────────────────────────────────────────────

/// Payload emitted as the `watcher:change` Tauri event on every filesystem change.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherChange {
    /// "create" | "remove" | "modify" | "rename"
    pub kind: String,
    pub path: String,
    /// Set only for renames — the destination path.
    pub new_path: Option<String>,
}

// ── WatcherService ────────────────────────────────────────────────────────────

/// Wraps a `notify` watcher and bridges its events into async cache updates.
///
/// Must be kept alive for watching to continue — dropping it stops all watches.
pub struct WatcherService {
    watcher:   RecommendedWatcher,
    recursive: bool,
}

impl WatcherService {
    /// Create a new watcher and spawn a background task to process events.
    ///
    /// `recursive` controls whether `watch()` descends into subdirectories
    /// automatically. Set to `false` (the default pref) on large NAS mounts.
    pub fn new(
        pool:       SqlitePool,
        app_handle: AppHandle,
        recursive:  bool,
    ) -> Result<Self, notify::Error> {
        let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<Event>>();

        // notify fires its callback on a native OS thread — UnboundedSender::send
        // is non-blocking so it is safe to call from outside the Tokio runtime.
        let watcher = recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        tauri::async_runtime::spawn(async move {
            while let Some(res) = rx.recv().await {
                handle_event(res, &pool, &app_handle).await;
            }
        });

        Ok(Self { watcher, recursive })
    }

    /// Start watching `path`. Recursive mode is determined by the stored preference.
    pub fn watch(&mut self, path: &std::path::Path) -> Result<(), notify::Error> {
        let mode = if self.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        self.watcher.watch(path, mode)
    }

    /// Stop watching `path`.
    pub fn unwatch(&mut self, path: &std::path::Path) -> Result<(), notify::Error> {
        self.watcher.unwatch(path)
    }
}

// ── Event dispatch ────────────────────────────────────────────────────────────

async fn handle_event(
    res:        notify::Result<Event>,
    pool:       &SqlitePool,
    app_handle: &AppHandle,
) {
    let event = match res {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[watcher] error: {e}");
            return;
        }
    };

    match event.kind {
        EventKind::Create(_) => {
            for path in &event.paths {
                on_create(pool, app_handle, path).await;
            }
        }

        EventKind::Remove(_) => {
            for path in &event.paths {
                on_remove(pool, app_handle, path).await;
            }
        }

        // Platforms that report both ends of a rename in one event (e.g. macOS FSEvents)
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if event.paths.len() == 2 => {
            on_rename(pool, app_handle, &event.paths[0], &event.paths[1]).await;
        }

        // Platforms that emit separate From/To rename events — treat each as create
        EventKind::Modify(ModifyKind::Name(_)) => {
            for path in &event.paths {
                on_create(pool, app_handle, path).await;
            }
        }

        EventKind::Modify(_) => {
            for path in &event.paths {
                on_modify(pool, app_handle, path).await;
            }
        }

        _ => {} // ignore access events and other noise
    }
}

// ── Per-event handlers ────────────────────────────────────────────────────────

async fn on_create(pool: &SqlitePool, app_handle: &AppHandle, path: &PathBuf) {
    let Some(entry) = read_entry(path).await else { return };

    let is_dir   = entry.is_dir;
    let path_str = entry.path.clone();

    if let Err(e) = cache::upsert_file(pool, &entry, now_unix()).await {
        eprintln!("[watcher] upsert error on create ({path_str}): {e}");
        return;
    }

    // New directory — index its contents in the background
    if is_dir {
        indexer::scan_dir(pool.clone(), path_str.clone(), app_handle.clone());
    }

    emit(app_handle, "create", path_str, None);
}

async fn on_remove(pool: &SqlitePool, app_handle: &AppHandle, path: &PathBuf) {
    let path_str = path.to_string_lossy().into_owned();

    // Remove the entry and cascade to any children (handles directory removal)
    let _ = cache::remove_file(pool, &path_str).await;
    let _ = cache::remove_children(pool, &path_str).await;

    emit(app_handle, "remove", path_str, None);
}

async fn on_modify(pool: &SqlitePool, app_handle: &AppHandle, path: &PathBuf) {
    let Some(entry) = read_entry(path).await else { return };

    let path_str = entry.path.clone();

    if let Err(e) = cache::upsert_file(pool, &entry, now_unix()).await {
        eprintln!("[watcher] upsert error on modify ({path_str}): {e}");
        return;
    }

    emit(app_handle, "modify", path_str, None);
}

async fn on_rename(
    pool:       &SqlitePool,
    app_handle: &AppHandle,
    old_path:   &PathBuf,
    new_path:   &PathBuf,
) {
    let old_str = old_path.to_string_lossy().into_owned();
    let new_str = new_path.to_string_lossy().into_owned();

    // Evict the old location and all its children from the cache
    let _ = cache::remove_file(pool, &old_str).await;
    let _ = cache::remove_children(pool, &old_str).await;

    // Index the new location
    if let Some(entry) = read_entry(new_path).await {
        let is_dir = entry.is_dir;

        let _ = cache::upsert_file(pool, &entry, now_unix()).await;

        if is_dir {
            indexer::scan_dir(pool.clone(), new_str.clone(), app_handle.clone());
        }
    }

    emit(app_handle, "rename", old_str, Some(new_str));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a `FileEntry` from live filesystem metadata at `path`.
/// Returns `None` if the path is no longer accessible (e.g. already deleted).
async fn read_entry(path: &PathBuf) -> Option<cache::FileEntry> {
    use std::time::UNIX_EPOCH;
    use tokio::io::AsyncReadExt;

    let metadata  = tokio::fs::metadata(path).await.ok()?;
    let name      = path.file_name()?.to_string_lossy().into_owned();
    let path_str  = path.to_string_lossy().into_owned();
    let is_dir    = metadata.is_dir();

    let size = if is_dir { None } else { Some(metadata.len() as i64) };

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    let mime_type = if is_dir {
        None
    } else {
        let mut file = tokio::fs::File::open(path).await.ok()?;
        let mut buf  = [0u8; 36];
        let n        = file.read(&mut buf).await.ok()?;
        infer::get(&buf[..n]).map(|t| t.mime_type().to_string())
    };

    Some(cache::FileEntry { path: path_str, name, is_dir, size, modified_at, mime_type })
}

fn emit(app_handle: &AppHandle, kind: &str, path: String, new_path: Option<String>) {
    let _ = app_handle.emit(
        "watcher:change",
        WatcherChange { kind: kind.to_string(), path, new_path },
    );
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
