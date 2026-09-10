//! Session-scoped runtime-control transport primitives for MT-703.
//!
//! The transport is deliberately local to the supervised MAME child: Rust owns
//! the anonymous stdin writer, while protocol output shares MAME stdout and is
//! separated from diagnostics by a per-session unpredictable frame token.

use std::{
    collections::{HashSet, VecDeque},
    ffi::OsString,
    io::{self, Write},
    path::Path,
    process::ChildStdin,
    sync::{Arc, Mutex, MutexGuard},
};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tempfile::{Builder, NamedTempFile};

use crate::errors::{AppError, AppResult};

pub(super) const PROTOCOL_VERSION: u32 = 1;
pub(super) const MAX_DECODED_MESSAGE_BYTES: usize = 16_384;
pub(super) const MAX_ENCODED_LINE_BYTES: usize = 24_576;
pub(super) const CONTROL_READY_TIMEOUT_MS: u64 = 15_000;

const FRAME_PREFIX: &str = "@@MAME_TAURI_CONTROL_V1@@";
const TOKEN_BYTES: usize = 32;
const TOKEN_REDACTION: &[u8] = b"[CONTROL_TOKEN_REDACTED]";
const KNOWN_COMMANDS: [&str; 9] = [
    "pause",
    "resume",
    "reset",
    "exit",
    "save_state",
    "load_state",
    "set_mute",
    "set_volume",
    "query_state",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlChannelState {
    Initializing,
    Ready,
    Closing,
    Closed,
    Failed,
}

pub(super) type SharedControlState = Arc<Mutex<ControlChannelState>>;

pub(super) struct ControlBootstrap {
    file: NamedTempFile,
    frame_token: String,
}

impl ControlBootstrap {
    pub(super) fn create(session_id: &str) -> AppResult<Self> {
        let frame_token = generate_frame_token()?;
        let ready = ReadyEnvelope {
            version: PROTOCOL_VERSION,
            message_type: "event".to_owned(),
            session_id: session_id.to_owned(),
            event: "ready".to_owned(),
            payload: ReadyPayload {
                commands: Vec::new(),
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

        let script = format!(
            "local expected_token = \"{frame_token}\"\n\
function mame_tauri_control_v1(token, payload)\n\
\tif token ~= expected_token then\n\
\t\treturn\n\
\tend\n\
\t-- MT-703 establishes the scoped transport only. Runtime commands are\n\
\t-- advertised and dispatched by MT-704 and later tasks.\n\
end\n\
local ready_frame = \"{ready_frame}\"\n\
io.write(\"\\n\", ready_frame, \"\\n\")\n\
io.flush()\n"
        );

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

        Ok(Self { file, frame_token })
    }

    pub(super) fn append_launch_arguments(&self, argv: &mut Vec<OsString>) {
        argv.push(OsString::from("-console"));
        argv.push(OsString::from("-autoboot_script"));
        argv.push(self.file.path().as_os_str().to_owned());
    }

    pub(super) fn frame_token(&self) -> &str {
        &self.frame_token
    }

    #[cfg(test)]
    fn path(&self) -> &Path {
        self.file.path()
    }
}

pub(super) struct ControlChannel {
    writer: Option<Arc<Mutex<ChildStdin>>>,
    state: SharedControlState,
    frame_token: String,
}

impl ControlChannel {
    pub(super) fn new(writer: ChildStdin, frame_token: String) -> Self {
        Self {
            writer: Some(Arc::new(Mutex::new(writer))),
            state: Arc::new(Mutex::new(ControlChannelState::Initializing)),
            frame_token,
        }
    }

    pub(super) fn shared_state(&self) -> SharedControlState {
        self.state.clone()
    }

    pub(super) fn begin_close(&mut self) {
        self.writer.take();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closing);
        }
    }

    pub(super) fn mark_closed(&mut self) {
        self.writer.take();
        self.frame_token.clear();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closed);
        }
    }

    pub(super) fn mark_failed(&mut self) {
        self.writer.take();
        self.frame_token.clear();
        set_control_state(&self.state, ControlChannelState::Failed);
    }

    #[cfg(test)]
    pub(super) fn state(&self) -> ControlChannelState {
        control_state(&self.state)
    }
}

impl Drop for ControlChannel {
    fn drop(&mut self) {
        self.writer.take();
        self.frame_token.clear();
        if control_state(&self.state) != ControlChannelState::Failed {
            set_control_state(&self.state, ControlChannelState::Closed);
        }
    }
}

pub(super) fn control_state(state: &SharedControlState) -> ControlChannelState {
    *recover_lock(state)
}

