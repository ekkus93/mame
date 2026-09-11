// MT-709 user-mute request path.
//
// This file is included by `sessions::control` so it reuses the authenticated,
// session-scoped writer and serialized request gate. The exact MAME revision in
// this repository exposes `sound.ui_mute` to Lua, but not the live master-volume
// setter; therefore MT-709 advertises `set_mute` and deliberately leaves
// `set_volume` unsupported rather than adding a PCM path or generic Lua eval.

pub(super) fn set_session_ui_mute(session_id: &str, muted: bool) -> AppResult<(bool, bool)> {
    let handle = active_mute_handle(session_id)?;
    handle.set_ui_mute(session_id, muted)
}

fn active_mute_handle(session_id: &str) -> AppResult<ControlRequestHandle> {
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
    fn set_ui_mute(&self, session_id: &str, muted: bool) -> AppResult<(bool, bool)> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }
        if !runtime_supports(&self.runtime, "set_mute") {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise user-mute control.",
            )
            .with_details(serde_json::json!({ "command": "set_mute" })));
        }

        let request_id = format!(
            "req-{}",
            self.next_request_id.fetch_add(1, Ordering::Relaxed)
        );
        let mut params = Map::new();
        params.insert("muted".to_owned(), Value::Bool(muted));
        let request = RequestEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "request",
            session_id,
            request_id: &request_id,
            command: "set_mute",
            params,
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control mute request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control mute request exceeds the decoded message limit.",
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
                "The runtime-control mute request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        // This synchronous command only needs the legacy correlation slot to
        // route its response. A Reset token is intentionally fail-closed if a
        // peer incorrectly attempts asynchronous completion instead.
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
                "The runtime-control mute request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control mute request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_mute_response(receiver, &request_id, muted)
    }

    fn wait_for_mute_response(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        requested_muted: bool,
    ) -> AppResult<(bool, bool)> {
        let signal = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not answer the mute request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not answer the mute request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "set_mute",
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
                "completed" => extract_mute_result(response.result.as_ref(), requested_muted),
                "accepted" => Err(protocol_output_error(
                    "User mute is synchronous; an accepted response is not valid completion.",
                )),
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
                "A user-mute completion event arrived instead of a synchronous response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }
}

fn extract_mute_result(
    result: Option<&Map<String, Value>>,
    requested_muted: bool,
) -> AppResult<(bool, bool)> {
    let result = result.ok_or_else(|| {
        protocol_output_error("A completed user-mute operation omitted its result object.")
    })?;
    if result.len() != 2 {
        return Err(protocol_output_error(
            "A completed user-mute operation returned unexpected result fields.",
        ));
    }
    let ui_muted = result
        .get("uiMuted")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("User-mute completion omitted uiMuted."))?;
    let effective_muted = result
        .get("effectiveMuted")
        .and_then(Value::as_bool)
        .ok_or_else(|| protocol_output_error("User-mute completion omitted effectiveMuted."))?;
    if ui_muted != requested_muted {
        return Err(AppError::new(
            "CONTROL_OPERATION_FAILED",
            "MAME reported a user-mute state that contradicts the requested value.",
        )
        .with_details(serde_json::json!({
            "requestedMuted": requested_muted,
            "observedUiMuted": ui_muted,
            "effectiveMuted": effective_muted
        })));
    }
    Ok((ui_muted, effective_muted))
}

#[cfg(test)]
mod mt709_mute_tests {
    use std::fs;

    use super::ControlBootstrap;

    #[test]
    fn bootstrap_advertises_only_native_user_mute_and_never_pcm_transport() {
        let bootstrap = ControlBootstrap::create("mame-709-1").expect("MT-709 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("\"set_mute\""));
        assert!(!script.contains("\"set_volume\""));
        assert!(script.contains("manager.machine.sound.ui_mute"));
        assert!(script.contains("manager.machine.sound.muted"));
        assert!(!script.contains("get_samples"));
        assert!(!script.contains("register_sound_update"));
        assert!(!script.contains("add_machine_sound"));
    }
}
