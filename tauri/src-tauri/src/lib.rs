#![deny(unsafe_code)]

pub mod app;
pub mod config;
pub mod errors;
pub mod library;
pub mod mame;
pub mod metadata;
pub mod platform;
pub mod sessions;
pub mod storage;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = config::settings_path(app.handle())?;
            if let Some(parent) = settings_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            app::emit_ready(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app::get_app_info])
        .run(tauri::generate_context!())
}
