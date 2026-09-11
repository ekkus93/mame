// MT-708 load-state request path.
//
// This file is included by `sessions::control` so it can reuse the authenticated
// session-scoped writer and request gate without exposing a raw control surface
// to the WebView. MAME's post-load notifier writes a one-shot completion marker
// to an application-owned path; Rust verifies the marker token and request ID.

const LOAD_STATE_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const LOAD_STATE_COMPLETION_POLL: Duration = Duration::from_millis(50);
const LOAD_STATE_COMPLETION_MARKER_LIMIT: u64 = 4_096;

pub(super) fn request_session_load(
    session_id: &str,
    path: &str,
    slot: &str,
    completion_path: &str,
) -> AppResult<()> {
    let handle = active_load_handle(session_id)?;
    handle.request_load_state(session_id, path, slot, completion_path)
}

fn active_load_handle(session_id: &str) -> AppResult<ControlRequestHandle> {
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
    fn request_load_state(
        &self,
        session_id: &str,
        path: &str,
        slot: &str,
        completion_path: &str,
    ) -> AppResult<()> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }
        if !runtime_supports(&self.runtime, "load_state") {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise load-state support.",
            )
            .with_details(serde_json::json!({ "command": "load_state" })));
        }

        remove_load_completion_marker(completion_path)?;
        let completion_token = generate_frame_token()?;
        let request_id = format!(
            "req-{}",
            self.next_request_id.fetch_add(1, Ordering::Relaxed)
        );
        let mut params = Map::new();
        params.insert("path".to_owned(), Value::String(path.to_owned()));
        params.insert("slot".to_owned(), Value::String(slot.to_owned()));
        params.insert(
            "completionPath".to_owned(),
            Value::String(completion_path.to_owned()),
        );
        params.insert(
            "completionToken".to_owned(),
            Value::String(completion_token.clone()),
        );
        let request = RequestEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "request",
            session_id,
            request_id: &request_id,
            command: "load_state",
            params,
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control load-state request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control load-state request exceeds the decoded message limit.",
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
                "The runtime-control load-state request exceeds the encoded line limit.",
            ));
        }

        let (sender, receiver) = mpsc::channel();
        // The legacy correlation enum is used only to route the single accepted
        // response. Load completion itself is proven by the authenticated,
        // one-shot completion marker written by the post-load notifier.
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
                "The runtime-control load-state request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control load-state request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_load_acceptance(receiver, &request_id, completion_path, &completion_token)
    }

    fn wait_for_load_acceptance(
        &self,
        receiver: Receiver<CommandSignal>,
        request_id: &str,
        completion_path: &str,
        completion_token: &str,
    ) -> AppResult<()> {
        let first = match recv_signal_until(&receiver, &self.state, CONTROL_RESPONSE_TIMEOUT) {
            Ok(signal) => signal,
            Err(ReceiveDeadlineError::Timeout) => {
                cancel_pending(&self.runtime, request_id);
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "The runtime-control peer did not acknowledge the load-state request before the deadline.",
                );
                return Err(AppError::new(
                    "CONTROL_RESPONSE_TIMEOUT",
                    "MAME did not acknowledge the load-state request before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "load_state",
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
                            "An accepted load-state response contained an unexpected result payload.",
                        ));
                    }
                    cancel_pending(&self.runtime, request_id);
                    self.wait_for_load_completion_marker(
                        request_id,
                        completion_path,
                        completion_token,
                    )
                }
                "completed" => {
                    cancel_pending(&self.runtime, request_id);
                    Err(protocol_output_error(
                        "Load state cannot complete synchronously; post-load notifier completion is required.",
                    ))
                }
                "rejected" => {
                    cancel_pending(&self.runtime, request_id);
                    Err(response
                        .error
                        .map(wire_error_to_app_error)
                        .unwrap_or_else(|| {
                            protocol_output_error("A rejected response omitted its error.")
                        }))
                }
                _ => {
                    cancel_pending(&self.runtime, request_id);
                    Err(protocol_output_error("The response status is invalid."))
                }
            },
            CommandSignal::Completion(_) => {
                cancel_pending(&self.runtime, request_id);
                Err(protocol_output_error(
                    "A load-state completion arrived before its request response.",
                ))
            }
            CommandSignal::Closed => Err(channel_closed_error()),
            CommandSignal::Failed(message) => Err(AppError::new("CONTROL_CHANNEL_FAILED", message)),
        }
    }

    fn wait_for_load_completion_marker(
        &self,
        request_id: &str,
        completion_path: &str,
        completion_token: &str,
    ) -> AppResult<()> {
        let deadline = Instant::now() + LOAD_STATE_COMPLETION_TIMEOUT;
        loop {
            let state = control_state(&self.state);
            if state != ControlChannelState::Ready {
                return Err(channel_state_error(state));
            }

            match read_load_completion_marker(completion_path, request_id, completion_token)? {
                Some(result) => {
                    remove_load_completion_marker(completion_path)?;
                    return result;
                }
                None => {}
            }

            if Instant::now() >= deadline {
                // A delayed post-load callback could otherwise be misattributed
                // to a later request. Fail the control channel closed on this
                // ambiguous boundary instead of permitting another command.
                set_control_state(&self.state, ControlChannelState::Failed);
                notify_channel_failed(
                    &self.runtime,
                    "MAME accepted the load-state request but did not confirm post-load completion before the deadline.",
                );
                return Err(AppError::new(
                    "LOAD_STATE_COMPLETION_TIMEOUT",
                    "MAME accepted the load-state request but did not confirm successful restoration before the deadline.",
                )
                .with_details(serde_json::json!({
                    "requestId": request_id,
                    "command": "load_state",
                    "timeoutMs": LOAD_STATE_COMPLETION_TIMEOUT.as_millis()
                })));
            }
            std::thread::sleep(LOAD_STATE_COMPLETION_POLL);
        }
    }
}

