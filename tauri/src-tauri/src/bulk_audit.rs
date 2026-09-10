//! Bounded whole-library MAME audit scheduling.
//!
//! A bulk run is intentionally restart-safe rather than resumable: every completed
//! per-machine result is committed independently through the MT-505 persistence
//! contract, so a cancelled or interrupted run may be started again without a
//! separate checkpoint file or partially committed batch transaction.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    config::settings_path,
    errors::{AppError, AppResult},
    library::audit::{resolve_bulk_audit_context, run_machine_audit_with_context, AuditContext},
    metadata::{CatalogRepository, CloneFilter, MachineQuery, MachineSort},
    storage,
};

const DEFAULT_PARALLELISM: u8 = 2;
const MAX_PARALLELISM: u8 = 4;
const QUERY_PAGE_SIZE: u32 = 200;
const MAX_BULK_MACHINE_COUNT: u64 = 100_000;
const MAX_STATUS_ERROR_CHARS: usize = 512;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BulkAuditRunState {
    Idle,
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BulkAuditStatus {
    pub schema_version: u32,
    pub job_id: Option<u64>,
    pub state: BulkAuditRunState,
    pub max_parallelism: u8,
    pub total: u64,
    pub completed: u64,
    pub failed: u64,
    pub active: u32,
    pub last_machine: Option<String>,
    pub last_error: Option<String>,
}

impl Default for BulkAuditStatus {
    fn default() -> Self {
        Self {
            schema_version: 1,
            job_id: None,
            state: BulkAuditRunState::Idle,
            max_parallelism: DEFAULT_PARALLELISM,
            total: 0,
            completed: 0,
            failed: 0,
            active: 0,
            last_machine: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StartBulkAuditRequest {
    #[serde(default = "default_parallelism")]
    pub max_parallelism: u8,
}

struct BulkAuditControl {
    status: Arc<Mutex<BulkAuditStatus>>,
    cancel: Option<Arc<AtomicBool>>,
}

pub struct BulkAuditSupervisor {
    control: Mutex<BulkAuditControl>,
    next_job_id: AtomicU64,
}

impl Default for BulkAuditSupervisor {
    fn default() -> Self {
        Self {
            control: Mutex::new(BulkAuditControl {
                status: Arc::new(Mutex::new(BulkAuditStatus::default())),
                cancel: None,
            }),
            next_job_id: AtomicU64::new(1),
        }
    }
}

impl BulkAuditSupervisor {
    fn start(
        &self,
        max_parallelism: u8,
    ) -> AppResult<(Arc<Mutex<BulkAuditStatus>>, Arc<AtomicBool>)> {
        let mut control = lock(&self.control, "MAME_BULK_AUDIT_STATE_FAILED")?;
        let current = lock(&control.status, "MAME_BULK_AUDIT_STATE_FAILED")?;
        if matches!(
            current.state,
            BulkAuditRunState::Running | BulkAuditRunState::Cancelling
        ) {
            return Err(AppError::new(
                "MAME_BULK_AUDIT_ALREADY_RUNNING",
                "A bulk MAME audit is already active.",
            ));
        }
        drop(current);

        let job_id = self.next_job_id.fetch_add(1, Ordering::Relaxed);
        let status = Arc::new(Mutex::new(BulkAuditStatus {
            schema_version: 1,
            job_id: Some(job_id),
            state: BulkAuditRunState::Running,
            max_parallelism,
            total: 0,
            completed: 0,
            failed: 0,
            active: 0,
            last_machine: None,
            last_error: None,
        }));
        let cancel = Arc::new(AtomicBool::new(false));
        control.status = Arc::clone(&status);
        control.cancel = Some(Arc::clone(&cancel));
        Ok((status, cancel))
    }

    fn snapshot(&self) -> AppResult<BulkAuditStatus> {
        let control = lock(&self.control, "MAME_BULK_AUDIT_STATE_FAILED")?;
        let status = lock(&control.status, "MAME_BULK_AUDIT_STATE_FAILED")?;
        Ok(status.clone())
    }

    fn cancel(&self) -> AppResult<BulkAuditStatus> {
        let control = lock(&self.control, "MAME_BULK_AUDIT_STATE_FAILED")?;
        let mut status = lock(&control.status, "MAME_BULK_AUDIT_STATE_FAILED")?;
        if matches!(status.state, BulkAuditRunState::Running) {
            if let Some(cancel) = &control.cancel {
                cancel.store(true, Ordering::Release);
                status.state = BulkAuditRunState::Cancelling;
            }
        }
        Ok(status.clone())
    }
}

#[tauri::command]
pub fn get_library_bulk_audit_status(
    supervisor: State<'_, BulkAuditSupervisor>,
) -> AppResult<BulkAuditStatus> {
    supervisor.snapshot()
}

#[tauri::command]
pub fn start_library_bulk_audit(
    request: StartBulkAuditRequest,
    app: AppHandle,
    supervisor: State<'_, BulkAuditSupervisor>,
) -> AppResult<BulkAuditStatus> {
    let max_parallelism = validate_parallelism(request.max_parallelism)?;
    let catalog_path = storage::catalog_path(&app)?;
    let settings_path = settings_path(&app)?;
    let (status, cancel) = supervisor.start(max_parallelism)?;
    let initial = clone_status(&status)?;

    tauri::async_runtime::spawn_blocking(move || {
        run_bulk_audit_job(catalog_path, settings_path, max_parallelism, status, cancel);
    });

    Ok(initial)
}

#[tauri::command]
pub fn cancel_library_bulk_audit(
    supervisor: State<'_, BulkAuditSupervisor>,
) -> AppResult<BulkAuditStatus> {
    supervisor.cancel()
}

fn run_bulk_audit_job(
    catalog_path: PathBuf,
    settings_path: PathBuf,
    max_parallelism: u8,
    status: Arc<Mutex<BulkAuditStatus>>,
    cancel: Arc<AtomicBool>,
) {
    let context = match resolve_bulk_audit_context(&catalog_path, &settings_path) {
        Ok(context) => context,
        Err(error) => {
            finish_with_error(&status, error);
            return;
        }
    };

    let machines = match load_runnable_machine_names(&catalog_path, &cancel) {
        Ok(machines) => machines,
        Err(_) if cancel.load(Ordering::Acquire) => {
            finish_cancelled(&status);
            return;
        }
        Err(error) => {
            finish_with_error(&status, error);
            return;
        }
    };

    if cancel.load(Ordering::Acquire) {
        finish_cancelled(&status);
        return;
    }

    let machine_count = machines.len();
    if let Err(error) = set_total(&status, machine_count) {
        finish_with_error(&status, error);
        return;
    }
    if machine_count == 0 {
        finish_completed(&status);
        return;
    }

    let queue = Arc::new(Mutex::new(VecDeque::from(machines)));
    let worker_count = usize::from(max_parallelism).min(machine_count);
    let mut worker_failure = None;

    thread::scope(|scope| {
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let status = Arc::clone(&status);
            let cancel = Arc::clone(&cancel);
            let catalog_path = catalog_path.clone();
            let context = context.clone();
            workers
                .push(scope.spawn(move || {
                    audit_worker(&catalog_path, &context, &queue, &status, &cancel)
                }));
        }
        for worker in workers {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    cancel.store(true, Ordering::Release);
                    if worker_failure.is_none() {
                        worker_failure = Some(error);
                    }
                }
                Err(_) => {
                    cancel.store(true, Ordering::Release);
                    if worker_failure.is_none() {
                        worker_failure = Some(AppError::new(
                            "MAME_BULK_AUDIT_WORKER_PANICKED",
                            "A bulk MAME audit worker terminated unexpectedly.",
                        ));
                    }
                }
            }
        }
    });

    if let Some(error) = worker_failure {
        finish_with_error(&status, error);
    } else if cancel.load(Ordering::Acquire) {
        finish_cancelled(&status);
    } else {
        finish_completed(&status);
    }
}

fn audit_worker(
    catalog_path: &Path,
    context: &AuditContext,
    queue: &Arc<Mutex<VecDeque<String>>>,
    status: &Arc<Mutex<BulkAuditStatus>>,
    cancel: &AtomicBool,
) -> AppResult<()> {
    loop {
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }

        let Some(machine) = next_machine(queue)? else {
            return Ok(());
        };
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }

        adjust_active(status, 1)?;
        let result = run_machine_audit_with_context(catalog_path, machine.clone(), context);
        record_result(status, &machine, result.as_ref().err())?;
        adjust_active(status, -1)?;
    }
}

