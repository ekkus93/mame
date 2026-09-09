//! Catalog queries and user-facing library domain operations.
//!
//! Generated metadata is never the storage owner for user favorites or history.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    errors::{AppError, AppResult},
    mame::{inspect_executable, MameExecutableIdentity, MameExecutableSource},
    metadata::{
        CatalogRepository, CloneFilter, FavoritePage, FavoriteState, MachineDetail, MachinePage,
        MachineQuery, MachineSort, MetadataGenerationSummary,
    },
    sessions::{self, SessionSnapshot, SessionSupervisor},
    storage,
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_SEARCH_TEXT_LENGTH: usize = 256;
const MAX_FILTER_TEXT_LENGTH: usize = 128;
const MAX_MACHINE_SHORT_NAME_LENGTH: usize = 16;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum CloneFilterRequest {
    #[default]
    All,
    ParentsOnly,
    ClonesOnly,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MachineSortRequest {
    #[default]
    DescriptionAsc,
    DescriptionDesc,
    ShortNameAsc,
    YearAsc,
    YearDesc,
    ManufacturerAsc,
    ManufacturerDesc,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineSearchRequest {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub driver_status: Option<String>,
    #[serde(default)]
    pub clone_filter: CloneFilterRequest,
    #[serde(default)]
    pub sort: MachineSortRequest,
    #[serde(default)]
    pub include_devices: bool,
    #[serde(default = "default_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineDetailRequest {
    pub short_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLibraryMachineRequest {
    pub short_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetLibraryFavoriteRequest {
    pub short_name: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FavoritePageRequest {
    #[serde(default = "default_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[tauri::command]
pub async fn query_mame_library(
    request: MachineSearchRequest,
    app: AppHandle,
) -> AppResult<MachinePage> {
    let query = validated_query(request)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.query_machines(&query)
    })
    .await
    .map_err(catalog_worker_error)?
}

#[tauri::command]
pub async fn get_mame_machine_detail(
    request: MachineDetailRequest,
    app: AppHandle,
) -> AppResult<MachineDetail> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.machine_detail(&short_name)
    })
    .await
    .map_err(catalog_worker_error)?
}

#[tauri::command]
pub async fn get_library_favorite(
    request: MachineDetailRequest,
    app: AppHandle,
) -> AppResult<FavoriteState> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.favorite_state(&short_name)
    })
    .await
    .map_err(catalog_worker_error)?
}

#[tauri::command]
pub async fn set_library_favorite(
    request: SetLibraryFavoriteRequest,
    app: AppHandle,
) -> AppResult<FavoriteState> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let favorite = request.favorite;
    let created_at_epoch_ms = now_epoch_ms()?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.set_favorite(
            &short_name,
            favorite,
            created_at_epoch_ms,
        )
    })
    .await
    .map_err(catalog_worker_error)?
}

#[tauri::command]
pub async fn query_library_favorites(
    request: FavoritePageRequest,
    app: AppHandle,
) -> AppResult<FavoritePage> {
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.query_favorites(request.limit, request.offset)
    })
    .await
    .map_err(catalog_worker_error)?
}

#[tauri::command]
pub fn launch_library_machine(
    request: LaunchLibraryMachineRequest,
    app: AppHandle,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<SessionSnapshot> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;
    let generation = CatalogRepository::open(&catalog_path)?
        .active_generation()?
        .ok_or_else(|| {
            AppError::new(
                "MAME_METADATA_NOT_READY",
                "No successfully imported MAME metadata generation is active.",
            )
        })?;

    let source = launch_source_from_generation(&generation)?;
    let current_identity = inspect_executable(source.clone())?;
    ensure_generation_matches_executable(&generation, &current_identity)?;

    sessions::launch_mame_with_source(source, short_name, None, Vec::new(), supervisor, app)
}

fn launch_source_from_generation(
    generation: &MetadataGenerationSummary,
) -> AppResult<MameExecutableSource> {
    match (generation.source_kind.as_str(), generation.trust.as_str()) {
        ("external", "userConfigured") => {
            Ok(MameExecutableSource::external(&generation.executable_path))
        }
        ("developmentTree", "development") => Ok(MameExecutableSource::development_tree(
            &generation.executable_path,
        )),
        ("bundled", "qualifiedBundled") => Err(AppError::new(
            "CATALOG_BUNDLED_EXECUTABLE_RESOLUTION_REQUIRED",
            "Bundled MAME launch requires package-owned executable resolution.",
        )),
        (source_kind, trust) => Err(AppError::new(
            "CATALOG_EXECUTABLE_PROVENANCE_INVALID",
            "The active catalog has an invalid executable source/trust pairing.",
        )
        .with_details(serde_json::json!({
            "sourceKind": source_kind,
            "trust": trust
        }))),
    }
}

