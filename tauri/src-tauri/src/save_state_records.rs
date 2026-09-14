//! Versioned save-state records and the MT-902 app-owned state index.
//!
//! Records are created only from authoritative supervised sessions plus a
//! completed MT-707 save result. The browser addresses records by database ID;
//! frontend input never selects an arbitrary host path for load or deletion.

use std::{
    fmt::Write as _,
    fs,
    path::{Component, Path, PathBuf},
    time::Duration,
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::{
    errors::{AppError, AppResult},
    mame::{validate_short_identifier, validate_software_identifier},
    sessions::{
        self, LoadMameStateRequest, LoadMameStateResult, SaveMameStateRequest, SaveMameStateResult,
        SessionSnapshot, SessionState, SessionSupervisor,
    },
};

pub const SAVE_STATE_RECORD_SCHEMA_VERSION: u32 = 1;
const SAVE_STATE_STORE_SCHEMA_VERSION: i64 = 1;
const DEFAULT_PAGE_LIMIT: u32 = 100;
const MAX_PAGE_LIMIT: u32 = 200;
const MAX_SLOT_BYTES: usize = 32;
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateMameIdentityV1 {
    pub version: String,
    pub build: Option<String>,
    pub raw_version_line: String,
}

/// Reserved metadata for a screenshot associated with a save-state record.
///
/// MT-901 does not capture screenshots. Keeping the field optional and
/// versioned lets a later task attach an opaque artwork identifier without
/// changing the meaning of existing records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateScreenshotMetadataV1 {
    pub schema_version: u32,
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateRecordV1 {
    pub schema_version: u32,
    pub machine: String,
    pub software: Option<String>,
    pub mame: SaveStateMameIdentityV1,
    pub saved_at_epoch_ms: u64,
    pub slot: String,
    pub path: String,
    pub bytes: u64,
    pub screenshot: Option<SaveStateScreenshotMetadataV1>,
}

impl SaveStateRecordV1 {
    /// Build a record from the exact session that produced a completed save.
    ///
    /// The consistency checks deliberately reject mismatched session/machine/
    /// software data instead of silently recording ambiguous provenance.
    pub fn from_completed_save(
        session: &SessionSnapshot,
        saved: &SaveMameStateResult,
    ) -> AppResult<Self> {
        validate_short_identifier("machine", &saved.machine)?;
        if let Some(software) = saved.software.as_deref() {
            validate_software_identifier(software)?;
        }
        validate_slot(&saved.slot)?;
        validate_record_path(&saved.path)?;

        if session.session_id != saved.session_id {
            return Err(record_mismatch(
                "sessionId",
                &session.session_id,
                &saved.session_id,
            ));
        }
        if session.machine != saved.machine {
            return Err(record_mismatch("machine", &session.machine, &saved.machine));
        }
        if session.software != saved.software {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_CONTEXT_MISMATCH",
                "The completed save-state software context does not match its producing session.",
            )
            .with_details(serde_json::json!({
                "field": "software",
                "sessionValue": session.software,
                "savedValue": saved.software,
            })));
        }
        if session.executable.version.trim().is_empty()
            || session.executable.raw_version_line.trim().is_empty()
        {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_MAME_IDENTITY_INVALID",
                "The producing MAME executable identity is incomplete.",
            ));
        }

        Ok(Self {
            schema_version: SAVE_STATE_RECORD_SCHEMA_VERSION,
            machine: saved.machine.clone(),
            software: saved.software.clone(),
            mame: SaveStateMameIdentityV1 {
                version: session.executable.version.clone(),
                build: session.executable.build.clone(),
                raw_version_line: session.executable.raw_version_line.clone(),
            },
            saved_at_epoch_ms: saved.saved_at_epoch_ms,
            slot: saved.slot.clone(),
            path: saved.path.clone(),
            bytes: saved.bytes,
            screenshot: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredSaveStateRecordV1 {
    pub id: i64,
    pub record: SaveStateRecordV1,
    pub file_present: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListSaveStateRecordsRequest {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateRecordPageV1 {
    pub schema_version: u32,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<StoredSaveStateRecordV1>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadKnownSaveStateRequest {
    pub session_id: String,
    pub record_id: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSaveStateRecordRequest {
    pub record_id: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSaveStateRecordResultV1 {
    pub schema_version: u32,
    pub record_id: i64,
    pub file_deleted: bool,
}

#[tauri::command]
pub fn save_known_state(
    request: SaveMameStateRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<StoredSaveStateRecordV1> {
    let session = current_running_session(&supervisor, &request.session_id)?;
    let saved = sessions::save_state::save_mame_state(request, supervisor, app.clone())?;
    let record = SaveStateRecordV1::from_completed_save(&session, &saved)?;
    let root = save_state_root(&app)?;
    authorize_record_path(&root, &record)?;

    let mut connection = open_store(&save_state_store_path(&app)?)?;
    let id = match upsert_record(&mut connection, &record) {
        Ok(id) => id,
        Err(mut error) => {
            let storage_details = std::mem::take(&mut error.details);
            error.details = serde_json::json!({
                "stateFilePreserved": true,
                "savedState": {
                    "machine": record.machine,
                    "software": record.software,
                    "slot": record.slot,
                    "path": record.path
                },
                "storage": storage_details
            });
            return Err(error);
        }
    };

    Ok(StoredSaveStateRecordV1 {
        id,
        file_present: authorized_state_file_present(&root, &record),
        record,
    })
}

#[tauri::command]
pub fn list_save_state_records(
    request: ListSaveStateRecordsRequest,
    app: AppHandle,
) -> AppResult<SaveStateRecordPageV1> {
    let limit = request
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let offset = request.offset.unwrap_or(0);
    let mut connection = open_store(&save_state_store_path(&app)?)?;
    let (total, records) = list_records(&mut connection, limit, offset)?;
    let root = save_state_root(&app)?;
    let items = records
        .into_iter()
        .map(|(id, record)| StoredSaveStateRecordV1 {
            id,
            file_present: authorized_state_file_present(&root, &record),
            record,
        })
        .collect();

    Ok(SaveStateRecordPageV1 {
        schema_version: 1,
        total,
        offset,
        limit,
        items,
    })
}

#[tauri::command]
pub fn load_known_save_state(
    request: LoadKnownSaveStateRequest,
    supervisor: State<'_, SessionSupervisor>,
    app: AppHandle,
) -> AppResult<LoadMameStateResult> {
    let mut connection = open_store(&save_state_store_path(&app)?)?;
    let record = load_record(&mut connection, request.record_id)?;
    let session = current_running_session(&supervisor, &request.session_id)?;
    ensure_record_matches_session(&record, &session)?;
    ensure_authorized_state_file(&save_state_root(&app)?, &record)?;

    // The record's MAME version is intentionally not treated as proof of
    // compatibility. The existing MT-708 load path performs a fresh structural
    // compatibility probe against the running session before restoration.
    sessions::load_mame_state(
        LoadMameStateRequest {
            session_id: request.session_id,
            slot: record.slot,
        },
        supervisor,
        app,
    )
}

#[tauri::command]
pub fn delete_save_state_record(
    request: DeleteSaveStateRecordRequest,
    app: AppHandle,
) -> AppResult<DeleteSaveStateRecordResultV1> {
    let store_path = save_state_store_path(&app)?;
    let mut connection = open_store(&store_path)?;
    let record = load_record(&mut connection, request.record_id)?;
    let expected_path = authorize_record_path(&save_state_root(&app)?, &record)?;

    let file_deleted = match fs::remove_file(&expected_path) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(AppError::new(
                "SAVE_STATE_DELETE_FILE_FAILED",
                "The application-owned save-state file could not be deleted.",
            )
            .with_details(serde_json::json!({
                "recordId": request.record_id,
                "cause": error.to_string()
            })))
        }
    };

    if let Err(mut error) = delete_record(&mut connection, request.record_id) {
        let storage_details = std::mem::take(&mut error.details);
        error.details = serde_json::json!({
            "recordId": request.record_id,
            "stateFileDeleted": file_deleted,
            "storage": storage_details
        });
        return Err(error);
    }

    Ok(DeleteSaveStateRecordResultV1 {
        schema_version: 1,
        record_id: request.record_id,
        file_deleted,
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
            "Save-state browser operations require a running MAME session.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "state": session.state
        })));
    }
    Ok(session)
}

fn ensure_record_matches_session(
    record: &SaveStateRecordV1,
    session: &SessionSnapshot,
) -> AppResult<()> {
    if record.machine != session.machine || record.software != session.software {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_CONTEXT_MISMATCH",
            "The selected save state belongs to a different machine or software context.",
        )
        .with_details(serde_json::json!({
            "recordMachine": record.machine,
            "recordSoftware": record.software,
            "sessionMachine": session.machine,
            "sessionSoftware": session.software
        })));
    }
    Ok(())
}

fn save_state_store_path(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("save-states").join("records.sqlite3"))
        .map_err(|error| {
            AppError::new(
                "SAVE_STATE_STORE_ROOT_UNAVAILABLE",
                "The platform application data directory for save-state records is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
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

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/save_state_store.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/save_state_validation.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/save_state_records_tests.rs"
));