fn load_runnable_machine_names(catalog_path: &Path, cancel: &AtomicBool) -> AppResult<Vec<String>> {
    let repository = CatalogRepository::open(catalog_path)?;
    let mut offset = 0_u32;
    let mut total = None;
    let mut names = Vec::new();

    loop {
        if cancel.load(Ordering::Acquire) {
            return Err(AppError::new(
                "MAME_BULK_AUDIT_CANCELLED",
                "The bulk MAME audit was cancelled while preparing its work queue.",
            ));
        }

        let page = repository.query_machines(&MachineQuery {
            text: None,
            manufacturer: None,
            year: None,
            driver_status: None,
            clone_filter: CloneFilter::All,
            sort: MachineSort::ShortNameAsc,
            include_devices: false,
            limit: QUERY_PAGE_SIZE,
            offset,
        })?;

        if page.total > MAX_BULK_MACHINE_COUNT {
            return Err(AppError::new(
                "MAME_BULK_AUDIT_TOO_LARGE",
                "The active catalog exceeds the supported bounded bulk-audit size.",
            )
            .with_details(serde_json::json!({
                "catalogCount": page.total,
                "maxCount": MAX_BULK_MACHINE_COUNT
            })));
        }
        total.get_or_insert(page.total);
        names.extend(
            page.items
                .iter()
                .filter(|machine| machine.runnable)
                .map(|machine| machine.short_name.clone()),
        );

        let catalog_total = total.unwrap_or(0);
        if u64::from(offset) + u64::from(page.limit) >= catalog_total {
            break;
        }
        if page.items.is_empty() {
            return Err(AppError::new(
                "MAME_BULK_AUDIT_CATALOG_PAGING_FAILED",
                "The catalog returned an empty page before the bulk audit reached the end.",
            ));
        }
        offset = offset.checked_add(page.limit).ok_or_else(|| {
            AppError::new(
                "MAME_BULK_AUDIT_CATALOG_PAGING_FAILED",
                "The bulk audit catalog offset exceeded its supported range.",
            )
        })?;
    }

    Ok(names)
}

