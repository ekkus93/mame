//! Session-scoped runtime-control transport primitives for MT-703/MT-704.
//!
//! Rust owns the anonymous stdin writer. Protocol output shares MAME stdout and
//! is separated from diagnostics by a per-session unpredictable frame token.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    io::{self, Write},
    path::Path,
    process::ChildStdin,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex, MutexGuard, OnceLock,
    },
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tempfile::{Builder, NamedTempFile};

use crate::errors::{AppError, AppResult};

pub(super) const PROTOCOL_VERSION: u32 = 1;
pub(super) const MAX_DECODED_MESSAGE_BYTES: usize = 16_384;
pub(super) const MAX_ENCODED_LINE_BYTES: usize = 24_576;
pub(super) const CONTROL_READY_TIMEOUT_MS: u64 = 15_000;
const CONTROL_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const PAUSE_RESUME_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

const FRAME_PREFIX: &str = "@@MAME_TAURI_CONTROL_V1@@";
const TOKEN_BYTES: usize = 32;
const TOKEN_REDACTION: &[u8] = b"[CONTROL_TOKEN_REDACTED]";
const ABANDONED_REQUEST_LIMIT: usize = 32;
const KNOWN_COMMANDS: [&str; 9] = [
    "pause",
    "resume",
    "reset",
    "exit",
    "save_state",
    "load_state",
    "set_mute",
    "set_volume",
    "query_state",
];
const MT704_COMMANDS: [&str; 2] = ["pause", "resume"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlChannelState {
    Initializing,
    Ready,
    Closing,
    Closed,
    Failed,
}

pub(super) type SharedControlState = Arc<Mutex<ControlChannelState>>;
pub(super) type SharedControlRuntime = Arc<ControlRuntime>;
pub(super) type PauseStateSink = Arc<dyn Fn(bool) + Send + Sync>;

#[derive(Default)]
struct ControlRegistry {
    bootstraps: HashMap<String, String>,
    active: HashMap<String, ActiveControl>,
}

#[derive(Clone)]
struct ActiveControl {
    handle: ControlRequestHandle,
    runtime: SharedControlRuntime,
}

static CONTROL_REGISTRY: OnceLock<Mutex<ControlRegistry>> = OnceLock::new();

fn control_registry() -> &'static Mutex<ControlRegistry> {
    CONTROL_REGISTRY.get_or_init(|| Mutex::new(ControlRegistry::default()))
}

fn register_bootstrap(session_id: &str, frame_token: &str) -> AppResult<()> {
    let mut registry = recover_lock(control_registry());
    if registry.bootstraps.contains_key(frame_token)
        || registry
            .active
            .values()
            .any(|active| active.handle.frame_token == frame_token)
    {
        return Err(AppError::new(
            "CONTROL_TOKEN_COLLISION",
            "The generated runtime-control token collided with an active session.",
        ));
    }
    registry
        .bootstraps
        .insert(frame_token.to_owned(), session_id.to_owned());
    Ok(())
}

fn unregister_bootstrap(frame_token: &str) {
    recover_lock(control_registry())
        .bootstraps
        .remove(frame_token);
}

fn activate_control(handle: &ControlRequestHandle) -> Option<String> {
    let mut registry = recover_lock(control_registry());
    let session_id = registry.bootstraps.remove(&handle.frame_token)?;
    registry.active.insert(
        session_id.clone(),
        ActiveControl {
            handle: handle.clone(),
            runtime: handle.runtime.clone(),
        },
    );
    Some(session_id)
}

fn deactivate_control(session_id: Option<&str>, frame_token: &str) {
    let Some(session_id) = session_id else {
        return;
    };
    let mut registry = recover_lock(control_registry());
    if registry
        .active
        .get(session_id)
        .is_some_and(|active| active.handle.frame_token == frame_token)
    {
        registry.active.remove(session_id);
    }
}

fn lookup_runtime(session_id: &str, frame_token: &str) -> Option<SharedControlRuntime> {
    let registry = recover_lock(control_registry());
    let active = registry.active.get(session_id)?;
    (active.handle.frame_token == frame_token).then(|| active.runtime.clone())
}

