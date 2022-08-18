#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod db;
mod error;

use crate::commands::local_files::{get_all_data, get_all_data_command};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_all_data, get_all_data_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
