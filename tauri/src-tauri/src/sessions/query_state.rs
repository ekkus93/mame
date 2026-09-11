use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::{AppError, AppResult};

use super::{control, SessionState, SessionSupervisor};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueryMameRuntimeStateRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueryMameRuntimeStateResult {
    pub schema_version: u32,
    pub session_id: String,
    pub running: bool,
    pub paused: bool,
    pub machine: String,
    pub software: Option<String>,
    pub ui_muted: bool,
    pub effective_muted: bool,
}

#[tauri::command]
pub fn query_mame_runtime_state(
    request: QueryMameRuntimeStateRequest,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<QueryMameRuntimeStateResult> {
    let session = supervisor
        .current_session()?
        .filter(|session| session.session_id == request.session_id)
        .ok_or_else(|| {
            AppError::new(
                "MAME_SESSION_NOT_FOUND",
                "The requested MAME session is not available.",
            )
            .with_details(serde_json::json!({ "sessionId": request.session_id }))
        })?;
    if session.state != SessionState::Running {
        return Err(AppError::new(
            "MAME_SESSION_NOT_RUNNING",
            "Runtime-state query requires a running MAME session.",
        )
        .with_details(serde_json::json!({
            "sessionId": session.session_id,
            "state": session.state
        })));
    }

    let observed = control::query_session_state(&session.session_id)?;
    if observed.machine != session.machine {
        return Err(AppError::new(
            "CONTROL_STATE_CONTEXT_MISMATCH",
            "MAME runtime state reported a machine identity that contradicts the supervised session.",
        )
        .with_details(serde_json::json!({
            "sessionId": session.session_id,
            "supervisedMachine": session.machine,
            "runtimeMachine": observed.machine
        })));
    }

    Ok(QueryMameRuntimeStateResult {
        schema_version: 1,
        session_id: session.session_id,
        running: observed.running,
        paused: observed.paused,
        machine: observed.machine,
        software: session.software,
        ui_muted: observed.ui_muted,
        effective_muted: observed.effective_muted,
    })
}
