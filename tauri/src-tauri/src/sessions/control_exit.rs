// MT-706 protocol-exit request path.
//
// This file is included by `sessions::control` so it can reuse the established
// session-scoped writer, request gate, correlation state, and protocol parser
// without exposing any of them to the WebView or sibling modules.

pub(super) fn request_session_exit(session_id: &str) -> AppResult<()> {
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
    handle.request_exit(session_id)
}

impl ControlRequestHandle {
    fn request_exit(&self, session_id: &str) -> AppResult<()> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }

        if !runtime_supports(&self.runtime, "exit") {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise clean exit.",
            )
            .with_details(serde_json::json!({ "command": "exit" })));
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
            command: "exit",
            params: Map::new(),
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control exit request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control exit request exceeds the decoded message limit.",
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
                "The runtime-control exit request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        // Exit is completed only by observed process termination, so the
        // existing MT-705 correlation record is used solely to route the one
        // request-response acknowledgement. No command-completion event is
        // accepted as exit success.
        begin_request(&self.runtime, request_id.clone(), CommandName::Reset, sender)?;
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
                "The runtime-control exit request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control exit request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_exit_acceptance(receiver, &request_id)
    }

    fn wait_for_exit_acceptance(
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
                    "The runtime-control peer did not acknowledge the exit request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the clean-exit request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "exit",
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
                        return Err(protocol_output_error(
                            "An accepted exit response contained an unexpected result payload.",
                        ));
                    }
                    // Make closing visible while this request still owns the
                    // request gate. Pause/reset cannot race an accepted exit.
                    set_control_state(&self.state, ControlChannelState::Closing);
                    Ok(())
                }
                "completed" => Err(protocol_output_error(
                    "Clean exit cannot complete synchronously; supervised process exit is required.",
                )),
                "rejected" => Err(response
                    .error
                    .map(wire_error_to_app_error)
                    .unwrap_or_else(|| protocol_output_error("A rejected response omitted its error."))),
                _ => Err(protocol_output_error("The response status is invalid.")),
            },
            CommandSignal::Completion(_) => Err(protocol_output_error(
                "An exit completion arrived before its request response.",
            )),
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }
}

#[cfg(test)]
mod mt706_exit_tests {
    use std::fs;

    use super::ControlBootstrap;

    #[test]
    fn bootstrap_dispatches_native_clean_exit() {
        let bootstrap = ControlBootstrap::create("mame-706-1").expect("MT-706 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("commands = { \"pause\", \"resume\", \"reset\", \"exit\" }"));
        assert!(script.contains("manager.machine:exit()"));
        assert!(script.contains("Pause, resume, and exit require an empty parameter object."));
    }
}
