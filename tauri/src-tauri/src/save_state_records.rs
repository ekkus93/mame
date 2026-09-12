//! Versioned save-state record model for MT-901.
//!
//! This module models durable save-state metadata without making compatibility
//! claims that MAME itself has not verified. The record is built only from an
//! authoritative supervised session plus a completed MT-707 save result.

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, AppResult},
    mame::{validate_short_identifier, validate_software_identifier},
    sessions::{SaveMameStateResult, SessionSnapshot},
};

pub const SAVE_STATE_RECORD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateMameIdentityV1 {
    pub version: String,
    pub build: Option<String>,
    pub raw_version_line: String,
}

/// Reserved metadata for a screenshot associated with a save-state record.
///
/// MT-901 does not capture screenshots. Keeping the field optional and
/// versioned lets a later task attach an opaque artwork identifier without
/// changing the meaning of existing records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateScreenshotMetadataV1 {
    pub schema_version: u32,
    pub asset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveStateRecordV1 {
    pub schema_version: u32,
    pub machine: String,
    pub software: Option<String>,
    pub mame: SaveStateMameIdentityV1,
    pub saved_at_epoch_ms: u64,
    pub slot: String,
    pub path: String,
    pub bytes: u64,
    pub screenshot: Option<SaveStateScreenshotMetadataV1>,
}

impl SaveStateRecordV1 {
    /// Build a record from the exact session that produced a completed save.
    ///
    /// The consistency checks deliberately reject mismatched session/machine/
    /// software data instead of silently recording ambiguous provenance.
    pub fn from_completed_save(
        session: &SessionSnapshot,
        saved: &SaveMameStateResult,
    ) -> AppResult<Self> {
        validate_short_identifier("machine", &saved.machine)?;
        if let Some(software) = saved.software.as_deref() {
            validate_software_identifier(software)?;
        }
        validate_record_path(&saved.path)?;

        if session.session_id != saved.session_id {
            return Err(record_mismatch(
                "sessionId",
                &session.session_id,
                &saved.session_id,
            ));
        }
        if session.machine != saved.machine {
            return Err(record_mismatch("machine", &session.machine, &saved.machine));
        }
        if session.software != saved.software {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_CONTEXT_MISMATCH",
                "The completed save-state software context does not match its producing session.",
            )
            .with_details(serde_json::json!({
                "field": "software",
                "sessionValue": session.software,
                "savedValue": saved.software,
            })));
        }
        if session.executable.version.trim().is_empty()
            || session.executable.raw_version_line.trim().is_empty()
        {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_MAME_IDENTITY_INVALID",
                "The producing MAME executable identity is incomplete.",
            ));
        }
        if saved.slot.trim().is_empty() {
            return Err(AppError::new(
                "SAVE_STATE_RECORD_SLOT_INVALID",
                "The completed save-state slot is empty.",
            ));
        }

        Ok(Self {
            schema_version: SAVE_STATE_RECORD_SCHEMA_VERSION,
            machine: saved.machine.clone(),
            software: saved.software.clone(),
            mame: SaveStateMameIdentityV1 {
                version: session.executable.version.clone(),
                build: session.executable.build.clone(),
                raw_version_line: session.executable.raw_version_line.clone(),
            },
            saved_at_epoch_ms: saved.saved_at_epoch_ms,
            slot: saved.slot.clone(),
            path: saved.path.clone(),
            bytes: saved.bytes,
            screenshot: None,
        })
    }
}

fn validate_record_path(path: &str) -> AppResult<()> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_PATH_INVALID",
            "A save-state record path must be an absolute application-owned path.",
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AppError::new(
            "SAVE_STATE_RECORD_PATH_INVALID",
            "A save-state record path must not contain parent-directory traversal.",
        ));
    }
    Ok(())
}

