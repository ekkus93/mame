#![deny(unsafe_code)]

use std::path::Path;

use serde::Serialize;

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
pub mod content_paths;
pub mod controller_profiles;
pub mod controller_settings;
pub mod diagnostics;
pub mod effective_runtime;
pub mod errors;
mod event_names;
pub mod general_settings;
pub mod history;
pub mod library;
pub mod machine_settings;
pub mod mame;
pub mod mame_ui;
pub mod mame_ui_export;
pub mod mame_ui_launch;
pub mod mame_ui_state;
pub mod metadata;
pub mod path_configuration;
pub mod platform;
pub mod save_state_records;
pub mod sessions;
pub mod software;
pub mod storage;

pub fn verify_bundled_runtime_resource_dir(
    resource_dir: impl AsRef<Path>,
) -> errors::AppResult<mame::MameExecutableIdentity> {
    let source = effective_runtime::resolve_effective_mame_source(
        &config::SettingsV2::default(),
        resource_dir.as_ref(),
    )?;

    if source.kind() != mame::MameExecutableSourceKind::Bundled {
        return Err(errors::AppError::new(
            "MAME_BUNDLED_RUNTIME_SOURCE_UNEXPECTED",
            "Default packaged runtime verification did not resolve the bundled MAME runtime.",
        )
        .with_details(serde_json::json!({
            "source": source.kind(),
            "path": source.path()
        })));
    }

    let identity = mame::inspect_executable(source)?;
    if identity.source != mame::MameExecutableSourceKind::Bundled
        || identity.trust != mame::MameExecutableTrust::QualifiedBundled
    {
        return Err(errors::AppError::new(
            "MAME_BUNDLED_RUNTIME_IDENTITY_UNEXPECTED",
            "Default packaged runtime verification returned a non-bundled runtime identity.",
        )
        .with_details(serde_json::json!({ "identity": identity })));
    }

    Ok(identity)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetContentPolicyGateReport {
    code: String,
    message: String,
    launch_attempted: bool,
    content_failure: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetContentPolicyVerificationReport {
    schema_version: u32,
    runtime: mame::MameExecutableIdentity,
    empty_effective_path_count: usize,
    unaudited_gate: ResetContentPolicyGateReport,
    configured_rompath: String,
    configured_path_status: config::PathValidationStatus,
    audit_classification: mame::MameAuditClassification,
    unavailable_gate: ResetContentPolicyGateReport,
    early_exit: ResetContentPolicyGateReport,
}

pub fn verify_reset_content_policy(
    resource_dir: impl AsRef<Path>,
    content_dir: impl AsRef<Path>,
    probe_machine: &str,
) -> errors::AppResult<ResetContentPolicyVerificationReport> {
    let source = effective_runtime::resolve_effective_mame_source(
        &config::SettingsV2::default(),
        resource_dir.as_ref(),
    )?;
    let runtime = mame::inspect_executable(source.clone())?;
    if runtime.source != mame::MameExecutableSourceKind::Bundled
        || runtime.trust != mame::MameExecutableTrust::QualifiedBundled
    {
        return Err(errors::AppError::new(
            "MAME_RESET_PACKAGE_RUNTIME_UNEXPECTED",
            "Reset package qualification did not resolve the qualified bundled MAME runtime.",
        )
        .with_details(serde_json::json!({ "identity": runtime })));
    }

    let empty_effective =
        content_paths::effective_content_paths(&config::ContentPathsV1::default());
    let mut unaudited_launch_attempted = false;
    let unaudited_error = sessions::launch_after_audit_gate(probe_machine, None, || {
        unaudited_launch_attempted = true;
        Ok(())
    })
    .expect_err("an unaudited packaged launch must be gated");
    if unaudited_error.code != "MAME_CONTENT_AUDIT_REQUIRED" || unaudited_launch_attempted {
        return Err(errors::AppError::new(
            "MAME_RESET_UNAUDITED_GATE_UNEXPECTED",
            "An unaudited packaged launch did not fail closed before process spawn.",
        )
        .with_details(serde_json::json!({
            "gateError": unaudited_error,
            "launchAttempted": unaudited_launch_attempted
        })));
    }

    let configured = config::ContentPathsV1 {
        rom_paths: vec![config::PlatformPath::new(content_dir.as_ref())],
        software_paths: Vec::new(),
        chd_paths: Vec::new(),
    };
    let effective = content_paths::effective_content_paths(&configured);
    let configured_entry = effective.entries.first().ok_or_else(|| {
        errors::AppError::new(
            "MAME_RESET_CONTENT_PATH_MISSING",
            "Reset package qualification did not produce the configured ROM path.",
        )
    })?;
    if configured_entry.validation.status != config::PathValidationStatus::Accessible {
        return Err(errors::AppError::new(
            "MAME_RESET_CONTENT_PATH_INVALID",
            "Reset package qualification requires an accessible empty ROM directory.",
        )
        .with_details(serde_json::json!({
            "validation": configured_entry.validation
        })));
    }

    let launch_paths =
        sessions::append_effective_content_project_paths(Vec::new(), &effective)?;
    let configured_rompath = launch_paths
        .iter()
        .find(|entry| entry.option == "rompath")
        .map(|entry| entry.path.clone())
        .ok_or_else(|| {
            errors::AppError::new(
                "MAME_RESET_ROMPATH_MISSING",
                "The configured effective content path was not propagated to launch rompath.",
            )
        })?;

    let audit = mame::audit_machine(&source, probe_machine, &effective)?;
    if !matches!(
        audit.classification,
        mame::MameAuditClassification::MissingRequired
            | mame::MameAuditClassification::Incorrect
            | mame::MameAuditClassification::MixedFailure
    ) {
        return Err(errors::AppError::new(
            "MAME_RESET_MISSING_CONTENT_AUDIT_UNEXPECTED",
            "The real bundled runtime did not classify the intentionally empty ROM directory as unavailable content.",
        )
        .with_details(serde_json::json!({ "audit": audit })));
    }

    let mut unavailable_launch_attempted = false;
    let unavailable_error =
        sessions::launch_after_audit_gate(probe_machine, Some(audit.classification), || {
            unavailable_launch_attempted = true;
            Ok(())
        })
        .expect_err("missing packaged content must be gated");
    if unavailable_error.code != "MAME_CONTENT_UNAVAILABLE" || unavailable_launch_attempted {
        return Err(errors::AppError::new(
            "MAME_RESET_UNAVAILABLE_GATE_UNEXPECTED",
            "Unavailable packaged content did not fail closed before process spawn.",
        )
        .with_details(serde_json::json!({
            "gateError": unavailable_error,
            "launchAttempted": unavailable_launch_attempted
        })));
    }

    let early_exit_error = sessions::classify_early_exit_output(&audit.raw_excerpt, "");
    let content_failure = early_exit_error
        .details
        .get("contentFailure")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if early_exit_error.code != "MAME_CONTENT_LAUNCH_FAILED" || !content_failure {
        return Err(errors::AppError::new(
            "MAME_RESET_EARLY_EXIT_CLASSIFICATION_UNEXPECTED",
            "Real bundled-MAME missing-content output was not classified as an actionable content launch failure.",
        )
        .with_details(serde_json::json!({ "classifiedError": early_exit_error })));
    }

    Ok(ResetContentPolicyVerificationReport {
        schema_version: 1,
        runtime,
        empty_effective_path_count: empty_effective.entries.len(),
        unaudited_gate: ResetContentPolicyGateReport {
            code: unaudited_error.code,
            message: unaudited_error.message,
            launch_attempted: unaudited_launch_attempted,
            content_failure: None,
        },
        configured_rompath,
        configured_path_status: configured_entry.validation.status.clone(),
        audit_classification: audit.classification,
        unavailable_gate: ResetContentPolicyGateReport {
            code: unavailable_error.code,
            message: unavailable_error.message,
            launch_attempted: unavailable_launch_attempted,
            content_failure: None,
        },
        early_exit: ResetContentPolicyGateReport {
            code: early_exit_error.code,
            message: early_exit_error.message,
            launch_attempted: false,
            content_failure: Some(content_failure),
        },
    })
}

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
            mame_ui_export::export_mame_ui_displayed_list,
            mame_ui_launch::launch_mame_empty,
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
