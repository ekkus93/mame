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

pub(super) fn reset_session_soft(session_id: &str) -> AppResult<()> {
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
    handle.reset_soft(session_id)
}
