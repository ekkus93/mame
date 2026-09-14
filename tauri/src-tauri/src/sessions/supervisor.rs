use std::{
    collections::VecDeque,
    io::{self, Read},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use super::control::{
    control_state, set_control_state, ControlBootstrap, ControlChannel, ControlChannelState,
    ControlStdoutParser, ParserEvent, SharedControlState, CONTROL_READY_TIMEOUT_MS,
};

use crate::{
    config::LaunchPreferencesV1,
    errors::{AppError, AppResult},
    mame::{
        build_launch_argv_with_preferences, inspect_executable, validate_executable_path,
        MameExecutableIdentity, MameExecutableSource, MameLaunchTarget,
    },
};

const DIAGNOSTIC_TAIL_LIMIT: usize = 64 * 1024;
const DIAGNOSTIC_ERROR_LIMIT: usize = 2 * 1024;
const CAPTURE_SETTLE_TIMEOUT: Duration = Duration::from_millis(250);
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(25);
const SOFT_STOP_TIMEOUT: Duration = Duration::from_millis(1500);
const FORCED_STOP_TIMEOUT: Duration = Duration::from_millis(1000);

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) type EventSink =
    Arc<dyn Fn(&str, SessionLifecycleEventV1) -> Result<(), String> + Send + Sync>;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionState {
    Created,
    Starting,
    Running,
    Stopping,
    Exited,
    Failed,
    Crashed,
}

impl SessionState {
    fn is_active(self) -> bool {
        matches!(
            self,
            Self::Created | Self::Starting | Self::Running | Self::Stopping
        )
    }

