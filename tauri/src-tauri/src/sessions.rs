//! Supervised MAME process lifecycle and authoritative runtime session state.

mod control;
mod supervisor;

use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::{
    config::{load_settings, settings_path, LaunchPreferencesV1},
    errors::AppResult,
    history, machine_settings,
    mame::{
        build_launch_argv, inspect_executable, MameExecutableIdentity, MameExecutableSource,
        MameLaunchTarget, ProjectPathArgument,
    },
    storage,
};

use supervisor::EventSink;
pub use supervisor::{
    EffectiveLaunchConfig, EffectiveProjectPath, SessionLifecycleEventV1, SessionSnapshot,
    SessionState, SessionSupervisor, StopSessionResult,
};

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
    let general_launch_preferences = load_settings(&settings_path(&app)?)?.launch_preferences;
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
        project_paths: project_paths
            .into_iter()
            .map(|project_path| ProjectPathArgument {
                option: project_path.option,
                path: PathBuf::from(project_path.path),
            })
            .collect(),
    };

    // Validate identifiers and project-controlled paths before persisting an
    // attempt, so invalid frontend input never becomes durable user history.
    build_launch_argv(&target)?;
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
        app_for_events
            .emit(name, event)
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
                    "session.paused"
                } else {
                    "session.resumed"
                };
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
                let _ = supervisor.stop(&session.session_id);
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
    supervisor.stop(&request.session_id)
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
    use super::{executable_source, MameExecutableRequest, MameExecutableSelectionKind};
    use crate::mame::{MameExecutableSourceKind, MameExecutableTrust};

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
}
