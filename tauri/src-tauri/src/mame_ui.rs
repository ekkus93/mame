use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    config::settings_path,
    errors::{AppError, AppResult},
    library::audit::resolve_bulk_audit_context,
    metadata::{CatalogRepository, MachinePage, MameUiMachineFilter, MameUiMachineQuery},
    storage,
};

const DEFAULT_PAGE_SIZE: u32 = 100;
const MAX_SEARCH_TEXT_LENGTH: usize = 256;
const MAX_FILTER_VALUE_LENGTH: usize = 256;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MameUiMachineFilterRequest {
    #[default]
    All,
    Available,
    Unavailable,
    Working,
    NotWorking,
    Mechanical,
    NotMechanical,
    Favorites,
    Bios,
    NotBios,
    Parents,
    Clones,
    Manufacturer,
    Year,
    SourceFile,
    SaveSupported,
    SaveUnsupported,
    VerticalScreen,
    HorizontalScreen,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameUiMachineSearchRequest {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub filter: MameUiMachineFilterRequest,
    #[serde(default)]
    pub filter_value: Option<String>,
    #[serde(default = "default_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[tauri::command]
pub async fn query_mame_ui_library(
    request: MameUiMachineSearchRequest,
    app: AppHandle,
) -> AppResult<MachinePage> {
    let catalog_path = storage::catalog_path(&app)?;
    let settings_path = settings_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        let query = validated_query(request, &catalog_path, &settings_path)?;
        CatalogRepository::open(&catalog_path)?.query_mame_ui_machines(&query)
    })
    .await
    .map_err(|error| {
        AppError::new(
            "CATALOG_QUERY_WORKER_FAILED",
            "The MAME UI catalog query worker did not complete normally.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?
}

fn validated_query(
    request: MameUiMachineSearchRequest,
    catalog_path: &std::path::Path,
    settings_path: &std::path::Path,
) -> AppResult<MameUiMachineQuery> {
    let text = normalize_optional(request.text, "text", MAX_SEARCH_TEXT_LENGTH)?;
    let filter_value =
        normalize_optional(request.filter_value, "filterValue", MAX_FILTER_VALUE_LENGTH)?;
    let filter = map_filter(request.filter);
    let needs_value = matches!(
        filter,
        MameUiMachineFilter::Manufacturer
            | MameUiMachineFilter::Year
            | MameUiMachineFilter::SourceFile
    );
    if needs_value && filter_value.is_none() {
        return Err(AppError::new(
            "MAME_UI_FILTER_VALUE_REQUIRED",
            "The selected MAME UI filter requires a value.",
        ));
    }

    let (audit_identity_json, audit_content_paths_json) =
        match resolve_bulk_audit_context(catalog_path, settings_path) {
            Ok(context) => (
                Some(serde_json::to_string(&context.identity).map_err(serialization_error)?),
                Some(serde_json::to_string(&context.content_paths).map_err(serialization_error)?),
            ),
            Err(error) if availability_provenance_unavailable(&error) => (None, None),
            Err(error) => return Err(error),
        };

    Ok(MameUiMachineQuery {
        text,
        filter,
        filter_value,
        limit: request.limit,
        offset: request.offset,
        audit_identity_json,
        audit_content_paths_json,
    })
}

fn map_filter(filter: MameUiMachineFilterRequest) -> MameUiMachineFilter {
    match filter {
        MameUiMachineFilterRequest::All => MameUiMachineFilter::All,
        MameUiMachineFilterRequest::Available => MameUiMachineFilter::Available,
        MameUiMachineFilterRequest::Unavailable => MameUiMachineFilter::Unavailable,
        MameUiMachineFilterRequest::Working => MameUiMachineFilter::Working,
        MameUiMachineFilterRequest::NotWorking => MameUiMachineFilter::NotWorking,
        MameUiMachineFilterRequest::Mechanical => MameUiMachineFilter::Mechanical,
        MameUiMachineFilterRequest::NotMechanical => MameUiMachineFilter::NotMechanical,
        MameUiMachineFilterRequest::Favorites => MameUiMachineFilter::Favorites,
        MameUiMachineFilterRequest::Bios => MameUiMachineFilter::Bios,
        MameUiMachineFilterRequest::NotBios => MameUiMachineFilter::NotBios,
        MameUiMachineFilterRequest::Parents => MameUiMachineFilter::Parents,
        MameUiMachineFilterRequest::Clones => MameUiMachineFilter::Clones,
        MameUiMachineFilterRequest::Manufacturer => MameUiMachineFilter::Manufacturer,
        MameUiMachineFilterRequest::Year => MameUiMachineFilter::Year,
        MameUiMachineFilterRequest::SourceFile => MameUiMachineFilter::SourceFile,
        MameUiMachineFilterRequest::SaveSupported => MameUiMachineFilter::SaveSupported,
        MameUiMachineFilterRequest::SaveUnsupported => MameUiMachineFilter::SaveUnsupported,
        MameUiMachineFilterRequest::VerticalScreen => MameUiMachineFilter::VerticalScreen,
        MameUiMachineFilterRequest::HorizontalScreen => MameUiMachineFilter::HorizontalScreen,
    }
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
            "A MAME UI query field exceeds the supported length.",
        )
        .with_details(serde_json::json!({ "field": field, "maxLength": max_length })));
    }
    Ok(Some(value.to_owned()))
}

fn availability_provenance_unavailable(error: &AppError) -> bool {
    matches!(
        error.code.as_str(),
        "CATALOG_BUNDLED_EXECUTABLE_RESOLUTION_REQUIRED"
            | "MAME_METADATA_STALE"
            | "MAME_EXECUTABLE_PATH_EMPTY"
            | "MAME_EXECUTABLE_NOT_FOUND"
            | "MAME_EXECUTABLE_PATH_INVALID"
            | "MAME_EXECUTABLE_METADATA_FAILED"
            | "MAME_EXECUTABLE_NOT_FILE"
            | "MAME_EXECUTABLE_NOT_EXECUTABLE"
            | "MAME_EXECUTABLE_LAUNCH_FAILED"
            | "MAME_VERSION_UNRECOGNIZED"
            | "MAME_VERSION_PROBE_WAIT_FAILED"
            | "MAME_VERSION_PROBE_TIMEOUT"
            | "MAME_VERSION_PROBE_OUTPUT_FAILED"
            | "MAME_VERSION_PROBE_FAILED"
    )
}

fn serialization_error(error: serde_json::Error) -> AppError {
    AppError::new(
        "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED",
        "Current MAME audit provenance could not be serialized for the MAME UI query.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

const fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::{map_filter, MameUiMachineFilterRequest};
    use crate::metadata::MameUiMachineFilter;

    #[test]
    fn request_filters_map_one_to_one_to_catalog_filters() {
        assert_eq!(
            map_filter(MameUiMachineFilterRequest::Favorites),
            MameUiMachineFilter::Favorites
        );
        assert_eq!(
            map_filter(MameUiMachineFilterRequest::HorizontalScreen),
            MameUiMachineFilter::HorizontalScreen
        );
    }
}
