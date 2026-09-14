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
        "reset" => {
            let request_id = event
                .request_id
                .as_deref()
                .ok_or_else(|| "A reset event omitted its request ID.".to_owned())?;
            if event.payload.len() != 1
                || event.payload.get("kind").and_then(Value::as_str) != Some("soft")
            {
                return Err("A reset event has invalid soft-reset semantics.".to_owned());
            }
            record_reset(runtime, request_id)
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
                Err("Non-fatal protocol_error events are not supported by MT-705.".to_owned())
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
            if pending.command.expected_paused() != Some(paused) {
                return Err("A pause-state event contradicted the pending command.".to_owned());
            }
            pending.observed_paused = Some(paused);
        } else if !correlation.abandoned.iter().any(|(abandoned_id, command)| {
            abandoned_id == request_id && command.expected_paused() == Some(paused)
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

fn record_reset(runtime: &SharedControlRuntime, request_id: &str) -> Result<(), String> {
    let mut correlation = recover_lock(&runtime.correlation);
    if let Some(pending) = correlation.pending.as_mut() {
        if pending.request_id != request_id {
            return Err("A reset event referenced an unknown request ID.".to_owned());
        }
        if pending.command != CommandName::Reset {
            return Err("A reset event contradicted the pending command.".to_owned());
        }
        pending.observed_reset = true;
        return Ok(());
    }

    if correlation
        .abandoned
        .iter()
        .any(|(abandoned_id, command)| abandoned_id == request_id && *command == CommandName::Reset)
    {
        return Ok(());
    }

    Err("A reset event referenced no outstanding or timed-out request.".to_owned())
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
