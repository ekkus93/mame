use std::{fs, path::Path};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{
    content_paths::EffectiveContentPaths,
    errors::{AppError, AppResult},
    storage,
};

use super::{
    MameAuditClassification, MameAuditParseResult, MameExecutableIdentity, MameExecutableSourceKind,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredMachineAuditResult {
    pub machine_short_name: String,
    pub result: MameAuditParseResult,
    pub audited_at_epoch_ms: u64,
}

pub fn save_machine_audit_result(
    catalog_path: &Path,
    machine_short_name: &str,
    result: &MameAuditParseResult,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
    audited_at_epoch_ms: u64,
) -> AppResult<()> {
    let connection = storage::open_catalog_connection(catalog_path)?;
    save_machine_audit_result_with_connection(
        &connection,
        machine_short_name,
        result,
        mame_identity,
        content_paths,
        audited_at_epoch_ms,
    )
}

pub fn load_current_machine_audit_result(
    catalog_path: &Path,
    machine_short_name: &str,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
) -> AppResult<Option<StoredMachineAuditResult>> {
    let connection = storage::open_catalog_connection(catalog_path)?;
    load_current_machine_audit_result_with_connection(
        &connection,
        machine_short_name,
        mame_identity,
        content_paths,
    )
}

pub fn invalidate_stale_machine_audit_results(
    catalog_path: &Path,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
) -> AppResult<u64> {
    let connection = storage::open_catalog_connection(catalog_path)?;
    invalidate_stale_with_connection(&connection, mame_identity, content_paths)
}

fn save_machine_audit_result_with_connection(
    connection: &Connection,
    machine_short_name: &str,
    result: &MameAuditParseResult,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
    audited_at_epoch_ms: u64,
) -> AppResult<()> {
    if machine_short_name.trim().is_empty() {
        return Err(AppError::new(
            "MAME_AUDIT_MACHINE_EMPTY",
            "The machine short name for an audit result cannot be empty.",
        ));
    }

    let audited_at = to_i64(audited_at_epoch_ms, "auditedAtEpochMs")?;
    let result_json = serialize_json(result, "MAME_AUDIT_RESULT_SERIALIZE_FAILED")?;
    let identity_json = serialize_identity_json(mame_identity)?;
    let content_paths_json =
        serialize_json(content_paths, "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED")?;

    connection
        .execute(
            r#"INSERT INTO machine_audit_results(
                machine_short_name,
                classification,
                result_json,
                audited_at_epoch_ms,
                mame_identity_json,
                content_paths_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(machine_short_name) DO UPDATE SET
                classification = excluded.classification,
                result_json = excluded.result_json,
                audited_at_epoch_ms = excluded.audited_at_epoch_ms,
                mame_identity_json = excluded.mame_identity_json,
                content_paths_json = excluded.content_paths_json"#,
            params![
                machine_short_name,
                classification_token(result.classification),
                result_json,
                audited_at,
                identity_json,
                content_paths_json,
            ],
        )
        .map_err(|error| database_error("MAME_AUDIT_RESULT_WRITE_FAILED", error))?;

    Ok(())
}

fn load_current_machine_audit_result_with_connection(
    connection: &Connection,
    machine_short_name: &str,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
) -> AppResult<Option<StoredMachineAuditResult>> {
    let identity_json = serialize_identity_json(mame_identity)?;
    let content_paths_json =
        serialize_json(content_paths, "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED")?;

    invalidate_stale_json_with_connection(connection, &identity_json, &content_paths_json)?;

    let stored = connection
        .query_row(
            r#"SELECT classification, result_json, audited_at_epoch_ms
            FROM machine_audit_results
            WHERE machine_short_name = ?1"#,
            [machine_short_name],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|error| database_error("MAME_AUDIT_RESULT_READ_FAILED", error))?;

    let Some((classification, result_json, audited_at)) = stored else {
        return Ok(None);
    };

    let result: MameAuditParseResult = serde_json::from_str(&result_json).map_err(|error| {
        AppError::new(
            "MAME_AUDIT_RESULT_INVALID",
            "A persisted MAME audit result could not be decoded.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    if classification != classification_token(result.classification) {
        return Err(AppError::new(
            "MAME_AUDIT_RESULT_INVALID",
            "A persisted MAME audit result has inconsistent classification data.",
        )
        .with_details(serde_json::json!({
            "storedClassification": classification,
            "decodedClassification": classification_token(result.classification)
        })));
    }

    let audited_at_epoch_ms = u64::try_from(audited_at).map_err(|_| {
        AppError::new(
            "MAME_AUDIT_RESULT_INVALID",
            "A persisted MAME audit result has an invalid negative timestamp.",
        )
        .with_details(serde_json::json!({ "auditedAtEpochMs": audited_at }))
    })?;

    Ok(Some(StoredMachineAuditResult {
        machine_short_name: machine_short_name.to_owned(),
        result,
        audited_at_epoch_ms,
    }))
}

fn invalidate_stale_with_connection(
    connection: &Connection,
    mame_identity: &MameExecutableIdentity,
    content_paths: &EffectiveContentPaths,
) -> AppResult<u64> {
    let identity_json = serialize_identity_json(mame_identity)?;
    let content_paths_json =
        serialize_json(content_paths, "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED")?;
    invalidate_stale_json_with_connection(connection, &identity_json, &content_paths_json)
}

fn invalidate_stale_json_with_connection(
    connection: &Connection,
    identity_json: &str,
    content_paths_json: &str,
) -> AppResult<u64> {
    // Older AppImage builds persisted their randomized mount path as part of the bundled
    // executable identity. Rebase matching audits to the stable identity before invalidation.
    connection
        .execute(
            r#"UPDATE machine_audit_results
            SET mame_identity_json = ?1
            WHERE json_extract(?1, '$.source') = 'bundled'
              AND json_extract(?1, '$.path') = '<bundled-mame>'
              AND json_extract(mame_identity_json, '$.source') = 'bundled'
              AND json_remove(mame_identity_json, '$.path') = json_remove(?1, '$.path')"#,
            [identity_json],
        )
        .map_err(|error| database_error("MAME_AUDIT_STALE_INVALIDATION_FAILED", error))?;

    let deleted = connection
        .execute(
            r#"DELETE FROM machine_audit_results
            WHERE mame_identity_json <> ?1 OR content_paths_json <> ?2"#,
            params![identity_json, content_paths_json],
        )
        .map_err(|error| database_error("MAME_AUDIT_STALE_INVALIDATION_FAILED", error))?;

    u64::try_from(deleted).map_err(|_| {
        AppError::new(
            "MAME_AUDIT_STALE_INVALIDATION_FAILED",
            "The database returned an invalid stale-audit deletion count.",
        )
    })
}

fn serialize_identity_json(identity: &MameExecutableIdentity) -> AppResult<String> {
    let baseline = identity.audit_provenance_json().map_err(|error| {
        AppError::new(
            "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED",
            "MAME audit identity could not be serialized.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    if identity.source != MameExecutableSourceKind::Bundled {
        return Ok(baseline);
    }

    let executable = Path::new(&identity.path);
    // Synthetic audit-store tests may supply a virtual bundled identity.
    // Production inspectors require the executable to exist before auditing.
    if !executable.is_file() {
        return Ok(baseline);
    }

    let root = executable.parent().and_then(Path::parent).ok_or_else(|| {
        AppError::new(
            "MAME_AUDIT_PROVENANCE_UNAVAILABLE",
            "The bundled MAME executable has no runtime resource root.",
        )
    })?;
    let manifest = root.join("runtime-provenance.txt");
    let raw = fs::read_to_string(&manifest).map_err(|error| {
        AppError::new(
            "MAME_AUDIT_PROVENANCE_UNAVAILABLE",
            "The bundled MAME runtime's immutable executable identity is unavailable.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    let digest = raw
        .lines()
        .find_map(|line| line.strip_prefix("mame_sha256="))
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| {
            AppError::new(
                "MAME_AUDIT_PROVENANCE_INVALID",
                "The bundled MAME runtime provenance has no valid executable SHA-256.",
            )
        })?;
    let mut value: serde_json::Value = serde_json::from_str(&baseline).map_err(|error| {
        AppError::new(
            "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED",
            "Bundled MAME audit provenance could not be decoded.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    value["runtimeDigestSha256"] = serde_json::Value::String(digest.to_ascii_lowercase());
    serde_json::to_string(&value).map_err(|error| {
        AppError::new(
            "MAME_AUDIT_PROVENANCE_SERIALIZE_FAILED",
            "Bundled MAME audit provenance could not be encoded.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn classification_token(classification: MameAuditClassification) -> &'static str {
    match classification {
        MameAuditClassification::Complete => "complete",
        MameAuditClassification::BestAvailable => "bestAvailable",
        MameAuditClassification::MissingRequired => "missingRequired",
        MameAuditClassification::Incorrect => "incorrect",
        MameAuditClassification::MixedFailure => "mixedFailure",
        MameAuditClassification::Unknown => "unknown",
    }
}

fn serialize_json<T: Serialize>(value: &T, code: &str) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| {
        AppError::new(code, "MAME audit persistence data could not be serialized.")
            .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn to_i64(value: u64, field: &str) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| {
        AppError::new(
            "MAME_AUDIT_VALUE_OUT_OF_RANGE",
            "A MAME audit persistence value cannot be represented by SQLite.",
        )
        .with_details(serde_json::json!({ "field": field, "value": value }))
    })
}

fn database_error(code: &str, error: rusqlite::Error) -> AppError {
    AppError::new(code, "The MAME audit database operation failed.")
        .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{ContentPathsV1, PlatformPath},
        content_paths::{effective_content_paths, EffectiveContentPaths},
        mame::{
            parse_mame_audit_output, MameExecutableIdentity, MameExecutableSourceKind,
            MameExecutableTrust,
        },
        storage,
    };

    use super::{
        invalidate_stale_with_connection, load_current_machine_audit_result_with_connection,
        save_machine_audit_result_with_connection,
    };

    fn identity(version: &str) -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::External,
            trust: MameExecutableTrust::UserConfigured,
            path: "/opt/mame/mame".to_owned(),
            version: version.to_owned(),
            build: Some("test-build".to_owned()),
            raw_version_line: format!("MAME {version} (test-build)"),
        }
    }

    fn bundled_identity(path: &str) -> MameExecutableIdentity {
        MameExecutableIdentity {
            source: MameExecutableSourceKind::Bundled,
            trust: MameExecutableTrust::QualifiedBundled,
            path: path.to_owned(),
            version: "0.289".to_owned(),
            build: Some("test-build".to_owned()),
            raw_version_line: "0.289 (test-build)".to_owned(),
        }
    }

    fn paths(first: &str, second: &str) -> EffectiveContentPaths {
        effective_content_paths(&ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(first), PlatformPath::new(second)],
            software_paths: vec![PlatformPath::new("/software")],
            chd_paths: vec![PlatformPath::new("/chd")],
        })
    }

    #[test]
    fn persists_and_loads_result_for_exact_provenance() {
        let connection = storage::open_catalog_memory().expect("catalog");
        let current_identity = identity("0.280");
        let current_paths = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &current_identity,
            &current_paths,
            1234,
        )
        .expect("persist audit result");

        let loaded = load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &current_identity,
            &current_paths,
        )
        .expect("load audit result")
        .expect("current audit result");

        assert_eq!(loaded.machine_short_name, "pacman");
        assert_eq!(loaded.result, result);
        assert_eq!(loaded.audited_at_epoch_ms, 1234);
    }

    #[test]
    fn bundled_appimage_mount_changes_preserve_existing_audits() {
        let connection = storage::open_catalog_memory().expect("catalog");
        let old_identity = bundled_identity("/tmp/.mount_old/usr/lib/mame");
        let current_identity = bundled_identity("/tmp/.mount_new/usr/lib/mame");
        let current_paths = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));
        let old_identity_json = serde_json::to_string(&old_identity).expect("old identity JSON");
        let result_json = serde_json::to_string(&result).expect("audit result JSON");
        let paths_json = serde_json::to_string(&current_paths).expect("paths JSON");

        connection
            .execute(
                r#"INSERT INTO machine_audit_results(
                    machine_short_name, classification, result_json, audited_at_epoch_ms,
                    mame_identity_json, content_paths_json
                ) VALUES ('pacman', 'complete', ?1, 1234, ?2, ?3)"#,
                rusqlite::params![result_json, old_identity_json, paths_json],
            )
            .expect("seed audit from previous AppImage mount");

        let loaded = load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &current_identity,
            &current_paths,
        )
        .expect("load audit across mount path change")
        .expect("matching bundled audit remains current");

        assert_eq!(loaded.result, result);
        let stored_identity: String = connection
            .query_row(
                "SELECT mame_identity_json FROM machine_audit_results WHERE machine_short_name = 'pacman'",
                [],
                |row| row.get(0),
            )
            .expect("read rebased identity");
        assert_eq!(
            stored_identity,
            current_identity.audit_provenance_json().unwrap()
        );
    }

    #[test]
    fn bundled_digest_preserves_audit_across_mounts_and_rejects_changed_binary() {
        use std::fs;
        let fixture = tempfile::tempdir().expect("tempdir");
        let create_runtime = |mount: &str, executable_bytes: &[u8], digest_char: char| {
            let runtime = fixture.path().join(mount).join("mame-runtime");
            let executable = runtime.join("bin").join("mame");
            fs::create_dir_all(executable.parent().expect("bin")).expect("runtime bin");
            fs::write(&executable, executable_bytes).expect("executable");
            fs::write(
                runtime.join("runtime-provenance.txt"),
                format!(
                    "mode=real\nrelease_qualified=true\nmame_sha256={}\n",
                    digest_char.to_string().repeat(64)
                ),
            )
            .expect("manifest");
            bundled_identity(executable.to_str().expect("utf8 fixture path"))
        };
        let first = create_runtime("mount-a", b"bundled MAME executable v1", 'a');
        let same = create_runtime("mount-b", b"bundled MAME executable v1", 'a');
        let changed = create_runtime("mount-c", b"bundled MAME executable v2", 'b');
        let connection = storage::open_catalog_memory().expect("catalog");
        let content = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &first,
            &content,
            1234,
        )
        .expect("save digest-bearing audit");
        assert!(load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &same,
            &content
        )
        .expect("different mount, same packaged executable digest")
        .is_some());
        assert!(load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &changed,
            &content
        )
        .expect("same version, changed bundled executable digest")
        .is_none());
    }

    #[test]
    fn legacy_bundled_audit_without_digest_is_invalidated_on_upgrade() {
        use std::fs;
        let fixture = tempfile::tempdir().expect("tempdir");
        let root = fixture.path().join("mame-runtime");
        let executable = root.join("bin").join("mame");
        fs::create_dir_all(executable.parent().expect("bin")).expect("runtime bin");
        fs::write(&executable, b"runtime").expect("executable");
        fs::write(
            root.join("runtime-provenance.txt"),
            format!("mame_sha256={}\n", "a".repeat(64)),
        )
        .expect("digest");
        let current = bundled_identity(executable.to_str().expect("fixture utf8"));
        let original = bundled_identity("/tmp/.mount_legacy/mame-runtime/bin/mame");
        let connection = storage::open_catalog_memory().expect("catalog");
        let content = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));
        let original_json = serde_json::to_string(&original).expect("legacy identity");
        connection
            .execute(
                r#"INSERT INTO machine_audit_results(
                machine_short_name, classification, result_json, audited_at_epoch_ms,
                mame_identity_json, content_paths_json
            ) VALUES ('pacman', 'complete', ?1, 1234, ?2, ?3)"#,
                rusqlite::params![
                    serde_json::to_string(&result).unwrap(),
                    original_json,
                    serde_json::to_string(&content).unwrap()
                ],
            )
            .expect("old audit");
        assert!(load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &current,
            &content
        )
        .expect("legacy audit requalification")
        .is_none());
    }

    #[test]
    fn changed_mame_identity_invalidates_persisted_results() {
        let connection = storage::open_catalog_memory().expect("catalog");
        let old_identity = identity("0.280");
        let current_paths = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &old_identity,
            &current_paths,
            1234,
        )
        .expect("persist audit result");

        let loaded = load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &identity("0.281"),
            &current_paths,
        )
        .expect("stale audit lookup");

        assert!(loaded.is_none());
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM machine_audit_results", [], |row| {
                row.get(0)
            })
            .expect("audit result count");
        assert_eq!(count, 0, "stale audit row must be deleted");
    }

    #[test]
    fn changed_or_reordered_paths_invalidate_persisted_results() {
        let connection = storage::open_catalog_memory().expect("catalog");
        let current_identity = identity("0.280");
        let original_paths = paths("/roms-a", "/roms-b");
        let reordered_paths = paths("/roms-b", "/roms-a");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &current_identity,
            &original_paths,
            1234,
        )
        .expect("persist audit result");

        let loaded = load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &current_identity,
            &reordered_paths,
        )
        .expect("stale audit lookup");

        assert!(loaded.is_none());
    }

    #[test]
    fn explicit_invalidation_preserves_only_current_provenance() {
        let connection = storage::open_catalog_memory().expect("catalog");
        let current_identity = identity("0.280");
        let current_paths = paths("/roms-a", "/roms-b");
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &current_identity,
            &current_paths,
            1234,
        )
        .expect("persist current audit result");

        connection
            .execute(
                r#"INSERT INTO machine_audit_results(
                    machine_short_name, classification, result_json, audited_at_epoch_ms,
                    mame_identity_json, content_paths_json
                ) SELECT 'galaga', classification, result_json, audited_at_epoch_ms,
                    'stale-identity', content_paths_json
                FROM machine_audit_results WHERE machine_short_name = 'pacman'"#,
                [],
            )
            .expect("seed stale row");

        let deleted =
            invalidate_stale_with_connection(&connection, &current_identity, &current_paths)
                .expect("invalidate stale audit rows");
        assert_eq!(deleted, 1);

        let names = connection
            .prepare(
                "SELECT machine_short_name FROM machine_audit_results ORDER BY machine_short_name",
            )
            .expect("prepare remaining audit query")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query remaining audits")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect remaining audits");
        assert_eq!(names, vec!["pacman"]);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_content_paths_participate_losslessly_in_provenance() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

        let connection = storage::open_catalog_memory().expect("catalog");
        let current_identity = identity("0.280");
        let original = effective_content_paths(&ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(PathBuf::from(OsString::from_vec(vec![
                b'/', b'r', b'o', b'm', 0xff,
            ])))],
            software_paths: Vec::new(),
            chd_paths: Vec::new(),
        });
        let changed = effective_content_paths(&ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(PathBuf::from(OsString::from_vec(vec![
                b'/', b'r', b'o', b'm', 0xfe,
            ])))],
            software_paths: Vec::new(),
            chd_paths: Vec::new(),
        });
        let result = parse_mame_audit_output("romset pacman is good\n", "", Some(0));

        save_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &result,
            &current_identity,
            &original,
            1234,
        )
        .expect("persist audit result with non-UTF-8 path");

        let loaded = load_current_machine_audit_result_with_connection(
            &connection,
            "pacman",
            &current_identity,
            &changed,
        )
        .expect("lookup changed non-UTF-8 provenance");

        assert!(loaded.is_none());
    }
}
