//! Supervised MAME process lifecycle and authoritative runtime session state.

mod control {
    include!(concat!(env!("OUT_DIR"), "/runtime_control_mt710.rs"));
}
mod load_state;
pub(crate) mod query_state;
pub(crate) mod save_state;
mod supervisor;
mod supervisor_exit;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    bundled_runtime::BundledRuntimeLayout,
    config::{load_settings, settings_path, LaunchPreferencesV1},
    content_paths::{effective_content_paths, EffectiveContentPaths},
    diagnostics,
    errors::{AppError, AppResult},
    event_names::{external_session_lifecycle_event, SESSION_PAUSED_EVENT, SESSION_RESUMED_EVENT},
    history, machine_settings,
    mame::{
        build_launch_argv, compose_mame_path_list, inspect_executable,
        load_current_machine_audit_result, MameAuditClassification, MameExecutableIdentity,
        MameExecutableSource, MameExecutableSourceKind, MameLaunchTarget, ProjectPathArgument,
    },
    storage,
};

pub use load_state::{LoadMameStateFailedEventV1, LoadMameStateRequest, LoadMameStateResult};
pub use save_state::{SaveMameStateFailedEventV1, SaveMameStateRequest, SaveMameStateResult};
use supervisor::EventSink;
pub use supervisor::{
    EffectiveLaunchConfig, EffectiveProjectPath, SessionLifecycleEventV1, SessionSnapshot,
    SessionState, SessionSupervisor, StopSessionResult,
};

const BUNDLED_MAME_STATE_DIR: &str = "mame-state";
const USER_WRITABLE_MAME_PATHS: [(&str, &str); 8] = [
    ("inipath", "ini"),
    ("cfg_directory", "cfg"),
    ("nvram_directory", "nvram"),
    ("input_directory", "input"),
    ("state_directory", "state"),
    ("snapshot_directory", "snap"),
    ("diff_directory", "diff"),
    ("comment_directory", "comments"),
];

