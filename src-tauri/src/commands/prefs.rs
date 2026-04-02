// Commandes Tauri pour les préférences par dossier.
// get_folder_prefs / save_folder_prefs : view_mode, sort_by, sort_dir par chemin.
// Les préférences globales (thème, locale, ui_scale, watch_recursive) sont dans l'issue 6.

use tauri::State;

use crate::services::cache::{self, FolderPrefs};
use crate::AppState;

// ── Commands ──────────────────────────────────────────────────────────────────

/// Return the display preferences for `path` (view mode, sort column, sort direction).
///
/// Returns sensible defaults (list / name / asc) if no preferences have been
/// saved for this folder yet.
#[tauri::command]
pub async fn get_folder_prefs(
    path:  String,
    state: State<'_, AppState>,
) -> Result<FolderPrefs, String> {
    cache::get_folder_prefs(&state.db, &path)
        .await
        .map_err(|e| e.to_string())
}

/// Persist the display preferences for the folder identified by `prefs.path`.
///
/// Called automatically on every view/sort change so preferences survive
/// between sessions without an explicit "save" step.
#[tauri::command]
pub async fn save_folder_prefs(
    prefs: FolderPrefs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cache::save_folder_prefs(&state.db, &prefs)
        .await
        .map_err(|e| e.to_string())
}
