//! Supervised MAME process lifecycle and authoritative runtime session state.

mod control {
    include!(concat!(env!("OUT_DIR"), "/runtime_control_mt710.rs"));
}
mod frame;
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
use tauri::{ipc::Response, AppHandle, Emitter, Manager, State};

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

pub use frame::FrameMetricsSnapshot;
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
pub struct GetMameGameFrameRequest {
    pub session_id: String,
    #[serde(default)]
    pub presented_sequence: Option<String>,
    #[serde(default)]
    pub presentation_duration_us: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetMameFrameMetricsRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameInputUpdate {
    pub token: String,
    pub value: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMameInputsRequest {
    pub session_id: String,
    pub updates: Vec<MameInputUpdate>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMameInputsResult {
    pub schema_version: u32,
    pub session_id: String,
    pub accepted: bool,
    pub update_count: usize,
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
pub fn get_mame_game_frame(
    request: GetMameGameFrameRequest,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<Response> {
    match (
        &request.presented_sequence,
        request.presentation_duration_us,
    ) {
        (Some(sequence), duration) => {
            let sequence = sequence.parse::<u64>().map_err(|error| {
                AppError::new(
                    "MAME_FRAME_PRESENTATION_ACK_INVALID",
                    "The gameplay frame presentation acknowledgement sequence is invalid.",
                )
                .with_details(serde_json::json!({
                    "sequence": sequence,
                    "cause": error.to_string()
                }))
            })?;
            supervisor.acknowledge_frame_presentation(&request.session_id, sequence, duration)?;
        }
        (None, Some(_)) => {
            return Err(AppError::new(
                "MAME_FRAME_PRESENTATION_ACK_INVALID",
                "A gameplay frame presentation duration requires a frame sequence.",
            )
            .with_details(serde_json::json!({ "sessionId": request.session_id })));
        }
        (None, None) => {}
    }

    let frame = supervisor.take_latest_frame(&request.session_id)?;
    Ok(Response::new(frame.to_client_bytes()?))
}

#[tauri::command]
pub fn get_mame_frame_metrics(
    request: GetMameFrameMetricsRequest,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<FrameMetricsSnapshot> {
    supervisor.frame_metrics(&request.session_id)
}

#[tauri::command]
pub fn set_mame_inputs(request: SetMameInputsRequest) -> AppResult<SetMameInputsResult> {
    let updates: Vec<_> = request
        .updates
        .iter()
        .map(|update| control::InputUpdate {
            token: update.token.clone(),
            value: update.value,
        })
        .collect();
    let accepted = control::send_session_inputs(&request.session_id, &updates)?;
    Ok(SetMameInputsResult {
        schema_version: 1,
        session_id: request.session_id,
        accepted,
        update_count: updates.len(),
    })
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
        current_machine_audit_classification(&app, &source, &machine, &effective_content_paths)?
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
