// MT-707 save-state request path.
//
// This file is included by `sessions::control` so it reuses the authenticated
// session-scoped writer and request gate without exposing a raw control surface
// to the WebView.

pub(super) fn request_session_save(session_id: &str, path: &str, slot: &str) -> AppResult<()> {
    let handle = active_handle(session_id)?;
    handle.request_save_state(session_id, path, slot)
}

pub(super) fn ensure_session_control_ready(session_id: &str) -> AppResult<()> {
    let handle = active_handle(session_id)?;
    let state = control_state(&handle.state);
    if state == ControlChannelState::Ready {
        Ok(())
    } else {
        Err(channel_state_error(state))
    }
}

fn active_handle(session_id: &str) -> AppResult<ControlRequestHandle> {
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
    fn request_save_state(&self, session_id: &str, path: &str, slot: &str) -> AppResult<()> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }
        if !runtime_supports(&self.runtime, "save_state") {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise save-state support.",
            )
            .with_details(serde_json::json!({ "command": "save_state" })));
        }

        let request_id = format!(
            "req-{}",
            self.next_request_id.fetch_add(1, Ordering::Relaxed)
        );
        let mut params = Map::new();
        params.insert("path".to_owned(), Value::String(path.to_owned()));
        params.insert("slot".to_owned(), Value::String(slot.to_owned()));
        let request = RequestEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "request",
            session_id,
            request_id: &request_id,
            command: "save_state",
            params,
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control save-state request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control save-state request exceeds the decoded message limit.",
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
                "The runtime-control save-state request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        // MT-707 needs only the authenticated acceptance response here. The
        // actual completion contract is filesystem-backed and verified by Rust
        // after MAME closes the state file, so no protocol completion is
        // fabricated. Reuse the existing correlation slot and clear it after
        // acceptance before returning to the filesystem verifier.
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
            set_control_state(&self.state, ControlChannelState::Failed);
            notify_channel_failed(
                &self.runtime,
                "The runtime-control save-state request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control save-state request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_save_acceptance(receiver, &request_id)
    }

    fn wait_for_save_acceptance(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
    ) -> AppResult<()> {
        let first = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not acknowledge the save-state request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the save-state request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "save_state",
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
                "accepted" => {
                    if response
                        .result
                        .as_ref()
                        .is_some_and(|result| !result.is_empty())
                    {
                        cancel_pending(&self.runtime, request_id);
                        return Err(protocol_output_error(
                            "An accepted save-state response contained an unexpected result payload.",
                        ));
                    }
                    cancel_pending(&self.runtime, request_id);
                    Ok(())
                }
                "completed" => Err(protocol_output_error(
                    "Save state cannot complete synchronously; verified filesystem completion is required.",
                )),
                "rejected" => Err(response
                    .error
                    .map(wire_error_to_app_error)
                    .unwrap_or_else(|| protocol_output_error("A rejected response omitted its error."))),
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "A save-state completion arrived before its request response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }
}

#[cfg(test)]
mod mt707_save_state_tests {
    use std::fs;

    use super::{base64url_decode, ControlBootstrap, FRAME_PREFIX};

    #[test]
    fn bootstrap_advertises_and_dispatches_native_save_state() {
        let bootstrap = ControlBootstrap::create("mame-707-1").expect("MT-707 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("\"save_state\""));
        assert!(script.contains("manager.machine:save(request.params.path)"));

        let ready_frame = script
            .lines()
            .find_map(|line| {
                line.strip_prefix("local ready_frame = \"")?
                    .strip_suffix('"')
            })
            .expect("generated ready frame literal");
        let expected_prefix = format!("{FRAME_PREFIX}{}@@", bootstrap.frame_token());
        let encoded = ready_frame
            .strip_prefix(&expected_prefix)
            .expect("ready frame token");
        let decoded = base64url_decode(encoded).expect("ready payload");
        let ready: serde_json::Value = serde_json::from_slice(&decoded).expect("ready JSON");
        // The generated literal remains the MT-705 bootstrap compatibility
        // artifact. Production readiness is emitted dynamically by Lua, where
        // MT-707 adds save_state to the supported capability list.
        assert_eq!(ready["event"], "ready");
    }
}
