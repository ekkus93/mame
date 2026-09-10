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

fn current_session<'a>(
    inner: &'a SupervisorInner,
    session_id: &str,
) -> AppResult<&'a ManagedSession> {
    let current = inner.current.as_ref().ok_or_else(|| {
        AppError::new("MAME_SESSION_NOT_FOUND", "No MAME session is available.")
            .with_details(serde_json::json!({ "sessionId": session_id }))
    })?;

    if current.snapshot.session_id != session_id {
        return Err(AppError::new(
            "MAME_SESSION_NOT_FOUND",
            "The requested MAME session is not available.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "currentSessionId": current.snapshot.session_id
        })));
    }

    Ok(current)
}

fn current_session_mut<'a>(
    inner: &'a mut SupervisorInner,
    session_id: &str,
) -> AppResult<&'a mut ManagedSession> {
    let current = inner.current.as_mut().ok_or_else(|| {
        AppError::new("MAME_SESSION_NOT_FOUND", "No MAME session is available.")
            .with_details(serde_json::json!({ "sessionId": session_id }))
    })?;

    if current.snapshot.session_id != session_id {
        return Err(AppError::new(
            "MAME_SESSION_NOT_FOUND",
            "The requested MAME session is not available.",
        )
        .with_details(serde_json::json!({
            "sessionId": session_id,
            "currentSessionId": current.snapshot.session_id
        })));
    }

    Ok(current)
}

fn transition(snapshot: &mut SessionSnapshot, next: SessionState) -> AppResult<()> {
    if !snapshot.state.allows(next) {
        return Err(AppError::new(
            "MAME_SESSION_INVALID_TRANSITION",
            "The MAME session lifecycle transition is invalid.",
        )
        .with_details(serde_json::json!({
            "sessionId": snapshot.session_id,
            "from": snapshot.state,
            "to": next
        })));
    }

    snapshot.state = next;
    Ok(())
}

fn snapshot_with_diagnostics(session: &ManagedSession) -> SessionSnapshot {
    let diagnostics = recover_lock(&session.diagnostics);
    let mut snapshot = session.snapshot.clone();
    snapshot.stdout_tail = diagnostics.stdout.text();
    snapshot.stderr_tail = diagnostics.stderr.text();
    snapshot.stdout_truncated = diagnostics.stdout.truncated;
    snapshot.stderr_truncated = diagnostics.stderr.truncated;
    snapshot.diagnostic_error = diagnostics.diagnostic_error.clone();
    snapshot
}

fn spawn_controlled_stdout_capture<R: Read + Send + 'static>(
    mut reader: R,
    diagnostics: SharedDiagnostics,
    mut parser: ControlStdoutParser,
    control_state: SharedControlState,
    ready_observed: Arc<AtomicBool>,
    capture_done: Arc<AtomicU8>,
) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    apply_control_parse_batch(
                        parser.finish(),
                        &diagnostics,
                        &control_state,
                        &ready_observed,
                    );
                    if matches!(
                        super::control::control_state(&control_state),
                        ControlChannelState::Initializing | ControlChannelState::Ready
                    ) {
                        set_control_state(&control_state, ControlChannelState::Closed);
                    }
                    break;
                }
                Ok(count) => apply_control_parse_batch(
                    parser.feed(&buffer[..count]),
                    &diagnostics,
                    &control_state,
                    &ready_observed,
                ),
                Err(error) => {
                    record_diagnostic_error(
                        &diagnostics,
                        format!("MAME stdout/control stream read failed: {error}"),
                    );
                    set_control_state(&control_state, ControlChannelState::Failed);
                    break;
                }
            }
        }
        capture_done.fetch_add(1, Ordering::Release);
    });
}

fn apply_control_parse_batch(
    batch: super::control::ParseBatch,
    diagnostics: &SharedDiagnostics,
    state: &SharedControlState,
    ready_observed: &AtomicBool,
) {
    if !batch.diagnostics.is_empty() {
        recover_lock(diagnostics).stdout.append(&batch.diagnostics);
    }
    for event in batch.events {
        match event {
            ParserEvent::Ready => {
                ready_observed.store(true, Ordering::Release);
                if control_state(state) == ControlChannelState::Initializing {
                    set_control_state(state, ControlChannelState::Ready);
                }
            }
            ParserEvent::Fatal(message) => {
                record_diagnostic_error(
                    diagnostics,
                    format!("Runtime-control protocol failed: {message}"),
                );
                set_control_state(state, ControlChannelState::Failed);
            }
        }
    }
}