fn next_machine(queue: &Arc<Mutex<VecDeque<String>>>) -> AppResult<Option<String>> {
    Ok(lock(queue, "MAME_BULK_AUDIT_QUEUE_FAILED")?.pop_front())
}

fn validate_parallelism(value: u8) -> AppResult<u8> {
    if (1..=MAX_PARALLELISM).contains(&value) {
        Ok(value)
    } else {
        Err(AppError::new(
            "MAME_BULK_AUDIT_PARALLELISM_INVALID",
            format!("Bulk audit parallelism must be between 1 and {MAX_PARALLELISM}."),
        )
        .with_details(serde_json::json!({
            "requested": value,
            "maxParallelism": MAX_PARALLELISM
        })))
    }
}

fn set_total(status: &Arc<Mutex<BulkAuditStatus>>, total: usize) -> AppResult<()> {
    let total = u64::try_from(total).map_err(|_| {
        AppError::new(
            "MAME_BULK_AUDIT_COUNT_INVALID",
            "The bulk MAME audit machine count cannot be represented.",
        )
    })?;
    lock(status, "MAME_BULK_AUDIT_STATE_FAILED")?.total = total;
    Ok(())
}

fn adjust_active(status: &Arc<Mutex<BulkAuditStatus>>, delta: i32) -> AppResult<()> {
    let mut status = lock(status, "MAME_BULK_AUDIT_STATE_FAILED")?;
    status.active = if delta >= 0 {
        status.active.saturating_add(delta as u32)
    } else {
        status.active.saturating_sub(delta.unsigned_abs())
    };
    Ok(())
}

