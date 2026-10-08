//! Bounded structured diagnostics and support-bundle export.
//!
//! Diagnostics are intentionally project-owned and append-only. The WebView
//! receives only typed snapshots/exports through explicit commands; there is no
//! generic filesystem or logging capability exposed to frontend code.

use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, Runtime, State};

use crate::{
    app::{self, AppInfoRequest, AppInfoResponse, APP_PROTOCOL_VERSION},
    config::{load_settings, settings_path, ContentPathKind, PathValidationStatus},
    content_paths::{effective_content_paths, EffectiveContentPathEntry, EffectiveContentPaths},
    errors::{AppError, AppResult},
    mame::{MameExecutableSourceKind, MameExecutableTrust},
    sessions::{FrameMetricsSnapshot, SessionSnapshot, SessionState, SessionSupervisor},
    storage,
};

const RECENT_LOG_LIMIT: usize = 200;
const LOG_ROTATE_BYTES: u64 = 1024 * 1024;
const LOG_FILE_NAME: &str = "diagnostics.jsonl";

static RECORDER: OnceLock<Mutex<DiagnosticRecorder>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticLogEntry {
    pub timestamp_epoch_ms: u64,
    pub level: String,
    pub category: String,
    pub message: String,
    pub context: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSchemaDiagnostics {
    pub supported_version: i64,
    pub current_version: Option<i64>,
    pub migration_state: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    pub status: String,
    pub active_source: Option<String>,
    pub trust: Option<String>,
    pub failure_domain: Option<String>,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContentPathDiagnostics {
    pub schema_version: u32,
    pub resolution_policy: String,
    pub entries: Vec<EffectiveContentPathEntry>,
    pub total: u32,
    pub rom: u32,
    pub software: u32,
    pub chd: u32,
    pub accessible: u32,
    pub missing: u32,
    pub not_directory: u32,
    pub permission_denied: u32,
    pub unreadable: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameplayInputDiagnostics {
    pub state: String,
    pub bridge: String,
    pub max_updates_per_batch: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayDiagnostics {
    pub session: Option<SessionSnapshot>,
    pub video_transport: String,
    pub video: Option<FrameMetricsSnapshot>,
    pub input: GameplayInputDiagnostics,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSnapshot {
    pub schema_version: u32,
    pub app: AppInfoResponse,
    pub runtime: RuntimeDiagnostics,
    pub platform: String,
    pub architecture: String,
    pub settings_path: String,
    pub catalog_path: String,
    pub catalog_schema: CatalogSchemaDiagnostics,
    pub content_paths: ContentPathDiagnostics,
    pub gameplay: GameplayDiagnostics,
    pub log_path: String,
    pub recent_logs: Vec<DiagnosticLogEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsExportResult {
    pub schema_version: u32,
    pub path: String,
}

struct DiagnosticRecorder {
    log_path: PathBuf,
    recent: VecDeque<DiagnosticLogEntry>,
}

pub fn initialize<R: Runtime>(handle: &AppHandle<R>) -> AppResult<()> {
    if RECORDER.get().is_some() {
        return Ok(());
    }

    let root = handle.path().app_data_dir().map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_ROOT_UNAVAILABLE",
            "The platform application data directory is unavailable for diagnostics.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;
    fs::create_dir_all(&root).map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_DIRECTORY_CREATE_FAILED",
            "The diagnostics directory could not be created.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;

    let log_path = root.join(LOG_FILE_NAME);
    let recent = load_recent_entries(&log_path);
    let _ = RECORDER.set(Mutex::new(DiagnosticRecorder { log_path, recent }));
    record(
        "info",
        "app.lifecycle",
        "Application diagnostics initialized.",
        json!({
            "appVersion": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH
        }),
    );
    Ok(())
}

pub fn record(level: &str, category: &str, message: &str, context: Value) {
    let Some(recorder) = RECORDER.get() else {
        return;
    };
    let Ok(mut recorder) = recorder.lock() else {
        return;
    };

    let entry = DiagnosticLogEntry {
        timestamp_epoch_ms: now_epoch_ms(),
        level: bounded_text(level, 32),
        category: bounded_text(category, 96),
        message: bounded_text(message, 1024),
        context: sanitize_context(context),
    };

    if recorder.recent.len() == RECENT_LOG_LIMIT {
        recorder.recent.pop_front();
    }
    recorder.recent.push_back(entry.clone());
    append_entry(&recorder.log_path, &entry);
}

pub fn record_error(code: &str, message: &str) {
    record(
        "error",
        error_category(code),
        message,
        json!({ "code": bounded_text(code, 128) }),
    );
}

pub fn error_category(code: &str) -> &'static str {
    if code.starts_with("CONFIG_") || code.starts_with("PATH_") {
        "configuration"
    } else if code.starts_with("CATALOG_")
        || code.starts_with("METADATA_")
        || code.starts_with("LIBRARY_")
        || code.contains("AUDIT")
    {
        "data"
    } else if code.starts_with("MAME_")
        || code.starts_with("SESSION_")
        || code.starts_with("CONTROL_")
        || code.starts_with("SAVE_")
        || code.starts_with("LOAD_")
    {
        "runtime"
    } else if code.starts_with("ARTWORK_") {
        "artwork"
    } else if code.starts_with("DIAGNOSTICS_") {
        "diagnostics"
    } else if code.starts_with("APP_") || code == "UNSUPPORTED_PROTOCOL_VERSION" {
        "application"
    } else {
        "backend"
    }
}

pub fn recent_logs() -> Vec<DiagnosticLogEntry> {
    RECORDER
        .get()
        .and_then(|recorder| recorder.lock().ok())
        .map(|recorder| recorder.recent.iter().cloned().collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_diagnostics(
    handle: AppHandle,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<DiagnosticsSnapshot> {
    diagnostics_snapshot(&handle, &supervisor)
}

#[tauri::command]
pub fn export_diagnostics_bundle(
    handle: AppHandle,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<DiagnosticsExportResult> {
    let snapshot = diagnostics_snapshot(&handle, &supervisor)?;
    let root = handle.path().app_data_dir().map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_ROOT_UNAVAILABLE",
            "The platform application data directory is unavailable for diagnostics export.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;
    let export_root = root.join("support-diagnostics");
    fs::create_dir_all(&export_root).map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_EXPORT_DIRECTORY_FAILED",
            "The support diagnostics directory could not be created.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;
    let path = export_root.join(format!("diagnostics-{}.json", now_epoch_ms()));
    let contents = serde_json::to_vec_pretty(&snapshot).map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_EXPORT_SERIALIZE_FAILED",
            "The diagnostics support bundle could not be serialized.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;
    fs::write(&path, contents).map_err(|error| {
        AppError::new(
            "DIAGNOSTICS_EXPORT_WRITE_FAILED",
            "The diagnostics support bundle could not be written.",
        )
        .with_details(json!({ "cause": error.to_string() }))
    })?;
    record(
        "info",
        "diagnostics",
        "Diagnostics support bundle exported.",
        json!({ "destination": "application-data/support-diagnostics" }),
    );

    Ok(DiagnosticsExportResult {
        schema_version: 1,
        path: path.to_string_lossy().into_owned(),
    })
}

fn diagnostics_snapshot(
    handle: &AppHandle,
    supervisor: &SessionSupervisor,
) -> AppResult<DiagnosticsSnapshot> {
    let settings = settings_path(handle)?;
    let catalog = storage::catalog_path(handle)?;
    let app = app::get_app_info(
        AppInfoRequest {
            protocol_version: APP_PROTOCOL_VERSION,
        },
        handle.clone(),
    )?;
    let runtime = runtime_diagnostics(&app.mame);
    let configured_settings = load_settings(&settings)?;
    let content_paths =
        summarize_content_paths(&effective_content_paths(&configured_settings.content_paths));
    let session = supervisor.current_session()?;
    let video = session
        .as_ref()
        .map(|session| supervisor.frame_metrics(&session.session_id))
        .transpose()?;
    let input_state = match session.as_ref().map(|session| session.state) {
        Some(SessionState::Running) => "ready",
        Some(SessionState::Created | SessionState::Starting) => "starting",
        Some(SessionState::Stopping) => "stopping",
        Some(SessionState::Exited | SessionState::Failed | SessionState::Crashed) => "ended",
        None => "inactive",
    }
    .to_owned();
    let gameplay = GameplayDiagnostics {
        session,
        video_transport: if cfg!(all(target_os = "linux", target_endian = "little")) {
            "privateAuthenticatedFifo"
        } else {
            "unsupported"
        }
        .to_owned(),
        video,
        input: GameplayInputDiagnostics {
            state: input_state,
            bridge: "boundedAuthenticatedRuntimeControl".to_owned(),
            max_updates_per_batch: 32,
        },
    };

    let log_path = RECORDER
        .get()
        .and_then(|recorder| recorder.lock().ok())
        .map(|recorder| recorder.log_path.clone())
        .unwrap_or_else(|| PathBuf::from(LOG_FILE_NAME));

    Ok(DiagnosticsSnapshot {
        schema_version: 3,
        app,
        runtime,
        platform: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        settings_path: settings.to_string_lossy().into_owned(),
        catalog_path: catalog.to_string_lossy().into_owned(),
        catalog_schema: inspect_catalog_schema(&catalog),
        content_paths,
        gameplay,
        log_path: log_path.to_string_lossy().into_owned(),
        recent_logs: recent_logs(),
    })
}

pub(crate) fn summarize_content_paths(paths: &EffectiveContentPaths) -> ContentPathDiagnostics {
    let mut summary = ContentPathDiagnostics {
        schema_version: 1,
        resolution_policy: "configuredWithConventionalMameDefaults".to_owned(),
        entries: paths.entries.clone(),
        total: 0,
        rom: 0,
        software: 0,
        chd: 0,
        accessible: 0,
        missing: 0,
        not_directory: 0,
        permission_denied: 0,
        unreadable: 0,
    };

    for entry in &paths.entries {
        summary.total = summary.total.saturating_add(1);
        match entry.kind {
            ContentPathKind::Rom => summary.rom = summary.rom.saturating_add(1),
            ContentPathKind::Software => summary.software = summary.software.saturating_add(1),
            ContentPathKind::Chd => summary.chd = summary.chd.saturating_add(1),
        }
        match entry.validation.status {
            PathValidationStatus::Accessible => {
                summary.accessible = summary.accessible.saturating_add(1)
            }
            PathValidationStatus::Missing => summary.missing = summary.missing.saturating_add(1),
            PathValidationStatus::NotDirectory => {
                summary.not_directory = summary.not_directory.saturating_add(1)
            }
            PathValidationStatus::PermissionDenied => {
                summary.permission_denied = summary.permission_denied.saturating_add(1)
            }
            PathValidationStatus::Unreadable => {
                summary.unreadable = summary.unreadable.saturating_add(1)
            }
        }
    }

    summary
}

fn runtime_diagnostics(report: &app::MameVersionReport) -> RuntimeDiagnostics {
    match report {
        app::MameVersionReport::Available { identity } => RuntimeDiagnostics {
            status: "available".to_owned(),
            active_source: Some(source_label(identity.source).to_owned()),
            trust: Some(trust_label(identity.trust).to_owned()),
            failure_domain: None,
            error_code: None,
            message: None,
        },
        app::MameVersionReport::Unavailable {
            path,
            error_code,
            error_message,
        } => RuntimeDiagnostics {
            status: "unavailable".to_owned(),
            active_source: None,
            trust: None,
            failure_domain: Some(runtime_failure_domain(path.as_deref(), error_code).to_owned()),
            error_code: Some(error_code.clone()),
            message: Some(error_message.clone()),
        },
        app::MameVersionReport::NotConfigured => RuntimeDiagnostics {
            status: "notConfigured".to_owned(),
            active_source: None,
            trust: None,
            failure_domain: Some("legacyNoRuntimeReported".to_owned()),
            error_code: None,
            message: Some(
                "Backend reported no configured runtime; packaged installs should normally resolve bundled MAME."
                    .to_owned(),
            ),
        },
    }
}

fn runtime_failure_domain(path: Option<&str>, error_code: &str) -> &'static str {
    if error_code.starts_with("CONFIG_") {
        return "configuration";
    }
    if error_code == "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE"
        || error_code.starts_with("MAME_BUNDLED_")
        || path.is_none()
    {
        return "bundledPackage";
    }
    "externalOverride"
}

fn source_label(source: MameExecutableSourceKind) -> &'static str {
    match source {
        MameExecutableSourceKind::Bundled => "bundled",
        MameExecutableSourceKind::External => "external",
        MameExecutableSourceKind::DevelopmentTree => "developmentTree",
    }
}

fn trust_label(trust: MameExecutableTrust) -> &'static str {
    match trust {
        MameExecutableTrust::QualifiedBundled => "qualifiedBundled",
        MameExecutableTrust::UserConfigured => "userConfigured",
        MameExecutableTrust::Development => "development",
    }
}

fn inspect_catalog_schema(path: &Path) -> CatalogSchemaDiagnostics {
    if !path.exists() {
        return CatalogSchemaDiagnostics {
            supported_version: storage::CATALOG_SCHEMA_VERSION,
            current_version: None,
            migration_state: "notInitialized".to_owned(),
            error_code: None,
        };
    }

    let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(connection) => connection,
        Err(_) => return unavailable_catalog_schema(),
    };

    let exists = match connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_schema_version')",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(exists) => exists,
        Err(_) => return unavailable_catalog_schema(),
    };

    let current_version = if exists == 0 {
        Some(0)
    } else {
        match connection
            .query_row(
                "SELECT version FROM app_schema_version WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()
        {
            Ok(version) => version,
            Err(_) => return unavailable_catalog_schema(),
        }
    };

    let migration_state = match current_version {
        Some(version) if version == storage::CATALOG_SCHEMA_VERSION => "current",
        Some(version) if version < storage::CATALOG_SCHEMA_VERSION => "migrationRequired",
        Some(_) => "newerThanSupported",
        None => "invalid",
    };

    CatalogSchemaDiagnostics {
        supported_version: storage::CATALOG_SCHEMA_VERSION,
        current_version,
        migration_state: migration_state.to_owned(),
        error_code: if current_version.is_none() {
            Some("CATALOG_SCHEMA_INVALID".to_owned())
        } else {
            None
        },
    }
}

fn unavailable_catalog_schema() -> CatalogSchemaDiagnostics {
    CatalogSchemaDiagnostics {
        supported_version: storage::CATALOG_SCHEMA_VERSION,
        current_version: None,
        migration_state: "unavailable".to_owned(),
        error_code: Some("CATALOG_SCHEMA_INSPECTION_FAILED".to_owned()),
    }
}

fn append_entry(path: &Path, entry: &DiagnosticLogEntry) {
    rotate_if_needed(path);
    let Ok(serialized) = serde_json::to_string(entry) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{serialized}");
}

fn rotate_if_needed(path: &Path) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() < LOG_ROTATE_BYTES {
        return;
    }
    let rotated = path.with_extension("jsonl.1");
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, rotated);
}

fn load_recent_entries(path: &Path) -> VecDeque<DiagnosticLogEntry> {
    let Ok(contents) = fs::read_to_string(path) else {
        return VecDeque::new();
    };
    let mut entries = contents
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<DiagnosticLogEntry>(line).ok())
        .take(RECENT_LOG_LIMIT)
        .collect::<Vec<_>>();
    entries.reverse();
    entries.into()
}

fn sanitize_context(value: Value) -> Value {
    match value {
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    let value = if lower.contains("token")
                        || lower.contains("password")
                        || lower.contains("secret")
                    {
                        Value::String("<redacted>".to_owned())
                    } else if lower.ends_with("path") || lower.ends_with("paths") {
                        Value::String("<path>".to_owned())
                    } else {
                        sanitize_context(value)
                    };
                    (key, value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(sanitize_context).collect()),
        Value::String(value) => Value::String(bounded_text(&value, 512)),
        other => other,
    }
}

fn bounded_text(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::Connection;
    use serde_json::json;
    use tempfile::tempdir;

    use super::{
        error_category, inspect_catalog_schema, runtime_diagnostics, sanitize_context,
        summarize_content_paths,
    };
    use crate::{
        app::MameVersionReport,
        config::{ContentPathsV1, PlatformPath},
        content_paths::effective_content_paths,
        mame::{MameExecutableIdentity, MameExecutableSourceKind, MameExecutableTrust},
        storage::CATALOG_SCHEMA_VERSION,
    };

    #[test]
    fn content_path_diagnostics_use_effective_contract_without_exposing_paths() {
        let root = tempdir().expect("temporary content directory");
        let rom = root.path().join("roms");
        fs::create_dir_all(&rom).expect("create ROM directory");
        let invalid = root.path().join("not-a-directory");
        fs::write(&invalid, b"file").expect("create invalid content path");
        let missing = root.path().join("missing");

        let effective = effective_content_paths(&ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(&rom), PlatformPath::new(&missing)],
            software_paths: vec![PlatformPath::new(&invalid)],
            chd_paths: Vec::new(),
        });
        let summary = summarize_content_paths(&effective);

        assert_eq!(summary.resolution_policy, "configuredOnly");
        assert_eq!(summary.total, 3);
        assert_eq!(summary.rom, 2);
        assert_eq!(summary.software, 1);
        assert_eq!(summary.chd, 0);
        assert_eq!(summary.accessible, 1);
        assert_eq!(summary.missing, 1);
        assert_eq!(summary.not_directory, 1);

        let json = serde_json::to_string(&summary).expect("serialize safe diagnostics");
        assert!(!json.contains(&rom.to_string_lossy().to_string()));
        assert!(!json.contains(&missing.to_string_lossy().to_string()));
    }

    #[test]
    fn maps_stable_error_codes_to_support_categories() {
        assert_eq!(error_category("CONFIG_READ_FAILED"), "configuration");
        assert_eq!(error_category("CATALOG_DATABASE_OPEN_FAILED"), "data");
        assert_eq!(error_category("MAME_IDENTIFIER_INVALID"), "runtime");
        assert_eq!(error_category("ARTWORK_ASSET_UNAVAILABLE"), "artwork");
        assert_eq!(error_category("APP_READY_EVENT_FAILED"), "application");
        assert_eq!(error_category("SOMETHING_ELSE"), "backend");
    }

    #[test]
    fn runtime_diagnostics_classify_bundled_external_and_configuration_failures() {
        let bundled = runtime_diagnostics(&MameVersionReport::Unavailable {
            path: None,
            error_code: "MAME_BUNDLED_RUNTIME_MISSING".to_owned(),
            error_message: "missing bundled runtime".to_owned(),
        });
        assert_eq!(bundled.failure_domain.as_deref(), Some("bundledPackage"));
        assert_eq!(
            bundled.error_code.as_deref(),
            Some("MAME_BUNDLED_RUNTIME_MISSING")
        );

        let external = runtime_diagnostics(&MameVersionReport::Unavailable {
            path: Some("/opt/mame/mame".to_owned()),
            error_code: "MAME_EXECUTABLE_NOT_FOUND".to_owned(),
            error_message: "missing external override".to_owned(),
        });
        assert_eq!(external.failure_domain.as_deref(), Some("externalOverride"));

        let configuration = runtime_diagnostics(&MameVersionReport::Unavailable {
            path: None,
            error_code: "CONFIG_READ_FAILED".to_owned(),
            error_message: "settings unreadable".to_owned(),
        });
        assert_eq!(
            configuration.failure_domain.as_deref(),
            Some("configuration")
        );
    }

    #[test]
    fn runtime_diagnostics_report_available_source_and_trust() {
        let diagnostics = runtime_diagnostics(&MameVersionReport::Available {
            identity: MameExecutableIdentity {
                source: MameExecutableSourceKind::Bundled,
                trust: MameExecutableTrust::QualifiedBundled,
                path: "/package/mame-runtime/bin/mame".to_owned(),
                version: "0.288".to_owned(),
                build: None,
                raw_version_line: "0.288".to_owned(),
            },
        });
        assert_eq!(diagnostics.status, "available");
        assert_eq!(diagnostics.active_source.as_deref(), Some("bundled"));
        assert_eq!(diagnostics.trust.as_deref(), Some("qualifiedBundled"));
        assert_eq!(diagnostics.failure_domain, None);
    }

    #[test]
    fn sanitizes_sensitive_and_path_context() {
        let value = sanitize_context(json!({
            "token": "secret-value",
            "romPath": "/home/user/private/roms",
            "machine": "pacman"
        }));
        assert_eq!(value["token"], "<redacted>");
        assert_eq!(value["romPath"], "<path>");
        assert_eq!(value["machine"], "pacman");
    }

    #[test]
    fn catalog_schema_diagnostics_report_not_initialized_without_creating_database() {
        let root = tempdir().expect("temporary directory");
        let path = root.path().join("catalog.sqlite3");
        let status = inspect_catalog_schema(&path);
        assert_eq!(status.current_version, None);
        assert_eq!(status.migration_state, "notInitialized");
        assert!(!path.exists());
    }

    #[test]
    fn catalog_schema_diagnostics_report_migration_required_read_only() {
        let root = tempdir().expect("temporary directory");
        let path = root.path().join("catalog.sqlite3");
        let connection = Connection::open(&path).expect("create test catalog");
        connection
            .execute_batch(
                "CREATE TABLE app_schema_version (singleton INTEGER PRIMARY KEY, version INTEGER NOT NULL);\nINSERT INTO app_schema_version(singleton, version) VALUES (1, 2);",
            )
            .expect("seed old schema version");
        drop(connection);

        let status = inspect_catalog_schema(&path);
        assert_eq!(status.current_version, Some(2));
        assert_eq!(status.supported_version, CATALOG_SCHEMA_VERSION);
        assert_eq!(status.migration_state, "migrationRequired");
    }

    #[test]
    fn catalog_schema_diagnostics_fail_closed_for_corrupt_database() {
        let root = tempdir().expect("temporary directory");
        let path = root.path().join("catalog.sqlite3");
        fs::write(&path, b"not a sqlite database").expect("seed corrupt catalog");

        let status = inspect_catalog_schema(&path);
        assert_eq!(status.current_version, None);
        assert_eq!(status.migration_state, "unavailable");
        assert_eq!(
            status.error_code.as_deref(),
            Some("CATALOG_SCHEMA_INSPECTION_FAILED")
        );
    }
}
