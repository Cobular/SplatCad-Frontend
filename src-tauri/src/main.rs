#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod db;
mod error;

use crate::commands::local_files::{get_file_diff, update_remote_state, update_local_state};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            db::make_db(app.config());
            Ok(())
        })
        .manage(db::get_db())
        .invoke_handler(tauri::generate_handler![get_file_diff, update_local_state, update_remote_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
