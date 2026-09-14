#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandName {
    Pause,
    Resume,
    Reset,
}

impl CommandName {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Reset => "reset",
        }
    }

    fn expected_paused(self) -> Option<bool> {
        match self {
            Self::Pause => Some(true),
            Self::Resume => Some(false),
            Self::Reset => None,
        }
    }
}

struct PendingCommand {
    request_id: String,
    command: CommandName,
    sender: Sender<CommandSignal>,
    accepted: bool,
    observed_paused: Option<bool>,
    observed_reset: bool,
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
