//! Durable recent-launch history and typed query commands.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::{AppError, AppResult},
    metadata::{CatalogRepository, RecentHistoryPage},
    storage,
};

const DEFAULT_HISTORY_PAGE_SIZE: u32 = 50;

struct PendingLaunchHistory {
    app: AppHandle,
    history_id: i64,
}

// The registry is backend-owned, keyed by the exact supervised session ID.
// A presentation acknowledgement and terminal lifecycle event race to claim
// the entry under one mutex: only the winner can finalize durable history.
static PENDING_LAUNCHES: OnceLock<Mutex<HashMap<String, PendingLaunchHistory>>> = OnceLock::new();

fn pending_launches() -> &'static Mutex<HashMap<String, PendingLaunchHistory>> {
    PENDING_LAUNCHES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn recover_pending() -> std::sync::MutexGuard<'static, HashMap<String, PendingLaunchHistory>> {
    pending_launches()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn register_pending_session_launch(
    app: &AppHandle,
    session_id: &str,
    history_id: i64,
) -> AppResult<()> {
    let mut pending = recover_pending();
    if pending.contains_key(session_id) {
        return Err(AppError::new(
            "PLAY_HISTORY_SESSION_DUPLICATE",
            "The session already has a pending launch-history record.",
        ));
    }
    pending.insert(
        session_id.to_owned(),
        PendingLaunchHistory {
            app: app.clone(),
            history_id,
        },
    );
    Ok(())
}

fn claim_pending<T>(map: &mut HashMap<String, T>, session_id: &str) -> Option<T> {
    map.remove(session_id)
}

pub(crate) fn settle_pending_session_launch(session_id: &str, succeeded: bool) -> AppResult<bool> {
    let Some(launch) = claim_pending(&mut recover_pending(), session_id) else {
        return Ok(false);
    };
    // Never hold the registry mutex across SQLite I/O. A failed write is
    // diagnostic but cannot convert a previously claimed outcome into another.
    finish_launch_history(&launch.app, launch.history_id, succeeded)?;
    Ok(true)
}

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

#[cfg(test)]
mod post_review_history_tests {
    use super::claim_pending;
    use std::{collections::HashMap, sync::{Arc, Mutex}, thread};

    #[test]
    fn readiness_keeps_history_pending_until_first_ack_or_terminal_claim() {
        let mut history = HashMap::from([("session-a".to_owned(), 101_i64)]);
        assert_eq!(history.get("session-a"), Some(&101));
        assert_eq!(claim_pending(&mut history, "session-b"), None);
        assert_eq!(claim_pending(&mut history, "session-a"), Some(101));
        assert_eq!(claim_pending(&mut history, "session-a"), None);
    }

    #[test]
    fn first_ack_and_terminal_event_settle_only_once_in_a_race() {
        let registry = Arc::new(Mutex::new(HashMap::from([
            ("original".to_owned(), 10_i64),
            ("retry".to_owned(), 11_i64),
        ])));
        let mut workers = Vec::new();
        for succeeded in [true, false] {
            let registry = Arc::clone(&registry);
            workers.push(thread::spawn(move || {
                let mut registry = registry.lock().expect("pending registry");
                claim_pending(&mut registry, "original").map(|history_id| (history_id, succeeded))
            }));
        }
        let claims: Vec<_> = workers.into_iter().filter_map(|worker| worker.join().expect("worker")).collect();
        assert_eq!(claims.len(), 1, "first valid ack or terminal failure wins");
        assert_eq!(claims[0].0, 10);
        let mut registry = registry.lock().expect("registry");
        assert_eq!(claim_pending(&mut registry, "retry"), Some(11));
        assert_eq!(claim_pending(&mut registry, "original"), None);
    }
}
