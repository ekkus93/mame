//! Versioned project-owned persistence for MAME-style browser view state.
//!
//! This state is deliberately separate from MAME-owned INI/CFG files and from
//! WebView local storage. The browser can fail back to safe defaults when the
//! file is absent, while malformed/future-version state is surfaced rather than
//! silently overwritten.

use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tempfile::Builder;

use crate::{
    artwork::ArtworkKind,
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
    mame_ui::MameUiMachineFilterRequest,
};

const MAME_UI_STATE_SCHEMA_VERSION: u32 = 1;
const MAX_FILTER_VALUE_LENGTH: usize = 256;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MameUiPanelMode {
    #[default]
    Images,
    Info,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameUiStateV1 {
    pub schema_version: u32,
    pub last_machine: Option<String>,
    pub filter: MameUiMachineFilterRequest,
    pub filter_value: Option<String>,
    pub right_panel_mode: MameUiPanelMode,
    pub artwork_kind: ArtworkKind,
    pub software_right_panel_mode: MameUiPanelMode,
    pub software_artwork_kind: ArtworkKind,
}

impl Default for MameUiStateV1 {
    fn default() -> Self {
        Self {
            schema_version: MAME_UI_STATE_SCHEMA_VERSION,
            last_machine: None,
            filter: MameUiMachineFilterRequest::All,
            filter_value: None,
            right_panel_mode: MameUiPanelMode::Images,
            artwork_kind: ArtworkKind::Screenshot,
            software_right_panel_mode: MameUiPanelMode::Images,
            software_artwork_kind: ArtworkKind::Screenshot,
        }
    }
}

#[tauri::command]
pub fn get_mame_ui_state(app: AppHandle) -> AppResult<MameUiStateV1> {
    load_state(&state_path(&app)?)
}

#[tauri::command]
pub fn set_mame_ui_state(state: MameUiStateV1, app: AppHandle) -> AppResult<MameUiStateV1> {
    validate_state(&state)?;
    persist_state(&state_path(&app)?, &state)?;
    Ok(state)
}

fn state_path(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|root| root.join("mame-ui-state.json"))
        .map_err(|error| {
            AppError::new(
                "MAME_UI_STATE_ROOT_UNAVAILABLE",
                "The platform application configuration directory is unavailable for MAME UI state.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn load_state(path: &Path) -> AppResult<MameUiStateV1> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(MameUiStateV1::default()),
        Err(error) => {
            return Err(AppError::new(
                "MAME_UI_STATE_READ_FAILED",
                "The MAME UI state file could not be read.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    };

    let value: Value = serde_json::from_str(&contents).map_err(|error| {
        AppError::new(
            "MAME_UI_STATE_INVALID_JSON",
            "The MAME UI state file is not valid JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    let version = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::new(
                "MAME_UI_STATE_SCHEMA_VERSION_MISSING",
                "The MAME UI state file has no valid schema version.",
            )
        })?;
    if version != u64::from(MAME_UI_STATE_SCHEMA_VERSION) {
        return Err(AppError::new(
            "MAME_UI_STATE_SCHEMA_UNSUPPORTED",
            format!("MAME UI state schema version {version} is not supported."),
        )
        .with_details(serde_json::json!({
            "supportedVersion": MAME_UI_STATE_SCHEMA_VERSION,
            "foundVersion": version,
        })));
    }

    let state: MameUiStateV1 = serde_json::from_value(value).map_err(|error| {
        AppError::new(
            "MAME_UI_STATE_SCHEMA_INVALID",
            "The MAME UI state file does not match the supported schema.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    validate_state(&state)?;
    Ok(state)
}

fn persist_state(path: &Path, state: &MameUiStateV1) -> AppResult<()> {
    validate_state(state)?;
    if path.exists() {
        // A malformed/future document must not be silently destroyed by a write.
        load_state(path)?;
    }

    let mut encoded = serde_json::to_vec_pretty(state).map_err(|error| {
        AppError::new(
            "MAME_UI_STATE_SERIALIZE_FAILED",
            "The MAME UI state could not be serialized.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    encoded.push(b'\n');

    let parent = path.parent().ok_or_else(|| {
        AppError::new(
            "MAME_UI_STATE_PATH_INVALID",
            "The MAME UI state path has no parent directory.",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        AppError::new(
            "MAME_UI_STATE_DIRECTORY_CREATE_FAILED",
            "The MAME UI state directory could not be created.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    let mut temp = Builder::new()
        .prefix(".mame-tauri-ui-state-")
        .tempfile_in(parent)
        .map_err(state_write_error)?;
    temp.write_all(&encoded).map_err(state_write_error)?;
    temp.as_file().sync_all().map_err(state_write_error)?;
    let persisted = temp
        .persist(path)
        .map_err(|error| state_write_error(error.error))?;
    persisted.sync_all().map_err(state_write_error)?;

    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(state_write_error)?;

    Ok(())
}

fn validate_state(state: &MameUiStateV1) -> AppResult<()> {
    if state.schema_version != MAME_UI_STATE_SCHEMA_VERSION {
        return Err(AppError::new(
            "MAME_UI_STATE_SCHEMA_UNSUPPORTED",
            "Only the current MAME UI state schema can be persisted.",
        ));
    }
    if let Some(machine) = state.last_machine.as_deref() {
        validate_short_identifier("lastMachine", machine)?;
    }
    if let Some(value) = state.filter_value.as_deref() {
        if value.chars().count() > MAX_FILTER_VALUE_LENGTH {
            return Err(AppError::new(
                "MAME_UI_STATE_FILTER_VALUE_TOO_LONG",
                "The persisted MAME UI filter value exceeds the supported length.",
            )
            .with_details(serde_json::json!({ "maxLength": MAX_FILTER_VALUE_LENGTH })));
        }
    }
    Ok(())
}

fn state_write_error(error: std::io::Error) -> AppError {
    AppError::new(
        "MAME_UI_STATE_ATOMIC_WRITE_FAILED",
        "The MAME UI state could not be atomically replaced.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{artwork::ArtworkKind, mame_ui::MameUiMachineFilterRequest};

    use super::{load_state, persist_state, MameUiPanelMode, MameUiStateV1};

    #[test]
    fn missing_state_uses_safe_defaults() {
        let path = temp_root("missing").join("mame-ui-state.json");
        let state = load_state(&path).expect("missing state should use defaults");
        assert_eq!(state.filter, MameUiMachineFilterRequest::All);
        assert_eq!(state.right_panel_mode, MameUiPanelMode::Images);
        assert_eq!(state.artwork_kind, ArtworkKind::Screenshot);
        assert_eq!(state.last_machine, None);
    }

    #[test]
    fn state_round_trips_machine_filter_and_panel_selection() {
        let root = temp_root("round-trip");
        let path = root.join("mame-ui-state.json");
        let state = MameUiStateV1 {
            last_machine: Some("pacman".to_owned()),
            filter: MameUiMachineFilterRequest::Manufacturer,
            filter_value: Some("Namco".to_owned()),
            right_panel_mode: MameUiPanelMode::Info,
            artwork_kind: ArtworkKind::Marquee,
            ..MameUiStateV1::default()
        };
        persist_state(&path, &state).expect("persist state");
        assert_eq!(load_state(&path).expect("reload state"), state);
        fs::remove_dir_all(root).expect("remove temporary state root");
    }

    #[test]
    fn malformed_existing_state_is_not_overwritten() {
        let root = temp_root("malformed");
        fs::create_dir_all(&root).expect("create temporary state root");
        let path = root.join("mame-ui-state.json");
        fs::write(&path, "{broken").expect("write malformed state");
        let error = persist_state(&path, &MameUiStateV1::default())
            .expect_err("malformed state must block replacement");
        assert_eq!(error.code, "MAME_UI_STATE_INVALID_JSON");
        assert_eq!(fs::read_to_string(&path).expect("read state"), "{broken");
        fs::remove_dir_all(root).expect("remove temporary state root");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-mui-state-{label}-{nonce}"))
    }
}
