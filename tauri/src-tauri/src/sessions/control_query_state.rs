// MT-710 bounded runtime-state query path.
//
// This file is included by `sessions::control` so the query reuses the
// authenticated session-scoped writer and serialized request gate. The result
// is intentionally small and read-only: running, paused, machine identity and
// native mute state only.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeQueryState {
    pub(super) running: bool,
    pub(super) paused: bool,
    pub(super) machine: String,
    pub(super) ui_muted: bool,
    pub(super) effective_muted: bool,
}

pub(super) fn query_session_state(session_id: &str) -> AppResult<RuntimeQueryState> {
    let handle = active_query_state_handle(session_id)?;
    handle.query_state(session_id)
}

fn active_query_state_handle(session_id: &str) -> AppResult<ControlRequestHandle> {
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
        })
}

impl ControlRequestHandle {
    fn query_state(&self, session_id: &str) -> AppResult<RuntimeQueryState> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }
        if !runtime_supports(&self.runtime, "query_state") {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise runtime-state queries.",
            )
            .with_details(serde_json::json!({ "command": "query_state" })));
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
            command: "query_state",
            params: Map::new(),
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control query-state request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control query-state request exceeds the decoded message limit.",
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
                "The runtime-control query-state request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        // Query-state is synchronous. The legacy correlation slot is used only
        // to route its single response; asynchronous completion is invalid.
        begin_request(
            &self.runtime,
            request_id.clone(),
            CommandName::Reset,
            sender,
        )?;
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
            cancel_pending(&self.runtime, &request_id);
            set_control_state(&self.state, ControlChannelState::Failed);
            notify_channel_failed(
                &self.runtime,
                "The runtime-control query-state request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control query-state request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_query_state_response(receiver, &request_id)
    }

    fn wait_for_query_state_response(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
    ) -> AppResult<RuntimeQueryState> {
        let signal = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not answer the query-state request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not answer the runtime-state query before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "query_state",
                    "timeoutMs": CONTROL_RESPONSE_TIMEOUT.as_millis()
                })));
            }
            Err(ReceiveDeadlineError::ChannelState(state)) => {
                cancel_pending(&self.runtime, request_id);
                return Err(channel_state_error(state));
            }
            Err(ReceiveDeadlineError::Disconnected) => return Err(channel_closed_error()),
        };

        match signal {
            CommandSignal::Response(response) => match response.status.as_str() {
                "completed" => extract_query_state_result(response.result.as_ref()),
                "accepted" => Err(protocol_output_error(
                    "Runtime-state query is synchronous; an accepted response is not valid completion.",
                )),
                "rejected" => Err(response
                    .error
                    .map(wire_error_to_app_error)
                    .unwrap_or_else(|| {
                        protocol_output_error("A rejected response omitted its error.")
                    })),
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "A runtime-state completion event arrived instead of a synchronous response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }
}

fn extract_query_state_result(result: Option<&Map<String, Value>>) -> AppResult<RuntimeQueryState> {
    let result = result.ok_or_else(|| {
        protocol_output_error("A completed runtime-state query omitted its result object.")
    })?;
    if result.len() != 5 {
        return Err(protocol_output_error(
            "A completed runtime-state query returned unexpected result fields.",
        ));
    }
    let running = result
        .get("running")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("Runtime-state query omitted running."))?;
    if !running {
        return Err(protocol_output_error(
            "A ready runtime-control peer reported that its machine is not running.",
        ));
    }
    let paused = result
        .get("paused")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("Runtime-state query omitted paused."))?;
    let machine = result
        .get("machine")
        .and_then(Value::as_str)
        .ok_or_else(|| protocol_output_error("Runtime-state query omitted machine identity."))?;
    if machine.is_empty() || machine.len() > 128 {
        return Err(protocol_output_error(
            "Runtime-state query returned an invalid machine identity.",
        ));
    }
    let ui_muted = result
        .get("uiMuted")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("Runtime-state query omitted uiMuted."))?;
    let effective_muted = result
        .get("effectiveMuted")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("Runtime-state query omitted effectiveMuted."))?;

    Ok(RuntimeQueryState {
        running,
        paused,
        machine: machine.to_owned(),
        ui_muted,
        effective_muted,
    })
}

#[cfg(test)]
mod mt710_query_state_tests {
    use std::fs;

    use super::ControlBootstrap;

    #[test]
    fn bootstrap_advertises_bounded_read_only_query_state() {
        let bootstrap = ControlBootstrap::create("mame-710-1").expect("MT-710 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("\"query_state\""));
        assert!(script.contains("manager.machine.paused"));
        assert!(script.contains("manager.machine.system.name"));
        assert!(script.contains("manager.machine.sound.ui_mute"));
        assert!(script.contains("manager.machine.sound.muted"));
        assert!(!script.contains("manager.machine.devices"));
        assert!(!script.contains("memory"));
    }
}
