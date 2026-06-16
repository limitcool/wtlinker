// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            pick_folder,
            get_wt_windows,
            launch_wt,
            get_sessions,
            get_session_details,
            convert_claude_to_codex,
            convert_codex_to_claude,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}