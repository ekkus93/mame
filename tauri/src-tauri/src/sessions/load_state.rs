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

const STATE_HEADER_BYTES: usize = 32;
const STATE_MAGIC: &[u8; 8] = b"MAMESAVE";
const STATE_FORMAT_VERSION: u8 = 2;
const STATE_MACHINE_FIELD_START: usize = 0x0a;
const STATE_MACHINE_FIELD_END: usize = 0x1c;
const STATE_SIGNATURE_START: usize = 0x1c;
const STATE_SIGNATURE_END: usize = 0x20;
const MAX_SLOT_BYTES: usize = 32;
const COMPATIBILITY_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_POLL_INTERVAL: Duration = Duration::from_millis(50);
const PROBE_STABLE_POLLS: usize = 3;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadMameStateRequest {
    pub session_id: String,
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadMameStateResult {
    pub schema_version: u32,
    pub session_id: String,
    pub machine: String,
    pub software: Option<String>,
    pub slot: String,
    pub path: String,
    pub bytes: u64,
    pub loaded_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadMameStateFailedEventV1 {
    pub schema_version: u32,
    pub session_id: String,
    pub machine: String,
    pub software: Option<String>,
    pub slot: String,
    pub error: AppError,
}

pub(super) fn load_mame_state_impl(
    request: LoadMameStateRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<LoadMameStateResult> {
    validate_slot(&request.slot)?;
    let session = current_running_session(&supervisor, &request.session_id)?;

    let result = load_state_for_session(&app, &session, &request.slot);
    match result {
        Ok(result) => {
            if let Err(error) = app.emit("session.state_loaded", result.clone()) {
                return Err(AppError::new(
                    "LOAD_STATE_EVENT_EMIT_FAILED",
                    "The state was restored successfully, but its success event could not be emitted.",
                )
                .with_details(serde_json::json!({
                    "event": "session.state_loaded",
                    "cause": error.to_string(),
                    "loaded": result
                })));
            }
            Ok(result)
        }
        Err(mut error) => {
            let failed_event = LoadMameStateFailedEventV1 {
                schema_version: 1,
                session_id: session.session_id,
                machine: session.machine,
                software: session.software,
                slot: request.slot,
                error: error.clone(),
            };
            if let Err(emit_error) = app.emit("session.state_load_failed", failed_event) {
                let operation_details = std::mem::take(&mut error.details);
                error.details = serde_json::json!({
                    "operation": operation_details,
                    "eventEmission": {
                        "event": "session.state_load_failed",
                        "cause": emit_error.to_string()
                    }
                });
            }
            Err(error)
        }
    }
}

fn load_state_for_session(
    app: &AppHandle,
    session: &SessionSnapshot,
    slot: &str,
) -> AppResult<LoadMameStateResult> {
    let paths = load_state_paths(app, session, slot)?;
    let target = inspect_target_state(&paths.final_path, &session.machine)?;

    fs::create_dir_all(&paths.directory).map_err(|error| {
        AppError::new(
            "LOAD_STATE_DIRECTORY_CREATE_FAILED",
            "The application-owned save-state directory could not be prepared for loading.",
        )
        .with_details(serde_json::json!({
            "path": paths.directory,
            "cause": error.to_string()
        }))
    })?;

    remove_if_exists(&paths.probe)?;
    let probe_utf8 = path_to_protocol(&paths.probe, "compatibility probe")?;
    control::request_session_save(&session.session_id, probe_utf8, "load-probe")?;
    let probe = match wait_for_probe(&paths.probe, &session.machine, &session.session_id) {
        Ok(probe) => probe,
        Err(mut error) => {
            if let Err(cleanup_error) = remove_if_exists(&paths.probe) {
                let operation_details = std::mem::take(&mut error.details);
                error.details = serde_json::json!({
                    "operation": operation_details,
                    "probeCleanup": {
                        "code": cleanup_error.code,
                        "message": cleanup_error.message,
                        "details": cleanup_error.details
                    }
                });
            }
            return Err(error);
        }
    };
    remove_if_exists(&paths.probe).map_err(|error| {
        AppError::new(
            "LOAD_STATE_PROBE_CLEANUP_FAILED",
            "The compatibility probe succeeded, but its temporary state file could not be removed.",
        )
        .with_details(serde_json::json!({
            "cleanup": {
                "code": error.code,
                "message": error.message,
                "details": error.details
            }
        }))
    })?;

    ensure_structural_compatibility(target, probe, &session.machine, slot)?;

    let target_utf8 = path_to_protocol(&paths.final_path, "saved state")?;
    let completion_utf8 = path_to_protocol(&paths.completion, "load completion marker")?;
    control::request_session_load(&session.session_id, target_utf8, slot, completion_utf8)?;

    Ok(LoadMameStateResult {
        schema_version: 1,
        session_id: session.session_id.clone(),
        machine: session.machine.clone(),
        software: session.software.clone(),
        slot: slot.to_owned(),
        path: target_utf8.to_owned(),
        bytes: target.bytes,
        loaded_at_epoch_ms: epoch_millis()?,
    })
}

struct LoadStatePaths {
    directory: PathBuf,
    final_path: PathBuf,
    probe: PathBuf,
    completion: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StateHeader {
    bytes: u64,
    signature: u32,
}

fn ensure_structural_compatibility(
    target: StateHeader,
    probe: StateHeader,
    machine: &str,
    slot: &str,
) -> AppResult<()> {
    if target.signature == probe.signature {
        return Ok(());
    }

    Err(AppError::new(
        "LOAD_STATE_INCOMPATIBLE",
        "The saved state is structurally incompatible with the currently running machine configuration.",
    )
    .with_details(serde_json::json!({
        "machine": machine,
        "slot": slot,
        "savedSignature": format!("{:08x}", target.signature),
        "currentSignature": format!("{:08x}", probe.signature),
        "stateFilePreserved": true
    })))
}

fn load_state_paths(
    app: &AppHandle,
    session: &SessionSnapshot,
    slot: &str,
) -> AppResult<LoadStatePaths> {
    let context = session
        .software
        .as_deref()
        .map(|software| format!("software-{}", encode_component(software)))
        .unwrap_or_else(|| "machine-only".to_owned());
    let directory = save_state_root(app)?
        .join(format!("machine-{}", encode_component(&session.machine)))
        .join(context);
    let session_component = encode_component(&session.session_id);
    Ok(LoadStatePaths {
        final_path: directory.join(format!("{slot}.sta")),
        probe: directory.join(format!(".{slot}.{session_component}.load-probe.sta")),
        completion: directory.join(format!(".{slot}.{session_component}.load-complete.json")),
        directory,
    })
}

fn save_state_root(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("save-states").join("v1"))
        .map_err(|error| {
            AppError::new(
                "LOAD_STATE_ROOT_UNAVAILABLE",
                "The platform application data directory for save states is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
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
            "Load state requires a running MAME session.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "state": session.state
        })));
    }
    Ok(session)
}

fn inspect_target_state(path: &Path, machine: &str) -> AppResult<StateHeader> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::new(
                "LOAD_STATE_NOT_FOUND",
                "The requested logical save-state slot does not exist.",
            )
            .with_details(serde_json::json!({ "path": path }))
        } else {
            AppError::new(
                "LOAD_STATE_READ_FAILED",
                "The requested save-state file could not be inspected.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            }))
        }
    })?;
    inspect_state_header(path, metadata.len(), machine, false)
}