pub(super) fn register_pause_state_sink(session_id: &str, sink: PauseStateSink) -> AppResult<()> {
    let runtime = {
        let registry = recover_lock(control_registry());
        registry
            .active
            .get(session_id)
            .map(|active| active.runtime.clone())
            .ok_or_else(|| {
                AppError::new(
                    "CONTROL_CHANNEL_CLOSED",
                    "The requested MAME session has no active runtime-control channel.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?
    };
    *recover_lock(&runtime.pause_state_sink) = Some(sink);
    Ok(())
}

pub(super) fn set_session_paused(session_id: &str, paused: bool) -> AppResult<bool> {
    let handle = {
        let registry = recover_lock(control_registry());
        registry
            .active
            .get(session_id)
            .map(|active| active.handle.clone())
            .ok_or_else(|| {
                AppError::new(
                    "CONTROL_CHANNEL_CLOSED",
                    "The requested MAME session has no active runtime-control channel.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?
    };
    handle.set_paused(session_id, paused)
}

pub(super) struct ControlBootstrap {
    file: NamedTempFile,
    frame_token: String,
}

impl ControlBootstrap {
    pub(super) fn create(session_id: &str) -> AppResult<Self> {
        let frame_token = generate_frame_token()?;
        let ready = ReadyEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "event".to_owned(),
            session_id: session_id.to_owned(),
            event: "ready".to_owned(),
            payload: ReadyPayload {
                commands: MT704_COMMANDS
                    .iter()
                    .map(|command| (*command).to_owned())
                    .collect(),
                max_message_bytes: MAX_DECODED_MESSAGE_BYTES,
            },
        };
        let ready_json = serde_json::to_vec(&ready).map_err(|error| {
            AppError::new(
                "CONTROL_BOOTSTRAP_SERIALIZE_FAILED",
                "The runtime-control ready message could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if ready_json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "CONTROL_BOOTSTRAP_MESSAGE_TOO_LARGE",
                "The runtime-control ready message exceeds the protocol limit.",
            ));
        }
        let encoded_ready = base64url_encode(&ready_json);
        let ready_frame = format!("{FRAME_PREFIX}{frame_token}@@{encoded_ready}");
        if ready_frame.len() + 1 > MAX_ENCODED_LINE_BYTES {
            return Err(AppError::new(
                "CONTROL_BOOTSTRAP_FRAME_TOO_LARGE",
                "The runtime-control ready frame exceeds the protocol line limit.",
            ));
        }

        let script = build_bootstrap_script(session_id, &frame_token, &ready_frame);
        let mut file = Builder::new()
            .prefix(".mame-tauri-control-")
            .suffix(".lua")
            .tempfile()
            .map_err(|error| bootstrap_io_error("create", error))?;
        restrict_bootstrap_permissions(file.path())?;
        file.write_all(script.as_bytes())
            .map_err(|error| bootstrap_io_error("write", error))?;
        file.flush()
            .map_err(|error| bootstrap_io_error("flush", error))?;
        file.as_file()
            .sync_all()
            .map_err(|error| bootstrap_io_error("sync", error))?;
        register_bootstrap(session_id, &frame_token)?;

        Ok(Self { file, frame_token })
    }

    pub(super) fn append_launch_arguments(&self, argv: &mut Vec<OsString>) {
        argv.push(OsString::from("-console"));
        argv.push(OsString::from("-autoboot_script"));
        argv.push(self.file.path().as_os_str().to_owned());
    }

    pub(super) fn frame_token(&self) -> &str {
        &self.frame_token
    }

    #[cfg(test)]
    fn path(&self) -> &Path {
        self.file.path()
    }
}

impl Drop for ControlBootstrap {
    fn drop(&mut self) {
        unregister_bootstrap(&self.frame_token);
    }
}

fn build_bootstrap_script(session_id: &str, frame_token: &str, ready_frame: &str) -> String {
    include_str!("control_pause_resume.lua")
        .replace("__TOKEN__", frame_token)
        .replace("__SESSION__", session_id)
        .replace("__READY_FRAME__", ready_frame)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandName {
    Pause,
    Resume,
}

impl CommandName {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
        }
    }

    fn expected_paused(self) -> bool {
        matches!(self, Self::Pause)
    }
}

struct PendingCommand {
    request_id: String,
    command: CommandName,
    sender: Sender<CommandSignal>,
    accepted: bool,
    observed_paused: Option<bool>,
}

#[derive(Default)]
struct CorrelationState {
    pending: Option<PendingCommand>,
    abandoned: VecDeque<(String, CommandName)>,
}

pub(super) struct ControlRuntime {
    capabilities: Mutex<HashSet<String>>,
    correlation: Mutex<CorrelationState>,
    pause_state_sink: Mutex<Option<PauseStateSink>>,
}

impl Default for ControlRuntime {
    fn default() -> Self {
        Self {
            capabilities: Mutex::new(HashSet::new()),
            correlation: Mutex::new(CorrelationState::default()),
            pause_state_sink: Mutex::new(None),
        }
    }
}

pub(super) struct ControlChannel {
    writer: Option<Arc<Mutex<ChildStdin>>>,
    state: SharedControlState,
    runtime: SharedControlRuntime,
    frame_token: String,
    request_gate: Arc<Mutex<()>>,
    session_id: Option<String>,
}

impl ControlChannel {
    pub(super) fn new(writer: ChildStdin, frame_token: String) -> Self {
        let writer = Arc::new(Mutex::new(writer));
        let state = Arc::new(Mutex::new(ControlChannelState::Initializing));
        let runtime = Arc::new(ControlRuntime::default());
        let request_gate = Arc::new(Mutex::new(()));
        let next_request_id = Arc::new(AtomicU64::new(1));
        let handle = ControlRequestHandle {
            writer: writer.clone(),
            state: state.clone(),
            runtime: runtime.clone(),
            frame_token: frame_token.clone(),
            request_gate: request_gate.clone(),
            next_request_id: next_request_id.clone(),
        };
        let session_id = activate_control(&handle);
        Self {
            writer: Some(writer),
            state,
            runtime,
            frame_token,
            request_gate,
            session_id,
        }
    }

    pub(super) fn shared_state(&self) -> SharedControlState {
        self.state.clone()
    }

    pub(super) fn begin_close(&mut self) {
        let request_gate = self.request_gate.clone();
        let _request_guard = recover_lock(&request_gate);
        deactivate_control(self.session_id.as_deref(), &self.frame_token);
        self.writer.take();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closing);
        }
        notify_channel_closed(&self.runtime);
        self.frame_token.clear();
    }

    pub(super) fn mark_closed(&mut self) {
        deactivate_control(self.session_id.as_deref(), &self.frame_token);
        self.writer.take();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closed);
        }
        notify_channel_closed(&self.runtime);
        self.frame_token.clear();
    }

    pub(super) fn mark_failed(&mut self) {
        deactivate_control(self.session_id.as_deref(), &self.frame_token);
        self.writer.take();
        set_control_state(&self.state, ControlChannelState::Failed);
        notify_channel_failed(&self.runtime, "The runtime-control channel failed.");
        self.frame_token.clear();
    }

    #[cfg(test)]
    pub(super) fn state(&self) -> ControlChannelState {
        control_state(&self.state)
    }
}

impl Drop for ControlChannel {
    fn drop(&mut self) {
        deactivate_control(self.session_id.as_deref(), &self.frame_token);
        self.writer.take();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closed);
        }
        notify_channel_closed(&self.runtime);
        self.frame_token.clear();
    }
}

#[derive(Clone)]
pub(super) struct ControlRequestHandle {
    writer: Arc<Mutex<ChildStdin>>,
    state: SharedControlState,
    runtime: SharedControlRuntime,
    frame_token: String,
    request_gate: Arc<Mutex<()>>,
    next_request_id: Arc<AtomicU64>,
}

