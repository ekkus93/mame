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

    pub(super) fn reset_soft(&self, session_id: &str) -> AppResult<()> {
        let _request_guard = recover_lock(&self.request_gate);
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }

        let command = CommandName::Reset;
        if !runtime_supports(&self.runtime, command.wire_name()) {
            return Err(AppError::new(
                "CONTROL_UNSUPPORTED",
                "The running MAME control shim does not advertise soft reset.",
            )
            .with_details(serde_json::json!({ "command": command.wire_name() })));
        }

        let request_id = format!(
            "req-{}",
            self.next_request_id.fetch_add(1, Ordering::Relaxed)
        );
        let mut params = Map::new();
        params.insert("kind".to_owned(), Value::String("soft".to_owned()));
        let request = RequestEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "request",
            session_id,
            request_id: &request_id,
            command: command.wire_name(),
            params,
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "CONTROL_REQUEST_SERIALIZE_FAILED",
                "The runtime-control reset request could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "PROTOCOL_MESSAGE_TOO_LARGE",
                "The runtime-control reset request exceeds the decoded message limit.",
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
                "The runtime-control reset request exceeds the encoded line limit.",
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
                "The runtime-control reset request could not be written to MAME.",
            );
            return Err(AppError::new(
                "CONTROL_CHANNEL_FAILED",
                "The runtime-control reset request could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }

        self.wait_for_reset(receiver, &request_id)
    }

}
