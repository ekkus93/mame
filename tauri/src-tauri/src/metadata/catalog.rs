use std::{io::BufRead, path::Path};

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::{
    errors::{AppError, AppResult},
    mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
    storage,
};

use super::{
    model::{
        ListXmlSummary, MachineListItem, MachineMetadata, MachinePage, MetadataFreshness,
        MetadataGenerationSummary, MetadataStatus,
    },
    parser::parse_listxml,
};

const MAX_QUERY_PAGE_SIZE: u32 = 200;

pub(crate) struct CatalogRepository {
    connection: Connection,
}

impl CatalogRepository {
    pub(crate) fn open(path: &Path) -> AppResult<Self> {
        Ok(Self {
            connection: storage::open_catalog_connection(path)?,
        })
    }

    #[cfg(test)]
    fn memory() -> AppResult<Self> {
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

        let total_i64: i64 = self
            .connection
            .query_row(
                &format!("SELECT COUNT(*) FROM machines m WHERE {}", QUERY_WHERE),
                params![
                    generation_id,
                    search_pattern,
                    query.manufacturer.as_deref(),
                    query.year.as_deref(),
                    query.driver_status.as_deref(),
                    clone_filter,
                    include_devices,
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
                (SELECT COUNT(*) FROM displays d
                    WHERE d.generation_id = m.generation_id
                      AND d.machine_short_name = m.short_name) AS display_count,
                (SELECT COUNT(*) FROM machine_software_lists sl
                    WHERE sl.generation_id = m.generation_id
                      AND sl.machine_short_name = m.short_name) AS software_list_count
            FROM machines m
            WHERE {}
            ORDER BY m.description COLLATE NOCASE, m.short_name COLLATE NOCASE
            LIMIT ?8 OFFSET ?9"#,
            QUERY_WHERE
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
                    i64::from(query.limit),
                    i64::from(query.offset),
                ],
                stored_machine_item_from_row,
            )
            .map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;

        let mut items = Vec::new();
        for row in rows {
            let stored = row.map_err(|error| database_error("CATALOG_QUERY_FAILED", error))?;
            items.push(stored.into_item()?);
        }

        Ok(MachinePage {
            schema_version: 1,
            generation_id,
            total,
            offset: query.offset,
            limit: query.limit,
            items,
        })
    }
}

pub(crate) struct CatalogImport<'connection> {
    transaction: Transaction<'connection>,
    generation_id: i64,
    identity: MameExecutableIdentity,
    generated_at_epoch_ms: u64,
    machine_count: u64,
}

