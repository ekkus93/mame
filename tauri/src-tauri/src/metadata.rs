//! MAME-generated metadata acquisition, parsing, import, and freshness tracking.
//!
//! MAME remains the metadata source of truth. The Rust backend invokes
//! `-listxml`, parses it as a stream, and activates a new SQLite generation only
//! after the complete process and import succeed.

mod catalog;
mod detail;
mod favorites;
mod generator;
mod model;
mod parser;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::{AppError, AppResult},
    mame::MameExecutableSource,
    sessions::{MameExecutableRequest, MameExecutableSelectionKind},
    storage,
};

pub use favorites::{FavoriteEntry, FavoritePage, FavoriteState};
pub use model::{
    MachineDetail, MachineDisplayInfo, MachineListItem, MachinePage, MetadataFreshness,
    MetadataGenerationSummary, MetadataRefreshResult, MetadataStatus,
};

pub(crate) use catalog::{CatalogRepository, CloneFilter, MachineQuery, MachineSort};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RefreshMameMetadataRequest {
    pub executable: MameExecutableRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatusRequest {
    pub executable: MameExecutableRequest,
}

#[tauri::command]
pub async fn refresh_mame_metadata(
    request: RefreshMameMetadataRequest,
    app: AppHandle,
) -> AppResult<MetadataRefreshResult> {
    let source = executable_source(&request.executable);
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || generator::refresh_catalog(source, &catalog_path))
        .await
        .map_err(metadata_worker_error)?
}

#[tauri::command]
pub async fn get_mame_metadata_status(
    request: MetadataStatusRequest,
    app: AppHandle,
) -> AppResult<MetadataStatus> {
    let source = executable_source(&request.executable);
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || generator::metadata_status(source, &catalog_path))
        .await
        .map_err(metadata_worker_error)?
}

fn executable_source(request: &MameExecutableRequest) -> MameExecutableSource {
    match request.source {
        MameExecutableSelectionKind::External => MameExecutableSource::external(&request.path),
        MameExecutableSelectionKind::DevelopmentTree => {
            MameExecutableSource::development_tree(&request.path)
        }
    }
}

fn metadata_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_METADATA_WORKER_FAILED",
        "The background MAME metadata worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}
