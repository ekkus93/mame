use std::{collections::BTreeMap, io::BufRead, path::Path};

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::{
    errors::{AppError, AppResult},
    mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
    storage,
};

use super::{
    model::{
        ListXmlSummary, MachineAvailability, MachineListItem, MachineMetadata, MachinePage,
        MetadataFreshness, MetadataGenerationSummary, MetadataStatus,
    },
    parser::parse_listxml,
};

const MAX_QUERY_PAGE_SIZE: u32 = 200;
const AUDIT_JOIN: &str = r#"
LEFT JOIN machine_audit_results ar
  ON ar.machine_short_name = m.short_name
 AND ar.mame_identity_json = ?8
 AND ar.content_paths_json = ?9
"#;

pub(crate) struct CatalogRepository {
    pub(super) connection: Connection,
}

impl CatalogRepository {
    pub(crate) fn open(path: &Path) -> AppResult<Self> {
        Ok(Self {
            connection: storage::open_catalog_connection(path)?,
        })
    }

    #[cfg(test)]
    pub(super) fn memory() -> AppResult<Self> {
        Ok(Self {
            connection: storage::open_catalog_memory()?,
        })
    }

    pub(crate) fn begin_import<'a>(
        &'a mut self,
        identity: &MameExecutableIdentity,
        generated_at_epoch_ms: u64,
    ) -> AppResult<CatalogImport<'a>> {
        let generated_at = to_i64(generated_at_epoch_ms, "generatedAtEpochMs")?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| database_error("CATALOG_IMPORT_BEGIN_FAILED", error))?;

        transaction
            .execute(
                r#"INSERT INTO metadata_generation(
                    source_kind, trust, executable_path, mame_version, mame_build,
                    raw_version_line, generated_at_epoch_ms, active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)"#,
                params![
                    source_kind_token(identity.source),
                    trust_token(identity.trust),
                    identity.path,
                    identity.version,
                    identity.build,
                    identity.raw_version_line,
                    generated_at
                ],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_BEGIN_FAILED", error))?;
        let generation_id = transaction.last_insert_rowid();

        Ok(CatalogImport {
            transaction,
            generation_id,
            identity: identity.clone(),
            generated_at_epoch_ms,
            machine_count: 0,
        })
    }

    pub(crate) fn active_generation(&self) -> AppResult<Option<MetadataGenerationSummary>> {
        let stored = self
            .connection
            .query_row(
                r#"SELECT
                    id, source_kind, trust, executable_path, mame_version, mame_build,
                    raw_version_line, listxml_build, mame_config, generated_at_epoch_ms,
                    imported_at_epoch_ms, machine_count
                FROM metadata_generation
                WHERE active = 1"#,
                [],
                stored_generation_from_row,
            )
            .optional()
            .map_err(|error| database_error("CATALOG_GENERATION_READ_FAILED", error))?;
        stored.map(StoredGeneration::into_summary).transpose()
    }

    pub(crate) fn metadata_status(
        &self,
        current_executable: MameExecutableIdentity,
    ) -> AppResult<MetadataStatus> {
        let active_generation = self.active_generation()?;
        let freshness = match &active_generation {
            None => MetadataFreshness::Empty,
            Some(active) if generation_matches_identity(active, &current_executable) => {
                MetadataFreshness::Fresh
            }
            Some(_) => MetadataFreshness::Stale,
        };

        Ok(MetadataStatus {
            schema_version: 1,
            freshness,
            current_executable,
            active_generation,
        })
    }

    pub(crate) fn query_machines(&self, query: &MachineQuery) -> AppResult<MachinePage> {
        self.query_machines_with_availability(
            query,
            &MachineAvailabilityQuery::unverified(AvailabilityFilter::All),
        )
    }

    pub(crate) fn query_machines_with_availability(
        &self,
        query: &MachineQuery,
        availability: &MachineAvailabilityQuery,
    ) -> AppResult<MachinePage> {
        if query.limit == 0 || query.limit > MAX_QUERY_PAGE_SIZE {
            return Err(AppError::new(
                "CATALOG_QUERY_LIMIT_INVALID",
                format!("Catalog query limit must be between 1 and {MAX_QUERY_PAGE_SIZE}."),
            )
            .with_details(serde_json::json!({
                "limit": query.limit,
                "maxLimit": MAX_QUERY_PAGE_SIZE
            })));
        }

        let generation_id: i64 = self
            .connection
            .query_row(
                "SELECT id FROM metadata_generation WHERE active = 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?
            .ok_or_else(|| {
                AppError::new(
                    "MAME_METADATA_NOT_READY",
                    "No successfully imported MAME metadata generation is active.",
                )
            })?;

        let search_pattern = query.text.as_deref().map(like_pattern);
        let clone_filter = query.clone_filter.as_token();
        let include_devices = if query.include_devices { 1_i64 } else { 0_i64 };
        let availability_filter = availability.filter.as_token();
        let audit_identity_json = availability.mame_identity_json.as_deref();
        let audit_content_paths_json = availability.content_paths_json.as_deref();

        let total_i64: i64 = self
            .connection
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM machines m {AUDIT_JOIN} WHERE {}",
                    QUERY_WHERE
                ),
                params![
                    generation_id,
                    search_pattern,
                    query.manufacturer.as_deref(),
                    query.year.as_deref(),
                    query.driver_status.as_deref(),
                    clone_filter,
                    include_devices,
                    audit_identity_json,
                    audit_content_paths_json,
                    availability_filter,
                ],
                |row| row.get(0),
            )
            .map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;
        let total = u64::try_from(total_i64).map_err(|_| {
            AppError::new(
                "CATALOG_QUERY_RESULT_INVALID",
                "The catalog returned an invalid negative result count.",
            )
        })?;

        let sql = format!(
            r#"SELECT
                m.short_name,
                m.description,
                m.year,
                m.manufacturer,
                m.source_file,
                m.clone_of,
                m.runnable,
                m.is_device,
                m.driver_status,
                CASE
                    WHEN ar.classification IN ('complete', 'bestAvailable') THEN 'available'
                    WHEN ar.classification IN ('missingRequired', 'incorrect', 'mixedFailure') THEN 'missing'
                    ELSE 'unknown'
                END AS availability,
                (SELECT COUNT(*) FROM displays d
                    WHERE d.generation_id = m.generation_id
                      AND d.machine_short_name = m.short_name) AS display_count,
                (SELECT COUNT(*) FROM machine_software_lists sl
                    WHERE sl.generation_id = m.generation_id
                      AND sl.machine_short_name = m.short_name) AS software_list_count
            FROM machines m
            {AUDIT_JOIN}
            WHERE {}
            ORDER BY {}
            LIMIT ?11 OFFSET ?12"#,
            QUERY_WHERE,
            query.sort.order_by()
        );
        let mut statement = self
            .connection
            .prepare(&sql)
            .map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;
        let rows = statement
            .query_map(
                params![
                    generation_id,
                    search_pattern,
                    query.manufacturer.as_deref(),
                    query.year.as_deref(),
                    query.driver_status.as_deref(),
                    clone_filter,
                    include_devices,
                    audit_identity_json,
                    audit_content_paths_json,
                    availability_filter,
                    i64::from(query.limit),
                    i64::from(query.offset),
                ],
                stored_machine_item_from_row,
            )
            .map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;

        let mut items = Vec::new();
        let mut availability_by_short_name = BTreeMap::new();
        for row in rows {
            let stored = row.map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;
            let (item, machine_availability) = stored.into_item()?;
            availability_by_short_name.insert(item.short_name.clone(), machine_availability);
            items.push(item);
        }

        Ok(MachinePage {
            schema_version: 1,
            generation_id,
            total,
            offset: query.offset,
            limit: query.limit,
            items,
            availability_by_short_name,
        })
    }
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/metadata/catalog_import.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/metadata/catalog_query.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/metadata/catalog_tests.rs"
));
