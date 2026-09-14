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
