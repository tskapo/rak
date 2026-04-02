// Räk application entry point.
// Initialises the SQLite database, registers all Tauri IPC commands, and starts the app.

mod commands;
mod services;

use services::{cache, indexer};
use sqlx::SqlitePool;
use tauri::Manager;

/// Shared application state injected into every Tauri command via `tauri::State`.
pub struct AppState {
    pub db: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");

            let db_path = app_dir.join("rak.db");
            let db_path_str = db_path.to_string_lossy().into_owned();

            let pool = tauri::async_runtime::block_on(cache::open(&db_path_str))
                .expect("failed to open SQLite database");

            // Determine the best startup path (last visited or home) then kick off
            // a background index scan so the cache is ready before the user navigates.
            let start_path =
                tauri::async_runtime::block_on(indexer::resolve_start_path(&pool));

            let app_handle = app.handle().clone();
            indexer::scan_dir(pool.clone(), start_path, app_handle);

            app.manage(AppState { db: pool });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
