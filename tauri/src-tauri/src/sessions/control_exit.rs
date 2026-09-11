// MT-706 protocol-exit bootstrap and request path.
//
// This file is textually included by the `sessions::control` module so it can
// reuse the established MT-703/704/705 transport internals without widening
// their visibility to the rest of the crate.

const MT706_COMMANDS: [&str; 4] = ["pause", "resume", "reset", "exit"];

impl ControlBootstrap {
    pub(super) fn create_mt706(session_id: &str) -> AppResult<Self> {
        let frame_token = generate_frame_token()?;
        let ready = ReadyEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "event".to_owned(),
            session_id: session_id.to_owned(),
            event: "ready".to_owned(),
            payload: ReadyPayload {
                commands: MT706_COMMANDS
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
}

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
        // Existing correlation records need a bounded enum value. Exit success
        // is terminally evidenced by the supervised child process, not by a
        // `command_completed` frame, so this internal sentinel is never exposed
        // as reset semantics on the wire. Any unexpected exit completion frame
        // fails closed through the existing command-mismatch check.
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
                    // Close the logical request gate before releasing it. This
                    // prevents pause/reset from racing the accepted exit.
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

    use serde_json::Value;

    use super::{base64url_decode, ControlBootstrap, FRAME_PREFIX, MAX_DECODED_MESSAGE_BYTES};

    #[test]
    fn mt706_bootstrap_advertises_clean_exit_and_uses_native_mame_exit() {
        let bootstrap = ControlBootstrap::create_mt706("mame-706-1").expect("MT-706 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("manager.machine:exit()"));

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
        assert_eq!(
            ready["payload"]["commands"],
            serde_json::json!(["pause", "resume", "reset", "exit"])
        );
        assert_eq!(
            ready["payload"]["maxMessageBytes"],
            MAX_DECODED_MESSAGE_BYTES
        );
    }
}
