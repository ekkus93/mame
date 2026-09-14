#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReadyEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    payload: ReadyPayload,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReadyPayload {
    commands: Vec<String>,
    #[serde(rename = "maxMessageBytes")]
    max_message_bytes: usize,
}

#[derive(Serialize)]
struct RequestEnvelope<'a> {
    version: u32,
    #[serde(rename = "type")]
    message_type: &'static str,
    #[serde(rename = "sessionId")]
    session_id: &'a str,
    #[serde(rename = "requestId")]
    request_id: &'a str,
    command: &'static str,
    params: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "requestId")]
    request_id: String,
    status: String,
    ok: bool,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventEnvelope {
    version: u32,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    event: String,
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    payload: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandCompletedPayload {
    command: String,
    ok: bool,
    result: Option<Map<String, Value>>,
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolErrorPayload {
    fatal: bool,
    error: WireError,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct WireError {
    code: String,
    message: String,
    details: Map<String, Value>,
    retryable: bool,
}

fn validate_ready(ready: &ReadyEnvelope, expected_session_id: &str) -> Result<(), String> {
    if ready.version != PROTOCOL_VERSION {
        return Err(format!(
            "Runtime-control ready version {} does not match supported version {PROTOCOL_VERSION}.",
            ready.version
        ));
    }
    if ready.message_type != "event" || ready.event != "ready" {
        return Err(
            "The first authenticated runtime-control message is not a ready event.".to_owned(),
        );
    }
    if ready.session_id != expected_session_id {
        return Err("Runtime-control ready event belongs to a different MAME session.".to_owned());
    }
    if ready.payload.max_message_bytes != MAX_DECODED_MESSAGE_BYTES {
        return Err(
            "Runtime-control peer advertised a different message-size contract.".to_owned(),
        );
    }

    let mut commands = HashSet::new();
    for command in &ready.payload.commands {
        if !KNOWN_COMMANDS.contains(&command.as_str()) {
            return Err(format!(
                "Runtime-control peer advertised unknown command {command:?}."
            ));
        }
        if !commands.insert(command.as_str()) {
            return Err(format!(
                "Runtime-control peer advertised duplicate command {command:?}."
            ));
        }
    }
    Ok(())
}

fn extract_paused_result(
    result: Option<&Map<String, Value>>,
    expected: bool,
    request_id: &str,
) -> AppResult<bool> {
    let result = result.ok_or_else(|| {
        protocol_output_error("A completed pause/resume operation omitted its result object.")
    })?;
    if result.len() != 1 {
        return Err(protocol_output_error(
            "A completed pause/resume operation returned unexpected result fields.",
        ));
    }
    let paused = result
        .get("paused")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            protocol_output_error("A completed pause/resume operation omitted its paused state.")
        })?;
    if paused != expected {
        return Err(AppError::new(
            "CONTROL_OPERATION_FAILED",
            "MAME reported a pause state that contradicts the requested operation.",
        )
        .with_details(serde_json::json!({
            "requestId": request_id,
            "expectedPaused": expected,
            "observedPaused": paused
        })));
    }
    Ok(paused)
}

fn extract_reset_result(result: Option<&Map<String, Value>>, request_id: &str) -> AppResult<()> {
    let result = result.ok_or_else(|| {
        protocol_output_error("A completed reset operation omitted its result object.")
    })?;
    if result.len() != 1 || result.get("kind").and_then(Value::as_str) != Some("soft") {
        return Err(AppError::new(
            "CONTROL_OPERATION_FAILED",
            "MAME reported reset completion with unsupported semantics.",
        )
        .with_details(serde_json::json!({
            "requestId": request_id,
            "expectedKind": "soft",
            "result": result
        })));
    }
    Ok(())
}

fn wire_error_to_app_error(error: WireError) -> AppError {
    AppError {
        code: error.code,
        message: error.message,
        details: Value::Object(error.details),
        retryable: error.retryable,
    }
}

fn channel_closed_error() -> AppError {
    AppError::new(
        "CONTROL_CHANNEL_CLOSED",
        "The MAME runtime-control channel is not available.",
    )
}

fn channel_state_error(state: ControlChannelState) -> AppError {
    match state {
        ControlChannelState::Failed => AppError::new(
            "CONTROL_CHANNEL_FAILED",
            "The MAME runtime-control channel has failed.",
        ),
        _ => channel_closed_error(),
    }
}

fn protocol_output_error(message: &str) -> AppError {
    AppError::new("CONTROL_CHANNEL_FAILED", message)
}

fn generate_frame_token() -> AppResult<String> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    fill_os_random(&mut bytes)?;
    let token = base64url_encode(&bytes);
    debug_assert_eq!(token.len(), 43);
    Ok(token)
}

