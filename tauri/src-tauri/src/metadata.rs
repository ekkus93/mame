//! MAME-generated metadata acquisition, parsing, import, and freshness tracking.
//!
//! MAME remains the metadata source of truth. The Rust backend invokes
//! `-listxml`, parses it as a stream, and activates a new SQLite generation only
//! after the complete process and import succeed.

mod catalog;
mod collections;
mod detail;
mod favorites;
mod generator;
mod history;
mod mame_ui_query;
mod model;
mod parser;
#[cfg(test)]
mod performance;
#[cfg(test)]
mod provenance_fixture;
mod software;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    config::{load_settings, settings_path},
    diagnostics,
    effective_runtime::resolve_effective_mame_source,
    errors::{AppError, AppResult},
    mame::MameExecutableSource,
    storage,
};

pub use collections::{
    CollectionListPage, CollectionMemberEntry, CollectionMemberPage, CollectionMembershipState,
    CollectionSummary,
};
pub use favorites::{FavoriteEntry, FavoritePage, FavoriteState};
pub use history::{RecentHistoryEntry, RecentHistoryPage, HISTORY_RETENTION_LIMIT};
pub use model::{
    MachineAvailability, MachineDetail, MachineDisplayInfo, MachineListItem, MachinePage,
    MachineSoftwareListInfo, MetadataFreshness, MetadataGenerationSummary, MetadataRefreshResult,
    MetadataStatus,
};
pub use software::{SoftwareItemSummary, SoftwareListFilter, SoftwarePartSummary};

pub(crate) use catalog::{
    AvailabilityFilter, CatalogRepository, CloneFilter, MachineAvailabilityQuery, MachineQuery,
    MachineSort,
};
pub(crate) use mame_ui_query::{MameUiMachineFilter, MameUiMachineQuery};
pub(crate) use software::{parse_software_item, parse_software_list_page};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MetadataExecutableSelectionKind {
    Bundled,
    External,
    DevelopmentTree,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataExecutableRequest {
    pub source: MetadataExecutableSelectionKind,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RefreshMameMetadataRequest {
    pub executable: MetadataExecutableRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatusRequest {
    pub executable: MetadataExecutableRequest,
}

#[tauri::command]
pub async fn refresh_mame_metadata(
    request: RefreshMameMetadataRequest,
    app: AppHandle,
) -> AppResult<MetadataRefreshResult> {
    let source = executable_source(&request.executable, &app)?;
    let catalog_path = storage::catalog_path(&app)?;
    diagnostics::record(
        "info",
        "metadata.refresh",
        "MAME metadata refresh requested.",
        serde_json::json!({ "source": format!("{:?}", request.executable.source) }),
    );

    let result = tauri::async_runtime::spawn_blocking(move || {
        generator::refresh_catalog(source, &catalog_path)
    })
    .await
    .map_err(metadata_worker_error)??;
    diagnostics::record(
        "info",
        "metadata.refresh",
        "MAME metadata refresh completed.",
        serde_json::json!({ "status": "completed" }),
    );
    Ok(result)
}

#[tauri::command]
pub async fn get_mame_metadata_status(
    request: MetadataStatusRequest,
    app: AppHandle,
) -> AppResult<MetadataStatus> {
    let source = executable_source(&request.executable, &app)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || generator::metadata_status(source, &catalog_path))
        .await
        .map_err(metadata_worker_error)?
}

fn executable_source(
    request: &MetadataExecutableRequest,
    app: &AppHandle,
) -> AppResult<MameExecutableSource> {
    match request.source {
        MetadataExecutableSelectionKind::Bundled => {
            let settings = load_settings(&settings_path(app)?)?;
            let resource_dir = app.path().resource_dir().map_err(|error| {
                AppError::new(
                    "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE",
                    "The application resource directory could not be resolved.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?;
            resolve_effective_mame_source(&settings, &resource_dir)
        }
        MetadataExecutableSelectionKind::External => {
            Ok(MameExecutableSource::external(&request.path))
        }
        MetadataExecutableSelectionKind::DevelopmentTree => {
            Ok(MameExecutableSource::development_tree(&request.path))
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
