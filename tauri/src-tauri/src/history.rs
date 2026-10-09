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

#[derive(Clone)]
struct SessionLaunchHistory<C> {
    context: C,
    history_id: i64,
    decided: Option<bool>,
    persistence_in_flight: bool,
}

type PendingLaunchHistory = SessionLaunchHistory<AppHandle>;

#[derive(Clone)]
struct HistoryPersistenceAttempt<C> {
    context: C,
    history_id: i64,
    succeeded: bool,
}

// The registry is backend-owned, keyed by the exact supervised session ID.
// Decision and persistence are separate: a transient SQLite failure must not
// erase which outcome won the presentation/terminal race.
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
        SessionLaunchHistory {
            context: app.clone(),
            history_id,
            decided: None,
            persistence_in_flight: false,
        },
    );
    Ok(())
}

fn prepare_settlement<C: Clone>(
    map: &mut HashMap<String, SessionLaunchHistory<C>>,
    session_id: &str,
    requested_outcome: Option<bool>,
) -> Option<HistoryPersistenceAttempt<C>> {
    let history = map.get_mut(session_id)?;
    if history.decided.is_none() {
        history.decided = requested_outcome;
    }
    let succeeded = history.decided?;
    if history.persistence_in_flight {
        return None;
    }
    history.persistence_in_flight = true;
    Some(HistoryPersistenceAttempt {
        context: history.context.clone(),
        history_id: history.history_id,
        succeeded,
    })
}

fn finish_settlement<C>(
    map: &mut HashMap<String, SessionLaunchHistory<C>>,
    session_id: &str,
    attempt: &HistoryPersistenceAttempt<C>,
    persisted: bool,
) {
    let Some(history) = map.get_mut(session_id) else {
        return;
    };
    if history.history_id != attempt.history_id || history.decided != Some(attempt.succeeded) {
        return;
    }
    if persisted {
        map.remove(session_id);
    } else {
        history.persistence_in_flight = false;
    }
}

fn persist_attempt(
    session_id: &str,
    attempt: HistoryPersistenceAttempt<AppHandle>,
) -> AppResult<bool> {
    let result = finish_launch_history(&attempt.context, attempt.history_id, attempt.succeeded);
    finish_settlement(
        &mut recover_pending(),
        session_id,
        &attempt,
        result.is_ok(),
    );
    result.map(|_| true)
}

pub(crate) fn settle_pending_session_launch(session_id: &str, succeeded: bool) -> AppResult<bool> {
    let attempt = {
        let mut pending = recover_pending();
        if !pending.contains_key(session_id) {
            return Ok(false);
        }
        prepare_settlement(&mut pending, session_id, Some(succeeded))
    };
    let Some(attempt) = attempt else {
        // Another callback is already persisting the winning outcome.
        return Ok(false);
    };
    persist_attempt(session_id, attempt)
}

fn retry_decided_session_launch(session_id: &str) -> AppResult<bool> {
    let attempt = {
        let mut pending = recover_pending();
        prepare_settlement(&mut pending, session_id, None)
    };
    match attempt {
        Some(attempt) => persist_attempt(session_id, attempt),
        None => Ok(false),
    }
}