impl ControlRequestHandle {
    pub(super) fn set_paused(&self, session_id: &str, paused: bool) -> AppResult<bool> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }

        let command = if paused {
            CommandName::Pause
        } else {
            CommandName::Resume
        };
        if !runtime_supports(&self.runtime, command.wire_name()) {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise this command.",
            )
            .with_details(serde_json::json!({ "command": command.wire_name() })));
        }

        let request_id = format!(
            "req-{}",
            self.next_request_id.fetch_add(1, Ordering::Relaxed)
        );
        let request = RequestEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "request",
            session_id,
            request_id: &request_id,
            command: command.wire_name(),
            params: Map::new(),
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control request exceeds the decoded message limit.",
            ));
        }
        let encoded = base64url_encode(&json);
        let line = format!(
            "mame_tauri_control_v1(\"{}\",\"{}\")\n",
            self.frame_token, encoded
        );
        if line.len() > MAX_ENCODED_LINE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        begin_request(&self.runtime, request_id.clone(), command, sender)?;
        if control_state(&self.state) != ControlChannelState::Ready {
            cancel_pending(&self.runtime, &request_id);
            return Err(channel_state_error(control_state(&self.state)));
        }

        let write_result = {
            let mut writer = recover_lock(&self.writer);
            writer
                .write_all(line.as_bytes())
                .and_then(|_| writer.flush())
        };
        if let Err(error) = write_result {
            set_control_state(&self.state, ControlChannelState::Failed);
            notify_channel_failed(
                &self.runtime,
                "The runtime-control request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_command(receiver, &request_id, command)
    }

    fn wait_for_command(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        command: CommandName,
    ) -> AppResult<bool> {
        let first = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not acknowledge the request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the runtime-control request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": command.wire_name(),
                    "timeoutMs": CONTROL_RESPONSE_TIMEOUT.as_millis()
                })));
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                return Err(channel_state_error(state));
            }
            Err(ReceiveDeadlineError::Disconnected) => return Err(channel_closed_error()),
        };

        match first {
            CommandSignal::Response(response) => match response.status.as_str() {
                "completed" => extract_paused_result(
                    response.result.as_ref(),
                    command.expected_paused(),
                    request_id,
                ),
                "accepted" => {
                    if response
                        .result
                        .as_ref()
                        .is_some_and(|result| !result.is_empty())
                    {
                        return Err(protocol_output_error(
                            "An accepted pause/resume response contained an unexpected result payload.",
                        ));
                    }
                    self.wait_for_completion(receiver, request_id, command)
                }
                "rejected" => {
                    Err(response
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A rejected response omitted its error.")
                        }))
                }
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "A command completion arrived before its request response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }

    fn wait_for_completion(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        command: CommandName,
    ) -> AppResult<bool> {
        match recv_signal_until(&receiver, &self.state, PAUSE_RESUME_COMPLETION_TIMEOUT) {
            Ok(CommandSignal::Completion(completion)) => {
                if completion.ok {
                    extract_paused_result(
                        completion.result.as_ref(),
                        command.expected_paused(),
                        request_id,
                    )
                } else {
                    Err(completion
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A failed command completion omitted its error.")
                        }))
                }
            }
            Ok(CommandSignal::Response(_)) => Err(protocol_output_error(
                "The runtime-control peer emitted more than one response for a request.",
            )),
            Ok(CommandSignal::Closed) => Err(channel_closed_error()),
            Ok(CommandSignal::Failed(message)) => {
                Err(AppError::new("CONTROL_CHANNEL_FAILED", message))
            }
            Err(ReceiveDeadlineError::Timeout) => {
                abandon_pending(&self.runtime, request_id, command);
                Err(AppError::new(
                    "CONTROL_COMPLETION_TIMEOUT",
                    "MAME accepted the pause/resume command but did not confirm the state transition before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": command.wire_name(),
                    "timeoutMs": PAUSE_RESUME_COMPLETION_TIMEOUT.as_millis()
                })))
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                Err(channel_state_error(state))
            }
            Err(ReceiveDeadlineError::Disconnected) => Err(channel_closed_error()),
        }
    }
}

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
        if completion.ok && pending.observed_paused != Some(pending.command.expected_paused()) {
            return Err(
                "A successful command completion arrived without its notifier-backed pause-state event."
                    .to_owned(),
            );
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

#[derive(Default)]
pub(super) struct ParseBatch {
    pub(super) diagnostics: Vec<u8>,
    pub(super) events: Vec<ParserEvent>,
}

pub(super) struct ControlStdoutParser {
    session_id: String,
    runtime: Option<SharedControlRuntime>,
    expected_prefix: Vec<u8>,
    line_mode: LineMode,
    redactor: SecretRedactor,
    ready_seen: bool,
    failed: bool,
}

enum LineMode {
    Matching(Vec<u8>),
    Diagnostic,
    Protocol(Vec<u8>),
    DiscardProtocol,
}

impl ControlStdoutParser {
    pub(super) fn new(session_id: String, frame_token: String) -> Self {
        let runtime = lookup_runtime(&session_id, &frame_token);
        let expected_prefix = format!("{FRAME_PREFIX}{frame_token}@@").into_bytes();
        Self {
            session_id,
            runtime,
            expected_prefix,
            line_mode: LineMode::Matching(Vec::new()),
            redactor: SecretRedactor::new(frame_token.into_bytes()),
            ready_seen: false,
            failed: false,
        }
    }

    #[cfg(test)]
    fn new_for_test(
        session_id: String,
        frame_token: String,
        runtime: SharedControlRuntime,
    ) -> Self {
        let expected_prefix = format!("{FRAME_PREFIX}{frame_token}@@").into_bytes();
        Self {
            session_id,
            runtime: Some(runtime),
            expected_prefix,
            line_mode: LineMode::Matching(Vec::new()),
            redactor: SecretRedactor::new(frame_token.into_bytes()),
            ready_seen: false,
            failed: false,
        }
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) -> ParseBatch {
        let mut batch = ParseBatch::default();
        for &byte in bytes {
            self.feed_byte(byte, &mut batch);
        }
        batch
    }

    pub(super) fn finish(&mut self) -> ParseBatch {
        let mut batch = ParseBatch::default();
        match std::mem::replace(&mut self.line_mode, LineMode::Matching(Vec::new())) {
            LineMode::Matching(buffer) => {
                self.redactor.feed(&buffer, &mut batch.diagnostics);
                self.redactor.finish_line(&mut batch.diagnostics);
            }
            LineMode::Diagnostic => self.redactor.finish_line(&mut batch.diagnostics),
            LineMode::Protocol(_) | LineMode::DiscardProtocol => {
                if !self.failed {
                    self.fail(
                        "Authenticated runtime-control output ended before a complete frame was received."
                            .to_owned(),
                        &mut batch,
                    );
                }
            }
        }
        if !self.failed {
            if let Some(runtime) = &self.runtime {
                notify_channel_closed(runtime);
            }
        }
        batch
    }

    fn feed_byte(&mut self, byte: u8, batch: &mut ParseBatch) {
        let mode = std::mem::replace(&mut self.line_mode, LineMode::Matching(Vec::new()));
        match mode {
            LineMode::Matching(mut buffer) => {
                buffer.push(byte);
                if byte == b'\n' {
                    self.redactor.feed(&buffer, &mut batch.diagnostics);
                    self.redactor.finish_line(&mut batch.diagnostics);
                    self.line_mode = LineMode::Matching(Vec::new());
                    return;
                }

                let index = buffer.len() - 1;
                if index < self.expected_prefix.len() && byte == self.expected_prefix[index] {
                    if buffer.len() == self.expected_prefix.len() {
                        self.line_mode = LineMode::Protocol(Vec::new());
                    } else {
                        self.line_mode = LineMode::Matching(buffer);
                    }
                } else {
                    self.redactor.feed(&buffer, &mut batch.diagnostics);
                    self.line_mode = LineMode::Diagnostic;
                }
            }
            LineMode::Diagnostic => {
                self.redactor.feed(&[byte], &mut batch.diagnostics);
                if byte == b'\n' {
                    self.redactor.finish_line(&mut batch.diagnostics);
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    self.line_mode = LineMode::Diagnostic;
                }
            }
            LineMode::Protocol(mut payload) => {
                if byte == b'\n' {
                    if payload.last() == Some(&b'\r') {
                        payload.pop();
                    }
                    if !self.failed {
                        match self.parse_protocol_payload(&payload) {
                            Ok(Some(event)) => batch.events.push(event),
                            Ok(None) => {}
                            Err(message) => self.fail(message, batch),
                        }
                    }
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    payload.push(byte);
                    let total_bytes = self.expected_prefix.len() + payload.len() + 1;
                    if total_bytes > MAX_ENCODED_LINE_BYTES {
                        if !self.failed {
                            self.fail(
                                "Authenticated runtime-control output exceeded the encoded line limit."
                                    .to_owned(),
                                batch,
                            );
                        }
                        self.line_mode = LineMode::DiscardProtocol;
                    } else {
                        self.line_mode = LineMode::Protocol(payload);
                    }
                }
            }
            LineMode::DiscardProtocol => {
                if byte == b'\n' {
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    self.line_mode = LineMode::DiscardProtocol;
                }
            }
        }
    }

    fn parse_protocol_payload(&mut self, encoded: &[u8]) -> Result<Option<ParserEvent>, String> {
        let encoded = std::str::from_utf8(encoded).map_err(|_| {
            "Authenticated runtime-control payload is not ASCII base64url.".to_owned()
        })?;
        let decoded = base64url_decode(encoded).map_err(|message| {
            format!("Authenticated runtime-control base64url is invalid: {message}")
        })?;
        if decoded.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(
                "Authenticated runtime-control message exceeded the decoded JSON limit.".to_owned(),
            );
        }

        let runtime = self.runtime.as_ref().ok_or_else(|| {
            "The authenticated runtime-control stream has no active session runtime.".to_owned()
        })?;

        if !self.ready_seen {
            let ready: ReadyEnvelope = serde_json::from_slice(&decoded).map_err(|error| {
                format!("Authenticated runtime-control JSON is invalid: {error}")
            })?;
            validate_ready(&ready, &self.session_id)?;
            {
                let mut capabilities = recover_lock(&runtime.capabilities);
                capabilities.clear();
                capabilities.extend(ready.payload.commands);
            }
            self.ready_seen = true;
            return Ok(Some(ParserEvent::Ready));
        }

        let value: Value = serde_json::from_slice(&decoded)
            .map_err(|error| format!("Authenticated runtime-control JSON is invalid: {error}"))?;
        let message_type = value.get("type").and_then(Value::as_str).ok_or_else(|| {
            "Authenticated runtime-control output omitted its message type.".to_owned()
        })?;
        match message_type {
            "response" => {
                let response = parse_response(value, &self.session_id)?;
                deliver_response(runtime, response)?;
            }
            "event" => handle_event_output(value, &self.session_id, runtime)?,
            _ => {
                return Err(
                    "Authenticated runtime-control output has an unknown message type.".to_owned(),
                )
            }
        }
        Ok(None)
    }

    fn fail(&mut self, message: String, batch: &mut ParseBatch) {
        self.failed = true;
        if let Some(runtime) = &self.runtime {
            notify_channel_failed(runtime, &message);
        }
        batch.events.push(ParserEvent::Fatal(message));
    }
}