impl CatalogImport<'_> {
    pub(crate) fn import_listxml<R: BufRead>(&mut self, reader: R) -> AppResult<ListXmlSummary> {
        parse_listxml(reader, |machine| self.insert_machine(&machine))
    }

    pub(crate) fn finish(
        self,
        listxml: ListXmlSummary,
        imported_at_epoch_ms: u64,
    ) -> AppResult<MetadataGenerationSummary> {
        if self.machine_count != listxml.machine_count {
            return Err(AppError::new(
                "CATALOG_IMPORT_COUNT_MISMATCH",
                "The parser and catalog importer disagree on the number of imported machines.",
            )
            .with_details(serde_json::json!({
                "parsed": listxml.machine_count,
                "inserted": self.machine_count
            })));
        }

        let imported_at = to_i64(imported_at_epoch_ms, "importedAtEpochMs")?;
        let machine_count = to_i64(self.machine_count, "machineCount")?;
        self.transaction
            .execute(
                r#"UPDATE metadata_generation
                   SET listxml_build = ?2,
                       mame_config = ?3,
                       imported_at_epoch_ms = ?4,
                       machine_count = ?5
                   WHERE id = ?1"#,
                params![
                    self.generation_id,
                    listxml.build,
                    listxml.mame_config,
                    imported_at,
                    machine_count
                ],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;

        self.transaction
            .execute(
                "UPDATE metadata_generation SET active = 0 WHERE active = 1",
                [],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;
        let activated = self
            .transaction
            .execute(
                "UPDATE metadata_generation SET active = 1 WHERE id = ?1",
                [self.generation_id],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_FINALIZE_FAILED", error))?;
        if activated != 1 {
            return Err(AppError::new(
                "CATALOG_IMPORT_ACTIVATION_FAILED",
                "The completed metadata generation could not be activated.",
            ));
        }

        // Keep generated metadata bounded. User-owned tables do not reference a
        // generation, so pruning superseded generations cannot delete favorites,
        // collections, tags, or history.
        self.transaction
            .execute(
                "DELETE FROM metadata_generation WHERE active = 0 AND id <> ?1",
                [self.generation_id],
            )
            .map_err(|error| database_error("CATALOG_IMPORT_PRUNE_FAILED", error))?;

        let summary = MetadataGenerationSummary {
            generation_id: self.generation_id,
            source_kind: source_kind_token(self.identity.source).to_owned(),
            trust: trust_token(self.identity.trust).to_owned(),
            executable_path: self.identity.path.clone(),
            mame_version: self.identity.version.clone(),
            mame_build: self.identity.build.clone(),
            raw_version_line: self.identity.raw_version_line.clone(),
            listxml_build: listxml.build,
            mame_config: listxml.mame_config,
            generated_at_epoch_ms: self.generated_at_epoch_ms,
            imported_at_epoch_ms,
            machine_count: self.machine_count,
        };

        self.transaction
            .commit()
            .map_err(|error| database_error("CATALOG_IMPORT_COMMIT_FAILED", error))?;
        Ok(summary)
    }

    fn insert_machine(&mut self, machine: &MachineMetadata) -> AppResult<()> {
        let driver = machine.driver.as_ref();
        self.transaction
            .execute(
                r#"INSERT INTO machines(
                    generation_id, short_name, description, year, manufacturer, source_file,
                    clone_of, rom_of, is_bios, is_device, is_mechanical, runnable,
                    driver_status, driver_emulation, driver_cocktail, driver_savestate,
                    driver_requires_artwork, driver_unofficial, driver_no_sound_hardware,
                    driver_incomplete
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20
                )"#,
                params![
                    self.generation_id,
                    machine.short_name,
                    machine.description,
                    machine.year,
                    machine.manufacturer,
                    machine.source_file,
                    machine.clone_of,
                    machine.rom_of,
                    bool_i64(machine.is_bios),
                    bool_i64(machine.is_device),
                    bool_i64(machine.is_mechanical),
                    bool_i64(machine.runnable),
                    driver.map(|value| value.status.as_str()),
                    driver.map(|value| value.emulation.as_str()),
                    driver.and_then(|value| value.cocktail.as_deref()),
                    driver.map(|value| value.savestate.as_str()),
                    bool_i64(driver.is_some_and(|value| value.requires_artwork)),
                    bool_i64(driver.is_some_and(|value| value.unofficial)),
                    bool_i64(driver.is_some_and(|value| value.no_sound_hardware)),
                    bool_i64(driver.is_some_and(|value| value.incomplete)),
                ],
            )
            .map_err(|error| {
                AppError::new(
                    "CATALOG_METADATA_INSERT_FAILED",
                    "The machine metadata could not be inserted into the catalog.",
                )
                .with_details(serde_json::json!({
                    "machine": machine.short_name,
                    "cause": error.to_string()
                }))
            })?;

        for chip in &machine.chips {
            let clock_hz = chip
                .clock_hz
                .map(|value| to_i64(value, "chipClockHz"))
                .transpose()?;
            self.transaction
                .execute(
                    r#"INSERT INTO chips(
                        generation_id, machine_short_name, chip_type, name, tag, clock_hz
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        chip.chip_type,
                        chip.name,
                        chip.tag,
                        clock_hz
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        for display in &machine.displays {
            let pixel_clock_hz = display
                .pixel_clock_hz
                .map(|value| to_i64(value, "displayPixelClockHz"))
                .transpose()?;
            self.transaction
                .execute(
                    r#"INSERT INTO displays(
                        generation_id, machine_short_name, tag, display_type, rotate, flip_x,
                        width, height, refresh_hz, pixel_clock_hz
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        display.tag,
                        display.display_type,
                        display.rotate.map(i64::from),
                        bool_i64(display.flip_x),
                        display.width.map(i64::from),
                        display.height.map(i64::from),
                        display.refresh_hz,
                        pixel_clock_hz
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        for device in &machine.devices {
            self.transaction
                .execute(
                    r#"INSERT INTO devices(
                        generation_id, machine_short_name, device_type, tag, fixed_image,
                        mandatory, interface, instance_name, instance_brief_name
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        device.device_type,
                        device.tag,
                        device.fixed_image,
                        device.mandatory,
                        device.interface,
                        device.instance_name,
                        device.instance_brief_name
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            let device_id = self.transaction.last_insert_rowid();
            for extension in &device.extensions {
                self.transaction
                    .execute(
                        "INSERT INTO device_extensions(device_id, extension) VALUES (?1, ?2)",
                        params![device_id, extension],
                    )
                    .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            }
        }

        for software_list in &machine.software_lists {
            self.transaction
                .execute(
                    "INSERT OR IGNORE INTO software_lists(generation_id, name) VALUES (?1, ?2)",
                    params![self.generation_id, software_list.name],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
            self.transaction
                .execute(
                    r#"INSERT INTO machine_software_lists(
                        generation_id, machine_short_name, tag, software_list_name, status, filter
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        self.generation_id,
                        machine.short_name,
                        software_list.tag,
                        software_list.name,
                        software_list.status,
                        software_list.filter
                    ],
                )
                .map_err(|error| database_error("CATALOG_METADATA_INSERT_FAILED", error))?;
        }

        self.machine_count = self.machine_count.checked_add(1).ok_or_else(|| {
            AppError::new(
                "CATALOG_MACHINE_COUNT_OVERFLOW",
                "The imported catalog contains more machines than can be counted.",
            )
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloneFilter {
    All,
    ParentsOnly,
    ClonesOnly,
}

impl CloneFilter {
    fn as_token(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::ParentsOnly => "parents",
            Self::ClonesOnly => "clones",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MachineQuery {
    pub text: Option<String>,
    pub manufacturer: Option<String>,
    pub year: Option<String>,
    pub driver_status: Option<String>,
    pub clone_filter: CloneFilter,
    pub include_devices: bool,
    pub limit: u32,
    pub offset: u32,
}

const QUERY_WHERE: &str = r#"
m.generation_id = ?1
AND (
    ?2 IS NULL
    OR m.short_name LIKE ?2 ESCAPE '\' COLLATE NOCASE
    OR m.description LIKE ?2 ESCAPE '\' COLLATE NOCASE
    OR m.manufacturer LIKE ?2 ESCAPE '\' COLLATE NOCASE
)
AND (?3 IS NULL OR m.manufacturer = ?3 COLLATE NOCASE)
AND (?4 IS NULL OR m.year = ?4)
AND (?5 IS NULL OR m.driver_status = ?5)
AND (
    ?6 = 'all'
    OR (?6 = 'parents' AND m.clone_of IS NULL)
    OR (?6 = 'clones' AND m.clone_of IS NOT NULL)
)
AND (?7 = 1 OR m.is_device = 0)
"#;

#[derive(Debug)]
struct StoredGeneration {
    generation_id: i64,
    source_kind: String,
    trust: String,
    executable_path: String,
    mame_version: String,
    mame_build: Option<String>,
    raw_version_line: String,
    listxml_build: Option<String>,
    mame_config: Option<String>,
    generated_at_epoch_ms: i64,
    imported_at_epoch_ms: i64,
    machine_count: i64,
}

impl StoredGeneration {
    fn into_summary(self) -> AppResult<MetadataGenerationSummary> {
        Ok(MetadataGenerationSummary {
            generation_id: self.generation_id,
            source_kind: self.source_kind,
            trust: self.trust,
            executable_path: self.executable_path,
            mame_version: self.mame_version,
            mame_build: self.mame_build,
            raw_version_line: self.raw_version_line,
            listxml_build: self.listxml_build,
            mame_config: self.mame_config,
            generated_at_epoch_ms: to_u64(self.generated_at_epoch_ms, "generatedAtEpochMs")?,
            imported_at_epoch_ms: to_u64(self.imported_at_epoch_ms, "importedAtEpochMs")?,
            machine_count: to_u64(self.machine_count, "machineCount")?,
        })
    }
}

fn stored_generation_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredGeneration> {
    Ok(StoredGeneration {
        generation_id: row.get(0)?,
        source_kind: row.get(1)?,
        trust: row.get(2)?,
        executable_path: row.get(3)?,
        mame_version: row.get(4)?,
        mame_build: row.get(5)?,
        raw_version_line: row.get(6)?,
        listxml_build: row.get(7)?,
        mame_config: row.get(8)?,
        generated_at_epoch_ms: row.get(9)?,
        imported_at_epoch_ms: row.get(10)?,
        machine_count: row.get(11)?,
    })
}

#[derive(Debug)]
struct StoredMachineItem {
    short_name: String,
    description: String,
    year: Option<String>,
    manufacturer: Option<String>,
    source_file: Option<String>,
    clone_of: Option<String>,
    runnable: i64,
    is_device: i64,
    driver_status: Option<String>,
    display_count: i64,
    software_list_count: i64,
}

impl StoredMachineItem {
    fn into_item(self) -> AppResult<MachineListItem> {
        Ok(MachineListItem {
            short_name: self.short_name,
            description: self.description,
            year: self.year,
            manufacturer: self.manufacturer,
            source_file: self.source_file,
            clone_of: self.clone_of,
            runnable: sqlite_bool(self.runnable, "runnable")?,
            is_device: sqlite_bool(self.is_device, "isDevice")?,
            driver_status: self.driver_status,
            display_count: to_u32(self.display_count, "displayCount")?,
            software_list_count: to_u32(self.software_list_count, "softwareListCount")?,
        })
    }
}

fn stored_machine_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredMachineItem> {
    Ok(StoredMachineItem {
        short_name: row.get(0)?,
        description: row.get(1)?,
        year: row.get(2)?,
        manufacturer: row.get(3)?,
        source_file: row.get(4)?,
        clone_of: row.get(5)?,
        runnable: row.get(6)?,
        is_device: row.get(7)?,
        driver_status: row.get(8)?,
        display_count: row.get(9)?,
        software_list_count: row.get(10)?,
    })
}

fn generation_matches_identity(
    generation: &MetadataGenerationSummary,
    identity: &MameExecutableIdentity,
) -> bool {
    generation.executable_path == identity.path
        && generation.mame_version == identity.version
        && generation.mame_build == identity.build
        && generation.raw_version_line == identity.raw_version_line
}

fn source_kind_token(source: MameExecutableSourceKind) -> &'static str {
    match source {
        MameExecutableSourceKind::Bundled => "bundled",
        MameExecutableSourceKind::External => "external",
        MameExecutableSourceKind::DevelopmentTree => "developmentTree",
    }
}

fn trust_token(trust: MameExecutableTrust) -> &'static str {
    match trust {
        MameExecutableTrust::QualifiedBundled => "qualifiedBundled",
        MameExecutableTrust::UserConfigured => "userConfigured",
        MameExecutableTrust::Development => "development",
    }
}

fn bool_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn sqlite_bool(value: i64, field: &str) -> AppResult<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog contains an invalid boolean value.",
        )
        .with_details(serde_json::json!({
            "field": field,
            "value": invalid
        }))),
    }
}

fn to_i64(value: u64, field: &str) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_INTEGER_OVERFLOW",
            "A catalog integer value exceeds SQLite's supported range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn to_u64(value: i64, field: &str) -> AppResult<u64> {
    u64::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog contains an invalid negative integer value.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn to_u32(value: i64, field: &str) -> AppResult<u32> {
    u32::try_from(value).map_err(|_| {
        AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog contains an integer outside the supported result range.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('%');
    for character in value.chars() {
        match character {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(character);
            }
            other => escaped.push(other),
        }
    }
    escaped.push('%');
    escaped
}

fn database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The catalog database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust};

    use super::{CatalogRepository, CloneFilter, MachineQuery};
    use crate::metadata::model::MetadataFreshness;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/listxml-representative.xml");

    #[test]
    fn import_activates_complete_generation_and_queries_it() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let generation = repository
            .active_generation()
            .expect("active generation query")
            .expect("active generation");
        assert_eq!(generation.machine_count, 4);
        assert_eq!(
            generation.listxml_build.as_deref(),
            Some("0.288 test-fixture")
        );

        let page = repository
            .query_machines(&MachineQuery {
                text: Some("galax".to_owned()),
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                include_devices: false,
                limit: 50,
                offset: 0,
            })
            .expect("search query");
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 2);
        assert!(page.items.iter().all(|item| !item.is_device));
    }

    #[test]
    fn filters_parent_clone_year_manufacturer_and_status() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let page = repository
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: Some("Namco".to_owned()),
                year: Some("1979".to_owned()),
                driver_status: Some("good".to_owned()),
                clone_filter: CloneFilter::ClonesOnly,
                include_devices: false,
                limit: 25,
                offset: 0,
            })
            .expect("filtered query");
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].short_name, "galaxiana");
    }

    #[test]
    fn device_records_are_hidden_unless_explicitly_requested() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200)
            .expect("fixture import must succeed");

        let hidden = repository
            .query_machines(&MachineQuery {
                text: Some("Zilog Z80".to_owned()),
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                include_devices: false,
                limit: 25,
                offset: 0,
            })
            .expect("hidden device query");
        assert_eq!(hidden.total, 0);

        let visible = repository
            .query_machines(&MachineQuery {
                include_devices: true,
                ..MachineQuery {
                    text: Some("Zilog Z80".to_owned()),
                    manufacturer: None,
                    year: None,
                    driver_status: None,
                    clone_filter: CloneFilter::All,
                    include_devices: false,
                    limit: 25,
                    offset: 0,
                }
            })
            .expect("visible device query");
        assert_eq!(visible.total, 1);
        assert!(visible.items[0].is_device);
    }

    #[test]
    fn failed_replacement_import_preserves_previous_active_generation() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        let first =
            import_fixture(&mut repository, &identity, FIXTURE, 100, 200).expect("first import");

        let duplicate_xml = r#"<mame build="0.288 test-fixture" mameconfig="10">
          <machine name="dup"><description>One</description></machine>
          <machine name="dup"><description>Two</description></machine>
        </mame>"#;
        let mut replacement = repository
            .begin_import(&identity, 300)
            .expect("replacement transaction");
        let error = replacement
            .import_listxml(Cursor::new(duplicate_xml.as_bytes()))
            .expect_err("duplicate machine must fail import");
        assert_eq!(error.code, "CATALOG_METADATA_INSERT_FAILED");
        drop(replacement);

        let active = repository
            .active_generation()
            .expect("active generation query")
            .expect("old active generation retained");
        assert_eq!(active.generation_id, first.generation_id);
        assert_eq!(active.machine_count, 4);
    }

    #[test]
    fn regeneration_does_not_delete_user_owned_state() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &identity, FIXTURE, 100, 200).expect("first import");
        repository
            .connection
            .execute(
                "INSERT INTO user_favorites(machine_short_name, created_at_epoch_ms) VALUES ('galaxian', 123)",
                [],
            )
            .expect("seed favorite");
        repository
            .connection
            .execute(
                "INSERT INTO user_collections(name, created_at_epoch_ms, updated_at_epoch_ms) VALUES ('Classics', 124, 125)",
                [],
            )
            .expect("seed collection");
        let collection_id = repository.connection.last_insert_rowid();
        repository
            .connection
            .execute(
                "INSERT INTO user_collection_machines(collection_id, machine_short_name, added_at_epoch_ms) VALUES (?1, 'galaxian', 126)",
                rusqlite::params![collection_id],
            )
            .expect("seed collection membership");
        repository
            .connection
            .execute(
                "INSERT INTO recent_history(machine_short_name, software_item, launched_at_epoch_ms, succeeded) VALUES ('galaxian', NULL, 127, 1)",
                [],
            )
            .expect("seed recent history");

        let replacement_xml = r#"<mame build="0.288 test-fixture" mameconfig="10">
          <machine name="newmachine"><description>New Machine</description></machine>
        </mame>"#;
        import_fixture(&mut repository, &identity, replacement_xml, 300, 400)
            .expect("replacement import");

        let favorite_count: i64 = repository
            .connection
            .query_row("SELECT COUNT(*) FROM user_favorites", [], |row| row.get(0))
            .expect("favorite count");
        assert_eq!(favorite_count, 1);
        let favorite: String = repository
            .connection
            .query_row("SELECT machine_short_name FROM user_favorites", [], |row| {
                row.get(0)
            })
            .expect("favorite value");
        assert_eq!(favorite, "galaxian");

        let collection_machine: String = repository
            .connection
            .query_row(
                "SELECT machine_short_name FROM user_collection_machines WHERE collection_id = ?1",
                rusqlite::params![collection_id],
                |row| row.get(0),
            )
            .expect("collection membership");
        assert_eq!(collection_machine, "galaxian");

        let recent_machine: String = repository
            .connection
            .query_row(
                "SELECT machine_short_name FROM recent_history ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("recent history");
        assert_eq!(recent_machine, "galaxian");
    }

    #[test]
    fn changed_executable_identity_marks_generation_stale() {
        let mut repository = CatalogRepository::memory().expect("catalog repository");
        let current_identity = identity("/opt/mame/mame", "0.288", "test-fixture");
        import_fixture(&mut repository, &current_identity, FIXTURE, 100, 200)
            .expect("fixture import");

        let fresh = repository
            .metadata_status(current_identity.clone())
            .expect("freshness query");
        assert_eq!(fresh.freshness, MetadataFreshness::Fresh);

        let changed = identity("/opt/mame/mame-new", "0.289", "next-build");
        let stale = repository
            .metadata_status(changed)
            .expect("stale freshness query");
        assert_eq!(stale.freshness, MetadataFreshness::Stale);
    }

    #[test]
    fn query_limit_is_bounded() {
        let repository = CatalogRepository::memory().expect("catalog repository");
        let error = repository
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::ParentsOnly,
                include_devices: false,
                limit: 201,
                offset: 0,
            })
            .expect_err("unbounded page must fail");
        assert_eq!(error.code, "CATALOG_QUERY_LIMIT_INVALID");
    }

    fn import_fixture(
        repository: &mut CatalogRepository,
        identity: &MameExecutableIdentity,
        xml: &str,
        generated_at: u64,
        imported_at: u64,
    ) -> crate::errors::AppResult<crate::metadata::model::MetadataGenerationSummary> {
        let mut import = repository.begin_import(identity, generated_at)?;
        let summary = import.import_listxml(Cursor::new(xml.as_bytes()))?;
        import.finish(summary, imported_at)
    }

    fn identity(path: &str, version: &str, build: &str) -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::External,
            trust: MameExecutableTrust::UserConfigured,
            path: path.to_owned(),
            version: version.to_owned(),
            build: Some(build.to_owned()),
            raw_version_line: format!("{version} {build}"),
        }
    }
}
