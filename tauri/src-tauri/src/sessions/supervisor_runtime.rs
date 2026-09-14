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
                if control_state(state) == ControlChannelState::Initializing {
                    set_control_state(state, ControlChannelState::Ready);
                    ready_observed.store(true, Ordering::Release);
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
        let current_state = control_state(state);
        if current_state == ControlChannelState::Failed {
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The MAME runtime-control channel failed during initialization.",
            ));
        }
        if ready_observed.load(Ordering::Acquire) {
            return Ok(());
        }
        match current_state {
            ControlChannelState::Closing | ControlChannelState::Closed => {
                return Err(AppError::new(
                    "CONTROL_CHANNEL_CLOSED",
                    "The MAME runtime-control channel closed before it became ready.",
                ));
            }
            ControlChannelState::Initializing | ControlChannelState::Ready => {}
            ControlChannelState::Failed => unreachable!("failed state handled above"),
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