pub(super) fn set_control_state(state: &SharedControlState, next: ControlChannelState) {
    *recover_lock(state) = next;
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ParserEvent {
    Ready,
    Fatal(String),
}

#[derive(Default)]
pub(super) struct ParseBatch {
    pub(super) diagnostics: Vec<u8>,
    pub(super) events: Vec<ParserEvent>,
}

pub(super) struct ControlStdoutParser {
    session_id: String,
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
        let expected_prefix = format!("{FRAME_PREFIX}{frame_token}@@").into_bytes();
        Self {
            session_id,
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
                        match self.validate_protocol_payload(&payload) {
                            Ok(()) => {
                                self.ready_seen = true;
                                batch.events.push(ParserEvent::Ready);
                            }
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

    fn validate_protocol_payload(&self, encoded: &[u8]) -> Result<(), String> {
        if self.ready_seen {
            return Err(
                "An unexpected authenticated runtime-control frame was received after ready."
                    .to_owned(),
            );
        }
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
        let ready: ReadyEnvelope = serde_json::from_slice(&decoded)
            .map_err(|error| format!("Authenticated runtime-control JSON is invalid: {error}"))?;
        validate_ready(&ready, &self.session_id)
    }

    fn fail(&mut self, message: String, batch: &mut ParseBatch) {
        self.failed = true;
        batch.events.push(ParserEvent::Fatal(message));
    }
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

fn generate_frame_token() -> AppResult<String> {
    // rusqlite is already a direct dependency. SQLite seeds its process-wide
    // high-quality PRNG from the platform VFS randomness source and exposes
    // those bytes through randomblob(). The inherited anonymous pipe is the
    // primary authorization boundary; this token provides unpredictable
    // session framing and defense in depth without adding another dependency.
    let connection = Connection::open_in_memory().map_err(token_generation_error)?;
    let bytes: Vec<u8> = connection
        .query_row("SELECT randomblob(?1)", [TOKEN_BYTES as i64], |row| {
            row.get(0)
        })
        .map_err(token_generation_error)?;
    if bytes.len() != TOKEN_BYTES {
        return Err(AppError::new(
            "CONTROL_TOKEN_GENERATION_FAILED",
            "The platform randomness source returned an invalid runtime-control token length.",
        ));
    }
    let token = base64url_encode(&bytes);
    debug_assert_eq!(token.len(), 43);
    Ok(token)
}

fn token_generation_error(error: rusqlite::Error) -> AppError {
    AppError::new(
        "CONTROL_TOKEN_GENERATION_FAILED",
        "A random per-session runtime-control token could not be generated.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn base64url_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let value = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(value & 0x3f) as usize] as char);
    }
    match chunks.remainder() {
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
        _ => unreachable!("chunks_exact remainder is shorter than three bytes"),
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

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;

    use super::{
        base64url_decode, base64url_encode, generate_frame_token, ControlBootstrap,
        ControlStdoutParser, ParserEvent, FRAME_PREFIX, MAX_DECODED_MESSAGE_BYTES, TOKEN_REDACTION,
    };

    #[test]
    fn generated_tokens_are_32_random_bytes_in_unpadded_base64url() {
        let first = generate_frame_token().expect("first token");
        let second = generate_frame_token().expect("second token");
        assert_eq!(first.len(), 43);
        assert_eq!(base64url_decode(&first).expect("decode first").len(), 32);
        assert!(first
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'));
        assert_ne!(first, second);
    }

    #[test]
    fn base64url_round_trips_protocol_payloads_without_padding() {
        for payload in [b"".as_slice(), b"a", b"ab", b"abc", b"protocol-v1"] {
            let encoded = base64url_encode(payload);
            assert!(!encoded.contains('='));
            assert_eq!(base64url_decode(&encoded).expect("decode"), payload);
        }
    }

    #[test]
    fn bootstrap_is_private_and_advertises_no_commands_before_mt704() {
        let bootstrap = ControlBootstrap::create("mame-123-1").expect("bootstrap");
        let script = fs::read_to_string(bootstrap.path()).expect("read bootstrap");
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
        assert_eq!(ready["version"], 1);
        assert_eq!(ready["type"], "event");
        assert_eq!(ready["sessionId"], "mame-123-1");
        assert_eq!(ready["event"], "ready");
        assert_eq!(ready["payload"]["commands"], serde_json::json!([]));
        assert_eq!(
            ready["payload"]["maxMessageBytes"],
            MAX_DECODED_MESSAGE_BYTES
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(bootstrap.path())
                .expect("bootstrap metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0);
        }
    }

    #[test]
    fn parser_accepts_split_ready_frame_and_preserves_other_stdout() {
        let token = "A".repeat(43);
        let ready = serde_json::json!({
            "version": 1,
            "type": "event",
            "sessionId": "mame-1-1",
            "event": "ready",
            "payload": {"commands": [], "maxMessageBytes": 16384}
        });
        let encoded = base64url_encode(&serde_json::to_vec(&ready).expect("serialize"));
        let stream = format!("banner\n{FRAME_PREFIX}{token}@@{encoded}\r\nafter\n");
        let split = stream.len() / 2;
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token);
        let first = parser.feed(&stream.as_bytes()[..split]);
        let second = parser.feed(&stream.as_bytes()[split..]);
        assert_eq!(
            [first.diagnostics, second.diagnostics].concat(),
            b"banner\nafter\n"
        );
        assert_eq!(
            [first.events, second.events].concat(),
            vec![ParserEvent::Ready]
        );
    }

    #[test]
    fn wrong_token_is_diagnostic_but_current_token_is_redacted_from_console_echo() {
        let token = "B".repeat(43);
        let wrong = "C".repeat(43);
        let input = format!(
            "{FRAME_PREFIX}{wrong}@@garbage\nmame_tauri_control_v1(\"{token}\",\"payload\")\n"
        );
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token.clone());
        let mut batch = parser.feed(input.as_bytes());
        let finished = parser.finish();
        batch.diagnostics.extend(finished.diagnostics);
        let text = String::from_utf8(batch.diagnostics).expect("diagnostics utf8");
        assert!(text.contains(&wrong));
        assert!(!text.contains(&token));
        assert!(text.contains(std::str::from_utf8(TOKEN_REDACTION).expect("redaction utf8")));
        assert!(batch.events.is_empty());
    }

    #[test]
    fn authenticated_malformed_frame_fails_closed() {
        let token = "D".repeat(43);
        let input = format!("{FRAME_PREFIX}{token}@@not+base64\n");
        let mut parser = ControlStdoutParser::new("mame-1-1".to_owned(), token);
        let batch = parser.feed(input.as_bytes());
        assert!(batch.diagnostics.is_empty());
        assert!(matches!(batch.events.as_slice(), [ParserEvent::Fatal(_)]));
    }
}
