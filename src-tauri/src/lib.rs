mod adapters;
mod commands;
mod core;
mod db;

use commands::register_commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