fn record_mismatch(field: &str, session_value: &str, saved_value: &str) -> AppError {
    AppError::new(
        "SAVE_STATE_RECORD_CONTEXT_MISMATCH",
        "The completed save-state context does not match its producing session.",
    )
    .with_details(serde_json::json!({
        "field": field,
        "sessionValue": session_value,
        "savedValue": saved_value,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
        sessions::{
            EffectiveLaunchConfig, SaveMameStateResult, SessionSnapshot, SessionState,
        },
    };

    use super::{SaveStateRecordV1, SAVE_STATE_RECORD_SCHEMA_VERSION};

    fn session() -> SessionSnapshot {
        SessionSnapshot {
            schema_version: 1,
            session_id: "session-7".to_owned(),
            state: SessionState::Running,
            machine: "pacman".to_owned(),
            software: Some("nes:mario".to_owned()),
            executable: MameExecutableIdentity {
                source: MameExecutableSourceKind::External,
                trust: MameExecutableTrust::UserConfigured,
                path: "/opt/mame/mame".to_owned(),
                version: "0.281".to_owned(),
                build: Some("mame0281".to_owned()),
                raw_version_line: "MAME v0.281 (mame0281)".to_owned(),
            },
            effective_argv: vec!["pacman".to_owned()],
            effective_config: EffectiveLaunchConfig {
                project_paths: Vec::new(),
            },
            created_at_epoch_ms: 10,
            started_at_epoch_ms: Some(11),
            ended_at_epoch_ms: None,
            pid: Some(1234),
            exit_code: None,
            termination_signal: None,
            forced_termination: false,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            diagnostic_error: None,
        }
    }

    fn saved() -> SaveMameStateResult {
        SaveMameStateResult {
            schema_version: 1,
            session_id: "session-7".to_owned(),
            machine: "pacman".to_owned(),
            software: Some("nes:mario".to_owned()),
            slot: "quick".to_owned(),
            path: "/tmp/mame/save-states/v1/machine-pacman/software-nes%3Amario/quick.sta"
                .to_owned(),
            bytes: 4096,
            saved_at_epoch_ms: 99,
        }
    }

    #[test]
    fn record_captures_required_mt901_provenance_without_executable_path() {
        let session = session();
        let record = SaveStateRecordV1::from_completed_save(&session, &saved())
            .expect("build save-state record");

        assert_eq!(record.schema_version, SAVE_STATE_RECORD_SCHEMA_VERSION);
        assert_eq!(record.machine, "pacman");
        assert_eq!(record.software.as_deref(), Some("nes:mario"));
        assert_eq!(record.mame.version, "0.281");
        assert_eq!(record.mame.build.as_deref(), Some("mame0281"));
        assert_eq!(record.saved_at_epoch_ms, 99);
        assert_eq!(record.slot, "quick");
        assert_eq!(record.bytes, 4096);
        assert!(record.screenshot.is_none());

        let json = serde_json::to_value(&record).expect("serialize record");
        let serialized = json.to_string();
        assert!(!serialized.contains("/opt/mame/mame"));
    }

    #[test]
    fn record_rejects_mismatched_session_provenance() {
        let mut saved = saved();
        saved.session_id = "different-session".to_owned();

        let error = SaveStateRecordV1::from_completed_save(&session(), &saved)
            .expect_err("mismatched session must fail");
        assert_eq!(error.code, "SAVE_STATE_RECORD_CONTEXT_MISMATCH");
    }

    #[test]
    fn record_rejects_relative_or_traversing_paths() {
        let mut relative = saved();
        relative.path = "save-states/quick.sta".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &relative)
                .expect_err("relative path must fail")
                .code,
            "SAVE_STATE_RECORD_PATH_INVALID"
        );

        let mut traversing = saved();
        traversing.path = "/tmp/mame/../escape.sta".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &traversing)
                .expect_err("traversing path must fail")
                .code,
            "SAVE_STATE_RECORD_PATH_INVALID"
        );
    }

    #[test]
    fn optional_screenshot_metadata_round_trips_as_absent() {
        let record = SaveStateRecordV1::from_completed_save(&session(), &saved())
            .expect("build save-state record");
        let encoded = serde_json::to_string(&record).expect("encode record");
        let decoded: SaveStateRecordV1 = serde_json::from_str(&encoded).expect("decode record");

        assert_eq!(decoded, record);
        assert!(decoded.screenshot.is_none());
    }
}