fn parse_response(value: Value, expected_session_id: &str) -> Result<ControlResponse, String> {
    let response: ResponseEnvelope = serde_json::from_value(value)
        .map_err(|error| format!("Runtime-control response envelope is invalid: {error}"))?;
    validate_common_output(
        response.version,
        &response.message_type,
        &response.session_id,
        expected_session_id,
    )?;
    validate_request_id(&response.request_id)?;
    match response.status.as_str() {
        "completed" | "accepted" => {
            if !response.ok || response.result.is_none() || response.error.is_some() {
                return Err(
                    "A successful runtime-control response has inconsistent fields.".to_owned(),
                );
            }
        }
        "rejected" => {
            if response.ok || response.error.is_none() || response.result.is_some() {
                return Err(
                    "A rejected runtime-control response has inconsistent fields.".to_owned(),
                );
            }
            validate_wire_error(response.error.as_ref().expect("checked above"))?;
        }
        _ => return Err("Runtime-control response status is not valid in protocol v1.".to_owned()),
    }
    Ok(ControlResponse {
        request_id: response.request_id,
        status: response.status,
        result: response.result,
        error: response.error,
    })
}

fn handle_event_output(
    value: Value,
    expected_session_id: &str,
    runtime: &SharedControlRuntime,
) -> Result<(), String> {
    let event: EventEnvelope = serde_json::from_value(value)
        .map_err(|error| format!("Runtime-control event envelope is invalid: {error}"))?;
    validate_common_output(
        event.version,
        &event.message_type,
        &event.session_id,
        expected_session_id,
    )?;
    if let Some(request_id) = event.request_id.as_deref() {
        validate_request_id(request_id)?;
    }

    match event.event.as_str() {
        "paused" | "resumed" => {
            if event.payload.len() != 1 {
                return Err("A pause-state event has unexpected payload fields.".to_owned());
            }
            let paused = event
                .payload
                .get("paused")
                .and_then(Value::as_bool)
                .ok_or_else(|| {
                    "A pause-state event omitted its boolean paused value.".to_owned()
                })?;
            if paused != (event.event == "paused") {
                return Err("A pause-state event contradicts its event name.".to_owned());
            }
            record_pause_state(runtime, paused, event.request_id.as_deref())?;
            Ok(())
        }
        "command_completed" => {
            let request_id = event
                .request_id
                .ok_or_else(|| "A command completion omitted its request ID.".to_owned())?;
            let payload: CommandCompletedPayload =
                serde_json::from_value(Value::Object(event.payload))
                    .map_err(|error| format!("Command completion payload is invalid: {error}"))?;
            if !KNOWN_COMMANDS.contains(&payload.command.as_str()) {
                return Err("A command completion named an unknown protocol command.".to_owned());
            }
            match payload.ok {
                true if payload.result.is_some() && payload.error.is_none() => {}
                false if payload.error.is_some() && payload.result.is_none() => {
                    validate_wire_error(payload.error.as_ref().expect("checked above"))?;
                }
                _ => {
                    return Err(
                        "A command completion has inconsistent success/error fields.".to_owned(),
                    )
                }
            }
            deliver_completion(
                runtime,
                CommandCompletion {
                    request_id,
                    command: payload.command,
                    ok: payload.ok,
                    result: payload.result,
                    error: payload.error,
                },
            )
        }
        "protocol_error" => {
            if event.request_id.is_some() {
                return Err("A protocol_error event must not carry a request ID.".to_owned());
            }
            let payload: ProtocolErrorPayload =
                serde_json::from_value(Value::Object(event.payload))
                    .map_err(|error| format!("Protocol error payload is invalid: {error}"))?;
            validate_wire_error(&payload.error)?;
            if payload.fatal {
                Err(format!("{}: {}", payload.error.code, payload.error.message))
            } else {
                Err("Non-fatal protocol_error events are not supported by MT-704.".to_owned())
            }
        }
        _ => Err("Authenticated runtime-control output used an unsupported event name.".to_owned()),
    }
}