fn inspect_state_header(
    path: &Path,
    bytes: u64,
    machine: &str,
    probe: bool,
) -> AppResult<StateHeader> {
    if bytes <= STATE_HEADER_BYTES as u64 {
        return Err(state_header_error(
            probe,
            "The MAME state file is truncated or contains no state payload.",
            path,
        ));
    }

    let mut header = [0_u8; STATE_HEADER_BYTES];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| {
            AppError::new(
                if probe {
                    "LOAD_STATE_PROBE_READ_FAILED"
                } else {
                    "LOAD_STATE_READ_FAILED"
                },
                "The MAME state-file header could not be read.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            }))
        })?;

    if &header[..STATE_MAGIC.len()] != STATE_MAGIC {
        return Err(state_header_error(
            probe,
            "The file does not have the MAME save-state signature.",
            path,
        ));
    }
    if header[8] != STATE_FORMAT_VERSION {
        return Err(AppError::new(
            if probe {
                "LOAD_STATE_PROBE_INVALID"
            } else {
                "LOAD_STATE_INCOMPATIBLE"
            },
            "The MAME save-state format version is not supported by the current runtime.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "observedVersion": header[8],
            "expectedVersion": STATE_FORMAT_VERSION
        })));
    }

    let machine_field = &header[STATE_MACHINE_FIELD_START..STATE_MACHINE_FIELD_END];
    let end = machine_field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(machine_field.len());
    if &machine_field[..end] != machine.as_bytes() {
        return Err(AppError::new(
            if probe {
                "LOAD_STATE_PROBE_INVALID"
            } else {
                "LOAD_STATE_CONTEXT_MISMATCH"
            },
            "The MAME state file belongs to a different machine context.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "expectedMachine": machine,
            "observedMachine": String::from_utf8_lossy(&machine_field[..end])
        })));
    }

    let signature = u32::from_le_bytes(
        header[STATE_SIGNATURE_START..STATE_SIGNATURE_END]
            .try_into()
            .expect("MAME state signature is four bytes"),
    );
    Ok(StateHeader { bytes, signature })
}