fn record_result(
    status: &Arc<Mutex<BulkAuditStatus>>,
    machine: &str,
    error: Option<&AppError>,
) -> AppResult<()> {
    let mut status = lock(status, "MAME_BULK_AUDIT_STATE_FAILED")?;
    status.last_machine = Some(machine.to_owned());
    if let Some(error) = error {
        status.failed = status.failed.saturating_add(1);
        status.last_error = Some(truncate_status_error(&format!(
            "{}: {}",
            error.code, error.message
        )));
    } else {
        status.completed = status.completed.saturating_add(1);
    }
    Ok(())
}

fn finish_completed(status: &Arc<Mutex<BulkAuditStatus>>) {
    if let Ok(mut status) = lock(status, "MAME_BULK_AUDIT_STATE_FAILED") {
        status.active = 0;
        status.state = BulkAuditRunState::Completed;
    }
}

fn finish_cancelled(status: &Arc<Mutex<BulkAuditStatus>>) {
    if let Ok(mut status) = lock(status, "MAME_BULK_AUDIT_STATE_FAILED") {
        status.active = 0;
        status.state = BulkAuditRunState::Cancelled;
    }
}

fn finish_with_error(status: &Arc<Mutex<BulkAuditStatus>>, error: AppError) {
    if let Ok(mut status) = lock(status, "MAME_BULK_AUDIT_STATE_FAILED") {
        status.active = 0;
        status.state = BulkAuditRunState::Failed;
        status.last_error = Some(truncate_status_error(&format!(
            "{}: {}",
            error.code, error.message
        )));
    }
}

fn clone_status(status: &Arc<Mutex<BulkAuditStatus>>) -> AppResult<BulkAuditStatus> {
    Ok(lock(status, "MAME_BULK_AUDIT_STATE_FAILED")?.clone())
}

fn lock<'a, T>(mutex: &'a Mutex<T>, code: &str) -> AppResult<MutexGuard<'a, T>> {
    mutex.lock().map_err(|_| {
        AppError::new(
            code,
            "Bulk MAME audit coordination state became unavailable.",
        )
    })
}

fn truncate_status_error(value: &str) -> String {
    value.chars().take(MAX_STATUS_ERROR_CHARS).collect()
}

const fn default_parallelism() -> u8 {
    DEFAULT_PARALLELISM
}

#[cfg(test)]
mod tests {
    use super::{
        truncate_status_error, validate_parallelism, BulkAuditRunState, BulkAuditStatus,
        BulkAuditSupervisor, DEFAULT_PARALLELISM, MAX_PARALLELISM, MAX_STATUS_ERROR_CHARS,
    };

    #[test]
    fn parallelism_is_explicitly_bounded() {
        assert_eq!(validate_parallelism(1).expect("one worker"), 1);
        assert_eq!(
            validate_parallelism(MAX_PARALLELISM).expect("maximum workers"),
            MAX_PARALLELISM
        );
        for invalid in [0, MAX_PARALLELISM + 1, u8::MAX] {
            let error = validate_parallelism(invalid).expect_err("invalid parallelism");
            assert_eq!(error.code, "MAME_BULK_AUDIT_PARALLELISM_INVALID");
        }
    }

    #[test]
    fn idle_status_is_bounded_and_versioned() {
        let status = BulkAuditStatus::default();
        assert_eq!(status.schema_version, 1);
        assert_eq!(status.state, BulkAuditRunState::Idle);
        assert_eq!(status.max_parallelism, DEFAULT_PARALLELISM);
        assert_eq!(status.total, 0);
        assert_eq!(status.active, 0);
    }

    #[test]
    fn supervisor_rejects_overlapping_jobs_and_cancels_active_job() {
        let supervisor = BulkAuditSupervisor::default();
        supervisor.start(2).expect("first job starts");
        let error = supervisor.start(2).expect_err("overlapping job rejected");
        assert_eq!(error.code, "MAME_BULK_AUDIT_ALREADY_RUNNING");

        let status = supervisor.cancel().expect("cancel active job");
        assert_eq!(status.state, BulkAuditRunState::Cancelling);
    }

    #[test]
    fn status_errors_are_bounded() {
        let input = "x".repeat(MAX_STATUS_ERROR_CHARS + 100);
        let output = truncate_status_error(&input);
        assert_eq!(output.chars().count(), MAX_STATUS_ERROR_CHARS);
    }
}
