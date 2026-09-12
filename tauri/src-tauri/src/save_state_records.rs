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

fn open_store(path: &Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::new(
                "SAVE_STATE_STORE_DIRECTORY_CREATE_FAILED",
                "The save-state record directory could not be created.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    }
    let mut connection = Connection::open(path)
        .map_err(|error| database_error("SAVE_STATE_STORE_OPEN_FAILED", error))?;
    connection
        .busy_timeout(SQLITE_BUSY_TIMEOUT)
        .map_err(|error| database_error("SAVE_STATE_STORE_CONFIG_FAILED", error))?;
    configure_store(&mut connection)?;
    Ok(connection)
}

fn configure_store(connection: &mut Connection) -> AppResult<()> {
    connection
        .execute_batch("PRAGMA foreign_keys = ON;\nPRAGMA synchronous = NORMAL;")
        .map_err(|error| database_error("SAVE_STATE_STORE_CONFIG_FAILED", error))?;
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| database_error("SAVE_STATE_STORE_SCHEMA_READ_FAILED", error))?;
    if version > SAVE_STATE_STORE_SCHEMA_VERSION {
        return Err(AppError::new(
            "SAVE_STATE_STORE_SCHEMA_UNSUPPORTED",
            "The save-state record store is newer than this application supports.",
        )
        .with_details(serde_json::json!({
            "foundVersion": version,
            "supportedVersion": SAVE_STATE_STORE_SCHEMA_VERSION
        })));
    }
    if version == 0 {
        let transaction = connection
            .transaction()
            .map_err(|error| database_error("SAVE_STATE_STORE_MIGRATION_FAILED", error))?;
        transaction
            .execute_batch(CREATE_STORE_V1)
            .map_err(|error| database_error("SAVE_STATE_STORE_MIGRATION_FAILED", error))?;
        transaction
            .pragma_update(None, "user_version", SAVE_STATE_STORE_SCHEMA_VERSION)
            .map_err(|error| database_error("SAVE_STATE_STORE_MIGRATION_FAILED", error))?;
        transaction
            .commit()
            .map_err(|error| database_error("SAVE_STATE_STORE_MIGRATION_FAILED", error))?;
    }
    Ok(())
}

