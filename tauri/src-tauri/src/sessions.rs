//! Supervised MAME process lifecycle and authoritative runtime session state.

mod supervisor;

use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::{
    errors::AppResult,
    mame::{
        inspect_executable, MameExecutableIdentity, MameExecutableSource, MameLaunchTarget,
        ProjectPathArgument,
    },
};

use supervisor::EventSink;
pub use supervisor::{
    EffectiveLaunchConfig, EffectiveProjectPath, SessionLifecycleEventV1, SessionSnapshot,
    SessionState, SessionSupervisor, StopSessionResult,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MameExecutableSelectionKind {
    Bundled,
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
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StopMameRequest {
    pub session_id: String,
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
    let effective_config = EffectiveLaunchConfig {
        project_paths: request
            .project_paths
            .iter()
            .map(|project_path| EffectiveProjectPath {
                option: project_path.option.clone(),
                path: project_path.path.clone(),
            })
            .collect(),
    };
    let target = MameLaunchTarget {
        machine: request.machine,
        software: request.software,
        project_paths: request
            .project_paths
            .into_iter()
            .map(|project_path| ProjectPathArgument {
                option: project_path.option,
                path: PathBuf::from(project_path.path),
            })
            .collect(),
    };

    let app_for_events = app.clone();
    let event_sink: EventSink = Arc::new(move |name, event| {
        app_for_events
            .emit(name, event)
            .map_err(|error| error.to_string())
    });

    supervisor.launch(source, target, effective_config, event_sink)
}

#[tauri::command]
pub fn get_mame_session(
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<Option<SessionSnapshot>> {
    supervisor.current_session()
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
        MameExecutableSelectionKind::Bundled => MameExecutableSource::bundled(&request.path),
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
    fn executable_selection_preserves_source_and_trust_boundary() {
        let bundled = executable_source(&MameExecutableRequest {
            source: MameExecutableSelectionKind::Bundled,
            path: "/tmp/mame".to_owned(),
        });
        assert_eq!(bundled.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(bundled.trust(), MameExecutableTrust::QualifiedBundled);

        let external = executable_source(&MameExecutableRequest {
            source: MameExecutableSelectionKind::External,
            path: "/tmp/mame".to_owned(),
        });
        assert_eq!(external.kind(), MameExecutableSourceKind::External);
        assert_eq!(external.trust(), MameExecutableTrust::UserConfigured);
    }
}