fn fill_os_random(bytes: &mut [u8]) -> AppResult<()> {
    getrandom::fill(bytes).map_err(|error| {
        AppError::new(
            "CONTROL_TOKEN_GENERATION_FAILED",
            "A cryptographically secure per-session runtime-control token could not be generated.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn base64url_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    let (chunks, remainder) = input.as_chunks::<3>();
    for chunk in chunks {
        let value = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(value & 0x3f) as usize] as char);
    }
    match remainder {
        [a] => {
            let value = u32::from(*a) << 16;
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        }
        [a, b] => {
            let value = (u32::from(*a) << 16) | (u32::from(*b) << 8);
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        }
        [] => {}
        _ => unreachable!("as_chunks remainder is shorter than three bytes"),
    }
    output
}

fn base64url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = input.as_bytes();
    if bytes.len() % 4 == 1 {
        return Err("invalid unpadded length");
    }
    let mut output = Vec::with_capacity((bytes.len() * 3) / 4 + 2);
    let mut index = 0;
    while index + 4 <= bytes.len() {
        let a = decode_sextet(bytes[index])?;
        let b = decode_sextet(bytes[index + 1])?;
        let c = decode_sextet(bytes[index + 2])?;
        let d = decode_sextet(bytes[index + 3])?;
        output.push((a << 2) | (b >> 4));
        output.push((b << 4) | (c >> 2));
        output.push((c << 6) | d);
        index += 4;
    }
    match bytes.len() - index {
        0 => {}
        2 => {
            let a = decode_sextet(bytes[index])?;
            let b = decode_sextet(bytes[index + 1])?;
            if b & 0x0f != 0 {
                return Err("non-canonical trailing bits");
            }
            output.push((a << 2) | (b >> 4));
        }
        3 => {
            let a = decode_sextet(bytes[index])?;
            let b = decode_sextet(bytes[index + 1])?;
            let c = decode_sextet(bytes[index + 2])?;
            if c & 0x03 != 0 {
                return Err("non-canonical trailing bits");
            }
            output.push((a << 2) | (b >> 4));
            output.push((b << 4) | (c >> 2));
        }
        _ => return Err("invalid unpadded length"),
    }
    Ok(output)
}

fn decode_sextet(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'-' => Ok(62),
        b'_' => Ok(63),
        _ => Err("invalid base64url character"),
    }
}

fn bootstrap_io_error(operation: &str, error: io::Error) -> AppError {
    AppError::new(
        "CONTROL_BOOTSTRAP_IO_FAILED",
        "The private runtime-control bootstrap script could not be prepared.",
    )
    .with_details(serde_json::json!({
        "operation": operation,
        "cause": error.to_string()
    }))
}

#[cfg(unix)]
fn restrict_bootstrap_permissions(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|error| {
        AppError::new(
            "CONTROL_BOOTSTRAP_PERMISSIONS_FAILED",
            "The private runtime-control bootstrap script permissions could not be restricted.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(not(unix))]
fn restrict_bootstrap_permissions(_path: &Path) -> AppResult<()> {
    Ok(())
}

fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
