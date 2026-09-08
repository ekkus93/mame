#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = mame_tauri_lib::run() {
        eprintln!("fatal Tauri runtime error: {error}");
        std::process::exit(1);
    }
}