fn wait_for_control_ready(
    state: &SharedControlState,
    ready_observed: &AtomicBool,
    timeout: Duration,
) -> AppResult<()> {
    let started = Instant::now();
    loop {
        if ready_observed.load(Ordering::Acquire) {
            return Ok(());
        }

        match control_state(state) {
            ControlChannelState::Failed => {
                return Err(AppError::new(
                    "CONTROL_CHANNEL_FAILED",
                    "The MAME runtime-control channel failed during initialization.",
                ));
            }
            ControlChannelState::Closing | ControlChannelState::Closed => {
                return Err(AppError::new(
                    "CONTROL_CHANNEL_CLOSED",
                    "The MAME runtime-control channel closed before it became ready.",
                ));
            }
            ControlChannelState::Initializing | ControlChannelState::Ready => {}
        }

        if started.elapsed() >= timeout {
            set_control_state(state, ControlChannelState::Failed);
            return Err(AppError::new(
                "CONTROL_READY_TIMEOUT",
                "MAME did not establish the authenticated runtime-control channel before the deadline.",
            )
            .with_details(serde_json::json!({ "timeoutMs": timeout.as_millis() })));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn fail_control_launch(
    inner: &Arc<Mutex<SupervisorInner>>,
    session_id: &str,
    child: &Arc<Mutex<Child>>,
    cause: &AppError,
) {
    {
        let mut inner = recover_lock(inner);
        if let Ok(current) = current_session_mut(&mut inner, session_id) {
            if let Some(control) = current.control.as_mut() {
                control.mark_failed();
            }
        }
    }

    let termination_error = terminate_supervised_child(child).err();
    mark_launch_failed(inner, session_id, &cause.code, cause.message.clone());
    if let Some(error) = termination_error {
        let inner = recover_lock(inner);
        if let Ok(current) = current_session(&inner, session_id) {
            record_diagnostic_error(
                &current.diagnostics,
                format!(
                    "{}: {}; child cleanup also failed: {error}",
                    cause.code, cause.message
                ),
            );
        }
    }
}

fn terminate_supervised_child(child: &Arc<Mutex<Child>>) -> io::Result<()> {
    let mut child = recover_lock(child);
    if child.try_wait()?.is_none() {
        child.kill()?;
        let _ = child.wait()?;
    }
    Ok(())
}

fn spawn_capture<R: Read + Send + 'static>(
    mut reader: R,
    diagnostics: SharedDiagnostics,
    capture_done: Arc<AtomicU8>,
) {
    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => recover_lock(&diagnostics).stderr.append(&buffer[..count]),
                Err(error) => {
                    record_diagnostic_error(
                        &diagnostics,
                        format!("MAME diagnostic stream read failed: {error}"),
                    );
                    break;
                }
            }
        }
        capture_done.fetch_add(1, Ordering::Release);
    });
}

fn spawn_exit_watcher(
    inner: Arc<Mutex<SupervisorInner>>,
    session_id: String,
    child: Arc<Mutex<Child>>,
    capture_done: Arc<AtomicU8>,
) {
    thread::spawn(move || loop {
        let status = {
            let mut child = recover_lock(&child);
            child.try_wait()
        };

        match status {
            Ok(Some(status)) => {
                wait_for_capture_completion(&capture_done);
                finalize_session(&inner, &session_id, status);
                break;
            }
            Ok(None) => thread::sleep(CHILD_POLL_INTERVAL),
            Err(error) => {
                mark_supervision_failed(
                    &inner,
                    &session_id,
                    format!("MAME child status observation failed: {error}"),
                );
                break;
            }
        }
    });
}

