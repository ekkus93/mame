//! Session-scoped binary gameplay frame protocol, transport, and latest-frame mailbox.

use std::{
    io::{self, Read},
    sync::{Arc, Mutex},
};

#[cfg(all(target_os = "linux", target_endian = "little"))]
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

#[cfg(all(target_os = "linux", target_endian = "little"))]
use rustix::fs::{mkfifoat, Mode, CWD};
use serde::Serialize;
#[cfg(all(target_os = "linux", target_endian = "little"))]
use tempfile::{Builder, TempDir};

use crate::errors::{AppError, AppResult};

pub(super) const FRAME_PROTOCOL_VERSION: u16 = 1;
const FRAME_WIRE_MAGIC: &[u8; 8] = b"MTFRAME1";
const FRAME_CLIENT_MAGIC: &[u8; 8] = b"MTGFRM01";
const FRAME_WIRE_FIXED_HEADER_BYTES: usize = 56;
const FRAME_CLIENT_FIXED_HEADER_BYTES: usize = 52;
const MAX_SESSION_ID_BYTES: usize = 96;
const MAX_AUTH_TOKEN_BYTES: usize = 128;
const MAX_FRAME_DIMENSION: u32 = 8192;
const MAX_FRAME_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
#[cfg(all(target_os = "linux", target_endian = "little"))]
const FRAME_AUTH_TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub(super) enum FramePixelFormat {
    Bgrx8888Le = 1,
}

