//! Session-scoped runtime-control transport primitives for MT-703/MT-704/MT-705.
//!
//! Rust owns the anonymous stdin writer. Protocol output shares MAME stdout and
//! is separated from diagnostics by a per-session unpredictable frame token.
//!
//! The protocol was split after MT-2200 so this orchestration file stays small
//! and the individual responsibilities remain reviewable:
//!
//! - registry/bootstrap session activation;
//! - request serialization and response/completion correlation;
//! - stdout frame parsing and diagnostic redaction;
//! - wire-format validation and protocol utility functions;
//! - focused regression tests.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    io::{self, Write},
    path::Path,
    process::ChildStdin,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex, MutexGuard, OnceLock,
    },
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tempfile::{Builder, NamedTempFile};

use crate::errors::{AppError, AppResult};

pub(super) const PROTOCOL_VERSION: u32 = 1;
pub(super) const MAX_DECODED_MESSAGE_BYTES: usize = 16_384;
pub(super) const MAX_ENCODED_LINE_BYTES: usize = 24_576;
pub(super) const CONTROL_READY_TIMEOUT_MS: u64 = 15_000;
const CONTROL_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const PAUSE_RESUME_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const RESET_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

const FRAME_PREFIX: &str = "@@MAME_TAURI_CONTROL_V1@@";
const TOKEN_BYTES: usize = 32;
const TOKEN_REDACTION: &[u8] = b"[CONTROL_TOKEN_REDACTED]";
const ABANDONED_REQUEST_LIMIT: usize = 32;
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
const MT705_COMMANDS: [&str; 3] = ["pause", "resume", "reset"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlChannelState {
    Initializing,
    Ready,
    Closing,
    Closed,
    Failed,
}

pub(super) type SharedControlState = Arc<Mutex<ControlChannelState>>;
pub(super) type SharedControlRuntime = Arc<ControlRuntime>;
pub(super) type PauseStateSink = Arc<dyn Fn(bool) + Send + Sync>;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_registry.rs"));
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_bootstrap.rs"));
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_transport.rs"));
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_parser.rs"));
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_protocol.rs"));

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sessions/control_tests.rs"));
