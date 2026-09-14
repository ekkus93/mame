#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::Connection;

    use crate::{
        mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
        sessions::{EffectiveLaunchConfig, SaveMameStateResult, SessionSnapshot, SessionState},
    };

    use super::{
        authorize_record_path, configure_store, delete_record, list_records, load_record,
        upsert_record, SaveStateRecordV1, SAVE_STATE_RECORD_SCHEMA_VERSION,
    };

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
            path: "/tmp/save-states/v1/machine-7061636d616e/software-6e65733a6d6172696f/quick.sta"
                .to_owned(),
            bytes: 4096,
            saved_at_epoch_ms: 99,
        }
    }

    fn record() -> SaveStateRecordV1 {
        SaveStateRecordV1::from_completed_save(&session(), &saved()).expect("record")
    }

    fn store() -> Connection {
        let mut connection = Connection::open_in_memory().expect("in-memory store");
        configure_store(&mut connection).expect("configure store");
        connection
    }

    #[test]
    fn record_captures_required_mt901_provenance_without_executable_path() {
        let record = record();

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
        assert!(!json.to_string().contains("/opt/mame/mame"));
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
    fn record_rejects_relative_traversing_and_malformed_slot_paths() {
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

        let mut slot_escape = saved();
        slot_escape.slot = "../escape".to_owned();
        assert_eq!(
            SaveStateRecordV1::from_completed_save(&session(), &slot_escape)
                .expect_err("slot traversal must fail")
                .code,
            "SAVE_STATE_INVALID_SLOT"
        );
    }

    #[test]
    fn optional_screenshot_metadata_round_trips_as_absent() {
        let record = record();
        let encoded = serde_json::to_string(&record).expect("encode record");
        let decoded: SaveStateRecordV1 = serde_json::from_str(&encoded).expect("decode record");

        assert_eq!(decoded, record);
        assert!(decoded.screenshot.is_none());
    }

    #[test]
    fn store_upserts_by_application_owned_state_path_and_lists_recent_first() {
        let mut connection = store();
        let mut first = record();
        first.saved_at_epoch_ms = 100;
        let first_id = upsert_record(&mut connection, &first).expect("insert first");

        let mut replacement = first.clone();
        replacement.saved_at_epoch_ms = 300;
        replacement.bytes = 8192;
        let replacement_id = upsert_record(&mut connection, &replacement).expect("replace slot");
        assert_eq!(
            replacement_id, first_id,
            "same state path must update one record"
        );

        let mut second = first.clone();
        second.slot = "auto".to_owned();
        second.path = second.path.replace("quick.sta", "auto.sta");
        second.saved_at_epoch_ms = 200;
        let second_id = upsert_record(&mut connection, &second).expect("insert second");
        assert_ne!(second_id, first_id);

        let (total, records) = list_records(&mut connection, 100, 0).expect("list records");
        assert_eq!(total, 2);
        assert_eq!(records[0].0, first_id);
        assert_eq!(records[0].1.bytes, 8192);
        assert_eq!(records[1].0, second_id);
    }

    #[test]
    fn record_lookup_and_delete_are_id_scoped() {
        let mut connection = store();
        let id = upsert_record(&mut connection, &record()).expect("insert record");
        assert_eq!(
            load_record(&mut connection, id).expect("load record"),
            record()
        );
        delete_record(&mut connection, id).expect("delete record");
        assert_eq!(
            load_record(&mut connection, id)
                .expect_err("deleted record must be absent")
                .code,
            "SAVE_STATE_RECORD_NOT_FOUND"
        );
    }

    #[test]
    fn delete_authorization_requires_exact_derived_application_path() {
        let root = Path::new("/tmp/save-states/v1");
        let record = record();
        assert_eq!(
            authorize_record_path(root, &record).expect("authorized path"),
            Path::new(&record.path)
        );

        let mut forged = record;
        forged.path = "/tmp/save-states/v1/other/quick.sta".to_owned();
        assert_eq!(
            authorize_record_path(root, &forged)
                .expect_err("forged path must be rejected")
                .code,
            "SAVE_STATE_RECORD_PATH_UNAUTHORIZED"
        );
    }

    #[test]
    fn future_store_schema_is_rejected() {
        let mut connection = Connection::open_in_memory().expect("in-memory store");
        connection
            .pragma_update(None, "user_version", 99_i64)
            .expect("set future version");
        assert_eq!(
            configure_store(&mut connection)
                .expect_err("future store must fail")
                .code,
            "SAVE_STATE_STORE_SCHEMA_UNSUPPORTED"
        );
    }
}
