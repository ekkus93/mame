#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MachineSort {
    DescriptionAsc,
    DescriptionDesc,
    ShortNameAsc,
    YearAsc,
    YearDesc,
    ManufacturerAsc,
    ManufacturerDesc,
}

impl MachineSort {
    fn order_by(self) -> &'static str {
        match self {
            Self::DescriptionAsc => "m.description COLLATE NOCASE ASC, m.short_name COLLATE NOCASE ASC",
            Self::DescriptionDesc => "m.description COLLATE NOCASE DESC, m.short_name COLLATE NOCASE ASC",
            Self::ShortNameAsc => "m.short_name COLLATE NOCASE ASC",
            Self::YearAsc => "m.year IS NULL, m.year ASC, m.description COLLATE NOCASE ASC",
            Self::YearDesc => "m.year IS NULL, m.year DESC, m.description COLLATE NOCASE ASC",
            Self::ManufacturerAsc => "m.manufacturer IS NULL, m.manufacturer COLLATE NOCASE ASC, m.description COLLATE NOCASE ASC",
            Self::ManufacturerDesc => "m.manufacturer IS NULL, m.manufacturer COLLATE NOCASE DESC, m.description COLLATE NOCASE ASC",
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AvailabilityFilter {
    All,
    Available,
    Missing,
    Unknown,
}

impl AvailabilityFilter {
    fn as_token(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MachineAvailabilityQuery {
    pub filter: AvailabilityFilter,
    pub mame_identity_json: Option<String>,
    pub content_paths_json: Option<String>,
}

impl MachineAvailabilityQuery {
    pub(crate) fn unverified(filter: AvailabilityFilter) -> Self {
        Self {
            filter,
            mame_identity_json: None,
            content_paths_json: None,
        }
    }

    pub(crate) fn current(
        filter: AvailabilityFilter,
        mame_identity_json: String,
        content_paths_json: String,
    ) -> Self {
        Self {
            filter,
            mame_identity_json: Some(mame_identity_json),
            content_paths_json: Some(content_paths_json),
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
    pub sort: MachineSort,
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
AND (
    ?10 = 'all'
    OR (?10 = 'available' AND ar.classification IN ('complete', 'bestAvailable'))
    OR (?10 = 'missing' AND ar.classification IN ('missingRequired', 'incorrect', 'mixedFailure'))
    OR (?10 = 'unknown' AND (ar.classification IS NULL OR ar.classification = 'unknown'))
)
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
    availability: String,
    display_count: i64,
    software_list_count: i64,
}

impl StoredMachineItem {
    fn into_item(self) -> AppResult<(MachineListItem, MachineAvailability)> {
        let availability = machine_availability_from_token(&self.availability)?;
        Ok((
            MachineListItem {
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
            },
            availability,
        ))
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
        availability: row.get(9)?,
        display_count: row.get(10)?,
        software_list_count: row.get(11)?,
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

fn machine_availability_from_token(value: &str) -> AppResult<MachineAvailability> {
    match value {
        "available" => Ok(MachineAvailability::Available),
        "missing" => Ok(MachineAvailability::Missing),
        "unknown" => Ok(MachineAvailability::Unknown),
        invalid => Err(AppError::new(
            "CATALOG_QUERY_RESULT_INVALID",
            "The catalog query produced an invalid machine availability value.",
        )
        .with_details(serde_json::json!({ "availability": invalid }))),
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
