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
use tauri::AppHandle;

use crate::{
    diagnostics,
    effective_runtime::effective_mame_source,
    errors::{AppError, AppResult},
    mame::{inspect_executable, MameExecutableIdentity, MameExecutableSource},
    sessions::{MameExecutableRequest, MameExecutableSelectionKind},
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RefreshMameMetadataRequest {
    #[serde(default)]
    pub executable: Option<MameExecutableRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataStatusRequest {
    #[serde(default)]
    pub executable: Option<MameExecutableRequest>,
}

#[tauri::command]
pub async fn refresh_mame_metadata(
    request: RefreshMameMetadataRequest,
    app: AppHandle,
) -> AppResult<MetadataRefreshResult> {
    let source = executable_source(request.executable.as_ref(), &app)?;
    let catalog_path = storage::catalog_path(&app)?;
    diagnostics::record(
        "info",
        "metadata.refresh",
        "MAME metadata refresh requested.",
        serde_json::json!({ "source": format!("{:?}", source.kind()) }),
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
    let source = executable_source(request.executable.as_ref(), &app)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || generator::metadata_status(source, &catalog_path))
        .await
        .map_err(metadata_worker_error)?
}

fn executable_source(
    request: Option<&MameExecutableRequest>,
    app: &AppHandle,
) -> AppResult<MameExecutableSource> {
    match request {
        None => effective_mame_source(app),
        Some(request) => Ok(match request.source {
            MameExecutableSelectionKind::External => MameExecutableSource::external(&request.path),
            MameExecutableSelectionKind::DevelopmentTree => {
                MameExecutableSource::development_tree(&request.path)
            }
        }),
    }
}

pub(crate) fn effective_source_for_generation(
    app: &AppHandle,
    generation: &MetadataGenerationSummary,
) -> AppResult<MameExecutableSource> {
    let source = effective_mame_source(app)?;
    let identity = inspect_executable(source.clone())?;
    ensure_generation_matches_identity(generation, &identity)?;
    Ok(source)
}

pub(crate) fn ensure_generation_matches_identity(
    generation: &MetadataGenerationSummary,
    identity: &MameExecutableIdentity,
) -> AppResult<()> {
    if generation.source_kind == source_kind_wire(identity)
        && generation.trust == trust_wire(identity)
        && generation.executable_path == identity.path
        && generation.mame_version == identity.version
        && generation.mame_build == identity.build
        && generation.raw_version_line == identity.raw_version_line
    {
        return Ok(());
    }

    Err(AppError::new(
        "MAME_METADATA_STALE",
        "The active MAME runtime no longer matches the imported metadata generation.",
    )
    .with_details(serde_json::json!({
        "catalogSource": generation.source_kind,
        "catalogTrust": generation.trust,
        "catalogPath": generation.executable_path,
        "catalogVersion": generation.mame_version,
        "currentSource": format!("{:?}", identity.source),
        "currentTrust": format!("{:?}", identity.trust),
        "currentPath": identity.path,
        "currentVersion": identity.version
    })))
}

fn source_kind_wire(identity: &MameExecutableIdentity) -> &'static str {
    use crate::mame::MameExecutableSourceKind;
    match identity.source {
        MameExecutableSourceKind::Bundled => "bundled",
        MameExecutableSourceKind::External => "external",
        MameExecutableSourceKind::DevelopmentTree => "developmentTree",
    }
}

fn trust_wire(identity: &MameExecutableIdentity) -> &'static str {
    use crate::mame::MameExecutableTrust;
    match identity.trust {
        MameExecutableTrust::QualifiedBundled => "qualifiedBundled",
        MameExecutableTrust::UserConfigured => "userConfigured",
        MameExecutableTrust::Development => "development",
    }
}

fn metadata_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_METADATA_WORKER_FAILED",
        "The background MAME metadata worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}