fn state_header_error(probe: bool, message: &str, path: &Path) -> AppError {
    AppError::new(
        if probe {
            "LOAD_STATE_PROBE_INVALID"
        } else {
            "LOAD_STATE_INVALID"
        },
        message,
    )
    .with_details(serde_json::json!({ "path": path }))
}

fn wait_for_probe(path: &Path, machine: &str, session_id: &str) -> AppResult<StateHeader> {
    let deadline = Instant::now() + COMPATIBILITY_PROBE_TIMEOUT;
    let mut previous_size = None;
    let mut stable_polls = 0_usize;
    loop {
        control::ensure_session_control_ready(session_id)?;
        match fs::metadata(path) {
            Ok(metadata) if metadata.len() > STATE_HEADER_BYTES as u64 => {
                if previous_size == Some(metadata.len()) {
                    stable_polls += 1;
                } else {
                    previous_size = Some(metadata.len());
                    stable_polls = 1;
                }
                if stable_polls >= PROBE_STABLE_POLLS {
                    return inspect_state_header(path, metadata.len(), machine, true);
                }
            }
            Ok(_) => {
                previous_size = None;
                stable_polls = 0;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                previous_size = None;
                stable_polls = 0;
            }
            Err(error) => {
                return Err(AppError::new(
                    "LOAD_STATE_PROBE_READ_FAILED",
                    "The compatibility probe state file could not be inspected.",
                )
                .with_details(serde_json::json!({
                    "path": path,
                    "cause": error.to_string()
                })))
            }
        }

        if Instant::now() >= deadline {
            let cleanup = remove_if_exists(path).err();
            return Err(AppError::new(
                "LOAD_STATE_PROBE_TIMEOUT",
                "MAME did not produce a compatibility-probe state before the deadline.",
            )
            .with_details(serde_json::json!({
                "sessionId": session_id,
                "timeoutMs": COMPATIBILITY_PROBE_TIMEOUT.as_millis(),
                "cleanupFailure": cleanup.map(|error| serde_json::json!({
                    "code": error.code,
                    "message": error.message,
                    "details": error.details
                }))
            })));
        }
        thread::sleep(PROBE_POLL_INTERVAL);
    }
}

fn path_to_protocol<'a>(path: &'a Path, purpose: &str) -> AppResult<&'a str> {
    path.to_str().ok_or_else(|| {
        AppError::new(
            "LOAD_STATE_PATH_UNREPRESENTABLE",
            "An application-owned load-state path cannot be represented in the UTF-8 runtime-control protocol.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "purpose": purpose
        }))
    })
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
        "LOAD_STATE_INVALID_SLOT",
        "Load-state slots must be 1-32 ASCII bytes, start with an alphanumeric character, and contain only alphanumerics, '_' or '-'.",
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

