// Commandes Tauri pour les préférences par dossier et les préférences globales de l'application.
// get_folder_prefs / save_folder_prefs : view_mode, sort_by, sort_dir par chemin.
// get_app_prefs / save_app_pref : clé/valeur globaux (theme, locale, ui_scale, watch_recursive).
// La langue et le thème sont stockés en SQLite — indépendants de l'OS et de localStorage.

use tauri::State;

use crate::services::cache::{self, AppPrefs, FolderPrefs};
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

// ── App preferences ───────────────────────────────────────────────────────────

/// Return all global application preferences with defaults applied.
///
/// Defaults: theme = "system", locale = "fr", ui_scale = 1.0, watch_recursive = false.
#[tauri::command]
pub async fn get_app_prefs(state: State<'_, AppState>) -> Result<AppPrefs, String> {
    cache::get_app_prefs(&state.db)
        .await
        .map_err(|e| e.to_string())
}

/// Return the raw string value of a single global preference key.
///
/// Returns `null` on the frontend if the key has never been saved.
#[tauri::command]
pub async fn get_app_pref(
    key:   String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    cache::get_app_pref_value(&state.db, &key)
        .await
        .map_err(|e| e.to_string())
}

/// Persist a single global preference by key.
///
/// Reserved keys: `"theme"` (`"light"` | `"dark"` | `"system"`),
/// `"locale"` (`"fr"` | `"en"` | `"ka"`), `"ui_scale"` (float string),
/// `"watch_recursive"` (`"true"` | `"false"`).
#[tauri::command]
pub async fn save_app_pref(
    key:   String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    cache::save_app_pref(&state.db, &key, &value)
        .await
        .map_err(|e| e.to_string())
}
