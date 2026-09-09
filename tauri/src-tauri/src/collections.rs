use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::{AppError, AppResult},
    metadata::{
        CatalogRepository, CollectionListPage, CollectionMemberPage, CollectionMembershipState,
        CollectionSummary,
    },
    storage,
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_COLLECTION_NAME_LENGTH: usize = 80;
const MAX_MACHINE_SHORT_NAME_LENGTH: usize = 16;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenameCollectionRequest {
    pub collection_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRequest {
    pub collection_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPageRequest {
    pub collection_id: i64,
    #[serde(default = "default_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionListRequest {
    #[serde(default = "default_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetCollectionMachineRequest {
    pub collection_id: i64,
    pub short_name: String,
    pub member: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCollectionResult {
    pub schema_version: u32,
    pub collection_id: i64,
    pub deleted: bool,
}

#[tauri::command]
pub async fn create_library_collection(
    request: CreateCollectionRequest,
    app: AppHandle,
) -> AppResult<CollectionSummary> {
    let name = validate_collection_name(request.name)?;
    let now = now_epoch_ms()?;
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.create_collection(&name, now)
    })
    .await
    .map_err(collection_worker_error)?
}

#[tauri::command]
pub async fn rename_library_collection(
    request: RenameCollectionRequest,
    app: AppHandle,
) -> AppResult<CollectionSummary> {
    let collection_id = validate_collection_id(request.collection_id)?;
    let name = validate_collection_name(request.name)?;
    let now = now_epoch_ms()?;
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.rename_collection(collection_id, &name, now)
    })
    .await
    .map_err(collection_worker_error)?
}

#[tauri::command]
pub async fn delete_library_collection(
    request: CollectionRequest,
    app: AppHandle,
) -> AppResult<DeleteCollectionResult> {
    let collection_id = validate_collection_id(request.collection_id)?;
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.delete_collection(collection_id)?;
        Ok(DeleteCollectionResult {
            schema_version: 1,
            collection_id,
            deleted: true,
        })
    })
    .await
    .map_err(collection_worker_error)?
}

#[tauri::command]
pub async fn query_library_collections(
    request: CollectionListRequest,
    app: AppHandle,
) -> AppResult<CollectionListPage> {
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.query_collections(request.limit, request.offset)
    })
    .await
    .map_err(collection_worker_error)?
}

#[tauri::command]
pub async fn query_library_collection_members(
    request: CollectionPageRequest,
    app: AppHandle,
) -> AppResult<CollectionMemberPage> {
    let collection_id = validate_collection_id(request.collection_id)?;
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.query_collection_members(
            collection_id,
            request.limit,
            request.offset,
        )
    })
    .await
    .map_err(collection_worker_error)?
}

#[tauri::command]
pub async fn set_library_collection_machine(
    request: SetCollectionMachineRequest,
    app: AppHandle,
) -> AppResult<CollectionMembershipState> {
    let collection_id = validate_collection_id(request.collection_id)?;
    let short_name = validate_machine_short_name(request.short_name)?;
    let member = request.member;
    // Removing a member must remain possible even if the wall clock is invalid.
    let now = if member { now_epoch_ms()? } else { 0 };
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.set_collection_machine(
            collection_id,
            &short_name,
            member,
            now,
        )
    })
    .await
    .map_err(collection_worker_error)?
}

fn validate_collection_name(value: String) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty()
        || value.chars().count() > MAX_COLLECTION_NAME_LENGTH
        || value.chars().any(char::is_control)
    {
        return Err(AppError::new(
            "CATALOG_COLLECTION_NAME_INVALID",
            "Collection names must be non-empty, printable text within the supported length.",
        )
        .with_details(serde_json::json!({ "maxLength": MAX_COLLECTION_NAME_LENGTH })));
    }
    Ok(value.to_owned())
}

fn validate_collection_id(value: i64) -> AppResult<i64> {
    if value <= 0 {
        return Err(AppError::new(
            "CATALOG_COLLECTION_ID_INVALID",
            "The collection identifier must be a positive integer.",
        ));
    }
    Ok(value)
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

fn collection_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "CATALOG_COLLECTION_WORKER_FAILED",
        "The background collection worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

const fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::{validate_collection_id, validate_collection_name, validate_machine_short_name};

    #[test]
    fn collection_name_is_trimmed_and_bounded() {
        assert_eq!(
            validate_collection_name("  Arcade Classics  ".to_owned()).expect("valid name"),
            "Arcade Classics"
        );
        assert_eq!(
            validate_collection_name("\n".to_owned())
                .expect_err("empty name must fail")
                .code,
            "CATALOG_COLLECTION_NAME_INVALID"
        );
    }

    #[test]
    fn collection_id_must_be_positive() {
        assert_eq!(
            validate_collection_id(0)
                .expect_err("zero id must fail")
                .code,
            "CATALOG_COLLECTION_ID_INVALID"
        );
    }

    #[test]
    fn collection_machine_identifier_uses_launch_policy() {
        assert_eq!(
            validate_machine_short_name("galaxian".to_owned()).expect("valid machine"),
            "galaxian"
        );
        assert_eq!(
            validate_machine_short_name("../mame".to_owned())
                .expect_err("invalid machine must fail")
                .code,
            "CATALOG_MACHINE_IDENTIFIER_INVALID"
        );
    }
}