fn ensure_generation_matches_executable(
    generation: &MetadataGenerationSummary,
    identity: &MameExecutableIdentity,
) -> AppResult<()> {
    if generation.executable_path == identity.path
        && generation.mame_version == identity.version
        && generation.mame_build == identity.build
        && generation.raw_version_line == identity.raw_version_line
    {
        return Ok(());
    }

    Err(AppError::new(
        "MAME_METADATA_STALE",
        "The selected MAME executable no longer matches the active metadata generation.",
    )
    .with_details(serde_json::json!({
        "catalogPath": generation.executable_path,
        "catalogVersion": generation.mame_version,
        "catalogBuild": generation.mame_build,
        "currentPath": identity.path,
        "currentVersion": identity.version,
        "currentBuild": identity.build
    })))
}

fn validated_query(request: MachineSearchRequest) -> AppResult<MachineQuery> {
    let text = normalize_optional(request.text, "text", MAX_SEARCH_TEXT_LENGTH)?;
    let manufacturer =
        normalize_optional(request.manufacturer, "manufacturer", MAX_FILTER_TEXT_LENGTH)?;
    let year = normalize_optional(request.year, "year", 32)?;
    let driver_status = normalize_optional(
        request.driver_status,
        "driverStatus",
        MAX_FILTER_TEXT_LENGTH,
    )?;

    if let Some(status) = driver_status.as_deref() {
        if !matches!(status, "good" | "imperfect" | "preliminary") {
            return Err(AppError::new(
                "CATALOG_QUERY_STATUS_INVALID",
                "The requested MAME driver status filter is not supported.",
            )
            .with_details(serde_json::json!({ "driverStatus": status })));
        }
    }

    Ok(MachineQuery {
        text,
        manufacturer,
        year,
        driver_status,
        clone_filter: match request.clone_filter {
            CloneFilterRequest::All => CloneFilter::All,
            CloneFilterRequest::ParentsOnly => CloneFilter::ParentsOnly,
            CloneFilterRequest::ClonesOnly => CloneFilter::ClonesOnly,
        },
        sort: match request.sort {
            MachineSortRequest::DescriptionAsc => MachineSort::DescriptionAsc,
            MachineSortRequest::DescriptionDesc => MachineSort::DescriptionDesc,
            MachineSortRequest::ShortNameAsc => MachineSort::ShortNameAsc,
            MachineSortRequest::YearAsc => MachineSort::YearAsc,
            MachineSortRequest::YearDesc => MachineSort::YearDesc,
            MachineSortRequest::ManufacturerAsc => MachineSort::ManufacturerAsc,
            MachineSortRequest::ManufacturerDesc => MachineSort::ManufacturerDesc,
        },
        include_devices: request.include_devices,
        limit: request.limit,
        offset: request.offset,
    })
}

fn validate_machine_short_name(value: String) -> AppResult<String> {
    let value = value.trim();
    let valid = !value.is_empty()
        && value.chars().count() <= MAX_MACHINE_SHORT_NAME_LENGTH
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if !valid {
        return Err(AppError::new(
            "CATALOG_MACHINE_IDENTIFIER_INVALID",
            "The machine short name is not a valid MAME identifier.",
        )
        .with_details(serde_json::json!({ "shortName": value })));
    }
    Ok(value.to_owned())
}

fn normalize_optional(
    value: Option<String>,
    field: &str,
    max_length: usize,
) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_length {
        return Err(AppError::new(
            "CATALOG_QUERY_TEXT_TOO_LONG",
            "A catalog query field exceeds the supported length.",
        )
        .with_details(serde_json::json!({
            "field": field,
            "maxLength": max_length
        })));
    }
    Ok(Some(value.to_owned()))
}

fn catalog_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "CATALOG_QUERY_WORKER_FAILED",
        "The background catalog query worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn now_epoch_ms() -> AppResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                "SYSTEM_CLOCK_INVALID",
                "The system clock is earlier than the Unix epoch.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    u64::try_from(duration.as_millis()).map_err(|_| {
        AppError::new(
            "SYSTEM_CLOCK_OUT_OF_RANGE",
            "The current system timestamp cannot be represented by the application.",
        )
    })
}

const fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_generation_matches_executable, launch_source_from_generation,
        validate_machine_short_name, validated_query, CloneFilterRequest, MachineSearchRequest,
        MachineSortRequest, DEFAULT_PAGE_SIZE,
    };
    use crate::{
        mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
        metadata::MetadataGenerationSummary,
    };

    fn generation(source_kind: &str, trust: &str) -> MetadataGenerationSummary {
        MetadataGenerationSummary {
            generation_id: 1,
            source_kind: source_kind.to_owned(),
            trust: trust.to_owned(),
            executable_path: "/opt/mame/mame".to_owned(),
            mame_version: "0.288".to_owned(),
            mame_build: Some("test-fixture".to_owned()),
            raw_version_line: "0.288 test-fixture".to_owned(),
            listxml_build: Some("0.288 test-fixture".to_owned()),
            mame_config: Some("10".to_owned()),
            generated_at_epoch_ms: 100,
            imported_at_epoch_ms: 200,
            machine_count: 4,
        }
    }

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

    fn base_request() -> MachineSearchRequest {
        MachineSearchRequest {
            text: None,
            manufacturer: None,
            year: None,
            driver_status: None,
            clone_filter: CloneFilterRequest::All,
            sort: MachineSortRequest::DescriptionAsc,
            include_devices: false,
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
        }
    }

    #[test]
    fn persisted_catalog_cannot_self_assert_bundled_trust() {
        let error = launch_source_from_generation(&generation("bundled", "qualifiedBundled"))
            .expect_err("bundled launch must require package-owned resolution");
        assert_eq!(error.code, "CATALOG_BUNDLED_EXECUTABLE_RESOLUTION_REQUIRED");
    }

    #[test]
    fn persisted_source_and_trust_pair_must_match() {
        let error = launch_source_from_generation(&generation("external", "qualifiedBundled"))
            .expect_err("mismatched persisted provenance must fail");
        assert_eq!(error.code, "CATALOG_EXECUTABLE_PROVENANCE_INVALID");

        let source = launch_source_from_generation(&generation("external", "userConfigured"))
            .expect("valid external provenance");
        assert_eq!(source.kind(), MameExecutableSourceKind::External);
        assert_eq!(source.trust(), MameExecutableTrust::UserConfigured);
    }

    #[test]
    fn launch_requires_catalog_identity_to_match_current_executable() {
        let generation = generation("external", "userConfigured");
        ensure_generation_matches_executable(&generation, &identity())
            .expect("matching identity must pass");

        let mut changed = identity();
        changed.version = "0.289".to_owned();
        let error = ensure_generation_matches_executable(&generation, &changed)
            .expect_err("changed executable must make catalog stale");
        assert_eq!(error.code, "MAME_METADATA_STALE");
    }

    #[test]
    fn query_defaults_are_bounded_and_parent_policy_is_explicit() {
        let query = validated_query(base_request()).expect("default query");
        assert_eq!(query.limit, DEFAULT_PAGE_SIZE);
        assert!(!query.include_devices);
    }

    #[test]
    fn whitespace_filters_normalize_to_none() {
        let mut request = base_request();
        request.text = Some("   ".to_owned());
        request.manufacturer = Some(" Namco ".to_owned());
        request.clone_filter = CloneFilterRequest::ParentsOnly;
        request.limit = 25;
        let query = validated_query(request).expect("normalized query");
        assert_eq!(query.text, None);
        assert_eq!(query.manufacturer.as_deref(), Some("Namco"));
    }

    #[test]
    fn unsupported_driver_status_is_rejected() {
        let mut request = base_request();
        request.driver_status = Some("unknown".to_owned());
        let error = validated_query(request).expect_err("unsupported status must fail");
        assert_eq!(error.code, "CATALOG_QUERY_STATUS_INVALID");
    }

    #[test]
    fn machine_identifier_validation_matches_launch_policy() {
        assert_eq!(
            validate_machine_short_name("apple2e".to_owned()).expect("valid identifier"),
            "apple2e"
        );
        for invalid in [
            "",
            "Apple2e",
            "../mame",
            "name-with-dash",
            "abcdefghijklmnopq",
        ] {
            let error = validate_machine_short_name(invalid.to_owned())
                .expect_err("invalid identifier must fail");
            assert_eq!(error.code, "CATALOG_MACHINE_IDENTIFIER_INVALID");
        }
    }
}
