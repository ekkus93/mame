use std::io::Cursor;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    errors::{AppError, AppResult},
    mame::{
        get_software_list_xml, inspect_executable, validate_short_identifier,
        validate_software_identifier, validate_software_list_identifier, MameExecutableIdentity,
        MameExecutableSource,
    },
    metadata::{
        parse_software_list_page, CatalogRepository, MetadataGenerationSummary, SoftwareItemSummary,
    },
    sessions::{self, SessionSnapshot, SessionSupervisor},
    storage,
};

const DEFAULT_SOFTWARE_PAGE_SIZE: u32 = 50;
const MAX_SOFTWARE_PAGE_SIZE: u32 = 200;
const MAX_SOFTWARE_SEARCH_LENGTH: usize = 256;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareListQueryRequest {
    pub short_name: String,
    pub software_list: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default = "default_software_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareListPage {
    pub schema_version: u32,
    pub machine_short_name: String,
    pub software_list_name: String,
    pub software_list_description: Option<String>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<SoftwareItemSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchLibrarySoftwareRequest {
    pub short_name: String,
    pub software_list: String,
    pub software_item: String,
}

#[tauri::command]
pub async fn query_mame_software_list(
    request: SoftwareListQueryRequest,
    app: AppHandle,
) -> AppResult<SoftwareListPage> {
    let request = validate_query_request(request)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        let repository = CatalogRepository::open(&catalog_path)?;
        let generation = active_generation(&repository)?;
        ensure_machine_software_list_association(
            &repository,
            &request.short_name,
            &request.software_list,
        )?;
        let source = validated_generation_source(&generation)?;
        let xml = get_software_list_xml(&source, &request.software_list)?;
        let parsed = parse_software_list_page(
            Cursor::new(xml.as_bytes()),
            &request.software_list,
            request.text.as_deref(),
            request.limit,
            request.offset,
        )?;

        Ok(SoftwareListPage {
            schema_version: 1,
            machine_short_name: request.short_name,
            software_list_name: parsed.name,
            software_list_description: parsed.description,
            total: parsed.total,
            offset: request.offset,
            limit: request.limit,
            items: parsed.items,
        })
    })
    .await
    .map_err(software_worker_error)?
}

#[tauri::command]
pub fn launch_library_software(
    request: LaunchLibrarySoftwareRequest,
    app: AppHandle,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<SessionSnapshot> {
    let short_name = validated_short_identifier("machine", request.short_name)?;
    let software_list = validated_software_list(request.software_list)?;
    let software_item = validated_short_identifier("softwareItem", request.software_item)?;
    let catalog_path = storage::catalog_path(&app)?;
    let repository = CatalogRepository::open(&catalog_path)?;
    let generation = active_generation(&repository)?;
    ensure_machine_software_list_association(&repository, &short_name, &software_list)?;
    let source = validated_generation_source(&generation)?;

    let software = format!("{software_list}:{software_item}");
    validate_software_identifier(&software)?;
    sessions::launch_mame_with_source(
        source,
        short_name,
        Some(software),
        Vec::new(),
        supervisor,
        app,
    )
}

fn validate_query_request(
    request: SoftwareListQueryRequest,
) -> AppResult<SoftwareListQueryRequest> {
    let short_name = validated_short_identifier("machine", request.short_name)?;
    let software_list = validated_software_list(request.software_list)?;
    let text = request
        .text
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if text
        .as_ref()
        .is_some_and(|value| value.chars().count() > MAX_SOFTWARE_SEARCH_LENGTH)
    {
        return Err(AppError::new(
            "MAME_SOFTWARE_SEARCH_TOO_LONG",
            "The software-list search text exceeds the supported length.",
        )
        .with_details(serde_json::json!({
            "maxLength": MAX_SOFTWARE_SEARCH_LENGTH
        })));
    }
    if request.limit == 0 || request.limit > MAX_SOFTWARE_PAGE_SIZE {
        return Err(AppError::new(
            "MAME_SOFTWARE_PAGE_LIMIT_INVALID",
            format!("Software-list page size must be between 1 and {MAX_SOFTWARE_PAGE_SIZE}."),
        )
        .with_details(serde_json::json!({
            "limit": request.limit,
            "maxLimit": MAX_SOFTWARE_PAGE_SIZE
        })));
    }
    Ok(SoftwareListQueryRequest {
        short_name,
        software_list,
        text,
        limit: request.limit,
        offset: request.offset,
    })
}

fn validated_short_identifier(field: &str, value: String) -> AppResult<String> {
    let value = value.trim();
    validate_short_identifier(field, value)?;
    Ok(value.to_owned())
}

fn validated_software_list(value: String) -> AppResult<String> {
    let value = value.trim();
    validate_software_list_identifier(value)?;
    Ok(value.to_owned())
}

fn active_generation(repository: &CatalogRepository) -> AppResult<MetadataGenerationSummary> {
    repository.active_generation()?.ok_or_else(|| {
        AppError::new(
            "MAME_METADATA_NOT_READY",
            "No successfully imported MAME metadata generation is active.",
        )
    })
}

fn ensure_machine_software_list_association(
    repository: &CatalogRepository,
    short_name: &str,
    software_list: &str,
) -> AppResult<()> {
    let detail = repository.machine_detail(short_name)?;
    if detail
        .software_lists
        .iter()
        .any(|association| association.name == software_list)
    {
        return Ok(());
    }
    Err(AppError::new(
        "MAME_SOFTWARE_LIST_NOT_ASSOCIATED",
        "The requested software list is not associated with the selected machine.",
    )
    .with_details(serde_json::json!({
        "shortName": short_name,
        "softwareList": software_list
    })))
}

fn validated_generation_source(
    generation: &MetadataGenerationSummary,
) -> AppResult<MameExecutableSource> {
    let source = launch_source_from_generation(generation)?;
    let identity = inspect_executable(source.clone())?;
    ensure_generation_matches_executable(generation, &identity)?;
    Ok(source)
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

fn default_software_page_size() -> u32 {
    DEFAULT_SOFTWARE_PAGE_SIZE
}

fn software_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_SOFTWARE_WORKER_FAILED",
        "The background software-list worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::{validate_query_request, SoftwareListQueryRequest};

    #[test]
    fn query_validation_is_bounded_and_normalizes_search_text() {
        let request = validate_query_request(SoftwareListQueryRequest {
            short_name: " apple2e ".to_owned(),
            software_list: " apple2_flop_clcracked ".to_owned(),
            text: Some("  archon  ".to_owned()),
            limit: 50,
            offset: 0,
        })
        .expect("valid request");
        assert_eq!(request.short_name, "apple2e");
        assert_eq!(request.software_list, "apple2_flop_clcracked");
        assert_eq!(request.text.as_deref(), Some("archon"));
    }

    #[test]
    fn query_validation_rejects_unbounded_page_size() {
        let error = validate_query_request(SoftwareListQueryRequest {
            short_name: "apple2e".to_owned(),
            software_list: "apple2_flop_orig".to_owned(),
            text: None,
            limit: 201,
            offset: 0,
        })
        .expect_err("oversized page must fail");
        assert_eq!(error.code, "MAME_SOFTWARE_PAGE_LIMIT_INVALID");
    }
}
