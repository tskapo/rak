// Commandes Tauri pour le système de tags et les labels de disques.
// get_tags / create_tag / delete_tag : CRUD sur la table tags.
// get_file_tags / set_file_tags : lecture et mise à jour des associations fichier ↔ tags.
// get_disk_label / save_disk_label : couleur hex et surnom par chemin de montage.

use tauri::State;

use crate::services::cache::{self, DiskLabel, Tag};
use crate::AppState;

// ── Tag commands ──────────────────────────────────────────────────────────────

/// Return all user-defined tags, sorted alphabetically by name.
#[tauri::command]
pub async fn get_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    cache::get_tags(&state.db)
        .await
        .map_err(|e| e.to_string())
}

/// Create a new tag with the given `name` and hex `color`.
///
/// Returns the created tag including its auto-generated `id`.
///
/// @throws if a tag with the same name already exists (UNIQUE constraint).
#[tauri::command]
pub async fn create_tag(
    name:  String,
    color: String,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    cache::create_tag(&state.db, &name, &color)
        .await
        .map_err(|e| e.to_string())
}

/// Delete the tag with the given `id`.
///
/// Cascades automatically to all `file_tags` rows (via SQLite `ON DELETE CASCADE`).
#[tauri::command]
pub async fn delete_tag(
    id:    i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cache::delete_tag(&state.db, id)
        .await
        .map_err(|e| e.to_string())
}

/// Return all tags assigned to the file or folder at `path`.
#[tauri::command]
pub async fn get_file_tags(
    path:  String,
    state: State<'_, AppState>,
) -> Result<Vec<Tag>, String> {
    cache::get_file_tags(&state.db, &path)
        .await
        .map_err(|e| e.to_string())
}

/// Replace the full tag set for the file at `path` with `tag_ids`.
///
/// Passing an empty list removes all tags from the file.
#[tauri::command]
pub async fn set_file_tags(
    path:    String,
    tag_ids: Vec<i64>,
    state:   State<'_, AppState>,
) -> Result<(), String> {
    cache::set_file_tags(&state.db, &path, &tag_ids)
        .await
        .map_err(|e| e.to_string())
}

// ── Disk label commands ───────────────────────────────────────────────────────

/// Return the label (color + optional name) for the mounted volume at `path`.
///
/// Returns `null` on the frontend if no label has been set for this path.
#[tauri::command]
pub async fn get_disk_label(
    path:  String,
    state: State<'_, AppState>,
) -> Result<Option<DiskLabel>, String> {
    cache::get_disk_label(&state.db, &path)
        .await
        .map_err(|e| e.to_string())
}

/// Persist (or update) the label for the mounted volume described by `label`.
#[tauri::command]
pub async fn save_disk_label(
    label: DiskLabel,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cache::save_disk_label(&state.db, &label)
        .await
        .map_err(|e| e.to_string())
}
