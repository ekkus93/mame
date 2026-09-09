use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use crate::errors::{AppError, AppResult};

use super::{catalog::CatalogRepository, model::MachineListItem};

const MAX_COLLECTION_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSummary {
    pub id: i64,
    pub name: String,
    pub created_at_epoch_ms: u64,
    pub updated_at_epoch_ms: u64,
    pub member_count: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionListPage {
    pub schema_version: u32,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<CollectionSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMemberEntry {
    pub short_name: String,
    pub added_at_epoch_ms: u64,
    pub machine: Option<MachineListItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMemberPage {
    pub schema_version: u32,
    pub collection: CollectionSummary,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<CollectionMemberEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMembershipState {
    pub schema_version: u32,
    pub collection_id: i64,
    pub short_name: String,
    pub member: bool,
    pub added_at_epoch_ms: Option<u64>,
}

impl CatalogRepository {
    pub(crate) fn create_collection(
        &self,
        name: &str,
        now_epoch_ms: u64,
    ) -> AppResult<CollectionSummary> {
        self.ensure_collection_name_available(name, None)?;
        let now = to_i64(now_epoch_ms, "timestamp")?;
        self.connection
            .execute(
                "INSERT INTO user_collections(name, created_at_epoch_ms, updated_at_epoch_ms) VALUES (?1, ?2, ?2)",
                params![name, now],
            )
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_CREATE_FAILED", error))?;
        let id = self.connection.last_insert_rowid();
        self.collection_summary(id)
    }

    pub(crate) fn rename_collection(
        &self,
        collection_id: i64,
        name: &str,
        now_epoch_ms: u64,
    ) -> AppResult<CollectionSummary> {
        self.require_collection(collection_id)?;
        self.ensure_collection_name_available(name, Some(collection_id))?;
        let now = to_i64(now_epoch_ms, "timestamp")?;
        self.connection
            .execute(
                "UPDATE user_collections SET name = ?1, updated_at_epoch_ms = ?2 WHERE id = ?3",
                params![name, now, collection_id],
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_RENAME_FAILED", error)
            })?;
        self.collection_summary(collection_id)
    }

    pub(crate) fn delete_collection(&self, collection_id: i64) -> AppResult<()> {
        self.require_collection(collection_id)?;
        self.connection
            .execute(
                "DELETE FROM user_collections WHERE id = ?1",
                [collection_id],
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_DELETE_FAILED", error)
            })?;
        Ok(())
    }

    pub(crate) fn query_collections(
        &self,
        limit: u32,
        offset: u32,
    ) -> AppResult<CollectionListPage> {
        validate_page_size(limit)?;
        let total = stored_u64(
            self.connection
                .query_row("SELECT COUNT(*) FROM user_collections", [], |row| {
                    row.get(0)
                })
                .map_err(|error| {
                    collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
                })?,
            "total",
        )?;
        let mut statement = self
            .connection
            .prepare(
                r#"SELECT c.id, c.name, c.created_at_epoch_ms, c.updated_at_epoch_ms,
                          COUNT(cm.machine_short_name)
                   FROM user_collections c
                   LEFT JOIN user_collection_machines cm ON cm.collection_id = c.id
                   GROUP BY c.id
                   ORDER BY c.name COLLATE NOCASE, c.id
                   LIMIT ?1 OFFSET ?2"#,
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
            })?;
        let rows = statement
            .query_map(
                params![i64::from(limit), i64::from(offset)],
                collection_summary_from_row,
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
            })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|error| {
                    collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
                })?
                .into_summary()?,
            );
        }
        Ok(CollectionListPage {
            schema_version: 1,
            total,
            offset,
            limit,
            items,
        })
    }

    pub(crate) fn collection_membership_state(
        &self,
        collection_id: i64,
        short_name: &str,
    ) -> AppResult<CollectionMembershipState> {
        self.require_collection(collection_id)?;
        let added_at: Option<i64> = self
            .connection
            .query_row(
                "SELECT added_at_epoch_ms FROM user_collection_machines WHERE collection_id = ?1 AND machine_short_name = ?2",
                params![collection_id, short_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_MEMBERSHIP_READ_FAILED", error))?;
        Ok(CollectionMembershipState {
            schema_version: 1,
            collection_id,
            short_name: short_name.to_owned(),
            member: added_at.is_some(),
            added_at_epoch_ms: added_at
                .map(|value| stored_u64(value, "addedAtEpochMs"))
                .transpose()?,
        })
    }

    pub(crate) fn set_collection_machine(
        &self,
        collection_id: i64,
        short_name: &str,
        member: bool,
        now_epoch_ms: u64,
    ) -> AppResult<CollectionMembershipState> {
        self.require_collection(collection_id)?;
        if member {
            self.require_active_machine_for_collection(short_name)?;
            let now = to_i64(now_epoch_ms, "timestamp")?;
            self.connection
                .execute(
                    "INSERT INTO user_collection_machines(collection_id, machine_short_name, added_at_epoch_ms) VALUES (?1, ?2, ?3) ON CONFLICT(collection_id, machine_short_name) DO NOTHING",
                    params![collection_id, short_name, now],
                )
                .map_err(|error| collections_database_error("CATALOG_COLLECTION_MEMBERSHIP_WRITE_FAILED", error))?;
        } else {
            self.connection
                .execute(
                    "DELETE FROM user_collection_machines WHERE collection_id = ?1 AND machine_short_name = ?2",
                    params![collection_id, short_name],
                )
                .map_err(|error| collections_database_error("CATALOG_COLLECTION_MEMBERSHIP_WRITE_FAILED", error))?;
        }
        self.collection_membership_state(collection_id, short_name)
    }

    pub(crate) fn query_collection_members(
        &self,
        collection_id: i64,
        limit: u32,
        offset: u32,
    ) -> AppResult<CollectionMemberPage> {
        validate_page_size(limit)?;
        let collection = self.collection_summary(collection_id)?;
        let total = stored_u64(
            self.connection
                .query_row(
                    "SELECT COUNT(*) FROM user_collection_machines WHERE collection_id = ?1",
                    [collection_id],
                    |row| row.get(0),
                )
                .map_err(|error| {
                    collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
                })?,
            "total",
        )?;
        let mut statement = self
            .connection
            .prepare(
                r#"SELECT
                    cm.machine_short_name,
                    cm.added_at_epoch_ms,
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
                FROM user_collection_machines cm
                LEFT JOIN machines m
                  ON m.generation_id = (SELECT id FROM metadata_generation WHERE active = 1)
                 AND m.short_name = cm.machine_short_name
                WHERE cm.collection_id = ?1
                ORDER BY
                    CASE WHEN m.short_name IS NULL THEN 1 ELSE 0 END,
                    COALESCE(m.description, cm.machine_short_name) COLLATE NOCASE,
                    cm.machine_short_name COLLATE NOCASE
                LIMIT ?2 OFFSET ?3"#,
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
            })?;
        let rows = statement
            .query_map(
                params![collection_id, i64::from(limit), i64::from(offset)],
                stored_collection_member_from_row,
            )
            .map_err(|error| {
                collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
            })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(
                row.map_err(|error| {
                    collections_database_error("CATALOG_COLLECTION_QUERY_FAILED", error)
                })?
                .into_entry()?,
            );
        }
        Ok(CollectionMemberPage {
            schema_version: 1,
            collection,
            total,
            offset,
            limit,
            items,
        })
    }

    fn collection_summary(&self, collection_id: i64) -> AppResult<CollectionSummary> {
        let stored = self
            .connection
            .query_row(
                r#"SELECT c.id, c.name, c.created_at_epoch_ms, c.updated_at_epoch_ms,
                          COUNT(cm.machine_short_name)
                   FROM user_collections c
                   LEFT JOIN user_collection_machines cm ON cm.collection_id = c.id
                   WHERE c.id = ?1
                   GROUP BY c.id"#,
                [collection_id],
                collection_summary_from_row,
            )
            .optional()
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_READ_FAILED", error))?
            .ok_or_else(|| collection_not_found(collection_id))?;
        stored.into_summary()
    }

    fn require_collection(&self, collection_id: i64) -> AppResult<()> {
        let exists: i64 = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM user_collections WHERE id = ?1)",
                [collection_id],
                |row| row.get(0),
            )
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_READ_FAILED", error))?;
        if exists == 0 {
            return Err(collection_not_found(collection_id));
        }
        Ok(())
    }

    fn ensure_collection_name_available(
        &self,
        name: &str,
        excluding_id: Option<i64>,
    ) -> AppResult<()> {
        let existing: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM user_collections WHERE name = ?1 COLLATE NOCASE",
                [name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_READ_FAILED", error))?;
        if existing.is_some_and(|id| Some(id) != excluding_id) {
            return Err(AppError::new(
                "CATALOG_COLLECTION_NAME_EXISTS",
                "A collection with that name already exists.",
            )
            .with_details(serde_json::json!({ "name": name })));
        }
        Ok(())
    }

    fn require_active_machine_for_collection(&self, short_name: &str) -> AppResult<()> {
        let generation_id: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM metadata_generation WHERE active = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_READ_FAILED", error))?;
        let Some(generation_id) = generation_id else {
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
            .map_err(|error| collections_database_error("CATALOG_COLLECTION_READ_FAILED", error))?;
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
struct StoredCollectionSummary {
    id: i64,
    name: String,
    created_at_epoch_ms: i64,
    updated_at_epoch_ms: i64,
    member_count: i64,
}

impl StoredCollectionSummary {
    fn into_summary(self) -> AppResult<CollectionSummary> {
        Ok(CollectionSummary {
            id: self.id,
            name: self.name,
            created_at_epoch_ms: stored_u64(self.created_at_epoch_ms, "createdAtEpochMs")?,
            updated_at_epoch_ms: stored_u64(self.updated_at_epoch_ms, "updatedAtEpochMs")?,
            member_count: stored_u64(self.member_count, "memberCount")?,
        })
    }
}

fn collection_summary_from_row(row: &Row<'_>) -> rusqlite::Result<StoredCollectionSummary> {
    Ok(StoredCollectionSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at_epoch_ms: row.get(2)?,
        updated_at_epoch_ms: row.get(3)?,
        member_count: row.get(4)?,
    })
}

#[derive(Debug)]
struct StoredCollectionMember {
    short_name: String,
    added_at_epoch_ms: i64,
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

impl StoredCollectionMember {
    fn into_entry(self) -> AppResult<CollectionMemberEntry> {
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
        Ok(CollectionMemberEntry {
            short_name: self.short_name,
            added_at_epoch_ms: stored_u64(self.added_at_epoch_ms, "addedAtEpochMs")?,
            machine,
        })
    }
}

fn stored_collection_member_from_row(row: &Row<'_>) -> rusqlite::Result<StoredCollectionMember> {
    Ok(StoredCollectionMember {
        short_name: row.get(0)?,
        added_at_epoch_ms: row.get(1)?,
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

fn validate_page_size(limit: u32) -> AppResult<()> {
    if limit == 0 || limit > MAX_COLLECTION_PAGE_SIZE {
        return Err(AppError::new(
            "CATALOG_COLLECTION_LIMIT_INVALID",
            format!("Collection page size must be between 1 and {MAX_COLLECTION_PAGE_SIZE}."),
        ));
    }
    Ok(())
}

fn collection_not_found(collection_id: i64) -> AppError {
    AppError::new(
        "CATALOG_COLLECTION_NOT_FOUND",
        "The requested collection does not exist.",
    )
    .with_details(serde_json::json!({ "collectionId": collection_id }))
}

fn to_i64(value: u64, field: &str) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_COLLECTION_VALUE_INVALID",
            "A collection numeric value is outside SQLite's supported range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn stored_u64(value: i64, field: &str) -> AppResult<u64> {
    u64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_COLLECTION_VALUE_INVALID",
            "The collections database contains an invalid numeric value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn stored_u32(value: i64, field: &str) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_COLLECTION_VALUE_INVALID",
            "The collections database contains an invalid numeric value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn collections_database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The collections database operation failed.")
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
            .expect("begin import");
        let summary = pending
            .import_listxml(Cursor::new(xml))
            .expect("parse fixture");
        pending
            .finish(summary, generated + 1)
            .expect("finish import");
    }

    #[test]
    fn collection_crud_and_membership_are_durable() {
        let mut repo = CatalogRepository::memory().expect("catalog");
        import(&mut repo, REPRESENTATIVE, 100);
        let collection = repo
            .create_collection("Favorites 2", 500)
            .expect("create collection");
        assert_eq!(collection.member_count, 0);
        let renamed = repo
            .rename_collection(collection.id, "Arcade", 600)
            .expect("rename collection");
        assert_eq!(renamed.name, "Arcade");

        let state = repo
            .set_collection_machine(collection.id, "galaxian", true, 700)
            .expect("add member");
        assert!(state.member);
        let page = repo
            .query_collection_members(collection.id, 20, 0)
            .expect("members");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "galaxian");
        assert!(page.items[0].machine.is_some());

        repo.delete_collection(collection.id)
            .expect("delete collection");
        let error = repo
            .query_collection_members(collection.id, 20, 0)
            .expect_err("deleted collection");
        assert_eq!(error.code, "CATALOG_COLLECTION_NOT_FOUND");
    }

    #[test]
    fn collection_member_survives_refresh_and_can_be_removed_when_stale() {
        let mut repo = CatalogRepository::memory().expect("catalog");
        import(&mut repo, REPRESENTATIVE, 100);
        let collection = repo
            .create_collection("Classics", 500)
            .expect("create collection");
        repo.set_collection_machine(collection.id, "galaxian", true, 600)
            .expect("add member");

        import(&mut repo, APPLE_ONLY, 200);
        let page = repo
            .query_collection_members(collection.id, 20, 0)
            .expect("members");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "galaxian");
        assert!(page.items[0].machine.is_none());

        let removed = repo
            .set_collection_machine(collection.id, "galaxian", false, 0)
            .expect("remove stale member");
        assert!(!removed.member);
        assert_eq!(
            repo.query_collection_members(collection.id, 20, 0)
                .expect("empty")
                .total,
            0
        );
    }

    #[test]
    fn collection_names_are_case_insensitively_unique() {
        let repo = CatalogRepository::memory().expect("catalog");
        repo.create_collection("Arcade", 100)
            .expect("create collection");
        let error = repo
            .create_collection("arcade", 200)
            .expect_err("duplicate name must fail");
        assert_eq!(error.code, "CATALOG_COLLECTION_NAME_EXISTS");
    }

    #[test]
    fn adding_member_requires_active_machine() {
        let mut repo = CatalogRepository::memory().expect("catalog");
        import(&mut repo, REPRESENTATIVE, 100);
        let collection = repo
            .create_collection("Arcade", 500)
            .expect("create collection");
        let error = repo
            .set_collection_machine(collection.id, "missing", true, 600)
            .expect_err("missing machine must fail");
        assert_eq!(error.code, "CATALOG_MACHINE_NOT_FOUND");
    }

    #[test]
    fn collection_pages_are_bounded() {
        let repo = CatalogRepository::memory().expect("catalog");
        let error = repo
            .query_collections(0, 0)
            .expect_err("zero page size must fail");
        assert_eq!(error.code, "CATALOG_COLLECTION_LIMIT_INVALID");
    }
}
