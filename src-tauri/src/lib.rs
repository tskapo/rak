// Räk application entry point.
// Initialises the SQLite database, registers all Tauri IPC commands, and starts the app.

mod commands;
mod services;

use services::{cache, indexer, watcher::WatcherService};
use sqlx::SqlitePool;
use std::sync::Mutex;
use tauri::Manager;

/// Shared application state injected into every Tauri command via `tauri::State`.
pub struct AppState {
    pub db:      SqlitePool,
    /// Mutex because `watch`/`unwatch` require `&mut self`.
    pub watcher: Mutex<WatcherService>,
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

            // Read prefs before starting background services
            let prefs = tauri::async_runtime::block_on(cache::get_app_prefs(&pool))
                .unwrap_or_else(|_| cache::AppPrefs {
                    theme:           "system".to_string(),
                    locale:          "fr".to_string(),
                    ui_scale:        1.0,
                    watch_recursive: false,
                });

            // Determine the best startup path (last visited or home) then kick off
            // a background index scan so the cache is ready before the user navigates.
            let start_path =
                tauri::async_runtime::block_on(indexer::resolve_start_path(&pool));

            let app_handle = app.handle().clone();
            indexer::scan_dir(pool.clone(), start_path.clone(), app_handle.clone());

            // Start the filesystem watcher on the same path
            let mut fs_watcher =
                WatcherService::new(pool.clone(), app_handle, prefs.watch_recursive)
                    .expect("failed to create filesystem watcher");

            if let Err(e) = fs_watcher.watch(std::path::Path::new(&start_path)) {
                eprintln!("[watcher] failed to watch {start_path}: {e}");
            }

            app.manage(AppState {
                db:      pool,
                watcher: Mutex::new(fs_watcher),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fs::list_dir,
            commands::fs::get_metadata,
            commands::fs::read_file_chunk,
            commands::fs::rescan_dir,
            commands::prefs::get_folder_prefs,
            commands::prefs::save_folder_prefs,
            commands::recent::get_recents,
            commands::recent::add_recent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
