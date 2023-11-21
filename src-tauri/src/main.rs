// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::process::exit;

#[tauri::command]
fn close() -> String {
  exit(0);
}

#[tauri::command]
fn launch() -> String {
  let command = Command::new("kate").output().expect("failed to execute process");
  return command.status.to_string();
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![close, launch])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
