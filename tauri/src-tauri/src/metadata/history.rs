use rusqlite::{params, Row};
use serde::Serialize;

use crate::errors::{AppError, AppResult};

use super::{catalog::CatalogRepository, model::MachineListItem};

pub const HISTORY_RETENTION_LIMIT: u32 = 500;
const MAX_HISTORY_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentHistoryEntry {
    pub id: i64,
    pub machine_short_name: String,
    pub software_item: Option<String>,
    pub launched_at_epoch_ms: u64,
    pub succeeded: Option<bool>,
    pub machine: Option<MachineListItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentHistoryPage {
    pub schema_version: u32,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub retention_limit: u32,
    pub items: Vec<RecentHistoryEntry>,
}

impl CatalogRepository {
    pub(crate) fn begin_history_entry(
        &self,
        machine_short_name: &str,
        software_item: Option<&str>,
        launched_at_epoch_ms: u64,
    ) -> AppResult<i64> {
        self.prune_history_before_insert()?;
        let launched_at = to_i64(launched_at_epoch_ms, "launchedAtEpochMs")?;
        self.connection
            .execute(
                "INSERT INTO recent_history(machine_short_name, software_item, launched_at_epoch_ms, succeeded) VALUES (?1, ?2, ?3, NULL)",
                params![machine_short_name, software_item, launched_at],
            )
            .map_err(|error| history_database_error("CATALOG_HISTORY_WRITE_FAILED", error))?;
        Ok(self.connection.last_insert_rowid())
    }

    pub(crate) fn finish_history_entry(&self, history_id: i64, succeeded: bool) -> AppResult<()> {
        let updated = self
            .connection
            .execute(
                "UPDATE recent_history SET succeeded = ?1 WHERE id = ?2",
                params![if succeeded { 1_i64 } else { 0_i64 }, history_id],
            )
            .map_err(|error| history_database_error("CATALOG_HISTORY_WRITE_FAILED", error))?;
        if updated != 1 {
            return Err(AppError::new(
                "CATALOG_HISTORY_ENTRY_NOT_FOUND",
                "The play-history entry could not be finalized because it no longer exists.",
            )
            .with_details(serde_json::json!({ "historyId": history_id })));
        }
        Ok(())
    }

    pub(crate) fn query_recent_history(
        &self,
        limit: u32,
        offset: u32,
    ) -> AppResult<RecentHistoryPage> {
        validate_page_size(limit)?;
        let total = stored_u64(
            self.connection
                .query_row("SELECT COUNT(*) FROM recent_history", [], |row| row.get(0))
                .map_err(|error| history_database_error("CATALOG_HISTORY_QUERY_FAILED", error))?,
            "total",
        )?;
        let mut statement = self
            .connection
            .prepare(
                r#"SELECT
                    h.id,
                    h.machine_short_name,
                    h.software_item,
                    h.launched_at_epoch_ms,
                    h.succeeded,
                    m.short_name,
                    m.description,
                    m.year,
                    m.manufacturer,
                    m.source_file,
                    m.clone_of,
                    m.runnable,
                    m.is_device,
                    m.driver_status,
                    (SELECT COUNT(*) FROM displays d
                        WHERE d.generation_id = m.generation_id
                          AND d.machine_short_name = m.short_name),
                    (SELECT COUNT(*) FROM machine_software_lists sl
                        WHERE sl.generation_id = m.generation_id
                          AND sl.machine_short_name = m.short_name)
                FROM recent_history h
                LEFT JOIN machines m
                  ON m.generation_id = (
                      SELECT id FROM metadata_generation WHERE active = 1
                  )
                 AND m.short_name = h.machine_short_name
                ORDER BY h.launched_at_epoch_ms DESC, h.id DESC
                LIMIT ?1 OFFSET ?2"#,
            )
            .map_err(|error| history_database_error("CATALOG_HISTORY_QUERY_FAILED", error))?;
        let rows = statement
            .query_map(
                params![i64::from(limit), i64::from(offset)],
                stored_history_from_row,
            )
            .map_err(|error| history_database_error("CATALOG_HISTORY_QUERY_FAILED", error))?;

        let mut items = Vec::new();
        for row in rows {
            let stored =
                row.map_err(|error| history_database_error("CATALOG_HISTORY_QUERY_FAILED", error))?;
            items.push(stored.into_entry()?);
        }

        Ok(RecentHistoryPage {
            schema_version: 1,
            total,
            offset,
            limit,
            retention_limit: HISTORY_RETENTION_LIMIT,
            items,
        })
    }

    fn prune_history_before_insert(&self) -> AppResult<()> {
        let keep = i64::from(HISTORY_RETENTION_LIMIT.saturating_sub(1));
        self.connection
            .execute(
                r#"DELETE FROM recent_history
                   WHERE id NOT IN (
                       SELECT id FROM recent_history
                       ORDER BY launched_at_epoch_ms DESC, id DESC
                       LIMIT ?1
                   )"#,
                [keep],
            )
            .map_err(|error| history_database_error("CATALOG_HISTORY_PRUNE_FAILED", error))?;
        Ok(())
    }
}

#[derive(Debug)]
struct StoredHistory {
    id: i64,
    machine_short_name: String,
    software_item: Option<String>,
    launched_at_epoch_ms: i64,
    succeeded: Option<i64>,
    active_machine_short_name: Option<String>,
    description: Option<String>,
    year: Option<String>,
    manufacturer: Option<String>,
    source_file: Option<String>,
    clone_of: Option<String>,
    runnable: Option<i64>,
    is_device: Option<i64>,
    driver_status: Option<String>,
    display_count: i64,
    software_list_count: i64,
}

impl StoredHistory {
    fn into_entry(self) -> AppResult<RecentHistoryEntry> {
        let machine = self
            .active_machine_short_name
            .map(|short_name| {
                Ok(MachineListItem {
                    description: self.description.unwrap_or_else(|| short_name.clone()),
                    short_name,
                    year: self.year,
                    manufacturer: self.manufacturer,
                    source_file: self.source_file,
                    clone_of: self.clone_of,
                    runnable: self.runnable.unwrap_or(0) != 0,
                    is_device: self.is_device.unwrap_or(0) != 0,
                    driver_status: self.driver_status,
                    display_count: stored_u32(self.display_count, "displayCount")?,
                    software_list_count: stored_u32(self.software_list_count, "softwareListCount")?,
                })
            })
            .transpose()?;
        let succeeded = self
            .succeeded
            .map(|value| stored_bool(value, "succeeded"))
            .transpose()?;

        Ok(RecentHistoryEntry {
            id: self.id,
            machine_short_name: self.machine_short_name,
            software_item: self.software_item,
            launched_at_epoch_ms: stored_u64(self.launched_at_epoch_ms, "launchedAtEpochMs")?,
            succeeded,
            machine,
        })
    }
}

fn stored_history_from_row(row: &Row<'_>) -> rusqlite::Result<StoredHistory> {
    Ok(StoredHistory {
        id: row.get(0)?,
        machine_short_name: row.get(1)?,
        software_item: row.get(2)?,
        launched_at_epoch_ms: row.get(3)?,
        succeeded: row.get(4)?,
        active_machine_short_name: row.get(5)?,
        description: row.get(6)?,
        year: row.get(7)?,
        manufacturer: row.get(8)?,
        source_file: row.get(9)?,
        clone_of: row.get(10)?,
        runnable: row.get(11)?,
        is_device: row.get(12)?,
        driver_status: row.get(13)?,
        display_count: row.get(14)?,
        software_list_count: row.get(15)?,
    })
}

fn validate_page_size(limit: u32) -> AppResult<()> {
    if limit == 0 || limit > MAX_HISTORY_PAGE_SIZE {
        return Err(AppError::new(
            "CATALOG_HISTORY_LIMIT_INVALID",
            format!("History page size must be between 1 and {MAX_HISTORY_PAGE_SIZE}."),
        )
        .with_details(serde_json::json!({
            "limit": limit,
            "maxLimit": MAX_HISTORY_PAGE_SIZE
        })));
    }
    Ok(())
}

fn stored_bool(value: i64, field: &str) -> AppResult<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(AppError::new(
            "CATALOG_HISTORY_DATA_INVALID",
            "Stored play-history data contains an invalid boolean value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))),
    }
}

fn stored_u32(value: i64, field: &str) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_HISTORY_DATA_INVALID",
            "Stored play-history data is outside the supported range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn stored_u64(value: i64, field: &str) -> AppResult<u64> {
    u64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_HISTORY_DATA_INVALID",
            "Stored play-history data is outside the supported range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn to_i64(value: u64, field: &str) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_HISTORY_VALUE_INVALID",
            "The play-history value is outside SQLite's supported range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn history_database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The play-history database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::{CatalogRepository, HISTORY_RETENTION_LIMIT};

    fn repository_with_machine() -> CatalogRepository {
        let repository = CatalogRepository::memory().expect("in-memory catalog");
        repository
            .connection
            .execute(
                r#"INSERT INTO metadata_generation(
                    source_kind, trust, executable_path, mame_version, raw_version_line,
                    generated_at_epoch_ms, imported_at_epoch_ms, machine_count, active
                ) VALUES ('external', 'userConfigured', '/tmp/mame', '0.288', 'MAME 0.288', 1, 1, 1, 1)"#,
                [],
            )
            .expect("generation");
        let generation_id = repository.connection.last_insert_rowid();
        repository
            .connection
            .execute(
                r#"INSERT INTO machines(
                    generation_id, short_name, description, year, manufacturer, source_file,
                    is_bios, is_device, is_mechanical, runnable, driver_status,
                    driver_requires_artwork, driver_unofficial, driver_no_sound_hardware,
                    driver_incomplete
                ) VALUES (?1, 'galaxian', 'Galaxian', '1979', 'Namco', 'galaxian.cpp',
                    0, 0, 0, 1, 'good', 0, 0, 0, 0)"#,
                [generation_id],
            )
            .expect("machine");
        repository
    }

    #[test]
    fn records_machine_software_timestamp_and_outcome() {
        let repository = repository_with_machine();
        let history_id = repository
            .begin_history_entry("galaxian", Some("galaxian:demo"), 100)
            .expect("begin history");

        let pending = repository
            .query_recent_history(20, 0)
            .expect("query pending history");
        assert_eq!(pending.items[0].id, history_id);
        assert_eq!(pending.items[0].machine_short_name, "galaxian");
        assert_eq!(
            pending.items[0].software_item.as_deref(),
            Some("galaxian:demo")
        );
        assert_eq!(pending.items[0].launched_at_epoch_ms, 100);
        assert_eq!(pending.items[0].succeeded, None);
        assert_eq!(
            pending.items[0]
                .machine
                .as_ref()
                .map(|machine| machine.description.as_str()),
            Some("Galaxian")
        );

        repository
            .finish_history_entry(history_id, true)
            .expect("finish history");
        let finished = repository
            .query_recent_history(20, 0)
            .expect("query finished history");
        assert_eq!(finished.items[0].succeeded, Some(true));
    }

    #[test]
    fn distinguishes_failed_launches() {
        let repository = repository_with_machine();
        let history_id = repository
            .begin_history_entry("galaxian", None, 200)
            .expect("begin history");
        repository
            .finish_history_entry(history_id, false)
            .expect("finish history");

        let page = repository
            .query_recent_history(20, 0)
            .expect("query history");
        assert_eq!(page.items[0].succeeded, Some(false));
    }

    #[test]
    fn history_survives_metadata_removal() {
        let repository = repository_with_machine();
        let history_id = repository
            .begin_history_entry("galaxian", None, 300)
            .expect("begin history");
        repository
            .finish_history_entry(history_id, true)
            .expect("finish history");
        repository
            .connection
            .execute("DELETE FROM metadata_generation WHERE active = 1", [])
            .expect("remove generated metadata");

        let page = repository
            .query_recent_history(20, 0)
            .expect("query stale history");
        assert_eq!(page.items[0].machine_short_name, "galaxian");
        assert_eq!(page.items[0].machine, None);
    }

    #[test]
    fn retention_keeps_only_newest_entries() {
        let repository = repository_with_machine();
        for timestamp in 0..u64::from(HISTORY_RETENTION_LIMIT + 5) {
            let history_id = repository
                .begin_history_entry("galaxian", None, timestamp)
                .expect("begin history");
            repository
                .finish_history_entry(history_id, true)
                .expect("finish history");
        }

        let (count, oldest): (i64, i64) = repository
            .connection
            .query_row(
                "SELECT COUNT(*), MIN(launched_at_epoch_ms) FROM recent_history",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("retention query");
        assert_eq!(count, i64::from(HISTORY_RETENTION_LIMIT));
        assert_eq!(oldest, 5);
    }

    #[test]
    fn rejects_unbounded_page_sizes() {
        let repository = repository_with_machine();
        let error = repository
            .query_recent_history(201, 0)
            .expect_err("oversized history query must fail");
        assert_eq!(error.code, "CATALOG_HISTORY_LIMIT_INVALID");
    }
}
