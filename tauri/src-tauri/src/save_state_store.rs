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
