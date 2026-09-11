use std::{
    fmt::Write as _,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::errors::{AppError, AppResult};

use super::{control, SessionSnapshot, SessionState, SessionSupervisor};

const SAVE_STATE_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const SAVE_STATE_POLL_INTERVAL: Duration = Duration::from_millis(50);
const SAVE_STATE_STABLE_POLLS: usize = 3;
const SAVE_STATE_HEADER_BYTES: usize = 32;
const SAVE_STATE_MAGIC: &[u8; 8] = b"MAMESAVE";
const SAVE_STATE_MACHINE_FIELD_START: usize = 0x0a;
const SAVE_STATE_MACHINE_FIELD_END: usize = 0x1c;
const MAX_SLOT_BYTES: usize = 32;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveMameStateRequest {
    pub session_id: String,
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveMameStateResult {
    pub schema_version: u32,
    pub session_id: String,
    pub machine: String,
    pub software: Option<String>,
    pub slot: String,
    pub path: String,
    pub bytes: u64,
    pub saved_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveMameStateFailedEventV1 {
    pub schema_version: u32,
    pub session_id: String,
    pub machine: String,
    pub software: Option<String>,
    pub slot: String,
    pub error: AppError,
}

#[tauri::command]
pub fn save_mame_state(
    request: SaveMameStateRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<SaveMameStateResult> {
    validate_slot(&request.slot)?;
    let session = current_running_session(&supervisor, &request.session_id)?;

    let result = save_state_for_session(&app, &session, &request.slot);
    match result {
        Ok(result) => {
            if let Err(error) = app.emit("session.state_saved", result.clone()) {
                return Err(AppError::new(
                    "SAVE_STATE_EVENT_EMIT_FAILED",
                    "The save state was written successfully, but its success event could not be emitted.",
                )
                .with_details(serde_json::json!({
                    "event": "session.state_saved",
                    "cause": error.to_string(),
                    "saved": result
                })));
            }
            Ok(result)
        }
        Err(mut error) => {
            let failed_event = SaveMameStateFailedEventV1 {
                schema_version: 1,
                session_id: session.session_id,
                machine: session.machine,
                software: session.software,
                slot: request.slot,
                error: error.clone(),
            };
            if let Err(emit_error) = app.emit("session.state_save_failed", failed_event) {
                let operation_details = std::mem::take(&mut error.details);
                error.details = serde_json::json!({
                    "operation": operation_details,
                    "eventEmission": {
                        "event": "session.state_save_failed",
                        "cause": emit_error.to_string()
                    }
                });
            }
            Err(error)
        }
    }
}

fn current_running_session(
    supervisor: &SessionSupervisor,
    session_id: &str,
) -> AppResult<SessionSnapshot> {
    let session = supervisor
        .current_session()?
        .filter(|session| session.session_id == session_id)
        .ok_or_else(|| {
            AppError::new(
                "MAME_SESSION_NOT_FOUND",
                "The requested MAME session is not available.",
            )
            .with_details(serde_json::json!({ "sessionId": session_id }))
        })?;
    if session.state != SessionState::Running {
        return Err(AppError::new(
            "MAME_SESSION_NOT_RUNNING",
            "Save state requires a running MAME session.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "state": session.state
        })));
    }
    Ok(session)
}

fn save_state_for_session(
    app: &AppHandle,
    session: &SessionSnapshot,
    slot: &str,
) -> AppResult<SaveMameStateResult> {
    let paths = save_state_paths(app, session, slot)?;
    fs::create_dir_all(&paths.directory).map_err(|error| {
        AppError::new(
            "SAVE_STATE_DIRECTORY_CREATE_FAILED",
            "The application-owned save-state directory could not be created.",
        )
        .with_details(serde_json::json!({
            "path": paths.directory,
            "cause": error.to_string()
        }))
    })?;
    remove_if_exists(&paths.pending)?;

    let pending_utf8 = paths.pending.to_str().ok_or_else(|| {
        AppError::new(
            "SAVE_STATE_PATH_UNREPRESENTABLE",
            "The application save-state path cannot be represented in the UTF-8 runtime-control protocol.",
        )
        .with_details(serde_json::json!({ "path": paths.pending }))
    })?;
    control::request_session_save(&session.session_id, pending_utf8, slot)?;

    let bytes = wait_for_save_file(&paths.pending, &session.machine, &session.session_id)?;
    promote_verified_state(&paths.pending, &paths.final_path, &paths.backup)?;
    let final_path = paths.final_path.to_str().ok_or_else(|| {
        AppError::new(
            "SAVE_STATE_PATH_UNREPRESENTABLE",
            "The completed save-state path cannot be represented for the frontend.",
        )
        .with_details(serde_json::json!({ "path": paths.final_path }))
    })?;

    Ok(SaveMameStateResult {
        schema_version: 1,
        session_id: session.session_id.clone(),
        machine: session.machine.clone(),
        software: session.software.clone(),
        slot: slot.to_owned(),
        path: final_path.to_owned(),
        bytes,
        saved_at_epoch_ms: epoch_millis()?,
    })
}

struct SaveStatePaths {
    directory: PathBuf,
    pending: PathBuf,
    final_path: PathBuf,
    backup: PathBuf,
}

fn save_state_paths(
    app: &AppHandle,
    session: &SessionSnapshot,
    slot: &str,
) -> AppResult<SaveStatePaths> {
    let context = session
        .software
        .as_deref()
        .map(|software| format!("software-{}", encode_component(software)))
        .unwrap_or_else(|| "machine-only".to_owned());
    let directory = save_state_root(app)?
        .join(format!("machine-{}", encode_component(&session.machine)))
        .join(context);
    let session_component = encode_component(&session.session_id);
    Ok(SaveStatePaths {
        pending: directory.join(format!(".{slot}.{session_component}.pending.sta")),
        final_path: directory.join(format!("{slot}.sta")),
        backup: directory.join(format!(".{slot}.previous.sta")),
        directory,
    })
}

fn save_state_root(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("save-states").join("v1"))
        .map_err(|error| {
            AppError::new(
                "SAVE_STATE_ROOT_UNAVAILABLE",
                "The platform application data directory for save states is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn wait_for_save_file(path: &Path, machine: &str, session_id: &str) -> AppResult<u64> {
    let deadline = Instant::now() + SAVE_STATE_COMPLETION_TIMEOUT;
    let mut previous_size = None;
    let mut stable_polls = 0_usize;

    loop {
        control::ensure_session_control_ready(session_id)?;
        match validate_save_file(path, machine) {
            Ok(size) => {
                if previous_size == Some(size) {
                    stable_polls += 1;
                } else {
                    previous_size = Some(size);
                    stable_polls = 1;
                }
                if stable_polls >= SAVE_STATE_STABLE_POLLS {
                    return Ok(size);
                }
            }
            Err(error) if error.code == "SAVE_STATE_NOT_READY" => {
                previous_size = None;
                stable_polls = 0;
            }
            Err(error) => return Err(with_pending_cleanup(error, path)),
        }

        if Instant::now() >= deadline {
            return Err(with_pending_cleanup(
                AppError::new(
                    "SAVE_STATE_COMPLETION_TIMEOUT",
                    "MAME accepted the save-state request but no complete, valid state file appeared before the deadline.",
                )
                .with_details(serde_json::json!({
                    "sessionId": session_id,
                    "machine": machine,
                    "timeoutMs": SAVE_STATE_COMPLETION_TIMEOUT.as_millis()
                })),
                path,
            ));
        }
        thread::sleep(SAVE_STATE_POLL_INTERVAL);
    }
}

fn validate_save_file(path: &Path, machine: &str) -> AppResult<u64> {
    let metadata = fs::metadata(path).map_err(|error| {
        AppError::new(
            "SAVE_STATE_NOT_READY",
            "The pending MAME save-state file is not available yet.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    if metadata.len() <= SAVE_STATE_HEADER_BYTES as u64 {
        return Err(AppError::new(
            "SAVE_STATE_NOT_READY",
            "The pending MAME save-state file is incomplete.",
        ));
    }

    let mut header = [0_u8; SAVE_STATE_HEADER_BYTES];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| {
            AppError::new(
                "SAVE_STATE_NOT_READY",
                "The pending MAME save-state header is not readable yet.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    if &header[..SAVE_STATE_MAGIC.len()] != SAVE_STATE_MAGIC {
        return Err(AppError::new(
            "SAVE_STATE_INVALID_OUTPUT",
            "MAME produced a file without the expected save-state signature.",
        ));
    }
    let machine_field = &header[SAVE_STATE_MACHINE_FIELD_START..SAVE_STATE_MACHINE_FIELD_END];
    let end = machine_field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(machine_field.len());
    if &machine_field[..end] != machine.as_bytes() {
        return Err(AppError::new(
            "SAVE_STATE_CONTEXT_MISMATCH",
            "The produced MAME state file belongs to a different machine context.",
        )
        .with_details(serde_json::json!({
            "expectedMachine": machine,
            "observedMachine": String::from_utf8_lossy(&machine_field[..end])
        })));
    }
    Ok(metadata.len())
}

fn promote_verified_state(pending: &Path, final_path: &Path, backup: &Path) -> AppResult<()> {
    remove_if_exists(backup)?;
    let had_previous = final_path.exists();
    if had_previous {
        fs::rename(final_path, backup).map_err(|error| {
            AppError::new(
                "SAVE_STATE_PROMOTE_FAILED",
                "The previous save-state slot could not be staged for replacement.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    }

    if let Err(error) = fs::rename(pending, final_path) {
        let rollback_error = if had_previous {
            fs::rename(backup, final_path)
                .err()
                .map(|rollback| rollback.to_string())
        } else {
            None
        };
        return Err(AppError::new(
            "SAVE_STATE_PROMOTE_FAILED",
            "The verified MAME state file could not be promoted into its logical slot.",
        )
        .with_details(serde_json::json!({
            "cause": error.to_string(),
            "previousSlotRollbackFailure": rollback_error
        })));
    }
    if had_previous {
        remove_if_exists(backup).map_err(|error| {
            AppError::new(
                "SAVE_STATE_BACKUP_CLEANUP_FAILED",
                "The new save state was promoted, but the previous-slot backup could not be removed.",
            )
            .with_details(serde_json::json!({
                "finalStatePreserved": true,
                "cleanup": {
                    "code": error.code,
                    "message": error.message,
                    "details": error.details
                }
            }))
        })?;
    }
    Ok(())
}

fn with_pending_cleanup(mut error: AppError, path: &Path) -> AppError {
    if let Err(cleanup_error) = remove_if_exists(path) {
        let operation_details = std::mem::take(&mut error.details);
        error.details = serde_json::json!({
            "operation": operation_details,
            "pendingCleanup": {
                "code": cleanup_error.code,
                "message": cleanup_error.message,
                "details": cleanup_error.details
            }
        });
    }
    error
}

fn remove_if_exists(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::new(
            "SAVE_STATE_FILE_REMOVE_FAILED",
            "A stale application-owned save-state staging file could not be removed.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string()
        }))),
    }
}

fn validate_slot(slot: &str) -> AppResult<()> {
    let mut bytes = slot.bytes();
    let first = bytes.next().ok_or_else(|| invalid_slot(slot))?;
    if !first.is_ascii_alphanumeric()
        || slot.len() > MAX_SLOT_BYTES
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(invalid_slot(slot));
    }
    Ok(())
}

fn invalid_slot(slot: &str) -> AppError {
    AppError::new(
        "SAVE_STATE_INVALID_SLOT",
        "Save-state slots must be 1-32 ASCII bytes, start with an alphanumeric character, and contain only alphanumerics, '_' or '-'.",
    )
    .with_details(serde_json::json!({ "slot": slot }))
}

fn encode_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.as_bytes() {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn epoch_millis() -> AppResult<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                "SYSTEM_CLOCK_INVALID",
                "The system clock is earlier than the Unix epoch.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?
        .as_millis();
    u64::try_from(millis).map_err(|error| {
        AppError::new(
            "SYSTEM_CLOCK_INVALID",
            "The system timestamp does not fit the application time representation.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::tempdir;

    use super::{encode_component, validate_save_file, validate_slot, SAVE_STATE_HEADER_BYTES};

    #[test]
    fn slots_reject_path_syntax_and_accept_bounded_logical_names() {
        for valid in ["1", "quick", "slot-01", "player_2"] {
            validate_slot(valid).expect("valid logical slot");
        }
        for invalid in ["", ".hidden", "../escape", "a/b", "a\\b", "white space"] {
            assert!(
                validate_slot(invalid).is_err(),
                "{invalid:?} must be rejected"
            );
        }
        assert!(validate_slot(&"x".repeat(33)).is_err());
    }

    #[test]
    fn context_components_are_filesystem_safe_and_unambiguous() {
        assert_eq!(encode_component("pacman"), "7061636d616e");
        assert_eq!(encode_component("list:item"), "6c6973743a6974656d");
        assert!(!encode_component("../escape").contains('/'));
    }

    #[test]
    fn save_file_validation_binds_output_to_machine_identity() {
        let root = tempdir().expect("tempdir");
        let path = root.path().join("quick.sta");
        let mut header = [0_u8; SAVE_STATE_HEADER_BYTES];
        header[..8].copy_from_slice(b"MAMESAVE");
        header[8] = 2;
        header[10..16].copy_from_slice(b"pacman");
        let mut file = fs::File::create(&path).expect("state fixture");
        file.write_all(&header).expect("header");
        file.write_all(&[1, 2, 3, 4]).expect("payload");
        file.sync_all().expect("sync fixture");

        assert_eq!(
            validate_save_file(&path, "pacman").expect("valid state"),
            36
        );
        assert!(validate_save_file(&path, "galaga").is_err());
    }
}
