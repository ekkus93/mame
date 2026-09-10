//! Non-destructive, recoverable persistence for application-owned settings.
//!
//! MT-603 deliberately keeps application settings separate from MAME-owned INI
//! and CFG files. This module preserves fields the current application does not
//! understand, maintains a last-known-good backup before replacement, and uses
//! a same-directory temporary file for atomic replacement.

use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};
use tempfile::Builder;

use crate::{
    config::{parse_settings_json, SettingsV2, SETTINGS_SCHEMA_VERSION},
    errors::{AppError, AppResult},
};

/// Persist current settings without discarding unknown fields from an existing
/// settings document.
///
/// If an existing primary file is valid, its exact bytes are atomically copied
/// to `<settings-path>.bak` before the new primary is committed. Invalid or
/// unsupported existing settings are never overwritten.
pub fn persist_settings(path: &Path, settings: &SettingsV2) -> AppResult<()> {
    if settings.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(AppError::new(
            "CONFIG_SCHEMA_INVALID",
            "Only the current settings schema can be persisted.",
        )
        .with_details(serde_json::json!({
            "supportedVersion": SETTINGS_SCHEMA_VERSION,
            "foundVersion": settings.schema_version,
        })));
    }

    let existing = read_existing_document(path)?;
    let base = existing
        .as_ref()
        .map(|document| document.value.clone())
        .unwrap_or_else(|| Value::Object(Map::new()));
    let merged = merge_settings_value(base, settings)?;
    let encoded = encode_document(&merged)?;

    if let Some(document) = existing {
        let backup = settings_backup_path(path);
        atomic_replace(&backup, &document.bytes).map_err(|error| {
            AppError::new(
                "CONFIG_BACKUP_WRITE_FAILED",
                "The previous application settings could not be saved as a recovery backup.",
            )
            .with_details(serde_json::json!({
                "backupPath": backup,
                "cause": error.to_string(),
            }))
        })?;
    }

    atomic_replace(path, &encoded).map_err(|error| {
        AppError::new(
            "CONFIG_ATOMIC_WRITE_FAILED",
            "The application settings could not be atomically replaced.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "cause": error.to_string(),
        }))
    })
}

/// Restore the exact last-known-good backup into the primary settings path.
///
/// Recovery is explicit rather than an implicit fallback from `load_settings`:
/// callers must decide to recover after a surfaced primary-file error. The
/// backup itself is not rotated while recovery is in progress, so a failed
/// restore cannot destroy the recovery source.
pub fn recover_settings_from_backup(path: &Path) -> AppResult<SettingsV2> {
    let backup = settings_backup_path(path);
    let bytes = fs::read(&backup).map_err(|error| {
        AppError::new(
            "CONFIG_BACKUP_READ_FAILED",
            "The application settings recovery backup could not be read.",
        )
        .with_details(serde_json::json!({
            "backupPath": backup,
            "cause": error.to_string(),
        }))
    })?;

    let text = std::str::from_utf8(&bytes).map_err(|error| {
        AppError::new(
            "CONFIG_BACKUP_INVALID",
            "The application settings recovery backup is not UTF-8 JSON.",
        )
        .with_details(serde_json::json!({
            "backupPath": backup,
            "cause": error.to_string(),
        }))
    })?;
    let settings = parse_settings_json(text).map_err(|error| {
        AppError::new(
            "CONFIG_BACKUP_INVALID",
            "The application settings recovery backup is not a supported settings document.",
        )
        .with_details(serde_json::json!({
            "backupPath": backup,
            "causeCode": error.code,
            "causeMessage": error.message,
        }))
    })?;

    atomic_replace(path, &bytes).map_err(|error| {
        AppError::new(
            "CONFIG_RECOVERY_WRITE_FAILED",
            "The application settings recovery backup could not be restored atomically.",
        )
        .with_details(serde_json::json!({
            "path": path,
            "backupPath": backup,
            "cause": error.to_string(),
        }))
    })?;

    Ok(settings)
}

pub fn settings_backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".bak");
    PathBuf::from(backup)
}

#[derive(Debug)]
struct ExistingDocument {
    bytes: Vec<u8>,
    value: Value,
}