fn settle_capture(inner: &Arc<Mutex<SupervisorInner>>, session_id: &str, _status: &ExitStatus) {
    let diagnostics = {
        let inner = recover_lock(inner);
        match current_session(&inner, session_id) {
            Ok(session) => session.diagnostics.clone(),
            Err(_) => return,
        }
    };

    let started = Instant::now();
    while started.elapsed() < CAPTURE_SETTLE_TIMEOUT {
        let diagnostics = recover_lock(&diagnostics);
        if diagnostics.diagnostic_error.is_some() {
            break;
        }
        drop(diagnostics);
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_capture_completion(capture_done: &AtomicU8) {
    let started = Instant::now();
    while capture_done.load(Ordering::Acquire) < 2 && started.elapsed() < CAPTURE_SETTLE_TIMEOUT {
        thread::sleep(Duration::from_millis(10));
    }
}

fn finalize_session(inner: &Arc<Mutex<SupervisorInner>>, session_id: &str, status: ExitStatus) {
    let (event_name, event_sink, event) = {
        let mut inner = recover_lock(inner);
        let current = match current_session_mut(&mut inner, session_id) {
            Ok(current) => current,
            Err(_) => return,
        };

        if !current.snapshot.state.is_active() {
            return;
        }

        if let Some(control) = current.control.as_mut() {
            control.mark_closed();
        }
        current.snapshot.exit_code = status.code();
        current.snapshot.termination_signal = termination_signal(&status);
        match epoch_millis() {
            Ok(timestamp) => current.snapshot.ended_at_epoch_ms = Some(timestamp),
            Err(error) => record_diagnostic_error(
                &current.diagnostics,
                format!(
                    "Could not record MAME session end timestamp: {}",
                    error.message
                ),
            ),
        }

        let next = if current.snapshot.state == SessionState::Stopping || status.success() {
            SessionState::Exited
        } else {
            SessionState::Crashed
        };

        if let Err(error) = transition(&mut current.snapshot, next) {
            record_diagnostic_error(
                &current.diagnostics,
                format!("MAME lifecycle finalization failed: {}", error.message),
            );
            current.snapshot.state = SessionState::Failed;
        }

        let event_name = match current.snapshot.state {
            SessionState::Crashed => "session.crashed",
            SessionState::Failed => "session.failed",
            _ => "session.exited",
        };
        let event_sink = current.event_sink.clone();
        let event = SessionLifecycleEventV1 {
            schema_version: 1,
            session: snapshot_with_diagnostics(current),
        };
        (event_name, event_sink, event)
    };

    if let Err(error) = event_sink(event_name, event) {
        let diagnostics = {
            let inner = recover_lock(inner);
            match current_session(&inner, session_id) {
                Ok(current) => current.diagnostics.clone(),
                Err(_) => return,
            }
        };
        record_diagnostic_error(
            &diagnostics,
            format!("{event_name} event emission failed: {error}"),
        );
    }
}

fn mark_launch_failed(
    inner: &Arc<Mutex<SupervisorInner>>,
    session_id: &str,
    code: &str,
    message: String,
) {
    let mut inner = recover_lock(inner);
    let current = match current_session_mut(&mut inner, session_id) {
        Ok(current) => current,
        Err(_) => return,
    };
    if let Some(control) = current.control.as_mut() {
        control.mark_failed();
    }
    current.snapshot.state = SessionState::Failed;
    record_diagnostic_error(&current.diagnostics, format!("{code}: {message}"));
    if let Ok(timestamp) = epoch_millis() {
        current.snapshot.ended_at_epoch_ms = Some(timestamp);
    }
}

fn mark_supervision_failed(inner: &Arc<Mutex<SupervisorInner>>, session_id: &str, message: String) {
    let (event_sink, event) = {
        let mut inner = recover_lock(inner);
        let current = match current_session_mut(&mut inner, session_id) {
            Ok(current) => current,
            Err(_) => return,
        };
        if let Some(control) = current.control.as_mut() {
            control.mark_failed();
        }
        current.snapshot.state = SessionState::Failed;
        record_diagnostic_error(&current.diagnostics, message);
        match epoch_millis() {
            Ok(timestamp) => current.snapshot.ended_at_epoch_ms = Some(timestamp),
            Err(error) => record_diagnostic_error(
                &current.diagnostics,
                format!("Could not record MAME failure timestamp: {}", error.message),
            ),
        }
        (
            current.event_sink.clone(),
            SessionLifecycleEventV1 {
                schema_version: 1,
                session: snapshot_with_diagnostics(current),
            },
        )
    };

    if let Err(error) = event_sink("session.failed", event) {
        let inner = recover_lock(inner);
        if let Ok(current) = current_session(&inner, session_id) {
            record_diagnostic_error(
                &current.diagnostics,
                format!("session.failed event emission failed: {error}"),
            );
        }
    }
}

fn record_diagnostic_error(diagnostics: &SharedDiagnostics, message: String) {
    let mut diagnostics = recover_lock(diagnostics);
    diagnostics.diagnostic_error = Some(message.chars().take(DIAGNOSTIC_ERROR_LIMIT).collect());
}

fn wait_for_child(child: &Arc<Mutex<Child>>, timeout: Duration) -> io::Result<Option<ExitStatus>> {
    let started = Instant::now();
    loop {
        let status = {
            let mut child = recover_lock(child);
            child.try_wait()?
        };
        if status.is_some() {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            return Ok(None);
        }
        thread::sleep(CHILD_POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn request_soft_stop(pid: u32) -> Result<(), String> {
    let status = Command::new("/bin/kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("could not execute /bin/kill: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("/bin/kill exited with status {status}"))
    }
}

#[cfg(windows)]
fn request_soft_stop(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill.exe")
        .args(["/PID", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("could not execute taskkill.exe: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("taskkill.exe exited with status {status}"))
    }
}

#[cfg(not(any(unix, windows)))]
fn request_soft_stop(_pid: u32) -> Result<(), String> {
    Err("soft process termination is not implemented for this platform".to_owned())
}

fn terminate_unusable_child(child: &mut Child) {
    if let Err(error) = child.kill() {
        eprintln!("failed to terminate unusable MAME child: {error}");
    }
    if let Err(error) = child.wait() {
        eprintln!("failed to reap unusable MAME child: {error}");
    }
}

fn epoch_millis() -> AppResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                "SYSTEM_CLOCK_INVALID",
                "The system clock cannot represent the current session timestamp.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;

    u64::try_from(duration.as_millis()).map_err(|error| {
        AppError::new(
            "SYSTEM_CLOCK_OVERFLOW",
            "The current session timestamp exceeds the supported range.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn new_session_id(created_at_epoch_ms: u64) -> String {
    let sequence = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    format!("mame-{created_at_epoch_ms}-{sequence}")
}

fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(unix)]
fn termination_signal(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn termination_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use crate::mame::{MameExecutableSource, MameLaunchTarget};

    use super::{
        recover_lock, ControlChannelState, EffectiveLaunchConfig, EventSink,
        SessionLifecycleEventV1, SessionState, SessionSupervisor, DIAGNOSTIC_TAIL_LIMIT,
    };

    fn no_op_sink() -> EventSink {
        Arc::new(|_, _| Ok(()))
    }

    fn recording_sink(events: Arc<Mutex<Vec<String>>>) -> EventSink {
        Arc::new(move |name, _event: SessionLifecycleEventV1| {
            recover_lock(&events).push(name.to_owned());
            Ok(())
        })
    }

    #[test]
    fn state_machine_rejects_terminal_transitions() {
        assert!(!SessionState::Exited.allows(SessionState::Running));
        assert!(!SessionState::Crashed.allows(SessionState::Running));
        assert!(SessionState::Running.allows(SessionState::Stopping));
        assert!(SessionState::Running.allows(SessionState::Crashed));
    }

    #[cfg(unix)]
    #[test]
    fn starts_captures_and_records_clean_exit() {
        let root = unique_temp_dir("clean-exit 日本語");
        let executable = write_fake_mame(
            &root,
            "printf '%s\\n' 'stdout-from-mame'\nprintf '%s\\n' 'stderr-from-mame' >&2\nexit 0\n",
        );
        let events = Arc::new(Mutex::new(Vec::new()));
        let supervisor = SessionSupervisor::default();

        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                recording_sink(events.clone()),
            )
            .expect("fake MAME must launch");
        assert_eq!(started.state, SessionState::Running);
        assert!(started.pid.is_some());

        let exited = wait_for_terminal(&supervisor);
        assert_eq!(exited.state, SessionState::Exited);
        assert_eq!(exited.exit_code, Some(0));
        assert!(exited.stdout_tail.contains("stdout-from-mame"));
        assert!(exited.stderr_tail.contains("stderr-from-mame"));
        let events = recover_lock(&events);
        assert!(events.iter().any(|name| name == "session.started"));
        assert!(events.iter().any(|name| name == "session.exited"));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_exit_is_reported_as_crash() {
        let root = unique_temp_dir("crash-exit");
        let executable =
            write_fake_mame(&root, "printf '%s\\n' 'fatal fake failure' >&2\nexit 7\n");
        let supervisor = SessionSupervisor::default();

        supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let crashed = wait_for_terminal(&supervisor);
        assert_eq!(crashed.state, SessionState::Crashed);
        assert_eq!(crashed.exit_code, Some(7));
        assert!(crashed.stderr_tail.contains("fatal fake failure"));

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn diagnostics_are_bounded_to_recent_tail() {
        let root = unique_temp_dir("bounded-output");
        let executable = write_fake_mame(
            &root,
            "i=0\nwhile [ \"$i\" -lt 9000 ]; do\n  printf '0123456789'\n  i=$((i + 1))\ndone\nexit 0\n",
        );
        let supervisor = SessionSupervisor::default();

        supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");

        let exited = wait_for_terminal(&supervisor);
        assert!(exited.stdout_truncated);
        assert!(exited.stdout_tail.len() <= DIAGNOSTIC_TAIL_LIMIT);

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn duplicate_active_launch_is_rejected() {
        let root = unique_temp_dir("duplicate-session");
        let executable = write_fake_mame(&root, "trap 'exit 0' TERM\nwhile :; do :; done\n");
        let supervisor = SessionSupervisor::default();

        let first = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("first fake MAME must launch");

        let error = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("galaga"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect_err("second active launch must be rejected");
        assert_eq!(error.code, "MAME_SESSION_ALREADY_ACTIVE");

        supervisor
            .stop(&first.session_id)
            .expect("first fake MAME must stop");
        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn control_endpoint_is_session_scoped_and_torn_down_with_session() {
        let root = unique_temp_dir("control-endpoint");
        let executable = write_fake_mame(&root, "trap 'exit 0' TERM\nwhile :; do :; done\n");
        let supervisor = SessionSupervisor::default();

        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME control endpoint must become ready");

        assert!(started.effective_argv.iter().any(|arg| arg == "-console"));
        assert!(started
            .effective_argv
            .iter()
            .any(|arg| arg == "-autoboot_script"));
        assert!(!started.stdout_tail.contains("@@MAME_TAURI_CONTROL_V1@@"));
        {
            let inner = recover_lock(&supervisor.inner);
            let current = inner.current.as_ref().expect("managed session");
            let control = current.control.as_ref().expect("control channel");
            assert_eq!(control.state(), ControlChannelState::Ready);
        }

        supervisor
            .stop(&started.session_id)
            .expect("fake MAME must stop");
        {
            let inner = recover_lock(&supervisor.inner);
            let current = inner.current.as_ref().expect("managed session");
            let control = current.control.as_ref().expect("control channel");
            assert_eq!(control.state(), ControlChannelState::Closed);
        }

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    #[test]
    fn stop_escalates_when_soft_termination_is_ignored() {
        let root = unique_temp_dir("forced-stop");
        let executable = write_fake_mame(&root, "trap '' TERM\nwhile :; do :; done\n");
        let supervisor = SessionSupervisor::default();

        let started = supervisor
            .launch(
                MameExecutableSource::external(&executable),
                target("pacman"),
                EffectiveLaunchConfig {
                    project_paths: Vec::new(),
                },
                no_op_sink(),
            )
            .expect("fake MAME must launch");
        let stopped = supervisor
            .stop(&started.session_id)
            .expect("forced stop must complete");

        assert!(stopped.soft_stop_requested);
        assert!(stopped.forced_termination);
        assert_eq!(stopped.session.state, SessionState::Exited);
        assert!(stopped.session.forced_termination);

        fs::remove_dir_all(root).expect("remove fake MAME directory");
    }

    #[cfg(unix)]
    fn write_fake_mame(root: &PathBuf, launch_body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(root).expect("create fake MAME directory");
        let executable = root.join("fake mame executable");
        let script = format!(
            r#"#!/bin/sh
if [ "$1" = '-noreadconfig' ] && [ "$2" = '-version' ]; then
  printf '%s\n' '0.288 test-build'
  exit 0
fi
bootstrap=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = '-autoboot_script' ]; then
    shift
    bootstrap=$1
  fi
  shift
done
if [ -n "$bootstrap" ]; then
  ready_frame=$(sed -n 's/^local ready_frame = "\(.*\)"$/\1/p' "$bootstrap")
  if [ -n "$ready_frame" ]; then
    printf '\n%s\n' "$ready_frame"
  fi
fi
{launch_body}"#
        );
        fs::write(&executable, script).expect("write fake MAME executable");
        let mut permissions = fs::metadata(&executable)
            .expect("read fake MAME metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("mark fake MAME executable");
        executable
    }

    #[cfg(unix)]
    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-session-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[cfg(unix)]
    fn wait_for_terminal(supervisor: &SessionSupervisor) -> super::SessionSnapshot {
        let started = std::time::Instant::now();
        loop {
            let snapshot = supervisor
                .current_session()
                .expect("session snapshot must be available")
                .expect("session must exist");
            if !snapshot.state.is_active() {
                return snapshot;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "fake MAME did not reach terminal state"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn target(machine: &str) -> MameLaunchTarget {
        MameLaunchTarget {
            machine: machine.to_owned(),
            software: None,
            project_paths: Vec::new(),
        }
    }
}
