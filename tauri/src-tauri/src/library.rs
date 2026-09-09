//! Catalog queries and user-facing library domain operations.
//!
//! Generated metadata is never the storage owner for user favorites or history.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::{AppError, AppResult},
    metadata::{CatalogRepository, CloneFilter, MachinePage, MachineQuery},
    storage,
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_SEARCH_TEXT_LENGTH: usize = 256;
const MAX_FILTER_TEXT_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum CloneFilterRequest {
    #[default]
    All,
    ParentsOnly,
    ClonesOnly,
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
    pub include_devices: bool,
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
    .map_err(|error| {
        AppError::new(
            "CATALOG_QUERY_WORKER_FAILED",
            "The background catalog query worker did not complete normally.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?
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
        include_devices: request.include_devices,
        limit: request.limit,
        offset: request.offset,
    })
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

const fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::{validated_query, CloneFilterRequest, MachineSearchRequest, DEFAULT_PAGE_SIZE};

    #[test]
    fn query_defaults_are_bounded_and_parent_policy_is_explicit() {
        let query = validated_query(MachineSearchRequest {
            text: None,
            manufacturer: None,
            year: None,
            driver_status: None,
            clone_filter: CloneFilterRequest::All,
            include_devices: false,
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
        })
        .expect("default query");
        assert_eq!(query.limit, DEFAULT_PAGE_SIZE);
        assert!(!query.include_devices);
    }

    #[test]
    fn whitespace_filters_normalize_to_none() {
        let query = validated_query(MachineSearchRequest {
            text: Some("   ".to_owned()),
            manufacturer: Some(" Namco ".to_owned()),
            year: None,
            driver_status: None,
            clone_filter: CloneFilterRequest::ParentsOnly,
            include_devices: false,
            limit: 25,
            offset: 0,
        })
        .expect("normalized query");
        assert_eq!(query.text, None);
        assert_eq!(query.manufacturer.as_deref(), Some("Namco"));
    }

    #[test]
    fn unsupported_driver_status_is_rejected() {
        let error = validated_query(MachineSearchRequest {
            text: None,
            manufacturer: None,
            year: None,
            driver_status: Some("unknown".to_owned()),
            clone_filter: CloneFilterRequest::All,
            include_devices: false,
            limit: 25,
            offset: 0,
        })
        .expect_err("unsupported status must fail");
        assert_eq!(error.code, "CATALOG_QUERY_STATUS_INVALID");
    }
}
