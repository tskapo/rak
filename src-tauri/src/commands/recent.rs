// Commandes Tauri pour l'historique des fichiers/dossiers récemment visités.
// get_recents : retourne les N entrées les plus récentes depuis SQLite (newest first).
// add_recent : insère ou met à jour une entrée dans la table recents (FIFO, max 50).

use tauri::State;

use crate::services::cache::{self, FileEntry};
use crate::AppState;

// ── Commands ──────────────────────────────────────────────────────────────────

/// Return the `limit` most recently visited files and folders, newest first.
///
/// `modified_at` in each returned `FileEntry` contains the `visited_at` timestamp
/// (when the entry was last opened), not the file's own modification date.
#[tauri::command]
pub async fn get_recents(
    limit: i64,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    cache::get_recents(&state.db, limit)
        .await
        .map_err(|e| e.to_string())
}

/// Record a visit to `entry`.
///
/// Upserts the entry in the `recents` table and prunes the table to 50 rows
/// (FIFO — oldest entries are discarded automatically).
#[tauri::command]
pub async fn add_recent(
    entry: FileEntry,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cache::add_recent(&state.db, &entry)
        .await
        .map_err(|e| e.to_string())
}