fn upsert_record(connection: &mut Connection, record: &SaveStateRecordV1) -> AppResult<i64> {
    validate_persisted_record(record)?;
    let screenshot_json = record
        .screenshot
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| {
            AppError::new(
                "SAVE_STATE_RECORD_SERIALIZE_FAILED",
                "Save-state screenshot metadata could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    let saved_at = i64::try_from(record.saved_at_epoch_ms)
        .map_err(|error| numeric_error("timestamp", error))?;
    let bytes = i64::try_from(record.bytes).map_err(|error| numeric_error("bytes", error))?;

    connection
        .execute(
            r#"
INSERT INTO save_state_records(
    schema_version, machine, software, mame_version, mame_build, mame_raw_version_line,
    saved_at_epoch_ms, slot, state_path, bytes, screenshot_json
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT(state_path) DO UPDATE SET
    schema_version = excluded.schema_version,
    machine = excluded.machine,
    software = excluded.software,
    mame_version = excluded.mame_version,
    mame_build = excluded.mame_build,
    mame_raw_version_line = excluded.mame_raw_version_line,
    saved_at_epoch_ms = excluded.saved_at_epoch_ms,
    slot = excluded.slot,
    bytes = excluded.bytes,
    screenshot_json = excluded.screenshot_json
"#,
            params![
                i64::from(record.schema_version),
                record.machine,
                record.software,
                record.mame.version,
                record.mame.build,
                record.mame.raw_version_line,
                saved_at,
                record.slot,
                record.path,
                bytes,
                screenshot_json
            ],
        )
        .map_err(|error| database_error("SAVE_STATE_RECORD_PERSIST_FAILED", error))?;

    connection
        .query_row(
            "SELECT id FROM save_state_records WHERE state_path = ?1",
            [&record.path],
            |row| row.get(0),
        )
        .map_err(|error| database_error("SAVE_STATE_RECORD_PERSIST_FAILED", error))
}

fn list_records(
    connection: &mut Connection,
    limit: u32,
    offset: u32,
) -> AppResult<(u64, Vec<(i64, SaveStateRecordV1)>)> {
    let total_i64: i64 = connection
        .query_row("SELECT COUNT(*) FROM save_state_records", [], |row| {
            row.get(0)
        })
        .map_err(|error| database_error("SAVE_STATE_RECORD_LIST_FAILED", error))?;
    let total = u64::try_from(total_i64).map_err(|error| numeric_error("total", error))?;

    let mut statement = connection
        .prepare(
            r#"
SELECT id, schema_version, machine, software, mame_version, mame_build,
       mame_raw_version_line, saved_at_epoch_ms, slot, state_path, bytes, screenshot_json
FROM save_state_records
ORDER BY saved_at_epoch_ms DESC, id DESC
LIMIT ?1 OFFSET ?2
"#,
        )
        .map_err(|error| database_error("SAVE_STATE_RECORD_LIST_FAILED", error))?;
    let rows = statement
        .query_map(params![i64::from(limit), i64::from(offset)], row_to_record)
        .map_err(|error| database_error("SAVE_STATE_RECORD_LIST_FAILED", error))?;
    let mut records = Vec::new();
    for row in rows {
        let record = row.map_err(|error| database_error("SAVE_STATE_RECORD_LIST_FAILED", error))?;
        validate_persisted_record(&record.1)?;
        records.push(record);
    }
    Ok((total, records))
}

fn load_record(connection: &mut Connection, record_id: i64) -> AppResult<SaveStateRecordV1> {
    let record = connection
        .query_row(
            r#"
SELECT id, schema_version, machine, software, mame_version, mame_build,
       mame_raw_version_line, saved_at_epoch_ms, slot, state_path, bytes, screenshot_json
FROM save_state_records
WHERE id = ?1
"#,
            [record_id],
            row_to_record,
        )
        .optional()
        .map_err(|error| database_error("SAVE_STATE_RECORD_READ_FAILED", error))?
        .ok_or_else(|| {
            AppError::new(
                "SAVE_STATE_RECORD_NOT_FOUND",
                "The selected save-state record no longer exists.",
            )
            .with_details(serde_json::json!({ "recordId": record_id }))
        })?
        .1;
    validate_persisted_record(&record)?;
    Ok(record)
}

fn delete_record(connection: &mut Connection, record_id: i64) -> AppResult<()> {
    let changed = connection
        .execute("DELETE FROM save_state_records WHERE id = ?1", [record_id])
        .map_err(|error| database_error("SAVE_STATE_RECORD_DELETE_FAILED", error))?;
    if changed != 1 {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_NOT_FOUND",
            "The selected save-state record no longer exists.",
        )
        .with_details(serde_json::json!({ "recordId": record_id })));
    }
    Ok(())
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<(i64, SaveStateRecordV1)> {
    let id: i64 = row.get(0)?;
    let schema_version_i64: i64 = row.get(1)?;
    let saved_at_i64: i64 = row.get(7)?;
    let bytes_i64: i64 = row.get(10)?;
    let screenshot_json: Option<String> = row.get(11)?;
    let screenshot = screenshot_json
        .map(|value| {
            serde_json::from_str(&value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .transpose()?;
    let schema_version = u32::try_from(schema_version_i64).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;
    let saved_at_epoch_ms = u64::try_from(saved_at_i64).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            7,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;
    let bytes = u64::try_from(bytes_i64).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            10,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;

    Ok((
        id,
        SaveStateRecordV1 {
            schema_version,
            machine: row.get(2)?,
            software: row.get(3)?,
            mame: SaveStateMameIdentityV1 {
                version: row.get(4)?,
                build: row.get(5)?,
                raw_version_line: row.get(6)?,
            },
            saved_at_epoch_ms,
            slot: row.get(8)?,
            path: row.get(9)?,
            bytes,
            screenshot,
        },
    ))
}

fn validate_persisted_record(record: &SaveStateRecordV1) -> AppResult<()> {
    if record.schema_version != SAVE_STATE_RECORD_SCHEMA_VERSION {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_SCHEMA_UNSUPPORTED",
            "A save-state record uses an unsupported schema version.",
        )
        .with_details(serde_json::json!({
            "foundVersion": record.schema_version,
            "supportedVersion": SAVE_STATE_RECORD_SCHEMA_VERSION
        })));
    }
    validate_short_identifier("machine", &record.machine)?;
    if let Some(software) = record.software.as_deref() {
        validate_software_identifier(software)?;
    }
    validate_slot(&record.slot)?;
    validate_record_path(&record.path)?;
    if record.mame.version.trim().is_empty() || record.mame.raw_version_line.trim().is_empty() {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_MAME_IDENTITY_INVALID",
            "A save-state record has incomplete MAME identity provenance.",
        ));
    }
    Ok(())
}

fn authorize_record_path(root: &Path, record: &SaveStateRecordV1) -> AppResult<PathBuf> {
    validate_persisted_record(record)?;
    let expected = expected_state_path(root, record);
    if Path::new(&record.path) != expected {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_PATH_UNAUTHORIZED",
            "The save-state record path is not the application-owned path for its machine, software, and slot.",
        ));
    }

    if expected.exists() {
        let canonical_root = fs::canonicalize(root).map_err(|error| {
            AppError::new(
                "SAVE_STATE_ROOT_UNAVAILABLE",
                "The application-owned save-state root could not be resolved.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        let canonical_state = fs::canonicalize(&expected).map_err(|error| {
            AppError::new(
                "SAVE_STATE_RECORD_FILE_UNAVAILABLE",
                "The selected save-state file could not be resolved.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if !canonical_state.starts_with(&canonical_root) {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_PATH_UNAUTHORIZED",
                "The selected save-state file resolves outside the application-owned save-state root.",
            ));
        }
    }
    Ok(expected)
}

