const MAX_INPUT_UPDATES: usize = 32;
const MAX_INPUT_TOKEN_BYTES: usize = 64;

#[derive(Debug, Clone)]
pub(super) struct InputUpdate {
    pub token: String,
    pub value: i16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InputEnvelope<'a> {
    version: u32,
    session_id: &'a str,
    updates: &'a [InputUpdateWire<'a>],
}

#[derive(Serialize)]
struct InputUpdateWire<'a> {
    token: &'a str,
    value: i16,
}

pub(super) fn send_session_inputs(
    session_id: &str,
    updates: &[InputUpdate],
) -> AppResult<bool> {
    if updates.is_empty() || updates.len() > MAX_INPUT_UPDATES {
        return Err(AppError::new(
            "MAME_INPUT_BATCH_INVALID",
            "A gameplay input update batch must contain between one and 32 updates.",
        )
        .with_details(serde_json::json!({ "updates": updates.len() })));
    }
    for update in updates {
        if update.token.is_empty()
            || update.token.len() > MAX_INPUT_TOKEN_BYTES
            || !update
                .token
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(AppError::new(
                "MAME_INPUT_TOKEN_INVALID",
                "A gameplay input token is invalid.",
            )
            .with_details(serde_json::json!({ "token": update.token })));
        }
    }

    let handle = {
        let registry = recover_lock(control_registry());
        registry
            .active
            .get(session_id)
            .map(|active| active.handle.clone())
            .ok_or_else(|| {
                AppError::new(
                    "CONTROL_CHANNEL_CLOSED",
                    "The requested MAME session has no active gameplay input channel.",
                )
                .with_details(serde_json::json!({ "sessionId": session_id }))
            })?
    };
    handle.send_inputs(session_id, updates)
}

impl ControlRequestHandle {
    fn send_inputs(&self, session_id: &str, updates: &[InputUpdate]) -> AppResult<bool> {
        if control_state(&self.state) != ControlChannelState::Ready {
            return Err(channel_state_error(control_state(&self.state)));
        }

        let request_guard = match self.request_gate.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(false),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        };

        if control_state(&self.state) != ControlChannelState::Ready {
            drop(request_guard);
            return Err(channel_state_error(control_state(&self.state)));
        }

        let wire_updates: Vec<_> = updates
            .iter()
            .map(|update| InputUpdateWire {
                token: &update.token,
                value: update.value,
            })
            .collect();
        let request = InputEnvelope {
            version: PROTOCOL_VERSION,
            session_id,
            updates: &wire_updates,
        };
        let json = serde_json::to_vec(&request).map_err(|error| {
            AppError::new(
                "MAME_INPUT_SERIALIZE_FAILED",
                "The gameplay input update could not be serialized.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
        if json.len() > MAX_DECODED_MESSAGE_BYTES {
            return Err(AppError::new(
                "MAME_INPUT_BATCH_TOO_LARGE",
                "The gameplay input update exceeds the bounded message limit.",
            ));
        }
        let encoded = base64url_encode(&json);
        let line = format!(
            "mame_tauri_input_v1(\"{}\",\"{}\")\n",
            self.frame_token, encoded
        );
        if line.len() > MAX_ENCODED_LINE_BYTES {
            return Err(AppError::new(
                "MAME_INPUT_BATCH_TOO_LARGE",
                "The encoded gameplay input update exceeds the bounded line limit.",
            ));
        }

        let mut writer = match self.writer.try_lock() {
            Ok(writer) => writer,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(false),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        };
        if let Err(error) = writer
            .write_all(line.as_bytes())
            .and_then(|_| writer.flush())
        {
            set_control_state(&self.state, ControlChannelState::Failed);
            notify_channel_failed(
                &self.runtime,
                "The gameplay input update could not be written to MAME.",
            );
            return Err(AppError::new(
                "MAME_INPUT_CHANNEL_FAILED",
                "The gameplay input update could not be written to MAME.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })));
        }
        Ok(true)
    }
}