fn record_pause_state(
    runtime: &SharedControlRuntime,
    paused: bool,
    request_id: Option<&str>,
) -> Result<(), String> {
    if let Some(request_id) = request_id {
        let mut correlation = recover_lock(&runtime.correlation);
        if let Some(pending) = correlation.pending.as_mut() {
            if pending.request_id != request_id {
                return Err("A pause-state event referenced an unknown request ID.".to_owned());
            }
            if pending.command.expected_paused() != paused {
                return Err("A pause-state event contradicted the pending command.".to_owned());
            }
            pending.observed_paused = Some(paused);
        } else if !correlation.abandoned.iter().any(|(abandoned_id, command)| {
            abandoned_id == request_id && command.expected_paused() == paused
        }) {
            return Err(
                "A pause-state event referenced no outstanding or timed-out request.".to_owned(),
            );
        }
    }

    let sink = recover_lock(&runtime.pause_state_sink).clone();
    if let Some(sink) = sink {
        sink(paused);
    }
    Ok(())
}

fn validate_common_output(
    version: u32,
    message_type: &str,
    session_id: &str,
    expected_session_id: &str,
) -> Result<(), String> {
    if version != PROTOCOL_VERSION {
        return Err(format!(
            "Runtime-control output version {version} does not match supported version {PROTOCOL_VERSION}."
        ));
    }
    if message_type != "response" && message_type != "event" {
        return Err("Runtime-control output has an invalid type field.".to_owned());
    }
    if session_id != expected_session_id {
        return Err("Runtime-control output belongs to a different MAME session.".to_owned());
    }
    Ok(())
}

fn validate_request_id(request_id: &str) -> Result<(), String> {
    if request_id.is_empty()
        || request_id.len() > 64
        || !request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("Runtime-control output contains an invalid request ID.".to_owned());
    }
    Ok(())
}

fn validate_wire_error(error: &WireError) -> Result<(), String> {
    if error.code.is_empty()
        || error.code.len() > 64
        || !error.code.bytes().enumerate().all(|(index, byte)| {
            if index == 0 {
                byte.is_ascii_uppercase()
            } else {
                byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
            }
        })
    {
        return Err("Runtime-control error contains an invalid code.".to_owned());
    }
    if error.message.is_empty() {
        return Err("Runtime-control error contains an empty message.".to_owned());
    }
    Ok(())
}