/// MAME executable sources that an untrusted frontend may select by path.
///
/// A release-qualified bundled executable is intentionally not representable
/// here. Bundled sidecar resolution is owned by the Rust/package layer so the
/// frontend cannot self-assert `qualifiedBundled` trust for an arbitrary path.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MameExecutableSelectionKind {
    External,
    DevelopmentTree,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameExecutableRequest {
    pub source: MameExecutableSelectionKind,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPathRequest {
    pub option: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchMameRequest {
    pub executable: MameExecutableRequest,
    pub machine: String,
    pub software: Option<String>,
    #[serde(default)]
    pub project_paths: Vec<ProjectPathRequest>,
    #[serde(default)]
    pub launch_overrides: Option<LaunchPreferencesV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StopMameRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PauseMameRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PauseMameResult {
    pub schema_version: u32,
    pub session_id: String,
    pub paused: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionPauseEventV1 {
    pub schema_version: u32,
    pub session_id: String,
    pub paused: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetMameRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResetKind {
    Soft,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetMameResult {
    pub schema_version: u32,
    pub session_id: String,
    pub kind: ResetKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMameMuteRequest {
    pub session_id: String,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMameMuteResult {
    pub schema_version: u32,
    pub session_id: String,
    pub ui_muted: bool,
    pub effective_muted: bool,
}

#[tauri::command]
pub fn inspect_mame_executable(
    request: MameExecutableRequest,
) -> AppResult<MameExecutableIdentity> {
    inspect_executable(executable_source(&request))
}

#[tauri::command]
pub fn launch_mame(
    request: LaunchMameRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    let source = executable_source(&request.executable);
    launch_mame_with_source(
        source,
        request.machine,
        request.software,
        request.project_paths,
        request.launch_overrides,
        supervisor,
        app,
    )
}

pub(crate) fn launch_mame_with_source(
    source: MameExecutableSource,
    machine: String,
    software: Option<String>,
    project_paths: Vec<ProjectPathRequest>,
    transient_launch_overrides: Option<LaunchPreferencesV1>,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    launch_mame_with_source_and_bios_policy(
        source,
        machine,
        software,
        None,
        project_paths,
        transient_launch_overrides,
        false,
        supervisor,
        app,
    )
}

pub(crate) fn launch_catalog_mame_with_source(
    source: MameExecutableSource,
    machine: String,
    software: Option<String>,
    project_paths: Vec<ProjectPathRequest>,
    transient_launch_overrides: Option<LaunchPreferencesV1>,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    launch_mame_with_source_and_bios_policy(
        source,
        machine,
        software,
        None,
        project_paths,
        transient_launch_overrides,
        true,
        supervisor,
        app,
    )
}

// This internal adapter mirrors the typed launch boundary and keeps BIOS an
// explicit value rather than exposing a generic argv escape hatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_mame_with_source_and_bios(
    source: MameExecutableSource,
    machine: String,
    software: Option<String>,
    bios: Option<String>,
    project_paths: Vec<ProjectPathRequest>,
    transient_launch_overrides: Option<LaunchPreferencesV1>,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    launch_mame_with_source_and_bios_policy(
        source,
        machine,
        software,
        bios,
        project_paths,
        transient_launch_overrides,
        false,
        supervisor,
        app,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_catalog_mame_with_source_and_bios(
    source: MameExecutableSource,
    machine: String,
    software: Option<String>,
    bios: Option<String>,
    project_paths: Vec<ProjectPathRequest>,
    transient_launch_overrides: Option<LaunchPreferencesV1>,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    launch_mame_with_source_and_bios_policy(
        source,
        machine,
        software,
        bios,
        project_paths,
        transient_launch_overrides,
        true,
        supervisor,
        app,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_mame_with_source_and_bios_policy(
    source: MameExecutableSource,
    machine: String,
    software: Option<String>,
    bios: Option<String>,
    project_paths: Vec<ProjectPathRequest>,
    transient_launch_overrides: Option<LaunchPreferencesV1>,
    require_current_audit: bool,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SessionSnapshot> {
    let settings = load_settings(&settings_path(&app)?)?;
    let effective_content_paths = effective_content_paths(&settings.content_paths);
    let audit_classification = if require_current_audit {
        current_machine_audit_classification(
            &app,
            &source,
            &machine,
            &effective_content_paths,
        )?
    } else {
        None
    };
    if require_current_audit {
        launch_after_audit_gate(&machine, audit_classification, || Ok(()))?;
    }
    let project_paths =
        append_effective_content_project_paths(project_paths, &effective_content_paths)?;
    let project_paths = append_bundled_runtime_project_paths(&app, &source, project_paths)?;
    let general_launch_preferences = settings.launch_preferences;
    let effective_config = EffectiveLaunchConfig {
        project_paths: project_paths
            .iter()
            .map(|project_path| EffectiveProjectPath {
                option: project_path.option.clone(),
                path: project_path.path.clone(),
            })
            .collect(),
    };
    let target = MameLaunchTarget {
        machine,
        software,
        bios,
        project_paths: project_paths
            .into_iter()
            .map(|project_path| ProjectPathArgument {
                option: project_path.option,
                path: PathBuf::from(project_path.path),
            })
            .collect(),
    };

    let launch_context = serde_json::json!({
        "machine": &target.machine,
        "software": &target.software,
        "bios": &target.bios,
        "auditState": audit_state_label(audit_classification, require_current_audit),
        "effectiveContentPaths": diagnostics::summarize_content_paths(&effective_content_paths)
    });

    // Validate identifiers and project-controlled paths before persisting an
    // attempt, so invalid frontend input never becomes durable user history.
    build_launch_argv(&target)?;
    diagnostics::record(
        "info",
        "mame.lifecycle",
        "MAME launch requested.",
        serde_json::json!({
            "machine": &target.machine,
            "software": &target.software,
            "bios": &target.bios,
            "effectiveProjectPaths": &effective_config.project_paths
        }),
    );
    let catalog_path = storage::catalog_path(&app)?;
    let launch_preferences = machine_settings::effective_launch_preferences(
        &catalog_path,
        &general_launch_preferences,
        &target.machine,
        transient_launch_overrides.as_ref(),
    )?;
    let history_id =
        history::begin_launch_history(&app, &target.machine, target.software.as_deref())?;

    let app_for_events = app.clone();
    let event_sink: EventSink = Arc::new(move |name, event| {
        diagnostics::record(
            "info",
            "mame.lifecycle",
            name,
            serde_json::json!({
                "sessionId": &event.session.session_id,
                "machine": &event.session.machine,
                "software": &event.session.software,
                "state": event.session.state,
                "exitCode": event.session.exit_code,
                "forcedTermination": event.session.forced_termination
            }),
        );
        let external_name = external_session_lifecycle_event(name)
            .ok_or_else(|| format!("unsupported session lifecycle event: {name}"))?;
        app_for_events
            .emit(external_name, event)
            .map_err(|error| error.to_string())
    });

    match supervisor.launch_with_preferences(
        source,
        target,
        effective_config,
        launch_preferences,
        event_sink,
    ) {
        Ok(mut session) => {
            let app_for_pause_events = app.clone();
            let pause_session_id = session.session_id.clone();
            let pause_sink: control::PauseStateSink = Arc::new(move |paused| {
                let event_name = if paused {
                    SESSION_PAUSED_EVENT
                } else {
                    SESSION_RESUMED_EVENT
                };
                diagnostics::record(
                    "info",
                    "mame.lifecycle",
                    event_name,
                    serde_json::json!({
                        "sessionId": &pause_session_id,
                        "paused": paused
                    }),
                );
                let _ = app_for_pause_events.emit(
                    event_name,
                    SessionPauseEventV1 {
                        schema_version: 1,
                        session_id: pause_session_id.clone(),
                        paused,
                    },
                );
            });
            if let Err(error) = control::register_pause_state_sink(&session.session_id, pause_sink)
            {
                let _ = supervisor.stop_with_protocol_exit(&session.session_id);
                let _ = history::finish_launch_history(&app, history_id, false);
                return Err(error);
            }

            if let Err(history_error) = history::finish_launch_history(&app, history_id, true) {
                let warning = format!("PLAY_HISTORY_FINALIZE_FAILED: {}", history_error.message);
                session.diagnostic_error = Some(match session.diagnostic_error.take() {
                    Some(existing) => format!("{existing}; {warning}"),
                    None => warning,
                });
            }
            Ok(session)
        }
        Err(mut launch_error) => {
            let runtime_details = launch_error.details.clone();
            launch_error.details = serde_json::json!({
                "launchContext": launch_context,
                "runtime": runtime_details
            });
            if let Err(history_error) = history::finish_launch_history(&app, history_id, false) {
                launch_error.details = serde_json::json!({
                    "launch": launch_error.details,
                    "historyFinalization": {
                        "code": history_error.code,
                        "message": history_error.message,
                        "details": history_error.details
                    }
                });
            }
            Err(launch_error)
        }
    }
}

fn audit_state_label(
    classification: Option<MameAuditClassification>,
    required: bool,
) -> &'static str {
    if !required {
        return "notRequired";
    }
    match classification {
        Some(MameAuditClassification::Complete) => "available",
        Some(MameAuditClassification::BestAvailable) => "bestAvailable",
        Some(MameAuditClassification::MissingRequired) => "missingRequired",
        Some(MameAuditClassification::Incorrect) => "incorrect",
        Some(MameAuditClassification::MixedFailure) => "mixedFailure",
        Some(MameAuditClassification::Unknown) | None => "unknownOrStale",
    }
}

fn current_machine_audit_classification(
    app: &AppHandle,
    source: &MameExecutableSource,
    machine: &str,
    effective_content_paths: &EffectiveContentPaths,
) -> AppResult<Option<MameAuditClassification>> {
    let catalog_path = storage::catalog_path(app)?;
    let identity = inspect_executable(source.clone())?;
    Ok(load_current_machine_audit_result(
        &catalog_path,
        machine,
        &identity,
        effective_content_paths,
    )?
    .map(|stored| stored.result.classification))
}

fn launch_after_audit_gate<T>(
    machine: &str,
    classification: Option<MameAuditClassification>,
    launch: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    match classification {
        Some(MameAuditClassification::Complete | MameAuditClassification::BestAvailable) => {
            launch()
        }
        Some(
            MameAuditClassification::MissingRequired
            | MameAuditClassification::Incorrect
            | MameAuditClassification::MixedFailure,
        ) => Err(AppError::new(
            "MAME_CONTENT_UNAVAILABLE",
            "Required ROM/content is missing or incorrect. Configure content paths, fix the content, and run an audit before starting.",
        )
        .with_details(serde_json::json!({
            "machine": machine,
            "availability": "missingContent"
        }))),
        Some(MameAuditClassification::Unknown) | None => Err(AppError::new(
            "MAME_CONTENT_AUDIT_REQUIRED",
            "This machine has not been verified against the current content paths. Run an audit before starting.",
        )
        .with_details(serde_json::json!({
            "machine": machine,
            "availability": "notAudited"
        }))),
    }
}

fn append_effective_content_project_paths(
    mut project_paths: Vec<ProjectPathRequest>,
    effective_content_paths: &EffectiveContentPaths,
) -> AppResult<Vec<ProjectPathRequest>> {
    let media_paths = effective_content_paths.media_search_paths();
    let media_path = compose_mame_path_list(media_paths.into_iter().map(|path| path.as_path()))?;
    let Some(media_path) = media_path else {
        return Ok(project_paths);
    };

    project_paths.retain(|project_path| project_path.option != "rompath");
    project_paths.push(ProjectPathRequest {
        option: "rompath".to_owned(),

        path: media_path.to_string_lossy().into_owned(),
    });
    Ok(project_paths)
}


fn append_bundled_runtime_project_paths(
    app: &AppHandle,
    source: &MameExecutableSource,
    mut project_paths: Vec<ProjectPathRequest>,
) -> AppResult<Vec<ProjectPathRequest>> {
    if source.kind() != MameExecutableSourceKind::Bundled {
        return Ok(project_paths);
    }

    let resource_dir = app.path().resource_dir().map_err(|error| {
        AppError::new(
            "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE",
            "The application resource directory could not be resolved.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    let layout = BundledRuntimeLayout::from_resource_dir(&resource_dir);
    layout.validate()?;

    let user_state_root = app
        .path()
        .app_data_dir()
        .map(|path| path.join(BUNDLED_MAME_STATE_DIR))
        .map_err(|error| {
            AppError::new(
                "MAME_USER_STATE_ROOT_UNAVAILABLE",
                "The platform application data directory is unavailable for MAME user state.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    let bundled_paths = bundled_runtime_project_paths(&layout, &user_state_root);
    ensure_user_state_directories(&bundled_paths)?;
    project_paths.extend(bundled_paths);
    Ok(project_paths)
}

fn bundled_runtime_project_paths(
    layout: &BundledRuntimeLayout,
    user_state_root: &Path,
) -> Vec<ProjectPathRequest> {
    let mut paths = vec![
        project_path("hashpath", layout.hash_dir.clone()),
        project_path("bgfx_path", layout.bgfx_dir.clone()),
    ];
    paths.extend(
        USER_WRITABLE_MAME_PATHS
            .iter()
            .map(|(option, directory)| project_path(option, user_state_root.join(directory))),
    );
    paths
}

fn ensure_user_state_directories(paths: &[ProjectPathRequest]) -> AppResult<()> {
    for path in paths
        .iter()
        .filter(|path| is_user_writable_mame_path(&path.option))
    {
        fs::create_dir_all(&path.path).map_err(|error| {
            AppError::new(
                "MAME_USER_STATE_DIRECTORY_CREATE_FAILED",
                "A MAME user-writable state directory could not be created.",
            )
            .with_details(serde_json::json!({
                "option": path.option,
                "path": path.path,
                "cause": error.to_string()
            }))
        })?;
    }
    Ok(())
}

fn is_user_writable_mame_path(option: &str) -> bool {
    USER_WRITABLE_MAME_PATHS
        .iter()
        .any(|(known_option, _)| *known_option == option)
}

fn project_path(option: &str, path: PathBuf) -> ProjectPathRequest {
    ProjectPathRequest {
        option: option.to_owned(),
        path: path.to_string_lossy().into_owned(),
    }
}

#[tauri::command]
pub fn get_mame_session(
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<Option<SessionSnapshot>> {
    supervisor.current_session()
}

#[tauri::command]
pub fn pause_mame(request: PauseMameRequest) -> AppResult<PauseMameResult> {
    set_mame_paused(request.session_id, true)
}

#[tauri::command]
pub fn resume_mame(request: PauseMameRequest) -> AppResult<PauseMameResult> {
    set_mame_paused(request.session_id, false)
}

#[tauri::command]
pub fn reset_mame(request: ResetMameRequest) -> AppResult<ResetMameResult> {
    control::reset_session_soft(&request.session_id)?;
    Ok(ResetMameResult {
        schema_version: 1,
        session_id: request.session_id,
        kind: ResetKind::Soft,
    })
}

#[tauri::command]
pub fn load_mame_state(
    request: LoadMameStateRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<LoadMameStateResult> {
    load_state::load_mame_state_impl(request, supervisor, app)
}

#[tauri::command]
pub fn set_mame_mute(request: SetMameMuteRequest) -> AppResult<SetMameMuteResult> {
    let (ui_muted, effective_muted) =
        control::set_session_ui_mute(&request.session_id, request.muted)?;
    Ok(SetMameMuteResult {
        schema_version: 1,
        session_id: request.session_id,
        ui_muted,
        effective_muted,
    })
}

fn set_mame_paused(session_id: String, paused: bool) -> AppResult<PauseMameResult> {
    let observed = control::set_session_paused(&session_id, paused)?;
    Ok(PauseMameResult {
        schema_version: 1,
        session_id,
        paused: observed,
    })
}

#[tauri::command]
pub fn stop_mame(
    request: StopMameRequest,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<StopSessionResult> {
    diagnostics::record(
        "info",
        "mame.lifecycle",
        "MAME stop requested.",
        serde_json::json!({ "sessionId": &request.session_id }),
    );
    supervisor.stop_with_protocol_exit(&request.session_id)
}

fn executable_source(request: &MameExecutableRequest) -> MameExecutableSource {
    match request.source {
        MameExecutableSelectionKind::External => MameExecutableSource::external(&request.path),
        MameExecutableSelectionKind::DevelopmentTree => {
            MameExecutableSource::development_tree(&request.path)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        append_effective_content_project_paths, bundled_runtime_project_paths, executable_source,
        launch_after_audit_gate, MameExecutableRequest, MameExecutableSelectionKind,
    };
    use crate::{
        bundled_runtime::BundledRuntimeLayout,
        config::{ContentPathsV1, PlatformPath},
        content_paths::effective_content_paths,
        errors::AppResult,
        mame::{MameAuditClassification, MameExecutableSourceKind, MameExecutableTrust},
    };

    #[test]
    fn frontend_selectable_sources_cannot_self_assert_bundled_trust() {
        let external = executable_source(&MameExecutableRequest {
            source: MameExecutableSelectionKind::External,
            path: "/tmp/mame".to_owned(),
        });
        assert_eq!(external.kind(), MameExecutableSourceKind::External);
        assert_eq!(external.trust(), MameExecutableTrust::UserConfigured);

        let development = executable_source(&MameExecutableRequest {
            source: MameExecutableSelectionKind::DevelopmentTree,
            path: "/tmp/mame".to_owned(),
        });
        assert_eq!(
            development.kind(),
            MameExecutableSourceKind::DevelopmentTree
        );
        assert_eq!(development.trust(), MameExecutableTrust::Development);
    }

    #[test]
    fn bundled_launch_paths_pin_package_resources_and_user_writable_state() {
        let resource_dir = PathBuf::from("/package/resources");
        let state_root = PathBuf::from("/user/data/mame-state");
        let layout = BundledRuntimeLayout::from_resource_dir(&resource_dir);
        let paths = bundled_runtime_project_paths(&layout, &state_root);

        assert_eq!(option_path(&paths, "hashpath"), layout.hash_dir);
        assert_eq!(option_path(&paths, "bgfx_path"), layout.bgfx_dir);
        assert_eq!(option_path(&paths, "inipath"), state_root.join("ini"));
        assert_eq!(option_path(&paths, "cfg_directory"), state_root.join("cfg"));
        assert_eq!(
            option_path(&paths, "nvram_directory"),
            state_root.join("nvram")
        );
        assert_eq!(
            option_path(&paths, "snapshot_directory"),
            state_root.join("snap")
        );
        assert!(paths.iter().all(|path| path.option != "rompath"));
    }

    #[test]
    fn effective_content_paths_append_authoritative_rompath() {
        let rom_a = PathBuf::from("/rom-a");
        let rom_b = PathBuf::from("/rom-b");
        let configured = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(&rom_a), PlatformPath::new(&rom_b)],
            software_paths: Vec::new(),
            chd_paths: Vec::new(),
        };
        let effective = effective_content_paths(&configured);
        let paths = append_effective_content_project_paths(
            vec![super::ProjectPathRequest {
                option: "rompath".to_owned(),
                path: "/stale-rom".to_owned(),
            }],
            &effective,
        )
        .expect("append effective paths");

        let expected = format!("{};{}", rom_a.display(), rom_b.display());
        assert_eq!(option_path(&paths, "rompath"), PathBuf::from(expected));
        assert_eq!(
            paths.iter().filter(|path| path.option == "rompath").count(),
            1
        );
    }

    #[test]
    fn gated_catalog_launch_never_reaches_spawn_closure_for_unknown_or_missing_content() {
        for classification in [
            None,
            Some(MameAuditClassification::Unknown),
            Some(MameAuditClassification::MissingRequired),
            Some(MameAuditClassification::Incorrect),
            Some(MameAuditClassification::MixedFailure),
        ] {
            let mut spawned = false;
            let result: AppResult<()> = launch_after_audit_gate("pacman", classification, || {
                spawned = true;
                Ok(())
            });
            assert!(result.is_err());
            assert!(!spawned, "gated launch must not invoke the spawn boundary");
        }
    }

    #[test]
    fn complete_and_best_available_audits_reach_spawn_closure() {
        for classification in [
            MameAuditClassification::Complete,
            MameAuditClassification::BestAvailable,
        ] {
            let mut spawned = false;
            launch_after_audit_gate("pacman", Some(classification), || {
                spawned = true;
                Ok(())
            })
            .expect("playable audit classification");
            assert!(spawned);
        }
    }

    #[test]
    fn catalog_gate_is_evaluated_before_session_spawn_in_source() {
        let source = include_str!("sessions.rs");
        let gate = source
            .find("launch_after_audit_gate(&machine, classification")
            .expect("catalog audit gate source");
        let spawn = source
            .find("supervisor.launch_with_preferences")
            .expect("session spawn source");
        assert!(gate < spawn, "catalog audit gate must run before session spawn");
    }

    fn option_path(paths: &[super::ProjectPathRequest], option: &str) -> PathBuf {
        Path::new(
            &paths
                .iter()
                .find(|path| path.option == option)
                .unwrap_or_else(|| panic!("missing {option}"))
                .path,
        )
        .to_path_buf()
    }
}
