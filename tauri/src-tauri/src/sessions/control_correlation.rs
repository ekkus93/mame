#[derive(Debug)]
enum ReceiveDeadlineError {
    Timeout,
    ChannelState(ControlChannelState),
    Disconnected,
}

fn recv_signal_until(
    receiver: &Receiver<CommandSignal>,
    state: &SharedControlState,
    timeout: Duration,
) -> Result<CommandSignal, ReceiveDeadlineError> {
    const POLL_INTERVAL: Duration = Duration::from_millis(50);
    let deadline = Instant::now() + timeout;
    loop {
        let current_state = control_state(state);
        if current_state != ControlChannelState::Ready {
            return Err(ReceiveDeadlineError::ChannelState(current_state));
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(ReceiveDeadlineError::Timeout);
        }
        let remaining = deadline.saturating_duration_since(now);
        let wait = remaining.min(POLL_INTERVAL);
        match receiver.recv_timeout(wait) {
            Ok(signal) => return Ok(signal),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Err(ReceiveDeadlineError::Disconnected),
        }
    }
}

fn begin_request(
    runtime: &SharedControlRuntime,
    request_id: String,
    command: CommandName,
    sender: Sender<CommandSignal>,
) -> AppResult<()> {
    let mut correlation = recover_lock(&runtime.correlation);
    if correlation.pending.is_some() {
        return Err(AppError::new(
            "CONTROL_OPERATION_FAILED",
            "Another runtime-control command is already in flight.",
        )
        .with_details(serde_json::json!({ "retryAfterCurrentCommand": true })));
    }
    correlation.pending = Some(PendingCommand {
        request_id,
        command,
        sender,
        accepted: false,
        observed_paused: None,
        observed_reset: false,
    });
    Ok(())
}

fn cancel_pending(runtime: &SharedControlRuntime, request_id: &str) {
    let mut correlation = recover_lock(&runtime.correlation);
    if correlation
        .pending
        .as_ref()
        .is_some_and(|pending| pending.request_id == request_id)
    {
        correlation.pending = None;
    }
}

fn abandon_pending(runtime: &SharedControlRuntime, request_id: &str, command: CommandName) {
    let mut correlation = recover_lock(&runtime.correlation);
    if correlation
        .pending
        .as_ref()
        .is_some_and(|pending| pending.request_id == request_id)
    {
        correlation.pending = None;
        correlation
            .abandoned
            .push_back((request_id.to_owned(), command));
        while correlation.abandoned.len() > ABANDONED_REQUEST_LIMIT {
            correlation.abandoned.pop_front();
        }
    }
}

fn runtime_supports(runtime: &SharedControlRuntime, command: &str) -> bool {
    recover_lock(&runtime.capabilities).contains(command)
}

pub(super) fn control_state(state: &SharedControlState) -> ControlChannelState {
    *recover_lock(state)
}

pub(super) fn set_control_state(state: &SharedControlState, next: ControlChannelState) {
    *recover_lock(state) = next;
}

pub(super) fn notify_channel_closed(runtime: &SharedControlRuntime) {
    recover_lock(&runtime.pause_state_sink).take();
    let mut correlation = recover_lock(&runtime.correlation);
    correlation.abandoned.clear();
    if let Some(pending) = correlation.pending.take() {
        let _ = pending.sender.send(CommandSignal::Closed);
    }
}

pub(super) fn notify_channel_failed(runtime: &SharedControlRuntime, message: &str) {
    recover_lock(&runtime.pause_state_sink).take();
    let mut correlation = recover_lock(&runtime.correlation);
    correlation.abandoned.clear();
    if let Some(pending) = correlation.pending.take() {
        let _ = pending
            .sender
            .send(CommandSignal::Failed(message.to_owned()));
    }
}

#[derive(Debug)]
enum CommandSignal {
    Response(ControlResponse),
    Completion(CommandCompletion),
    Closed,
    Failed(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ParserEvent {
    Ready,
    Fatal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ControlResponse {
    request_id: String,
    status: String,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CommandCompletion {
    request_id: String,
    command: String,
    ok: bool,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

fn deliver_response(
    runtime: &SharedControlRuntime,
    response: ControlResponse,
) -> Result<(), String> {
    let mut correlation = recover_lock(&runtime.correlation);
    let pending = correlation.pending.as_mut().ok_or_else(|| {
        "A response referenced no outstanding runtime-control request.".to_owned()
    })?;
    if pending.request_id != response.request_id {
        return Err("A response referenced an unknown runtime-control request ID.".to_owned());
    }
    if pending.accepted {
        return Err(
            "The runtime-control peer emitted more than one response for a request.".to_owned(),
        );
    }

    if response.status == "accepted" {
        pending.accepted = true;
        pending
            .sender
            .send(CommandSignal::Response(response))
            .map_err(|_| {
                "The runtime-control response receiver disappeared unexpectedly.".to_owned()
            })?;
        return Ok(());
    }

    let pending = correlation
        .pending
        .take()
        .expect("pending request checked above");
    let _ = pending.sender.send(CommandSignal::Response(response));
    Ok(())
}

fn deliver_completion(
    runtime: &SharedControlRuntime,
    completion: CommandCompletion,
) -> Result<(), String> {
    let mut correlation = recover_lock(&runtime.correlation);
    if let Some(pending) = correlation.pending.as_ref() {
        if pending.request_id != completion.request_id {
            return Err("A command completion referenced an unknown request ID.".to_owned());
        }
        if pending.command.wire_name() != completion.command {
            return Err(
                "A command completion named a different command than its request.".to_owned(),
            );
        }
        if !pending.accepted {
            return Err("A command completion arrived before its accepted response.".to_owned());
        }
        if completion.ok {
            match pending.command {
                CommandName::Pause | CommandName::Resume => {
                    if pending.observed_paused != pending.command.expected_paused() {
                        return Err(
                            "A successful pause/resume completion arrived without its notifier-backed pause-state event."
                                .to_owned(),
                        );
                    }
                }
                CommandName::Reset => {
                    if !pending.observed_reset {
                        return Err(
                            "A successful reset completion arrived without its notifier-backed reset event."
                                .to_owned(),
                        );
                    }
                }
            }
        }
        let pending = correlation
            .pending
            .take()
            .expect("pending request checked above");
        let _ = pending.sender.send(CommandSignal::Completion(completion));
        return Ok(());
    }

    if let Some(index) = correlation
        .abandoned
        .iter()
        .position(|(request_id, command)| {
            request_id == &completion.request_id && command.wire_name() == completion.command
        })
    {
        correlation.abandoned.remove(index);
        return Ok(());
    }

    Err("A command completion referenced no outstanding or timed-out request.".to_owned())
}