struct SecretRedactor {
    secret: Vec<u8>,
    pending: VecDeque<u8>,
}

impl SecretRedactor {
    fn new(secret: Vec<u8>) -> Self {
        Self {
            secret,
            pending: VecDeque::new(),
        }
    }

    fn feed(&mut self, bytes: &[u8], output: &mut Vec<u8>) {
        for &byte in bytes {
            self.pending.push_back(byte);
            self.flush_safe_prefix(output);
        }
    }

    fn finish_line(&mut self, output: &mut Vec<u8>) {
        while let Some(byte) = self.pending.pop_front() {
            output.push(byte);
        }
    }

    fn flush_safe_prefix(&mut self, output: &mut Vec<u8>) {
        if self.secret.is_empty() {
            while let Some(byte) = self.pending.pop_front() {
                output.push(byte);
            }
            return;
        }

        while self.pending.len() >= self.secret.len() {
            let is_secret = self
                .pending
                .iter()
                .take(self.secret.len())
                .copied()
                .eq(self.secret.iter().copied());
            if is_secret {
                self.pending.drain(..self.secret.len());
                output.extend_from_slice(TOKEN_REDACTION);
            } else if let Some(byte) = self.pending.pop_front() {
                output.push(byte);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReadyEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    payload: ReadyPayload,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReadyPayload {
    commands: Vec<String>,
    #[serde(rename = "maxMessageBytes")]
    max_message_bytes: usize,
}

#[derive(Serialize)]
struct RequestEnvelope<'a> {
    version: u32,
    #[serde(rename = "type")]
    message_type: &'static str,
    #[serde(rename = "sessionId")]
    session_id: &'a str,
    #[serde(rename = "requestId")]
    request_id: &'a str,
    command: &'static str,
    params: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "requestId")]
    request_id: String,
    status: String,
    ok: bool,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    payload: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandCompletedPayload {
    command: String,
    ok: bool,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolErrorPayload {
    fatal: bool,
    error: WireError,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct WireError {
    code: String,
    message: String,
    details: Map<String, Value>,
    retryable: bool,
}

fn validate_ready(ready: &ReadyEnvelope, expected_session_id: &str) -> Result<(), String> {
    if ready.version != PROTOCOL_VERSION {
        return Err(format!(
            "Runtime-control ready version {} does not match supported version {PROTOCOL_VERSION}.",
            ready.version
        ));
    }
    if ready.message_type != "event" || ready.event != "ready" {
        return Err(
            "The first authenticated runtime-control message is not a ready event.".to_owned(),
        );
    }
    if ready.session_id != expected_session_id {
        return Err("Runtime-control ready event belongs to a different MAME session.".to_owned());
    }
    if ready.payload.max_message_bytes != MAX_DECODED_MESSAGE_BYTES {
        return Err(
            "Runtime-control peer advertised a different message-size contract.".to_owned(),
        );
    }

    let mut commands = HashSet::new();
    for command in &ready.payload.commands {
        if !KNOWN_COMMANDS.contains(&command.as_str()) {
            return Err(format!(
                "Runtime-control peer advertised unknown command {command:?}."
            ));
        }
        if !commands.insert(command.as_str()) {
            return Err(format!(
                "Runtime-control peer advertised duplicate command {command:?}."
            ));
        }
    }
    Ok(())
}

fn extract_paused_result(
    result: Option<&Map<String, Value>>,
    expected: bool,
    request_id: &str,
) -> AppResult<bool> {
    let result = result.ok_or_else(|| {
        protocol_output_error("A completed pause/resume operation omitted its result object.")
    })?;
    if result.len() != 1 {
        return Err(protocol_output_error(
            "A completed pause/resume operation returned unexpected result fields.",
        ));
    }
    let paused = result
        .get("paused")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            protocol_output_error("A completed pause/resume operation omitted its paused state.")
        })?;
    if paused != expected {
        return Err(AppError::new(
            "CONTROL_OPERATION_FAILED",
            "MAME reported a pause state that contradicts the requested operation.",
        )
        .with_details(serde_json::json!({
            "requestId": request_id,
            "expectedPaused": expected,
            "observedPaused": paused
        })));
    }
    Ok(paused)
}

fn wire_error_to_app_error(error: WireError) -> AppError {
    AppError {
        code: error.code,
        message: error.message,
        details: Value::Object(error.details),
        retryable: error.retryable,
    }
}

fn channel_closed_error() -> AppError {
    AppError::new(
        "CONTROL_CHANNEL_CLOSED",
        "The MAME runtime-control channel is not available.",
    )
}

fn channel_state_error(state: ControlChannelState) -> AppError {
    match state {
        ControlChannelState::Failed => AppError::new(
            "CONTROL_CHANNEL_FAILED",
            "The MAME runtime-control channel has failed.",
        ),
        _ => channel_closed_error(),
    }
}

fn protocol_output_error(message: &str) -> AppError {
    AppError::new("CONTROL_CHANNEL_FAILED", message)
}

fn generate_frame_token() -> AppResult<String> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    fill_os_random(&mut bytes)?;
    let token = base64url_encode(&bytes);
    debug_assert_eq!(token.len(), 43);
    Ok(token)
}

#[cfg(unix)]
fn fill_os_random(bytes: &mut [u8]) -> AppResult<()> {
    use std::{fs::File, io::Read};

    File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(bytes))
        .map_err(random_source_error)
}

