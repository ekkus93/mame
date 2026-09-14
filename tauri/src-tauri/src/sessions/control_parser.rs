#[derive(Default)]
pub(super) struct ParseBatch {
    pub(super) diagnostics: Vec<u8>,
    pub(super) events: Vec<ParserEvent>,
}

pub(super) struct ControlStdoutParser {
    session_id: String,
    runtime: Option<SharedControlRuntime>,
    expected_prefix: Vec<u8>,
    line_mode: LineMode,
    redactor: SecretRedactor,
    ready_seen: bool,
    failed: bool,
}

enum LineMode {
    Matching(Vec<u8>),
    Diagnostic,
    Protocol(Vec<u8>),
    DiscardProtocol,
}

impl ControlStdoutParser {
    pub(super) fn new(session_id: String, frame_token: String) -> Self {
        let runtime = lookup_runtime(&session_id, &frame_token);
        let expected_prefix = format!("{FRAME_PREFIX}{frame_token}@@").into_bytes();
        Self {
            session_id,
            runtime,
            expected_prefix,
            line_mode: LineMode::Matching(Vec::new()),
            redactor: SecretRedactor::new(frame_token.into_bytes()),
            ready_seen: false,
            failed: false,
        }
    }

    #[cfg(test)]
    fn new_for_test(
        session_id: String,
        frame_token: String,
        runtime: SharedControlRuntime,
    ) -> Self {
        let expected_prefix = format!("{FRAME_PREFIX}{frame_token}@@").into_bytes();
        Self {
            session_id,
            runtime: Some(runtime),
            expected_prefix,
            line_mode: LineMode::Matching(Vec::new()),
            redactor: SecretRedactor::new(frame_token.into_bytes()),
            ready_seen: false,
            failed: false,
        }
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) -> ParseBatch {
        let mut batch = ParseBatch::default();
        for &byte in bytes {
            self.feed_byte(byte, &mut batch);
        }
        batch
    }

    pub(super) fn finish(&mut self) -> ParseBatch {
        let mut batch = ParseBatch::default();
        match std::mem::replace(&mut self.line_mode, LineMode::Matching(Vec::new())) {
            LineMode::Matching(buffer) => {
                self.redactor.feed(&buffer, &mut batch.diagnostics);
                self.redactor.finish_line(&mut batch.diagnostics);
            }
            LineMode::Diagnostic => self.redactor.finish_line(&mut batch.diagnostics),
            LineMode::Protocol(_) | LineMode::DiscardProtocol => {
                if !self.failed {
                    self.fail(
                        "Authenticated runtime-control output ended before a complete frame was received."
                            .to_owned(),
                        &mut batch,
                    );
                }
            }
        }
        if !self.failed {
            if let Some(runtime) = &self.runtime {
                notify_channel_closed(runtime);
            }
        }
        batch
    }

    fn feed_byte(&mut self, byte: u8, batch: &mut ParseBatch) {
        let mode = std::mem::replace(&mut self.line_mode, LineMode::Matching(Vec::new()));
        match mode {
            LineMode::Matching(mut buffer) => {
                buffer.push(byte);
                if byte == b'\n' {
                    self.redactor.feed(&buffer, &mut batch.diagnostics);
                    self.redactor.finish_line(&mut batch.diagnostics);
                    self.line_mode = LineMode::Matching(Vec::new());
                    return;
                }

                let index = buffer.len() - 1;
                if index < self.expected_prefix.len() && byte == self.expected_prefix[index] {
                    if buffer.len() == self.expected_prefix.len() {
                        self.line_mode = LineMode::Protocol(Vec::new());
                    } else {
                        self.line_mode = LineMode::Matching(buffer);
                    }
                } else {
                    self.redactor.feed(&buffer, &mut batch.diagnostics);
                    self.line_mode = LineMode::Diagnostic;
                }
            }
            LineMode::Diagnostic => {
                self.redactor.feed(&[byte], &mut batch.diagnostics);
                if byte == b'\n' {
                    self.redactor.finish_line(&mut batch.diagnostics);
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    self.line_mode = LineMode::Diagnostic;
                }
            }
            LineMode::Protocol(mut payload) => {
                if byte == b'\n' {
                    if payload.last() == Some(&b'\r') {
                        payload.pop();
                    }
                    if !self.failed {
                        match self.parse_protocol_payload(&payload) {
                            Ok(Some(event)) => batch.events.push(event),
                            Ok(None) => {}
                            Err(message) => self.fail(message, batch),
                        }
                    }
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    payload.push(byte);
                    let total_bytes = self.expected_prefix.len() + payload.len() + 1;
                    if total_bytes > MAX_ENCODED_LINE_BYTES {
                        if !self.failed {
                            self.fail(
                                "Authenticated runtime-control output exceeded the encoded line limit."
                                    .to_owned(),
                                batch,
                            );
                        }
                        self.line_mode = LineMode::DiscardProtocol;
                    } else {
                        self.line_mode = LineMode::Protocol(payload);
                    }
                }
            }
            LineMode::DiscardProtocol => {
                if byte == b'\n' {
                    self.line_mode = LineMode::Matching(Vec::new());
                } else {
                    self.line_mode = LineMode::DiscardProtocol;
                }
            }
        }
    }

    fn parse_protocol_payload(&mut self, encoded: &[u8]) -> Result<Option<ParserEvent>, String> {
        let encoded = std::str::from_utf8(encoded).map_err(|_| {
            "Authenticated runtime-control payload is not ASCII base64url.".to_owned()
        })?;
        let decoded = base64url_decode(encoded).map_err(|message| {
            format!("Authenticated runtime-control base64url is invalid: {message}")
        })?;
        if decoded.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(
                "Authenticated runtime-control message exceeded the decoded JSON limit.".to_owned(),
            );
        }

        let runtime = self.runtime.as_ref().ok_or_else(|| {
            "The authenticated runtime-control stream has no active session runtime.".to_owned()
        })?;

        if !self.ready_seen {
            let ready: ReadyEnvelope = serde_json::from_slice(&decoded).map_err(|error| {
                format!("Authenticated runtime-control JSON is invalid: {error}")
            })?;
            validate_ready(&ready, &self.session_id)?;
            {
                let mut capabilities = recover_lock(&runtime.capabilities);
                capabilities.clear();
                capabilities.extend(ready.payload.commands);
            }
            self.ready_seen = true;
            return Ok(Some(ParserEvent::Ready));
        }

        let value: Value = serde_json::from_slice(&decoded)
            .map_err(|error| format!("Authenticated runtime-control JSON is invalid: {error}"))?;
        let message_type = value.get("type").and_then(Value::as_str).ok_or_else(|| {
            "Authenticated runtime-control output omitted its message type.".to_owned()
        })?;
        match message_type {
            "response" => {
                let response = parse_response(value, &self.session_id)?;
                deliver_response(runtime, response)?;
            }
            "event" => handle_event_output(value, &self.session_id, runtime)?,
            _ => {
                return Err(
                    "Authenticated runtime-control output has an unknown message type.".to_owned(),
                )
            }
        }
        Ok(None)
    }

    fn fail(&mut self, message: String, batch: &mut ParseBatch) {
        self.failed = true;
        if let Some(runtime) = &self.runtime {
            notify_channel_failed(runtime, &message);
        }
        batch.events.push(ParserEvent::Fatal(message));
    }
}