fn read_existing_document(path: &Path) -> AppResult<Option<ExistingDocument>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(AppError::new(
                "CONFIG_READ_FAILED",
                "The application settings file could not be read before persistence.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    };

    let text = std::str::from_utf8(&bytes).map_err(|error| {
        AppError::new(
            "CONFIG_INVALID_JSON",
            "The application settings file is not UTF-8 JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    // Validate the schema before retaining the raw document. This ensures an
    // unsupported future schema or malformed known field is never rewritten.
    parse_settings_json(text)?;
    let value = serde_json::from_str(text).map_err(|error| {
        AppError::new(
            "CONFIG_INVALID_JSON",
            "The application settings file is not valid JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    Ok(Some(ExistingDocument { bytes, value }))
}

fn merge_settings_value(mut base: Value, settings: &SettingsV2) -> AppResult<Value> {
    let root = base.as_object_mut().ok_or_else(|| {
        AppError::new(
            "CONFIG_SCHEMA_INVALID",
            "The application settings document root must be a JSON object.",
        )
    })?;

    root.insert(
        "schemaVersion".to_owned(),
        Value::from(SETTINGS_SCHEMA_VERSION),
    );
    root.insert(
        "mameExecutable".to_owned(),
        serde_json::to_value(&settings.mame_executable).map_err(serialization_error)?,
    );

    let known_content_paths = serde_json::to_value(&settings.content_paths)
        .map_err(serialization_error)?
        .as_object()
        .cloned()
        .ok_or_else(|| {
            AppError::new(
                "CONFIG_SERIALIZE_FAILED",
                "The application content-path settings did not serialize as a JSON object.",
            )
        })?;

    match root.get_mut("contentPaths") {
        Some(Value::Object(existing)) => {
            for (key, value) in known_content_paths {
                existing.insert(key, value);
            }
        }
        _ => {
            root.insert(
                "contentPaths".to_owned(),
                Value::Object(known_content_paths),
            );
        }
    }

    Ok(base)
}

fn encode_document(value: &Value) -> AppResult<Vec<u8>> {
    let mut encoded = serde_json::to_vec_pretty(value).map_err(serialization_error)?;
    encoded.push(b'\n');
    Ok(encoded)
}

fn serialization_error(error: serde_json::Error) -> AppError {
    AppError::new(
        "CONFIG_SERIALIZE_FAILED",
        "The application settings could not be serialized.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

/// Write through a sibling temporary file and atomically replace the target.
/// `NamedTempFile::persist` provides replace semantics on supported desktop
/// platforms; keeping the temporary file in the target directory also avoids
/// cross-filesystem rename failures.
fn atomic_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "configuration path has no parent directory",
        )
    })?;
    fs::create_dir_all(parent)?;

    let mut temp = Builder::new()
        .prefix(".mame-tauri-settings-")
        .tempfile_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;

    let persisted = temp.persist(path).map_err(|error| error.error)?;
    persisted.sync_all()?;
    sync_parent_directory(parent)?;
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::{json, Value};

    use crate::config::{
        load_settings, ContentPathsV1, PlatformPath, SettingsV2, SETTINGS_SCHEMA_VERSION,
    };

    use super::{persist_settings, recover_settings_from_backup, settings_backup_path};

    #[test]
    fn round_trip_preserves_unrelated_top_level_and_nested_fields() {
        let root = temp_root("preserve-unknown");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let path = root.join("settings.json");
        fs::write(
            &path,
            r#"{
              "schemaVersion": 2,
              "mameExecutable": "/opt/mame/mame",
              "contentPaths": {
                "romPaths": [],
                "softwarePaths": [],
                "chdPaths": [],
                "futureContentPolicy": {"scan": "manual"}
              },
              "window": {"fullscreen": true, "monitor": 2},
              "futureFeature": {"enabled": true, "threshold": 7}
            }"#,
        )
        .expect("seed settings");

        let mut settings = load_settings(&path).expect("load supported settings");
        settings.content_paths.rom_paths = vec![PlatformPath::new("/games/roms")];
        persist_settings(&path, &settings).expect("persist merged settings");

        let persisted: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read persisted settings"))
                .expect("persisted settings remain JSON");
        assert_eq!(
            persisted["window"],
            json!({"fullscreen": true, "monitor": 2})
        );
        assert_eq!(
            persisted["futureFeature"],
            json!({"enabled": true, "threshold": 7})
        );
        assert_eq!(
            persisted["contentPaths"]["futureContentPolicy"],
            json!({"scan": "manual"})
        );
        assert_eq!(
            persisted["contentPaths"]["romPaths"],
            json!(["/games/roms"])
        );

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    #[test]
    fn schema_v1_write_upgrades_known_fields_without_dropping_unknown_fields() {
        let root = temp_root("migrate-preserve");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let path = root.join("settings.json");
        fs::write(
            &path,
            r#"{"schemaVersion":1,"mameExecutable":"/opt/mame/mame","legacyUserSetting":{"keep":true}}"#,
        )
        .expect("seed schema v1 settings");

        let settings = load_settings(&path).expect("migrate schema v1 in memory");
        persist_settings(&path, &settings).expect("persist migrated settings");

        let persisted: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read migrated settings"))
                .expect("migrated settings remain JSON");
        assert_eq!(persisted["schemaVersion"], SETTINGS_SCHEMA_VERSION);
        assert_eq!(persisted["legacyUserSetting"], json!({"keep": true}));
        assert!(persisted["contentPaths"].is_object());

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    #[test]
    fn replacement_rotates_exact_previous_primary_into_backup() {
        let root = temp_root("backup");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let path = root.join("settings.json");
        let previous = br#"{"schemaVersion":2,"mameExecutable":"/old/mame","contentPaths":{"romPaths":[],"softwarePaths":[],"chdPaths":[]},"preserveMe":42}"#;
        fs::write(&path, previous).expect("seed settings");

        let mut settings = load_settings(&path).expect("load seeded settings");
        settings.mame_executable = Some("/new/mame".to_owned());
        persist_settings(&path, &settings).expect("replace settings");

        assert_eq!(
            fs::read(settings_backup_path(&path)).expect("read backup"),
            previous
        );
        assert_eq!(
            load_settings(&path)
                .expect("load replacement")
                .mame_executable
                .as_deref(),
            Some("/new/mame")
        );

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    #[test]
    fn explicit_recovery_restores_last_known_good_backup_without_rotating_it() {
        let root = temp_root("recover");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let path = root.join("settings.json");
        let original = br#"{"schemaVersion":2,"mameExecutable":"/known-good/mame","contentPaths":{"romPaths":[],"softwarePaths":[],"chdPaths":[]},"keep":{"x":1}}"#;
        fs::write(&path, original).expect("seed settings");

        let mut settings = load_settings(&path).expect("load seeded settings");
        settings.mame_executable = Some("/new/mame".to_owned());
        persist_settings(&path, &settings).expect("create backup while replacing settings");
        let backup_path = settings_backup_path(&path);
        let backup_before = fs::read(&backup_path).expect("read recovery backup");
        fs::write(&path, b"{corrupt").expect("simulate corrupted primary");

        let recovered = recover_settings_from_backup(&path).expect("restore recovery backup");
        assert_eq!(
            recovered.mame_executable.as_deref(),
            Some("/known-good/mame")
        );
        assert_eq!(fs::read(&path).expect("read restored primary"), original);
        assert_eq!(
            fs::read(&backup_path).expect("backup remains intact"),
            backup_before
        );

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    #[test]
    fn malformed_existing_settings_are_never_overwritten() {
        let root = temp_root("invalid-primary");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let path = root.join("settings.json");
        let invalid = b"{definitely-not-json";
        fs::write(&path, invalid).expect("seed malformed settings");

        let error = persist_settings(&path, &SettingsV2::default())
            .expect_err("malformed settings must block persistence");
        assert_eq!(error.code, "CONFIG_INVALID_JSON");
        assert_eq!(fs::read(&path).expect("read untouched primary"), invalid);
        assert!(!settings_backup_path(&path).exists());

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    #[test]
    fn first_write_is_complete_and_does_not_invent_a_backup() {
        let root = temp_root("first-write");
        let path = root.join("settings.json");
        let settings = SettingsV2 {
            schema_version: SETTINGS_SCHEMA_VERSION,
            mame_executable: Some("/opt/mame/mame".to_owned()),
            content_paths: ContentPathsV1::default(),
        };

        persist_settings(&path, &settings).expect("persist first settings document");
        let loaded = load_settings(&path).expect("load first settings document");
        assert_eq!(loaded, settings);
        assert!(!settings_backup_path(&path).exists());

        let leaked_temp = fs::read_dir(&root)
            .expect("list settings directory")
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".mame-tauri-settings-")
            });
        assert!(!leaked_temp, "atomic replacement must not leak temp files");

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-mt603-{label}-{nonce}"))
    }
}