#[cfg(windows)]
fn fill_os_random(bytes: &mut [u8]) -> AppResult<()> {
    use std::{ffi::c_void, io};

    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x0000_0002;

    #[link(name = "bcrypt")]
    extern "system" {
        #[link_name = "BCryptGenRandom"]
        fn bcrypt_gen_random(
            algorithm: *mut c_void,
            buffer: *mut u8,
            buffer_len: u32,
            flags: u32,
        ) -> i32;
    }

    let buffer_len = u32::try_from(bytes.len()).map_err(|error| {
        AppError::new(
            "CONTROL_TOKEN_GENERATION_FAILED",
            "The runtime-control token buffer exceeds the Windows random-source limit.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    // SAFETY: BCryptGenRandom receives a valid writable buffer for exactly
    // `buffer_len` bytes, a null algorithm handle as required by
    // BCRYPT_USE_SYSTEM_PREFERRED_RNG, and does not retain either pointer.
    let status = unsafe {
        bcrypt_gen_random(
            std::ptr::null_mut(),
            bytes.as_mut_ptr(),
            buffer_len,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    if status >= 0 {
        Ok(())
    } else {
        Err(random_source_error(io::Error::other(format!(
            "BCryptGenRandom failed with NTSTATUS 0x{:08X}",
            status as u32
        ))))
    }
}

#[cfg(not(any(unix, windows)))]
fn fill_os_random(_bytes: &mut [u8]) -> AppResult<()> {
    Err(AppError::new(
        "CONTROL_TOKEN_GENERATION_FAILED",
        "No cryptographically secure runtime-control token source is implemented for this platform.",
    ))
}

fn random_source_error(error: io::Error) -> AppError {
    AppError::new(
        "CONTROL_TOKEN_GENERATION_FAILED",
        "A cryptographically secure per-session runtime-control token could not be generated.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn base64url_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    let (chunks, remainder) = input.as_chunks::<3>();
    for chunk in chunks {
        let value = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(value & 0x3f) as usize] as char);
    }
    match remainder {
        [a] => {
            let value = u32::from(*a) << 16;
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        }
        [a, b] => {
            let value = (u32::from(*a) << 16) | (u32::from(*b) << 8);
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        }
        [] => {}
        _ => unreachable!("as_chunks remainder is shorter than three bytes"),
    }
    output
}

fn base64url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = input.as_bytes();
    if bytes.len() % 4 == 1 {
        return Err("invalid unpadded length");
    }
    let mut output = Vec::with_capacity((bytes.len() * 3) / 4 + 2);
    let mut index = 0;
    while index + 4 <= bytes.len() {
        let a = decode_sextet(bytes[index])?;
        let b = decode_sextet(bytes[index + 1])?;
        let c = decode_sextet(bytes[index + 2])?;
        let d = decode_sextet(bytes[index + 3])?;
        output.push((a << 2) | (b >> 4));
        output.push((b << 4) | (c >> 2));
        output.push((c << 6) | d);
        index += 4;
    }
    match bytes.len() - index {
        0 => {}
        2 => {
            let a = decode_sextet(bytes[index])?;
            let b = decode_sextet(bytes[index + 1])?;
            if b & 0x0f != 0 {
                return Err("non-canonical trailing bits");
            }
            output.push((a << 2) | (b >> 4));
        }
        3 => {
            let a = decode_sextet(bytes[index])?;
            let b = decode_sextet(bytes[index + 1])?;
            let c = decode_sextet(bytes[index + 2])?;
            if c & 0x03 != 0 {
                return Err("non-canonical trailing bits");
            }
            output.push((a << 2) | (b >> 4));
            output.push((b << 4) | (c >> 2));
        }
        _ => return Err("invalid unpadded length"),
    }
    Ok(output)
}

fn decode_sextet(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'-' => Ok(62),
        b'_' => Ok(63),
        _ => Err("invalid base64url character"),
    }
}

fn bootstrap_io_error(operation: &str, error: io::Error) -> AppError {
    AppError::new(
        "CONTROL_BOOTSTRAP_IO_FAILED",
        "The private runtime-control bootstrap script could not be prepared.",
    )
    .with_details(serde_json::json!({
        "operation": operation,
        "cause": error.to_string()
    }))
}

#[cfg(unix)]
fn restrict_bootstrap_permissions(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|error| {
        AppError::new(
            "CONTROL_BOOTSTRAP_PERMISSIONS_FAILED",
            "The private runtime-control bootstrap script permissions could not be restricted.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(not(unix))]
fn restrict_bootstrap_permissions(_path: &Path) -> AppResult<()> {
    Ok(())
}

fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{mpsc, Arc, Mutex},
    };

    use serde_json::Value;

    use super::{
        abandon_pending, base64url_decode, base64url_encode, begin_request, deliver_completion,
        deliver_response, generate_frame_token, notify_channel_closed, record_pause_state,
        CommandCompletion, CommandName, CommandSignal, ControlBootstrap, ControlResponse,
        ControlRuntime, ControlStdoutParser, ParserEvent, FRAME_PREFIX, MAX_DECODED_MESSAGE_BYTES,
        TOKEN_REDACTION,
    };

    #[test]
    fn generated_tokens_are_32_random_bytes_in_unpadded_base64url() {
        let first = generate_frame_token().expect("first token");
        let second = generate_frame_token().expect("second token");
        assert_eq!(first.len(), 43);
        assert_eq!(base64url_decode(&first).expect("decode first").len(), 32);
        assert!(first
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'));
        assert_ne!(first, second);
    }

    #[test]
    fn base64url_round_trips_protocol_payloads_without_padding() {
        for payload in [b"".as_slice(), b"a", b"ab", b"abc", b"protocol-v1"] {
            let encoded = base64url_encode(payload);
            assert!(!encoded.contains('='));
            assert_eq!(base64url_decode(&encoded).expect("decode"), payload);
        }
    }

    #[test]
    fn bootstrap_is_private_and_advertises_pause_resume() {
        let bootstrap = ControlBootstrap::create("mame-123-1").expect("bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("emu.add_machine_pause_notifier"));
        assert!(script.contains("emu.add_machine_resume_notifier"));
        assert!(script.contains("pcall(emu.pause)"));
        assert!(script.contains("pcall(emu.unpause)"));
        let ready_frame = script
            .lines()
            .find_map(|line| {
                line.strip_prefix("local ready_frame = \"")?
                    .strip_suffix('"')
            })
            .expect("ready frame literal");
        let expected_prefix = format!("{FRAME_PREFIX}{}@@", bootstrap.frame_token());
        let encoded = ready_frame
            .strip_prefix(&expected_prefix)
            .expect("session token prefix");
        let decoded = base64url_decode(encoded).expect("ready base64url");
        let ready: Value = serde_json::from_slice(&decoded).expect("ready JSON");
        assert_eq!(ready["version"], 1);
        assert_eq!(ready["type"], "event");
        assert_eq!(ready["sessionId"], "mame-123-1");
        assert_eq!(ready["event"], "ready");
        assert_eq!(
            ready["payload"]["commands"],
            serde_json::json!(["pause", "resume"])
        );
        assert_eq!(
            ready["payload"]["maxMessageBytes"],
            MAX_DECODED_MESSAGE_BYTES
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(bootstrap.path())
                .expect("bootstrap metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0);
        }
    }

    #[test]
    fn parser_accepts_ready_response_state_and_completion_frames() {
        let token = "A".repeat(43);
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "ready",
            "payload": {"commands": ["pause", "resume"], "maxMessageBytes": 16384}
        });
        let accepted = serde_json::json!({
            "version": 1,
            "type": "response",
            "sessionId": "mame-1-1",
            "requestId": "req-1",
            "status": "accepted",
            "ok": true,
            "result": {}
        });
        let paused = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "paused",
            "requestId": "req-1",
            "payload": {"paused": true}
        });
        let completed = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "command_completed",
            "requestId": "req-1",
            "payload": {"command": "pause", "ok": true, "result": {"paused": true}}
        });
        let stream = [ready, accepted, paused, completed]
            .into_iter()
            .map(|message| protocol_frame(&token, &message))
            .collect::<String>();
        let split = stream.len() / 2;
        let mut parser =
            ControlStdoutParser::new_for_test("mame-1-1".to_owned(), token, runtime.clone());
        let first = parser.feed(&stream.as_bytes()[..split]);
        let second = parser.feed(&stream.as_bytes()[split..]);
        let events: Vec<_> = first.events.into_iter().chain(second.events).collect();
        assert_eq!(events, vec![ParserEvent::Ready]);
        assert!(matches!(
            receiver.recv().expect("accepted signal"),
            CommandSignal::Response(_)
        ));
        assert!(matches!(
            receiver.recv().expect("completion signal"),
            CommandSignal::Completion(_)
        ));
    }

    #[test]
    fn command_correlation_accepts_late_completion_after_local_timeout() {
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        let response = ControlResponse {
            request_id: "req-1".to_owned(),
            status: "accepted".to_owned(),
            result: Some(serde_json::Map::new()),
            error: None,
        };
        deliver_response(&runtime, response).expect("accepted response");
        assert!(matches!(
            receiver.recv().expect("accepted signal"),
            CommandSignal::Response(_)
        ));
        abandon_pending(&runtime, "req-1", CommandName::Pause);
        drop(receiver);
        record_pause_state(&runtime, true, Some("req-1")).expect("late pause state");
        let completion = CommandCompletion {
            request_id: "req-1".to_owned(),
            command: "pause".to_owned(),
            ok: true,
            result: Some(serde_json::Map::from_iter([(
                "paused".to_owned(),
                Value::Bool(true),
            )])),
            error: None,
        };
        deliver_completion(&runtime, completion).expect("late completion is correlation-safe");
    }

    #[test]
    fn pause_state_notifier_drives_registered_state_sink_until_disconnect() {
        let runtime = Arc::new(ControlRuntime::default());
        let observed = Arc::new(Mutex::new(Vec::new()));
        let observed_for_sink = observed.clone();
        *super::recover_lock(&runtime.pause_state_sink) = Some(Arc::new(move |paused| {
            super::recover_lock(&observed_for_sink).push(paused);
        }));

        record_pause_state(&runtime, true, None).expect("independent pause state");
        assert_eq!(*super::recover_lock(&observed), vec![true]);

        notify_channel_closed(&runtime);
        record_pause_state(&runtime, false, None).expect("late state remains parseable");
        assert_eq!(
            *super::recover_lock(&observed),
            vec![true],
            "disconnect must detach the application state-event sink"
        );
    }

    #[test]
    fn channel_close_resolves_outstanding_command_immediately() {
        let runtime = std::sync::Arc::new(ControlRuntime::default());
        let (sender, receiver) = mpsc::channel();
        begin_request(&runtime, "req-1".to_owned(), CommandName::Pause, sender)
            .expect("begin request");
        notify_channel_closed(&runtime);
        assert!(matches!(
            receiver.recv().expect("closed signal"),
            CommandSignal::Closed
        ));
    }

    #[test]
    fn wrong_token_is_diagnostic_but_current_token_is_redacted_from_console_echo() {
        let token = "B".repeat(43);
        let wrong = "C".repeat(43);
        let input = format!(
            "{FRAME_PREFIX}{wrong}@@garbage\nmame_tauri_control_v1(\"{token}\",\"payload\")\n"
        );
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token.clone());
        let mut batch = parser.feed(input.as_bytes());
        let finished = parser.finish();
        batch.diagnostics.extend(finished.diagnostics);
        let text = String::from_utf8(batch.diagnostics).expect("diagnostics utf8");
        assert!(text.contains(&wrong));
        assert!(!text.contains(&token));
        assert!(text.contains(std::str::from_utf8(TOKEN_REDACTION).expect("redaction utf8")));
        assert!(batch.events.is_empty());
    }

    #[test]
    fn authenticated_malformed_frame_fails_closed() {
        let token = "D".repeat(43);
        let input = format!("{FRAME_PREFIX}{token}@@not+base64\n");
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token);
        let batch = parser.feed(input.as_bytes());
        assert!(batch.diagnostics.is_empty());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }

    fn protocol_frame(token: &str, message: &Value) -> String {
        let encoded = base64url_encode(&serde_json::to_vec(message).expect("serialize frame"));
        format!("{FRAME_PREFIX}{token}@@{encoded}\n")
    }
}