    fn allows(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Starting | Self::Failed)
                | (Self::Starting, Self::Running | Self::Failed)
                | (
                    Self::Running,
                    Self::Stopping | Self::Exited | Self::Failed | Self::Crashed
                )
                | (Self::Stopping, Self::Exited | Self::Failed | Self::Crashed)
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveLaunchConfig {
    pub project_paths: Vec<EffectiveProjectPath>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveProjectPath {
    pub option: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub schema_version: u32,
    pub session_id: String,
    pub state: SessionState,
    pub machine: String,
    pub software: Option<String>,
    pub executable: MameExecutableIdentity,
    pub effective_argv: Vec<String>,
    pub effective_config: EffectiveLaunchConfig,
    pub created_at_epoch_ms: u64,
    pub started_at_epoch_ms: Option<u64>,
    pub ended_at_epoch_ms: Option<u64>,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub termination_signal: Option<i32>,
    pub forced_termination: bool,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub diagnostic_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionLifecycleEventV1 {
    pub schema_version: u32,
    pub session: SessionSnapshot,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StopSessionResult {
    pub schema_version: u32,
    pub soft_stop_requested: bool,
    pub forced_termination: bool,
    pub session: SessionSnapshot,
}

#[derive(Default)]
pub struct SessionSupervisor {
    launch_gate: Mutex<()>,
    inner: Arc<Mutex<SupervisorInner>>,
}

#[derive(Default)]
struct SupervisorInner {
    current: Option<ManagedSession>,
}

struct ManagedSession {
    snapshot: SessionSnapshot,
    child: Option<Arc<Mutex<Child>>>,
    control: Option<ControlChannel>,
    diagnostics: SharedDiagnostics,
    event_sink: EventSink,
}

type SharedDiagnostics = Arc<Mutex<SessionDiagnostics>>;

#[derive(Default)]
struct SessionDiagnostics {
    stdout: TailBuffer,
    stderr: TailBuffer,
    diagnostic_error: Option<String>,
}

struct TailBuffer {
    bytes: VecDeque<u8>,
    truncated: bool,
}

impl Default for TailBuffer {
    fn default() -> Self {
        Self {
            bytes: VecDeque::with_capacity(DIAGNOSTIC_TAIL_LIMIT),
            truncated: false,
        }
    }
}

impl TailBuffer {
    fn append(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        if bytes.len() >= DIAGNOSTIC_TAIL_LIMIT {
            self.bytes.clear();
            self.bytes
                .extend(bytes[bytes.len() - DIAGNOSTIC_TAIL_LIMIT..].iter().copied());
            self.truncated = true;
            return;
        }

        let overflow = self
            .bytes
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(DIAGNOSTIC_TAIL_LIMIT);
        if overflow > 0 {
            self.bytes.drain(..overflow);
            self.truncated = true;
        }
        self.bytes.extend(bytes.iter().copied());
    }

    fn text(&self) -> String {
        let bytes: Vec<u8> = self.bytes.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl SessionSupervisor {
    #[cfg(test)]
    pub(crate) fn launch(
        &self,
        source: MameExecutableSource,
        target: MameLaunchTarget,
        effective_config: EffectiveLaunchConfig,
        event_sink: EventSink,
    ) -> AppResult<SessionSnapshot> {
        self.launch_with_preferences(
            source,
            target,
            effective_config,
            LaunchPreferencesV1::default(),
            event_sink,
        )
    }

    pub(crate) fn launch_with_preferences(
        &self,
        source: MameExecutableSource,
        target: MameLaunchTarget,
        effective_config: EffectiveLaunchConfig,
        launch_preferences: LaunchPreferencesV1,
        event_sink: EventSink,
    ) -> AppResult<SessionSnapshot> {
        let _launch_guard = recover_lock(&self.launch_gate);

        {
            let inner = recover_lock(&self.inner);
            if let Some(current) = &inner.current {
                if current.snapshot.state.is_active() {
                    return Err(AppError::new(
                        "MAME_SESSION_ALREADY_ACTIVE",
                        "A MAME session is already active.",
                    )
                    .with_details(serde_json::json!({
                        "sessionId": current.snapshot.session_id,
                        "state": current.snapshot.state
                    })));
                }
            }
        }

        let executable_path = validate_executable_path(source.path())?;
        let executable = inspect_executable(source)?;
        let created_at_epoch_ms = epoch_millis()?;
        let session_id = new_session_id(created_at_epoch_ms);
        let control_bootstrap = ControlBootstrap::create(&session_id)?;
        let frame_token = control_bootstrap.frame_token().to_owned();
        let mut argv = build_launch_argv_with_preferences(&target, &launch_preferences)?.into_vec();
        control_bootstrap.append_launch_arguments(&mut argv);
        let diagnostics = Arc::new(Mutex::new(SessionDiagnostics::default()));
        let effective_argv = argv
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();

        let mut snapshot = SessionSnapshot {
            schema_version: 1,
            session_id: session_id.clone(),
            state: SessionState::Created,
            machine: target.machine,
            software: target.software,
            executable,
            effective_argv,
            effective_config,
            created_at_epoch_ms,
            started_at_epoch_ms: None,
            ended_at_epoch_ms: None,
            pid: None,
            exit_code: None,
            termination_signal: None,
            forced_termination: false,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            diagnostic_error: None,
        };
        transition(&mut snapshot, SessionState::Starting)?;

        {
            let mut inner = recover_lock(&self.inner);
            inner.current = Some(ManagedSession {
                snapshot,
                child: None,
                control: None,
                diagnostics: diagnostics.clone(),
                event_sink: event_sink.clone(),
            });
        }

        let mut command = Command::new(&executable_path);
        command
            .args(&argv)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                mark_launch_failed(
                    &self.inner,
                    &session_id,
                    "MAME_LAUNCH_FAILED",
                    format!("MAME could not be launched: {error}"),
                );
                return Err(
                    AppError::new("MAME_LAUNCH_FAILED", "MAME could not be launched.")
                        .with_details(serde_json::json!({
                            "sessionId": session_id,
                            "path": executable_path,
                            "cause": error.to_string()
                        })),
                );
            }
        };

        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                terminate_unusable_child(&mut child);
                mark_launch_failed(
                    &self.inner,
                    &session_id,
                    "MAME_STDIN_PIPE_MISSING",
                    "MAME launched without the requested control stdin pipe.".to_owned(),
                );
                return Err(AppError::new(
                    "MAME_STDIN_PIPE_MISSING",
                    "MAME launched without the requested control stdin pipe.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id })));
            }
        };
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                terminate_unusable_child(&mut child);
                mark_launch_failed(
                    &self.inner,
                    &session_id,
                    "MAME_STDOUT_PIPE_MISSING",
                    "MAME launched without the requested stdout pipe.".to_owned(),
                );
                return Err(AppError::new(
                    "MAME_STDOUT_PIPE_MISSING",
                    "MAME launched without the requested stdout pipe.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id })));
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                terminate_unusable_child(&mut child);
                mark_launch_failed(
                    &self.inner,
                    &session_id,
                    "MAME_STDERR_PIPE_MISSING",
                    "MAME launched without the requested stderr pipe.".to_owned(),
                );
                return Err(AppError::new(
                    "MAME_STDERR_PIPE_MISSING",
                    "MAME launched without the requested stderr pipe.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id })));
            }
        };

        let pid = child.id();
        let child = Arc::new(Mutex::new(child));
        let control = ControlChannel::new(stdin, frame_token.clone());
        let control_state = control.shared_state();
        let ready_observed = Arc::new(AtomicBool::new(false));

        {
            let mut inner = recover_lock(&self.inner);
            let current = current_session_mut(&mut inner, &session_id)?;
            current.child = Some(child.clone());
            current.control = Some(control);
            current.snapshot.pid = Some(pid);
        }

        let capture_done = Arc::new(AtomicU8::new(0));
        spawn_controlled_stdout_capture(
            stdout,
            diagnostics.clone(),
            ControlStdoutParser::new(session_id.clone(), frame_token),
            control_state.clone(),
            ready_observed.clone(),
            capture_done.clone(),
        );
        spawn_capture(stderr, diagnostics.clone(), capture_done.clone());

        if let Err(error) = wait_for_control_ready(
            &control_state,
            &ready_observed,
            Duration::from_millis(CONTROL_READY_TIMEOUT_MS),
        ) {
            fail_control_launch(&self.inner, &session_id, &child, &error);
            return Err(error.with_details(serde_json::json!({ "sessionId": session_id })));
        }

        let started_at_epoch_ms = match epoch_millis() {
            Ok(timestamp) => timestamp,
            Err(error) => {
                fail_control_launch(&self.inner, &session_id, &child, &error);
                return Err(error);
            }
        };
        let started_snapshot = {
            let mut inner = recover_lock(&self.inner);
            let current = current_session_mut(&mut inner, &session_id)?;
            current.snapshot.started_at_epoch_ms = Some(started_at_epoch_ms);
            transition(&mut current.snapshot, SessionState::Running)?;
            snapshot_with_diagnostics(current)
        };

        if let Err(error) = event_sink(
            "session.started",
            SessionLifecycleEventV1 {
                schema_version: 1,
                session: started_snapshot.clone(),
            },
        ) {
            record_diagnostic_error(
                &diagnostics,
                format!("session.started event emission failed: {error}"),
            );
        }

        spawn_exit_watcher(self.inner.clone(), session_id, child, capture_done);

        Ok(started_snapshot)
    }

    pub(crate) fn current_session(&self) -> AppResult<Option<SessionSnapshot>> {
        let inner = recover_lock(&self.inner);
        Ok(inner.current.as_ref().map(snapshot_with_diagnostics))
    }

    pub(crate) fn stop(&self, session_id: &str) -> AppResult<StopSessionResult> {
        let (child, pid) = {
            let mut inner = recover_lock(&self.inner);
            let current = current_session_mut(&mut inner, session_id)?;

            if current.snapshot.state != SessionState::Running {
                return Err(AppError::new(
                    "MAME_SESSION_NOT_RUNNING",
                    "The requested MAME session is not running.",
                )
                .with_details(serde_json::json!({
                    "sessionId": session_id,
                    "state": current.snapshot.state
                })));
            }

            transition(&mut current.snapshot, SessionState::Stopping)?;
            if let Some(control) = current.control.as_mut() {
                control.begin_close();
            }
            let child = current.child.clone().ok_or_else(|| {
                AppError::new(
                    "MAME_SESSION_CHILD_MISSING",
                    "The running MAME session has no supervised child process.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?;
            let pid = current.snapshot.pid.ok_or_else(|| {
                AppError::new(
                    "MAME_SESSION_PID_MISSING",
                    "The running MAME session has no recorded child process identifier.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?;
            (child, pid)
        };

        let soft_stop_requested = match request_soft_stop(pid) {
            Ok(()) => true,
            Err(error) => {
                let diagnostics = {
                    let inner = recover_lock(&self.inner);
                    current_session(&inner, session_id)?.diagnostics.clone()
                };
                record_diagnostic_error(
                    &diagnostics,
                    format!("Soft stop request failed; escalating: {error}"),
                );
                false
            }
        };

        let mut status = if soft_stop_requested {
            wait_for_child(&child, SOFT_STOP_TIMEOUT).map_err(|error| {
                AppError::new(
                    "MAME_STOP_WAIT_FAILED",
                    "The MAME process could not be observed during shutdown.",
                )
                .with_details(serde_json::json!({
                    "sessionId": session_id,
                    "cause": error.to_string()
                }))
            })?
        } else {
            None
        };

        let forced_termination = status.is_none();
        if forced_termination {
            {
                let mut inner = recover_lock(&self.inner);
                let current = current_session_mut(&mut inner, session_id)?;
                current.snapshot.forced_termination = true;
            }

            {
                let mut child = recover_lock(&child);
                child.kill().map_err(|error| {
                    AppError::new(
                        "MAME_FORCE_STOP_FAILED",
                        "MAME did not accept the forced termination request.",
                    )
                    .with_details(serde_json::json!({
                        "sessionId": session_id,
                        "cause": error.to_string()
                    }))
                })?;
            }

            status = wait_for_child(&child, FORCED_STOP_TIMEOUT).map_err(|error| {
                AppError::new(
                    "MAME_FORCE_STOP_WAIT_FAILED",
                    "The forced MAME termination could not be observed.",
                )
                .with_details(serde_json::json!({
                    "sessionId": session_id,
                    "cause": error.to_string()
                }))
            })?;
        }

        let status = status.ok_or_else(|| {
            AppError::new(
                "MAME_STOP_TIMEOUT",
                "MAME remained active after the bounded shutdown sequence.",
            )
            .with_details(serde_json::json!({
                "sessionId": session_id,
                "softTimeoutMs": SOFT_STOP_TIMEOUT.as_millis(),
                "forcedTimeoutMs": FORCED_STOP_TIMEOUT.as_millis()
            }))
        })?;

        settle_capture(&self.inner, session_id, &status);
        finalize_session(&self.inner, session_id, status);
        let session = self
            .current_session()?
            .filter(|session| session.session_id == session_id)
            .ok_or_else(|| {
                AppError::new(
                    "MAME_SESSION_NOT_FOUND",
                    "The requested MAME session is no longer available.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?;

        Ok(StopSessionResult {
            schema_version: 1,
            soft_stop_requested,
            forced_termination,
            session,
        })
    }
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/sessions/supervisor_runtime.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/sessions/supervisor_tests.rs"
));
