//! Durable recent-launch history and typed query commands.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::{AppError, AppResult},
    metadata::{CatalogRepository, RecentHistoryPage},
    storage,
};

const DEFAULT_HISTORY_PAGE_SIZE: u32 = 50;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentHistoryPageRequest {
    #[serde(default = "default_history_page_size")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

#[tauri::command]
pub async fn query_library_history(
    request: RecentHistoryPageRequest,
    app: AppHandle,
) -> AppResult<RecentHistoryPage> {
    let catalog_path = storage::catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        CatalogRepository::open(&catalog_path)?.query_recent_history(request.limit, request.offset)
    })
    .await
    .map_err(history_worker_error)?
}

pub(crate) fn begin_launch_history(
    app: &AppHandle,
    machine_short_name: &str,
    software_item: Option<&str>,
) -> AppResult<i64> {
    let catalog_path = storage::catalog_path(app)?;
    CatalogRepository::open(&catalog_path)?.begin_history_entry(
        machine_short_name,
        software_item,
        now_epoch_ms()?,
    )
}

pub(crate) fn finish_launch_history(
    app: &AppHandle,
    history_id: i64,
    succeeded: bool,
) -> AppResult<()> {
    let catalog_path = storage::catalog_path(app)?;
    CatalogRepository::open(&catalog_path)?.finish_history_entry(history_id, succeeded)
}

fn history_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "CATALOG_HISTORY_WORKER_FAILED",
        "The background play-history query worker did not complete normally.",
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

const fn default_history_page_size() -> u32 {
    DEFAULT_HISTORY_PAGE_SIZE
}
