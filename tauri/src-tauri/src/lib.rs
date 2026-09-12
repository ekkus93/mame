#![deny(unsafe_code)]

pub mod app;
pub mod artwork;
pub mod bulk_audit;
pub mod collections;
pub mod config;
pub mod config_persistence;
pub mod configuration_explainability;
pub mod configuration_precedence;
pub mod controller_profiles;
pub mod controller_settings;
pub mod errors;
pub mod general_settings;
pub mod history;
pub mod library;
pub mod machine_settings;
pub mod mame;
pub mod metadata;
pub mod path_configuration;
pub mod platform;
pub mod sessions;
pub mod software;
pub mod storage;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(sessions::SessionSupervisor::default())
        .manage(bulk_audit::BulkAuditSupervisor::default())
        .setup(|app| {
            let settings_path = config::settings_path(app.handle())?;
            if let Some(parent) = settings_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            app::emit_ready(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::get_app_info,
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
            sessions::stop_mame,
            metadata::refresh_mame_metadata,
            metadata::get_mame_metadata_status,
            library::query_mame_library,
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
            machine_settings::get_machine_launch_settings,
            machine_settings::set_machine_launch_settings,
            machine_settings::reset_machine_launch_settings,
            controller_settings::get_controller_profile_configuration,
            controller_settings::set_controller_profile_selection,
            controller_settings::create_browser_controller_profile,
            software::query_mame_software_list,
            software::launch_library_software,
            path_configuration::get_content_path_configuration,
            path_configuration::set_content_path_configuration,
            path_configuration::pick_content_directory
        ])
        .run(tauri::generate_context!())
}
