#![deny(unsafe_code)]

pub mod app;
pub mod artwork;
pub mod artwork_assets;
pub mod bulk_audit;
pub mod bundled_runtime;
pub mod collections;
pub mod config;
pub mod config_persistence;
pub mod configuration_explainability;
pub mod configuration_precedence;
pub mod controller_profiles;
pub mod controller_settings;
pub mod diagnostics;
pub mod errors;
mod event_names;
pub mod general_settings;
pub mod history;
pub mod library;
pub mod machine_settings;
pub mod mame;
pub mod mame_ui;
pub mod mame_ui_state;
pub mod metadata;
pub mod path_configuration;
pub mod platform;
pub mod save_state_records;
pub mod sessions;
pub mod software;
pub mod storage;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(sessions::SessionSupervisor::default())
        .manage(bulk_audit::BulkAuditSupervisor::default())
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                diagnostics::record(
                    "info",
                    "app.lifecycle",
                    "Main application window destroyed.",
                    serde_json::json!({ "window": "main" }),
                );
            }
        })
        .setup(|app| {
            diagnostics::initialize(app.handle())?;
            let settings_path = config::settings_path(app.handle())?;
            if let Some(parent) = settings_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            app::emit_ready(app.handle())?;
            diagnostics::record(
                "info",
                "app.lifecycle",
                "Application backend ready.",
                serde_json::json!({ "appVersion": env!("CARGO_PKG_VERSION") }),
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::get_app_info,
            diagnostics::get_diagnostics,
            diagnostics::export_diagnostics_bundle,
            sessions::inspect_mame_executable,
            sessions::launch_mame,
            sessions::get_mame_session,
            sessions::pause_mame,
            sessions::resume_mame,
            sessions::reset_mame,
            sessions::set_mame_mute,
            sessions::query_state::query_mame_runtime_state,
            sessions::load_mame_state,
            sessions::save_state::save_mame_state,
            save_state_records::save_known_state,
            save_state_records::list_save_state_records,
            save_state_records::load_known_save_state,
            save_state_records::delete_save_state_record,
            sessions::stop_mame,
            metadata::refresh_mame_metadata,
            metadata::get_mame_metadata_status,
            library::query_mame_library,
            mame_ui::query_mame_ui_library,
            mame_ui_state::get_mame_ui_state,
            mame_ui_state::set_mame_ui_state,
            library::get_mame_machine_detail,
            library::launch_library_machine,
            library::audit::get_library_machine_audit,
            library::audit::run_library_machine_audit,
            bulk_audit::get_library_bulk_audit_status,
            bulk_audit::start_library_bulk_audit,
            bulk_audit::cancel_library_bulk_audit,
            library::get_library_favorite,
            library::set_library_favorite,
            library::query_library_favorites,
            collections::create_library_collection,
            collections::rename_library_collection,
            collections::delete_library_collection,
            collections::query_library_collections,
            collections::query_library_collection_members,
            collections::set_library_collection_machine,
            history::query_library_history,
            general_settings::get_general_settings,
            general_settings::set_general_mame_executable,
            general_settings::set_general_launch_preferences,
            general_settings::pick_mame_executable,
            artwork::get_artwork_configuration,
            artwork::set_artwork_configuration,
            artwork::pick_artwork_directory,
            artwork::discover_machine_artwork,
            artwork_assets::read_artwork_asset,
            machine_settings::get_machine_launch_settings,
            machine_settings::set_machine_launch_settings,
            machine_settings::reset_machine_launch_settings,
            controller_settings::get_controller_profile_configuration,
            controller_settings::set_controller_profile_selection,
            controller_settings::create_browser_controller_profile,
            software::query_mame_software_list,
            software::query_mame_bios_choices,
            software::launch_library_software,
            path_configuration::get_content_path_configuration,
            path_configuration::set_content_path_configuration,
            path_configuration::pick_content_directory
        ])
        .run(tauri::generate_context!())
}