pub(crate) fn retry_decided_session_launches_once() -> Vec<(String, AppError)> {
    let session_ids = {
        let pending = recover_pending();
        pending
            .iter()
            .filter_map(|(session_id, history)| {
                (history.decided.is_some() && !history.persistence_in_flight)
                    .then(|| session_id.clone())
            })
            .collect::<Vec<_>>()
    };
    session_ids
        .into_iter()
        .filter_map(|session_id| match retry_decided_session_launch(&session_id) {
            Ok(_) => None,
            Err(error) => Some((session_id, error)),
        })
        .collect()
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
    use super::{
        finish_settlement, prepare_settlement, HistoryPersistenceAttempt, SessionLaunchHistory,
    };
    use crate::metadata::CatalogRepository;
    use rusqlite::{params, Connection};
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{Arc, Mutex},
        thread,
    };
    use tempfile::tempdir;

    fn state<C>(context: C, history_id: i64) -> SessionLaunchHistory<C> {
        SessionLaunchHistory {
            context,
            history_id,
            decided: None,
            persistence_in_flight: false,
        }
    }

    #[test]
    fn readiness_keeps_history_pending_until_first_ack_or_terminal_decision() {
        let mut histories = HashMap::from([("session-a".to_owned(), state((), 101))]);
        assert_eq!(histories["session-a"].decided, None);
        let attempt = prepare_settlement(&mut histories, "session-a", Some(true)).unwrap();
        assert!(attempt.succeeded);
        assert_eq!(histories["session-a"].decided, Some(true));
        finish_settlement(&mut histories, "session-a", &attempt, true);
        assert!(!histories.contains_key("session-a"));
    }

    #[test]
    fn first_decision_wins_and_failed_persistence_can_retry_same_outcome() {
        let registry = Arc::new(Mutex::new(HashMap::from([(
            "session".to_owned(),
            state((), 10),
        )])));
        let mut workers = Vec::new();
        for succeeded in [true, false] {
            let registry = Arc::clone(&registry);
            workers.push(thread::spawn(move || {
                let mut registry = registry.lock().expect("history registry");
                prepare_settlement(&mut registry, "session", Some(succeeded))
            }));
        }
        let attempts: Vec<_> = workers
            .into_iter()
            .filter_map(|worker| worker.join().expect("worker"))
            .collect();
        assert_eq!(
            attempts.len(),
            1,
            "only one persistence attempt may be in flight"
        );
        let first = &attempts[0];
        let mut registry = registry.lock().expect("history registry");
        finish_settlement(&mut registry, "session", first, false);
        let retry = prepare_settlement(&mut registry, "session", Some(!first.succeeded))
            .expect("retry after failure");
        assert_eq!(
            retry.succeeded, first.succeeded,
            "competing callback cannot change decision"
        );
        finish_settlement(&mut registry, "session", &retry, true);
        assert!(!registry.contains_key("session"));
    }

    fn persist_path(attempt: &HistoryPersistenceAttempt<PathBuf>) -> crate::errors::AppResult<()> {
        CatalogRepository::open(&attempt.context)?
            .finish_history_entry(attempt.history_id, attempt.succeeded)
    }

    #[test]
    fn sqlite_failure_retains_decision_and_retry_persists_same_history_row() {
        let temp = tempdir().expect("tempdir");
        let catalog = temp.path().join("catalog.sqlite3");
        let repository = CatalogRepository::open(&catalog).expect("catalog");
        let history_id = repository
            .begin_history_entry("pacman", None, 100)
            .expect("begin history");
        drop(repository);

        let connection = Connection::open(&catalog).expect("raw catalog");
        connection
            .execute("DELETE FROM recent_history WHERE id = ?1", [history_id])
            .expect("delete row to simulate persistence failure");
        drop(connection);

        let mut histories = HashMap::from([(
            "session-a".to_owned(),
            state(catalog.clone(), history_id),
        )]);
        let first = prepare_settlement(&mut histories, "session-a", Some(true)).unwrap();
        let error = persist_path(&first).expect_err("missing row must fail persistence");
        assert_eq!(error.code, "CATALOG_HISTORY_ENTRY_NOT_FOUND");
        finish_settlement(&mut histories, "session-a", &first, false);
        assert_eq!(histories["session-a"].decided, Some(true));
        assert!(!histories["session-a"].persistence_in_flight);

        let connection = Connection::open(&catalog).expect("raw catalog");
        connection
            .execute(
                "INSERT INTO recent_history(id, machine_short_name, software_item, launched_at_epoch_ms, succeeded) VALUES (?1, 'pacman', NULL, 100, NULL)",
                params![history_id],
            )
            .expect("restore row");
        drop(connection);

        let retry = prepare_settlement(&mut histories, "session-a", Some(false)).unwrap();
        assert!(
            retry.succeeded,
            "retry must preserve the original success decision"
        );
        persist_path(&retry).expect("retry persistence");
        finish_settlement(&mut histories, "session-a", &retry, true);
        assert!(!histories.contains_key("session-a"));

        let page = CatalogRepository::open(&catalog)
            .expect("catalog")
            .query_recent_history(10, 0)
            .expect("query history");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, history_id);
        assert_eq!(page.items[0].succeeded, Some(true));
    }

    #[test]
    fn accepted_ack_contract_keeps_draw_only_start_unsuccessful_if_terminal_event_wins() {
        let mut histories = HashMap::from([("session".to_owned(), state((), 20))]);
        let terminal = prepare_settlement(&mut histories, "session", Some(false)).unwrap();
        assert!(!terminal.succeeded);
        finish_settlement(&mut histories, "session", &terminal, true);
        assert!(!histories.contains_key("session"));
    }

    #[test]
    fn accepted_first_frame_success_cannot_be_overwritten_by_later_failure() {
        let mut histories = HashMap::from([("session".to_owned(), state((), 21))]);
        let success = prepare_settlement(&mut histories, "session", Some(true)).unwrap();
        assert!(success.succeeded);
        assert!(
            prepare_settlement(&mut histories, "session", Some(false)).is_none(),
            "a competing failure cannot start while success persistence is in flight"
        );
        assert_eq!(histories["session"].decided, Some(true));
        finish_settlement(&mut histories, "session", &success, true);
        assert!(!histories.contains_key("session"));
    }

    #[test]
    fn independent_retry_session_cannot_claim_another_sessions_history() {
        let mut histories = HashMap::from([
            ("original".to_owned(), state((), 10)),
            ("retry".to_owned(), state((), 11)),
        ]);
        assert!(prepare_settlement(&mut histories, "missing", Some(true)).is_none());
        let original = prepare_settlement(&mut histories, "original", Some(false)).unwrap();
        finish_settlement(&mut histories, "original", &original, true);
        assert!(histories.contains_key("retry"));
        assert_eq!(histories["retry"].decided, None);
    }
}