fn ensure_authorized_state_file(root: &Path, record: &SaveStateRecordV1) -> AppResult<PathBuf> {
    let path = authorize_record_path(root, record)?;
    if !path.is_file() {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_FILE_MISSING",
            "The selected save-state file is missing from application storage.",
        ));
    }
    Ok(path)
}

fn authorized_state_file_present(root: &Path, record: &SaveStateRecordV1) -> bool {
    ensure_authorized_state_file(root, record).is_ok()
}

fn expected_state_path(root: &Path, record: &SaveStateRecordV1) -> PathBuf {
    let context = record
        .software
        .as_deref()
        .map(|software| format!("software-{}", encode_component(software)))
        .unwrap_or_else(|| "machine-only".to_owned());
    root.join(format!("machine-{}", encode_component(&record.machine)))
        .join(context)
        .join(format!("{}.sta", record.slot))
}

fn validate_record_path(path: &str) -> AppResult<()> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_PATH_INVALID",
            "A save-state record path must be an absolute application-owned path.",
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_PATH_INVALID",
            "A save-state record path must not contain parent-directory traversal.",
        ));
    }
    Ok(())
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

fn record_mismatch(field: &str, session_value: &str, saved_value: &str) -> AppError {
    AppError::new(
        "SAVE_STATE_RECORD_CONTEXT_MISMATCH",
        "The completed save-state context does not match its producing session.",
    )
    .with_details(serde_json::json!({
        "field": field,
        "sessionValue": session_value,
        "savedValue": saved_value,
    }))
}

fn numeric_error(field: &str, error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "SAVE_STATE_RECORD_NUMERIC_RANGE_INVALID",
        "A save-state record numeric value cannot be represented by the local index.",
    )
    .with_details(serde_json::json!({ "field": field, "cause": error.to_string() }))
}

fn database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The save-state record database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