fn remove_if_exists(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::new(
            "LOAD_STATE_FILE_REMOVE_FAILED",
            "An application-owned load-state temporary file could not be removed.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string()
        }))),
    }
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

    use super::{
        encode_component, ensure_structural_compatibility, inspect_target_state, validate_slot,
        StateHeader, STATE_FORMAT_VERSION, STATE_HEADER_BYTES,
    };

    #[test]
    fn load_slots_reject_path_syntax() {
        for valid in ["1", "quick", "slot-01", "player_2"] {
            validate_slot(valid).expect("valid logical slot");
        }
        for invalid in ["", ".hidden", "../escape", "a/b", "a\\b", "white space"] {
            assert!(
                validate_slot(invalid).is_err(),
                "{invalid:?} must be rejected"
            );
        }
    }

    #[test]
    fn context_components_match_save_state_encoding() {
        assert_eq!(encode_component("pacman"), "7061636d616e");
        assert_eq!(encode_component("list:item"), "6c6973743a6974656d");
    }

    #[test]
    fn incompatible_structural_probe_preserves_target_state_bytes() {
        let root = tempdir().expect("tempdir");
        let path = root.path().join("quick.sta");
        let original = b"known saved state bytes that must survive a failed compatibility check";
        fs::write(&path, original).expect("state fixture");

        let error = ensure_structural_compatibility(
            StateHeader {
                bytes: original.len() as u64,
                signature: 0x1122_3344,
            },
            StateHeader {
                bytes: original.len() as u64,
                signature: 0x5566_7788,
            },
            "pacman",
            "quick",
        )
        .expect_err("mismatched structural signatures must fail");

        assert_eq!(error.code, "LOAD_STATE_INCOMPATIBLE");
        assert_eq!(
            error
                .details
                .get("stateFilePreserved")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(fs::read(&path).expect("preserved target"), original);
    }

    #[test]
    fn target_preflight_rejects_missing_wrong_machine_and_wrong_format() {
        let root = tempdir().expect("tempdir");
        let missing = root.path().join("missing.sta");
        assert_eq!(
            inspect_target_state(&missing, "pacman")
                .expect_err("missing")
                .code,
            "LOAD_STATE_NOT_FOUND"
        );

        let path = root.path().join("quick.sta");
        let mut header = [0_u8; STATE_HEADER_BYTES];
        header[..8].copy_from_slice(b"MAMESAVE");
        header[8] = STATE_FORMAT_VERSION;
        header[10..16].copy_from_slice(b"pacman");
        header[28..32].copy_from_slice(&0x11223344_u32.to_le_bytes());
        let mut file = fs::File::create(&path).expect("state fixture");
        file.write_all(&header).expect("header");
        file.write_all(&[1, 2, 3, 4]).expect("payload");
        file.sync_all().expect("sync fixture");

        let before_wrong_machine = fs::read(&path).expect("snapshot target");
        assert_eq!(
            inspect_target_state(&path, "galaga")
                .expect_err("wrong machine")
                .code,
            "LOAD_STATE_CONTEXT_MISMATCH"
        );
        assert_eq!(
            fs::read(&path).expect("target after wrong-machine preflight"),
            before_wrong_machine
        );

        header[8] = STATE_FORMAT_VERSION + 1;
        let mut file = fs::File::create(&path).expect("rewrite fixture");
        file.write_all(&header).expect("header");
        file.write_all(&[1, 2, 3, 4]).expect("payload");
        file.sync_all().expect("sync fixture");
        let before_wrong_format = fs::read(&path).expect("snapshot incompatible target");
        assert_eq!(
            inspect_target_state(&path, "pacman")
                .expect_err("wrong version")
                .code,
            "LOAD_STATE_INCOMPATIBLE"
        );
        assert_eq!(
            fs::read(&path).expect("target after wrong-format preflight"),
            before_wrong_format
        );
    }
}
