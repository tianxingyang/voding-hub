mod adapters;
mod commands;
mod core;
mod db;

use commands::{register_commands, DbState};
use core::FileWatcher;
use db::init_db;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(register_commands())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let conn = init_db(&app_dir).expect("Failed to init database");
            app.manage(DbState(Mutex::new(conn)));

            let watcher = FileWatcher::new(app.handle().clone())
                .expect("Failed to create file watcher");
            watcher.start_global_watch().ok();
            app.manage(watcher);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