const CREATE_STORE_V1: &str = r#"
CREATE TABLE save_state_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    machine TEXT NOT NULL,
    software TEXT,
    mame_version TEXT NOT NULL,
    mame_build TEXT,
    mame_raw_version_line TEXT NOT NULL,
    saved_at_epoch_ms INTEGER NOT NULL CHECK (saved_at_epoch_ms >= 0),
    slot TEXT NOT NULL,
    state_path TEXT NOT NULL UNIQUE,
    bytes INTEGER NOT NULL CHECK (bytes >= 0),
    screenshot_json TEXT
);
CREATE INDEX save_state_records_recent
    ON save_state_records(saved_at_epoch_ms DESC, id DESC);
CREATE INDEX save_state_records_context
    ON save_state_records(machine, software, slot);
"#;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::Connection;

    use crate::{
        mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
        sessions::{EffectiveLaunchConfig, SaveMameStateResult, SessionSnapshot, SessionState},
    };

    use super::{
        authorize_record_path, configure_store, delete_record, list_records, load_record,
        upsert_record, SaveStateRecordV1, SAVE_STATE_RECORD_SCHEMA_VERSION,
    };

    fn session() -> SessionSnapshot {
        SessionSnapshot {
            schema_version: 1,
            session_id: "session-7".to_owned(),
            state: SessionState::Running,
            machine: "pacman".to_owned(),
            software: Some("nes:mario".to_owned()),
            executable: MameExecutableIdentity {
                source: MameExecutableSourceKind::External,
                trust: MameExecutableTrust::UserConfigured,
                path: "/opt/mame/mame".to_owned(),
                version: "0.281".to_owned(),
                build: Some("mame0281".to_owned()),
                raw_version_line: "MAME v0.281 (mame0281)".to_owned(),
            },
            effective_argv: vec!["pacman".to_owned()],
            effective_config: EffectiveLaunchConfig {
                project_paths: Vec::new(),
            },
            created_at_epoch_ms: 10,
            started_at_epoch_ms: Some(11),
            ended_at_epoch_ms: None,
            pid: Some(1234),
            exit_code: None,
            termination_signal: None,
            forced_termination: false,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            diagnostic_error: None,
        }
    }

    fn saved() -> SaveMameStateResult {
        SaveMameStateResult {
            schema_version: 1,
            session_id: "session-7".to_owned(),
            machine: "pacman".to_owned(),
            software: Some("nes:mario".to_owned()),
            slot: "quick".to_owned(),
            path: "/tmp/save-states/v1/machine-7061636d616e/software-6e65733a6d6172696f/quick.sta"
                .to_owned(),
            bytes: 4096,
            saved_at_epoch_ms: 99,
        }
    }

    fn record() -> SaveStateRecordV1 {
        SaveStateRecordV1::from_completed_save(&session(), &saved()).expect("record")
    }

    fn store() -> Connection {
        let mut connection = Connection::open_in_memory().expect("in-memory store");
        configure_store(&mut connection).expect("configure store");
        connection
    }

    #[test]
    fn record_captures_required_mt901_provenance_without_executable_path() {
        let record = record();

        assert_eq!(record.schema_version, SAVE_STATE_RECORD_SCHEMA_VERSION);
        assert_eq!(record.machine, "pacman");
        assert_eq!(record.software.as_deref(), Some("nes:mario"));
        assert_eq!(record.mame.version, "0.281");
        assert_eq!(record.mame.build.as_deref(), Some("mame0281"));
        assert_eq!(record.saved_at_epoch_ms, 99);
        assert_eq!(record.slot, "quick");
        assert_eq!(record.bytes, 4096);
        assert!(record.screenshot.is_none());

        let json = serde_json::to_value(&record).expect("serialize record");
        assert!(!json.to_string().contains("/opt/mame/mame"));
    }

    #[test]
    fn record_rejects_mismatched_session_provenance() {
        let mut saved = saved();
        saved.session_id = "different-session".to_owned();

        let error = SaveStateRecordV1::from_completed_save(&session(), &saved)
            .expect_err("mismatched session must fail");
        assert_eq!(error.code, "SAVE_STATE_RECORD_CONTEXT_MISMATCH");
    }

    #[test]
    fn record_rejects_relative_traversing_and_malformed_slot_paths() {
        let mut relative = saved();
        relative.path = "save-states/quick.sta".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &relative)
                .expect_err("relative path must fail")
                .code,
            "SAVE_STATE_RECORD_PATH_INVALID"
        );

        let mut traversing = saved();
        traversing.path = "/tmp/mame/../escape.sta".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &traversing)
                .expect_err("traversing path must fail")
                .code,
            "SAVE_STATE_RECORD_PATH_INVALID"
        );

        let mut slot_escape = saved();
        slot_escape.slot = "../escape".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &slot_escape)
                .expect_err("slot traversal must fail")
                .code,
            "SAVE_STATE_INVALID_SLOT"
        );
    }

    #[test]
    fn optional_screenshot_metadata_round_trips_as_absent() {
        let record = record();
        let encoded = serde_json::to_string(&record).expect("encode record");
        let decoded: SaveStateRecordV1 = serde_json::from_str(&encoded).expect("decode record");

        assert_eq!(decoded, record);
        assert!(decoded.screenshot.is_none());
    }

    #[test]
    fn store_upserts_by_application_owned_state_path_and_lists_recent_first() {
        let mut connection = store();
        let mut first = record();
        first.saved_at_epoch_ms = 100;
        let first_id = upsert_record(&mut connection, &first).expect("insert first");

        let mut replacement = first.clone();
        replacement.saved_at_epoch_ms = 300;
        replacement.bytes = 8192;
        let replacement_id = upsert_record(&mut connection, &replacement).expect("replace slot");
        assert_eq!(
            replacement_id, first_id,
            "same state path must update one record"
        );

        let mut second = first.clone();
        second.slot = "auto".to_owned();
        second.path = second.path.replace("quick.sta", "auto.sta");
        second.saved_at_epoch_ms = 200;
        let second_id = upsert_record(&mut connection, &second).expect("insert second");
        assert_ne!(second_id, first_id);

        let (total, records) = list_records(&mut connection, 100, 0).expect("list records");
        assert_eq!(total, 2);
        assert_eq!(records[0].0, first_id);
        assert_eq!(records[0].1.bytes, 8192);
        assert_eq!(records[1].0, second_id);
    }

    #[test]
    fn record_lookup_and_delete_are_id_scoped() {
        let mut connection = store();
        let id = upsert_record(&mut connection, &record()).expect("insert record");
        assert_eq!(
            load_record(&mut connection, id).expect("load record"),
            record()
        );
        delete_record(&mut connection, id).expect("delete record");
        assert_eq!(
            load_record(&mut connection, id)
                .expect_err("deleted record must be absent")
                .code,
            "SAVE_STATE_RECORD_NOT_FOUND"
        );
    }

    #[test]
    fn delete_authorization_requires_exact_derived_application_path() {
        let root = Path::new("/tmp/save-states/v1");
        let record = record();
        assert_eq!(
            authorize_record_path(root, &record).expect("authorized path"),
            Path::new(&record.path)
        );

        let mut forged = record;
        forged.path = "/tmp/save-states/v1/other/quick.sta".to_owned();
        assert_eq!(
            authorize_record_path(root, &forged)
                .expect_err("forged path must be rejected")
                .code,
            "SAVE_STATE_RECORD_PATH_UNAUTHORIZED"
        );
    }

    #[test]
    fn future_store_schema_is_rejected() {
        let mut connection = Connection::open_in_memory().expect("in-memory store");
        connection
            .pragma_update(None, "user_version", 99_i64)
            .expect("set future version");
        assert_eq!(
            configure_store(&mut connection)
                .expect_err("future store must fail")
                .code,
            "SAVE_STATE_STORE_SCHEMA_UNSUPPORTED"
        );
    }
}