fn read_load_completion_marker(
    path: &str,
    request_id: &str,
    completion_token: &str,
) -> AppResult<Option<AppResult<()>>> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(AppError::new(
                "LOAD_STATE_COMPLETION_READ_FAILED",
                "The application-owned load-state completion marker could not be inspected.",
            )
            .with_details(serde_json::json!({
                "path": path,
                "cause": error.to_string()
            })))
        }
    };
    if metadata.len() == 0 {
        return Ok(None);
    }
    if metadata.len() > LOAD_STATE_COMPLETION_MARKER_LIMIT {
        return Err(AppError::new(
            "LOAD_STATE_COMPLETION_INVALID",
            "The load-state completion marker exceeded its bounded size.",
        )
        .with_details(serde_json::json!({ "path": path, "bytes": metadata.len() })));
    }

    let text = std::fs::read_to_string(path).map_err(|error| {
        AppError::new(
            "LOAD_STATE_COMPLETION_READ_FAILED",
            "The load-state completion marker could not be read.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string()
        }))
    })?;
    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        // The notifier writes and closes the marker synchronously; tolerate a
        // single observation while the file is still being written.
        Err(_) => return Ok(None),
    };
    let object = value.as_object().ok_or_else(|| {
        AppError::new(
            "LOAD_STATE_COMPLETION_INVALID",
            "The load-state completion marker is not a JSON object.",
        )
    })?;
    let allowed = ["token", "requestId", "status", "error"];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(AppError::new(
            "LOAD_STATE_COMPLETION_INVALID",
            "The load-state completion marker contains unexpected fields.",
        ));
    }
    if object.get("token").and_then(Value::as_str) != Some(completion_token)
        || object.get("requestId").and_then(Value::as_str) != Some(request_id)
    {
        return Err(AppError::new(
            "LOAD_STATE_COMPLETION_INVALID",
            "The load-state completion marker failed request authentication.",
        ));
    }

    match object.get("status").and_then(Value::as_str) {
        Some("ok") if object.get("error").is_none() => Ok(Some(Ok(()))),
        Some("error") => {
            let error_value = object.get("error").cloned().ok_or_else(|| {
                AppError::new(
                    "LOAD_STATE_COMPLETION_INVALID",
                    "A failed load-state completion marker omitted its error.",
                )
            })?;
            let wire: WireError = serde_json::from_value(error_value).map_err(|error| {
                AppError::new(
                    "LOAD_STATE_COMPLETION_INVALID",
                    "A failed load-state completion marker contained an invalid error envelope.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?;
            validate_wire_error(&wire)
                .map_err(|message| AppError::new("LOAD_STATE_COMPLETION_INVALID", message))?;
            Ok(Some(Err(wire_error_to_app_error(wire))))
        }
        _ => Err(AppError::new(
            "LOAD_STATE_COMPLETION_INVALID",
            "The load-state completion marker contains an invalid status/error combination.",
        )),
    }
}

fn remove_load_completion_marker(path: &str) -> AppResult<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::new(
            "LOAD_STATE_COMPLETION_CLEANUP_FAILED",
            "The application-owned load-state completion marker could not be removed.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string()
        }))),
    }
}

#[cfg(test)]
mod mt708_load_state_tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{read_load_completion_marker, ControlBootstrap};

    #[test]
    fn bootstrap_advertises_native_load_and_post_load_completion() {
        let bootstrap = ControlBootstrap::create("mame-708-1").expect("MT-708 bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
        assert!(script.contains("\"load_state\""));
        assert!(script.contains("manager.machine:load(request.params.path)"));
        assert!(script.contains("emu.add_machine_post_load_notifier"));
        assert!(script.contains("completionToken"));
    }
    #[test]
    fn completion_marker_is_request_authenticated_and_explicit() {
        let root = tempdir().expect("tempdir");
        let path = root.path().join("complete.json");
        let token = "a".repeat(43);
        fs::write(
            &path,
            serde_json::json!({
                "token": token,
                "requestId": "req-9",
                "status": "ok"
            })
            .to_string(),
        )
        .expect("completion marker");

        let path_text = path.to_str().expect("utf8 path");
        let completion = read_load_completion_marker(path_text, "req-9", &"a".repeat(43))
            .expect("valid marker")
            .expect("marker present");
        completion.expect("successful completion");

        let error = read_load_completion_marker(path_text, "req-9", &"b".repeat(43))
            .expect_err("wrong token must fail");
        assert_eq!(error.code, "LOAD_STATE_COMPLETION_INVALID");
    }
}
