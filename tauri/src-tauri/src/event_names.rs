//! Tauri event-name contract shared by the Rust backend.
//!
//! Tauri event names may contain only ASCII alphanumerics plus `-`, `/`, `:`
//! and `_`. Keep all backend-to-WebView event names here so unit tests can
//! reject invalid names before a real Tauri runtime is required.

pub const APP_READY_EVENT: &str = "app:ready";
pub const SESSION_STARTED_EVENT: &str = "session:started";
pub const SESSION_EXITED_EVENT: &str = "session:exited";
pub const SESSION_CRASHED_EVENT: &str = "session:crashed";
pub const SESSION_FAILED_EVENT: &str = "session:failed";
pub const SESSION_PAUSED_EVENT: &str = "session:paused";
pub const SESSION_RESUMED_EVENT: &str = "session:resumed";
pub const SESSION_STATE_SAVED_EVENT: &str = "session:state-saved";
pub const SESSION_STATE_SAVE_FAILED_EVENT: &str = "session:state-save-failed";
pub const SESSION_STATE_LOADED_EVENT: &str = "session:state-loaded";
pub const SESSION_STATE_LOAD_FAILED_EVENT: &str = "session:state-load-failed";

pub fn external_session_lifecycle_event(internal_name: &str) -> Option<&'static str> {
    match internal_name {
        "session.started" => Some(SESSION_STARTED_EVENT),
        "session.exited" => Some(SESSION_EXITED_EVENT),
        "session.crashed" => Some(SESSION_CRASHED_EVENT),
        "session.failed" => Some(SESSION_FAILED_EVENT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_EXTERNAL_EVENTS: &[&str] = &[
        APP_READY_EVENT,
        SESSION_STARTED_EVENT,
        SESSION_EXITED_EVENT,
        SESSION_CRASHED_EVENT,
        SESSION_FAILED_EVENT,
        SESSION_PAUSED_EVENT,
        SESSION_RESUMED_EVENT,
        SESSION_STATE_SAVED_EVENT,
        SESSION_STATE_SAVE_FAILED_EVENT,
        SESSION_STATE_LOADED_EVENT,
        SESSION_STATE_LOAD_FAILED_EVENT,
    ];

    fn valid_tauri_event_name(name: &str) -> bool {
        !name.is_empty()
            && name.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'/' | b':' | b'_')
            })
    }

    #[test]
    fn all_external_event_names_satisfy_tauri_contract() {
        for name in ALL_EXTERNAL_EVENTS {
            assert!(
                valid_tauri_event_name(name),
                "invalid Tauri event name: {name}"
            );
        }
    }

    #[test]
    fn supervisor_lifecycle_names_map_to_external_contract() {
        assert_eq!(
            external_session_lifecycle_event("session.started"),
            Some(SESSION_STARTED_EVENT)
        );
        assert_eq!(
            external_session_lifecycle_event("session.exited"),
            Some(SESSION_EXITED_EVENT)
        );
        assert_eq!(
            external_session_lifecycle_event("session.crashed"),
            Some(SESSION_CRASHED_EVENT)
        );
        assert_eq!(
            external_session_lifecycle_event("session.failed"),
            Some(SESSION_FAILED_EVENT)
        );
        assert_eq!(external_session_lifecycle_event("session.unknown"), None);
    }
}
