use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use crate::errors::{AppError, AppResult};

use super::{catalog::CatalogRepository, model::MachineListItem};

const MAX_FAVORITES_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteState {
    pub schema_version: u32,
    pub short_name: String,
    pub favorite: bool,
    pub created_at_epoch_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteEntry {
    pub short_name: String,
    pub created_at_epoch_ms: u64,
    pub machine: Option<MachineListItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FavoritePage {
    pub schema_version: u32,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<FavoriteEntry>,
}

impl CatalogRepository {
    pub(crate) fn favorite_state(&self, short_name: &str) -> AppResult<FavoriteState> {
        let created_at: Option<i64> = self
            .connection
            .query_row(
                "SELECT created_at_epoch_ms FROM user_favorites WHERE machine_short_name = ?1",
                [short_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| favorites_database_error("CATALOG_FAVORITE_READ_FAILED", error))?;

        Ok(FavoriteState {
            schema_version: 1,
            short_name: short_name.to_owned(),
            favorite: created_at.is_some(),
            created_at_epoch_ms: created_at
                .map(|value| stored_u64(value, "createdAtEpochMs"))
                .transpose()?,
        })
    }

    pub(crate) fn set_favorite(
        &self,
        short_name: &str,
        favorite: bool,
        created_at_epoch_ms: u64,
    ) -> AppResult<FavoriteState> {
        if favorite {
            self.require_active_machine(short_name)?;
            let created_at = i64::try_from(created_at_epoch_ms).map_err(|_| {
                AppError::new(
                    "CATALOG_FAVORITE_TIMESTAMP_INVALID",
                    "The favorite timestamp is outside SQLite's supported range.",
                )
            })?;
            self.connection
                .execute(
                    "INSERT INTO user_favorites(machine_short_name, created_at_epoch_ms) VALUES (?1, ?2) ON CONFLICT(machine_short_name) DO NOTHING",
                    params![short_name, created_at],
                )
                .map_err(|error| favorites_database_error("CATALOG_FAVORITE_WRITE_FAILED", error))?;
        } else {
            self.connection
                .execute(
                    "DELETE FROM user_favorites WHERE machine_short_name = ?1",
                    [short_name],
                )
                .map_err(|error| {
                    favorites_database_error("CATALOG_FAVORITE_WRITE_FAILED", error)
                })?;
        }

        self.favorite_state(short_name)
    }

    pub(crate) fn query_favorites(&self, limit: u32, offset: u32) -> AppResult<FavoritePage> {
        if limit == 0 || limit > MAX_FAVORITES_PAGE_SIZE {
            return Err(AppError::new(
                "CATALOG_FAVORITES_LIMIT_INVALID",
                format!("Favorites page size must be between 1 and {MAX_FAVORITES_PAGE_SIZE}."),
            )
            .with_details(serde_json::json!({
                "limit": limit,
                "maxLimit": MAX_FAVORITES_PAGE_SIZE
            })));
        }

        let total_i64: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM user_favorites", [], |row| row.get(0))
            .map_err(|error| favorites_database_error("CATALOG_FAVORITES_QUERY_FAILED", error))?;
        let total = stored_u64(total_i64, "total")?;

        let mut statement = self
            .connection
            .prepare(
                r#"SELECT
                    f.machine_short_name,
                    f.created_at_epoch_ms,
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
                FROM user_favorites f
                LEFT JOIN machines m
                  ON m.generation_id = (
                      SELECT id FROM metadata_generation WHERE active = 1
                  )
                 AND m.short_name = f.machine_short_name
                ORDER BY
                    CASE WHEN m.short_name IS NULL THEN 1 ELSE 0 END,
                    COALESCE(m.description, f.machine_short_name) COLLATE NOCASE,
                    f.machine_short_name COLLATE NOCASE
                LIMIT ?1 OFFSET ?2"#,
            )
            .map_err(|error| favorites_database_error("CATALOG_FAVORITES_QUERY_FAILED", error))?;
        let rows = statement
            .query_map(
                params![i64::from(limit), i64::from(offset)],
                stored_favorite_from_row,
            )
            .map_err(|error| favorites_database_error("CATALOG_FAVORITES_QUERY_FAILED", error))?;

        let mut items = Vec::new();
        for row in rows {
            let stored = row.map_err(|error| {
                favorites_database_error("CATALOG_FAVORITES_QUERY_FAILED", error)
            })?;
            items.push(stored.into_entry()?);
        }

        Ok(FavoritePage {
            schema_version: 1,
            total,
            offset,
            limit,
            items,
        })
    }

    fn require_active_machine(&self, short_name: &str) -> AppResult<()> {
        let active_generation: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM metadata_generation WHERE active = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| favorites_database_error("CATALOG_FAVORITE_READ_FAILED", error))?;
        let Some(generation_id) = active_generation else {
            return Err(AppError::new(
                "MAME_METADATA_NOT_READY",
                "No successfully imported MAME metadata generation is active.",
            ));
        };

        let exists: i64 = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM machines WHERE generation_id = ?1 AND short_name = ?2)",
                params![generation_id, short_name],
                |row| row.get(0),
            )
            .map_err(|error| favorites_database_error("CATALOG_FAVORITE_READ_FAILED", error))?;
        if exists == 0 {
            return Err(AppError::new(
                "CATALOG_MACHINE_NOT_FOUND",
                "The requested machine is not present in the active MAME catalog.",
            )
            .with_details(serde_json::json!({ "shortName": short_name })));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct StoredFavorite {
    short_name: String,
    created_at_epoch_ms: i64,
    machine_short_name: Option<String>,
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

impl StoredFavorite {
    fn into_entry(self) -> AppResult<FavoriteEntry> {
        let machine = self
            .machine_short_name
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

        Ok(FavoriteEntry {
            short_name: self.short_name,
            created_at_epoch_ms: stored_u64(self.created_at_epoch_ms, "createdAtEpochMs")?,
            machine,
        })
    }
}

fn stored_favorite_from_row(row: &Row<'_>) -> rusqlite::Result<StoredFavorite> {
    Ok(StoredFavorite {
        short_name: row.get(0)?,
        created_at_epoch_ms: row.get(1)?,
        machine_short_name: row.get(2)?,
        description: row.get(3)?,
        year: row.get(4)?,
        manufacturer: row.get(5)?,
        source_file: row.get(6)?,
        clone_of: row.get(7)?,
        runnable: row.get(8)?,
        is_device: row.get(9)?,
        driver_status: row.get(10)?,
        display_count: row.get(11)?,
        software_list_count: row.get(12)?,
    })
}

fn stored_u64(value: i64, field: &str) -> AppResult<u64> {
    u64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_FAVORITE_VALUE_INVALID",
            "The favorites database contains an invalid numeric value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn stored_u32(value: i64, field: &str) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_FAVORITE_VALUE_INVALID",
            "The favorites database contains an invalid numeric value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn favorites_database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The favorites database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::CatalogRepository;
    use crate::mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust};

    const REPRESENTATIVE: &[u8] =
        include_bytes!("../../../tests/fixtures/listxml-representative.xml");
    const APPLE_ONLY: &[u8] = br#"<?xml version="1.0"?>
<mame build="0.288 test-fixture" debug="no" mameconfig="10">
  <machine name="apple2e" sourcefile="apple/apple2e.cpp">
    <description>Apple //e</description>
    <year>1983</year>
    <manufacturer>Apple Computer</manufacturer>
    <driver status="good" emulation="good" savestate="supported"/>
  </machine>
</mame>"#;

    fn identity() -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::External,
            trust: MameExecutableTrust::UserConfigured,
            path: "/opt/mame/mame".to_owned(),
            version: "0.288".to_owned(),
            build: Some("test-fixture".to_owned()),
            raw_version_line: "0.288 test-fixture".to_owned(),
        }
    }

    fn import(repo: &mut CatalogRepository, xml: &[u8], generated: u64) {
        let mut pending = repo
            .begin_import(&identity(), generated)
            .expect("begin metadata import");
        let summary = pending
            .import_listxml(Cursor::new(xml))
            .expect("parse metadata fixture");
        pending
            .finish(summary, generated + 1)
            .expect("finish metadata import");
    }

    #[test]
    fn favorite_survives_refresh_and_removed_machine() {
        let mut repo = CatalogRepository::memory().expect("catalog");
        import(&mut repo, REPRESENTATIVE, 100);

        let state = repo
            .set_favorite("galaxian", true, 500)
            .expect("favorite machine");
        assert!(state.favorite);
        assert_eq!(state.created_at_epoch_ms, Some(500));

        import(&mut repo, APPLE_ONLY, 200);
        assert!(
            repo.favorite_state("galaxian")
                .expect("favorite state")
                .favorite
        );

        let page = repo.query_favorites(20, 0).expect("favorites page");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "galaxian");
        assert!(page.items[0].machine.is_none());

        let removed = repo
            .set_favorite("galaxian", false, 999)
            .expect("remove stale favorite");
        assert!(!removed.favorite);
        assert_eq!(repo.query_favorites(20, 0).expect("empty page").total, 0);
    }

    #[test]
    fn favorite_requires_machine_in_active_catalog() {
        let mut repo = CatalogRepository::memory().expect("catalog");
        import(&mut repo, REPRESENTATIVE, 100);
        let error = repo
            .set_favorite("missing", true, 500)
            .expect_err("unknown favorite must fail");
        assert_eq!(error.code, "CATALOG_MACHINE_NOT_FOUND");
    }

    #[test]
    fn favorite_page_size_is_bounded() {
        let repo = CatalogRepository::memory().expect("catalog");
        let error = repo
            .query_favorites(0, 0)
            .expect_err("zero page size must fail");
        assert_eq!(error.code, "CATALOG_FAVORITES_LIMIT_INVALID");
    }
}