impl FramePixelFormat {
    fn parse(value: u16) -> AppResult<Self> {
        match value {
            1 => Ok(Self::Bgrx8888Le),
            _ => Err(protocol_error(
                "MAME_FRAME_PIXEL_FORMAT_UNSUPPORTED",
                "The MAME frame stream used an unsupported pixel format.",
                serde_json::json!({ "pixelFormat": value }),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct FrameMetricsSnapshot {
    schema_version: u32,
    session_id: String,
    received: u64,
    dropped: u64,
    delivered: u64,
    last_sequence: Option<u64>,
    last_capture_timestamp_us: Option<u64>,
    last_width: Option<u32>,
    last_height: Option<u32>,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GameFrame {
    session_id: String,
    sequence: u64,
    width: u32,
    height: u32,
    stride: u32,
    capture_timestamp_us: u64,
    orientation_degrees: u16,
    flags: u16,
    pixel_format: FramePixelFormat,
    payload: Vec<u8>,
}

impl GameFrame {
    pub(super) fn to_client_bytes(&self) -> AppResult<Vec<u8>> {
        let session = self.session_id.as_bytes();
        let session_len = u16::try_from(session.len()).map_err(|_| {
            protocol_error(
                "MAME_FRAME_SESSION_INVALID",
                "The MAME frame session identifier is too long.",
                serde_json::json!({}),
            )
        })?;
        let payload_len = u32::try_from(self.payload.len()).map_err(|_| {
            protocol_error(
                "MAME_FRAME_PAYLOAD_TOO_LARGE",
                "The MAME frame payload exceeds the supported client response size.",
                serde_json::json!({}),
            )
        })?;
        let header_len = FRAME_CLIENT_FIXED_HEADER_BYTES
            .checked_add(session.len())
            .and_then(|value| u16::try_from(value).ok())
            .ok_or_else(|| {
                protocol_error(
                    "MAME_FRAME_HEADER_INVALID",
                    "The MAME frame client header exceeds the supported size.",
                    serde_json::json!({}),
                )
            })?;
        let mut bytes = Vec::with_capacity(usize::from(header_len) + self.payload.len());
        bytes.extend_from_slice(FRAME_CLIENT_MAGIC);
        push_u16(&mut bytes, FRAME_PROTOCOL_VERSION);
        push_u16(&mut bytes, header_len);
        push_u64(&mut bytes, self.sequence);
        push_u32(&mut bytes, self.width);
        push_u32(&mut bytes, self.height);
        push_u32(&mut bytes, self.stride);
        push_u32(&mut bytes, payload_len);
        push_u64(&mut bytes, self.capture_timestamp_us);
        push_u16(&mut bytes, self.orientation_degrees);
        push_u16(&mut bytes, self.flags);
        push_u16(&mut bytes, self.pixel_format as u16);
        push_u16(&mut bytes, session_len);
        bytes.extend_from_slice(session);
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

#[derive(Clone)]
pub(super) struct FrameMailbox {
    inner: Arc<Mutex<FrameMailboxInner>>,
}

struct FrameMailboxInner {
    session_id: String,
    latest: Option<GameFrame>,
    received: u64,
    dropped: u64,
    delivered: u64,
    last_sequence: Option<u64>,
    last_capture_timestamp_us: Option<u64>,
    last_width: Option<u32>,
    last_height: Option<u32>,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
}

impl FrameMailbox {
    pub(super) fn new(session_id: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FrameMailboxInner {
                session_id,
                latest: None,
                received: 0,
                dropped: 0,
                delivered: 0,
                last_sequence: None,
                last_capture_timestamp_us: None,
                last_width: None,
                last_height: None,
                last_error_code: None,
                last_error_message: None,
            })),
        }
    }

    fn publish(&self, frame: GameFrame) -> AppResult<()> {
        let mut inner = recover_lock(&self.inner);
        if frame.session_id != inner.session_id {
            return Err(protocol_error(
                "MAME_FRAME_SESSION_MISMATCH",
                "A MAME frame belongs to a different gameplay session.",
                serde_json::json!({}),
            ));
        }
        if let Some(previous) = inner.last_sequence {
            if frame.sequence <= previous {
                return Err(protocol_error(
                    "MAME_FRAME_SEQUENCE_INVALID",
                    "The MAME frame sequence did not advance monotonically.",
                    serde_json::json!({
                        "previousSequence": previous, "observedSequence": frame.sequence
                    }),
                ));
            }
        }
        if inner.latest.is_some() {
            inner.dropped = inner.dropped.saturating_add(1);
        }
        inner.received = inner.received.saturating_add(1);
        inner.last_sequence = Some(frame.sequence);
        inner.last_capture_timestamp_us = Some(frame.capture_timestamp_us);
        inner.last_width = Some(frame.width);
        inner.last_height = Some(frame.height);
        inner.last_error_code = None;
        inner.last_error_message = None;
        inner.latest = Some(frame);
        Ok(())
    }

    pub(super) fn take_latest(&self) -> Option<GameFrame> {
        let mut inner = recover_lock(&self.inner);
        let frame = inner.latest.take();
        if frame.is_some() {
            inner.delivered = inner.delivered.saturating_add(1);
        }
        frame
    }

    pub(super) fn snapshot(&self) -> FrameMetricsSnapshot {
        let inner = recover_lock(&self.inner);
        FrameMetricsSnapshot {
            schema_version: 1,
            session_id: inner.session_id.clone(),
            received: inner.received,
            dropped: inner.dropped,
            delivered: inner.delivered,
            last_sequence: inner.last_sequence,
            last_capture_timestamp_us: inner.last_capture_timestamp_us,
            last_width: inner.last_width,
            last_height: inner.last_height,
            last_error_code: inner.last_error_code.clone(),
            last_error_message: inner.last_error_message.clone(),
        }
    }

    fn mark_error(&self, error: &AppError) {
        let mut inner = recover_lock(&self.inner);
        inner.last_error_code = Some(error.code.clone());
        inner.last_error_message = Some(error.message.clone());
    }
}

pub(super) struct FrameTransport {
    #[cfg(all(target_os = "linux", target_endian = "little"))]
    root: Arc<TempDir>,
    #[cfg(all(target_os = "linux", target_endian = "little"))]
    path: PathBuf,
    #[cfg(all(target_os = "linux", target_endian = "little"))]
    auth_token: String,
    #[cfg(all(target_os = "linux", target_endian = "little"))]
    reader_opened: Arc<AtomicBool>,
    #[cfg(all(target_os = "linux", target_endian = "little"))]
    cancelled: Arc<AtomicBool>,
}

impl FrameTransport {
    pub(super) fn create() -> AppResult<Option<Self>> {
        #[cfg(all(target_os = "linux", target_endian = "little"))]
        {
            let root = Arc::new(
                Builder::new()
                    .prefix(".mame-tauri-frame-")
                    .tempdir()
                    .map_err(|error| transport_io_error("create temporary directory", error))?,
            );
            let path = root.path().join("frames.fifo");
            mkfifoat(CWD, &path, Mode::RUSR | Mode::WUSR).map_err(|error| {
                AppError::new(
                    "MAME_FRAME_TRANSPORT_CREATE_FAILED",
                    "The private MAME frame transport could not be created.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?;
            return Ok(Some(Self {
                root,
                path,
                auth_token: generate_auth_token()?,
                reader_opened: Arc::new(AtomicBool::new(false)),
                cancelled: Arc::new(AtomicBool::new(false)),
            }));
        }
        #[cfg(not(all(target_os = "linux", target_endian = "little")))]
        {
            Ok(None)
        }
    }

    pub(super) fn configure_command(&self, command: &mut std::process::Command) -> AppResult<()> {
        #[cfg(all(target_os = "linux", target_endian = "little"))]
        {
            command.env("MAME_TAURI_FRAME_PIPE", path_text(&self.path)?);
            command.env("MAME_TAURI_FRAME_TOKEN", &self.auth_token);
            command.env(
                "MAME_TAURI_FRAME_PROTOCOL",
                FRAME_PROTOCOL_VERSION.to_string(),
            );
        }
        #[cfg(not(all(target_os = "linux", target_endian = "little")))]
        let _ = command;
        Ok(())
    }

    pub(super) fn start_reader(&self, session_id: String, mailbox: FrameMailbox) {
        #[cfg(all(target_os = "linux", target_endian = "little"))]
        {
            let _root = self.root.clone();
            let path = self.path.clone();
            let auth_token = self.auth_token.clone();
            let reader_opened = self.reader_opened.clone();
            let cancelled = self.cancelled.clone();
            thread::spawn(move || {
                let mut reader = match File::open(&path) {
                    Ok(reader) => {
                        reader_opened.store(true, Ordering::Release);
                        reader
                    }
                    Err(error) => {
                        if !cancelled.load(Ordering::Acquire) {
                            mailbox.mark_error(&transport_io_error("open private FIFO", error));
                        }
                        return;
                    }
                };
                loop {
                    match read_wire_frame(&mut reader, &session_id, &auth_token) {
                        Ok(frame) => {
                            if let Err(error) = mailbox.publish(frame) {
                                mailbox.mark_error(&error);
                                break;
                            }
                        }
                        Err(error) => {
                            if !cancelled.load(Ordering::Acquire) {
                                mailbox.mark_error(&error);
                            }
                            break;
                        }
                    }
                }
            });
        }
        #[cfg(not(all(target_os = "linux", target_endian = "little")))]
        let _ = (session_id, mailbox);
    }

    pub(super) fn cancel(&self) {
        #[cfg(all(target_os = "linux", target_endian = "little"))]
        {
            self.cancelled.store(true, Ordering::Release);
            if !self.reader_opened.load(Ordering::Acquire) {
                let _ = OpenOptions::new().write(true).open(&self.path);
            }
        }
    }
}

fn read_wire_frame(
    reader: &mut impl Read,
    expected_session_id: &str,
    expected_auth_token: &str,
) -> AppResult<GameFrame> {
    let mut fixed = [0_u8; FRAME_WIRE_FIXED_HEADER_BYTES];
    reader
        .read_exact(&mut fixed)
        .map_err(|error| frame_read_error("header", error))?;
    if &fixed[0..8] != FRAME_WIRE_MAGIC {
        return Err(protocol_error(
            "MAME_FRAME_MAGIC_INVALID",
            "The MAME frame stream magic value is invalid.",
            serde_json::json!({}),
        ));
    }
    let version = read_u16(&fixed, 8);
    if version != FRAME_PROTOCOL_VERSION {
        return Err(protocol_error(
            "MAME_FRAME_VERSION_UNSUPPORTED",
            "The MAME frame stream protocol version is unsupported.",
            serde_json::json!({
                "supportedVersion": FRAME_PROTOCOL_VERSION, "observedVersion": version
            }),
        ));
    }
    let header_len = usize::from(read_u16(&fixed, 10));
    let sequence = read_u64(&fixed, 12);
    let width = read_u32(&fixed, 20);
    let height = read_u32(&fixed, 24);
    let stride = read_u32(&fixed, 28);
    let payload_len = read_u32(&fixed, 32) as usize;
    let capture_timestamp_us = read_u64(&fixed, 36);
    let orientation_degrees = read_u16(&fixed, 44);
    let flags = read_u16(&fixed, 46);
    let pixel_format = FramePixelFormat::parse(read_u16(&fixed, 48))?;
    let session_len = usize::from(read_u16(&fixed, 50));
    let token_len = usize::from(read_u16(&fixed, 52));
    let reserved = read_u16(&fixed, 54);

    if reserved != 0 {
        return Err(protocol_error(
            "MAME_FRAME_HEADER_INVALID",
            "The MAME frame stream header contains unsupported reserved flags.",
            serde_json::json!({}),
        ));
    }
    if width == 0 || height == 0 || width > MAX_FRAME_DIMENSION || height > MAX_FRAME_DIMENSION {
        return Err(protocol_error(
            "MAME_FRAME_DIMENSIONS_INVALID",
            "The MAME frame dimensions exceed the supported bounds.",
            serde_json::json!({ "width": width, "height": height }),
        ));
    }
    if session_len == 0
        || session_len > MAX_SESSION_ID_BYTES
        || token_len == 0
        || token_len > MAX_AUTH_TOKEN_BYTES
    {
        return Err(protocol_error(
            "MAME_FRAME_IDENTITY_INVALID",
            "The MAME frame identity lengths are invalid.",
            serde_json::json!({}),
        ));
    }
    let expected_header_len = FRAME_WIRE_FIXED_HEADER_BYTES
        .checked_add(session_len)
        .and_then(|v| v.checked_add(token_len))
        .ok_or_else(|| {
            protocol_error(
                "MAME_FRAME_HEADER_INVALID",
                "The MAME frame header length overflowed.",
                serde_json::json!({}),
            )
        })?;
    if header_len != expected_header_len {
        return Err(protocol_error(
            "MAME_FRAME_HEADER_INVALID",
            "The MAME frame stream header length is inconsistent.",
            serde_json::json!({}),
        ));
    }
    let expected_stride = width.checked_mul(4).ok_or_else(|| {
        protocol_error(
            "MAME_FRAME_DIMENSIONS_INVALID",
            "The MAME frame stride overflowed.",
            serde_json::json!({}),
        )
    })?;
    if stride != expected_stride {
        return Err(protocol_error(
            "MAME_FRAME_STRIDE_INVALID",
            "The MAME frame stride does not match packed RGB32.",
            serde_json::json!({}),
        ));
    }
    let expected_payload_len = (stride as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| {
            protocol_error(
                "MAME_FRAME_PAYLOAD_TOO_LARGE",
                "The MAME frame payload size overflowed.",
                serde_json::json!({}),
            )
        })?;
    if payload_len != expected_payload_len || payload_len > MAX_FRAME_PAYLOAD_BYTES {
        return Err(protocol_error(
            "MAME_FRAME_PAYLOAD_INVALID",
            "The MAME frame payload size is inconsistent or exceeds the supported cap.",
            serde_json::json!({
                "expectedPayloadBytes": expected_payload_len, "observedPayloadBytes": payload_len
            }),
        ));
    }
    if !matches!(orientation_degrees, 0 | 90 | 180 | 270) {
        return Err(protocol_error(
            "MAME_FRAME_ORIENTATION_INVALID",
            "The MAME frame orientation is unsupported.",
            serde_json::json!({}),
        ));
    }

    let mut identity = vec![0_u8; session_len + token_len];
    reader
        .read_exact(&mut identity)
        .map_err(|error| frame_read_error("identity", error))?;
    let session_id = std::str::from_utf8(&identity[..session_len]).map_err(|error| {
        protocol_error(
            "MAME_FRAME_SESSION_INVALID",
            "The MAME frame session identifier is not valid UTF-8.",
            serde_json::json!({ "cause": error.to_string() }),
        )
    })?;
    let auth_token = std::str::from_utf8(&identity[session_len..]).map_err(|error| {
        protocol_error(
            "MAME_FRAME_AUTH_INVALID",
            "The MAME frame authentication token is not valid UTF-8.",
            serde_json::json!({ "cause": error.to_string() }),
        )
    })?;
    if session_id != expected_session_id {
        return Err(protocol_error(
            "MAME_FRAME_SESSION_MISMATCH",
            "The MAME frame stream belongs to a different gameplay session.",
            serde_json::json!({}),
        ));
    }
    if auth_token != expected_auth_token {
        return Err(protocol_error(
            "MAME_FRAME_AUTH_MISMATCH",
            "The MAME frame stream authentication token is invalid.",
            serde_json::json!({}),
        ));
    }

    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .map_err(|error| frame_read_error("payload", error))?;
    Ok(GameFrame {
        session_id: session_id.to_owned(),
        sequence,
        width,
        height,
        stride,
        capture_timestamp_us,
        orientation_degrees,
        flags,
        pixel_format,
        payload,
    })
}

#[cfg(all(target_os = "linux", target_endian = "little"))]
fn path_text(path: &Path) -> AppResult<&str> {
    path.to_str().ok_or_else(|| {
        AppError::new(
            "MAME_FRAME_TRANSPORT_PATH_INVALID",
            "The private MAME frame transport path is not valid UTF-8.",
        )
    })
}

#[cfg(all(target_os = "linux", target_endian = "little"))]
fn generate_auth_token() -> AppResult<String> {
    let mut bytes = [0_u8; FRAME_AUTH_TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| {
        AppError::new(
            "MAME_FRAME_AUTH_RANDOM_FAILED",
            "Secure randomness for the MAME frame transport is unavailable.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    let mut token = String::with_capacity(FRAME_AUTH_TOKEN_BYTES * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut token, "{byte:02x}").expect("String formatting cannot fail");
    }
    Ok(token)
}

#[cfg(all(target_os = "linux", target_endian = "little"))]
fn transport_io_error(action: &str, error: io::Error) -> AppError {
    AppError::new(
        "MAME_FRAME_TRANSPORT_IO_FAILED",
        "The private MAME frame transport could not be initialized or read.",
    )
    .with_details(serde_json::json!({ "action": action, "cause": error.to_string() }))
}

fn frame_read_error(part: &str, error: io::Error) -> AppError {
    AppError::new(
        "MAME_FRAME_STREAM_READ_FAILED",
        "The MAME frame stream ended or could not be read.",
    )
    .with_details(serde_json::json!({ "part": part, "cause": error.to_string() }))
}

fn protocol_error(code: &str, message: &str, details: serde_json::Value) -> AppError {
    AppError::new(code, message).with_details(details)
}

fn recover_lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
fn read_u16(bytes: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([bytes[o], bytes[o + 1]])
}
fn read_u32(bytes: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
}
fn read_u64(bytes: &[u8], o: usize) -> u64 {
    u64::from_le_bytes([
        bytes[o],
        bytes[o + 1],
        bytes[o + 2],
        bytes[o + 3],
        bytes[o + 4],
        bytes[o + 5],
        bytes[o + 6],
        bytes[o + 7],
    ])
}
fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SESSION: &str = "mame-123-1";
    const TOKEN: &str = "0123456789012345678901234567890123456789012";

    fn wire_frame(sequence: u64, width: u32, height: u32, session: &str, token: &str) -> Vec<u8> {
        let stride = width * 4;
        let payload_len = stride * height;
        let header_len = FRAME_WIRE_FIXED_HEADER_BYTES + session.len() + token.len();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(FRAME_WIRE_MAGIC);
        bytes.extend_from_slice(&FRAME_PROTOCOL_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(header_len as u16).to_le_bytes());
        bytes.extend_from_slice(&sequence.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&stride.to_le_bytes());
        bytes.extend_from_slice(&payload_len.to_le_bytes());
        bytes.extend_from_slice(&(42_u64 + sequence).to_le_bytes());
        bytes.extend_from_slice(&90_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&(FramePixelFormat::Bgrx8888Le as u16).to_le_bytes());
        bytes.extend_from_slice(&(session.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&(token.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(session.as_bytes());
        bytes.extend_from_slice(token.as_bytes());
        bytes.resize(bytes.len() + payload_len as usize, sequence as u8);
        bytes
    }

    #[test]
    fn parses_bounded_authenticated_frame() {
        let frame = read_wire_frame(
            &mut Cursor::new(wire_frame(7, 2, 3, SESSION, TOKEN)),
            SESSION,
            TOKEN,
        )
        .expect("valid frame");
        assert_eq!(
            (
                frame.sequence,
                frame.width,
                frame.height,
                frame.payload.len()
            ),
            (7, 2, 3, 24)
        );
    }

    #[test]
    fn rejects_cross_session_and_oversized_dimensions() {
        let error = read_wire_frame(
            &mut Cursor::new(wire_frame(1, 1, 1, SESSION, TOKEN)),
            "other",
            TOKEN,
        )
        .expect_err("cross session");
        assert_eq!(error.code, "MAME_FRAME_SESSION_MISMATCH");
        let mut bytes = wire_frame(1, 1, 1, SESSION, TOKEN);
        bytes[20..24].copy_from_slice(&9000_u32.to_le_bytes());
        let error =
            read_wire_frame(&mut Cursor::new(bytes), SESSION, TOKEN).expect_err("oversized");
        assert_eq!(error.code, "MAME_FRAME_DIMENSIONS_INVALID");
    }

    #[test]
    fn mailbox_replaces_stale_frame_instead_of_queueing() {
        let mailbox = FrameMailbox::new(SESSION.to_owned());
        for sequence in [1_u64, 2] {
            mailbox
                .publish(GameFrame {
                    session_id: SESSION.to_owned(),
                    sequence,
                    width: 1,
                    height: 1,
                    stride: 4,
                    capture_timestamp_us: sequence,
                    orientation_degrees: 0,
                    flags: 0,
                    pixel_format: FramePixelFormat::Bgrx8888Le,
                    payload: vec![sequence as u8; 4],
                })
                .expect("publish");
        }
        assert_eq!(mailbox.snapshot().dropped, 1);
        assert_eq!(mailbox.take_latest().expect("latest").sequence, 2);
        assert!(mailbox.take_latest().is_none());
    }

    #[test]
    fn client_buffer_does_not_expose_auth_token() {
        let frame = GameFrame {
            session_id: SESSION.to_owned(),
            sequence: 9,
            width: 1,
            height: 1,
            stride: 4,
            capture_timestamp_us: 99,
            orientation_degrees: 270,
            flags: 2,
            pixel_format: FramePixelFormat::Bgrx8888Le,
            payload: vec![1, 2, 3, 0],
        };
        let bytes = frame.to_client_bytes().expect("client bytes");
        assert_eq!(&bytes[..8], b"MTGFRM01");
        assert!(bytes
            .windows(TOKEN.len())
            .all(|window| window != TOKEN.as_bytes()));
    }
}
