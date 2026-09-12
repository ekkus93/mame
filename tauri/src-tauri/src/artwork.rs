//! Typed local artwork configuration, discovery, and frontend metadata.
//!
//! Artwork is presentation data, not a privileged filesystem API. Public
//! descriptors therefore identify the logical artwork asset and its provenance
//! without exposing a configured root or canonical host path to the WebView.

use std::{
    fs,
    io::{self, ErrorKind, Write},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::DialogExt;
use tempfile::Builder;

use crate::{
    config::{PathValidation, PathValidationStatus, PlatformPath},
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
};

const ARTWORK_SCHEMA_VERSION: u32 = 1;
const MAX_ARTWORK_ROOTS: usize = 32;
const MAX_ARTWORK_FILE_BYTES: u64 = 64 * 1024 * 1024;
const ARTWORK_EXTENSIONS: [(&str, &str); 4] = [
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
];

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkKind {
    Screenshot,
    Cabinet,
    Marquee,
    Flyer,
    Icon,
    SystemImage,
}

impl ArtworkKind {
    const ALL: [Self; 6] = [
        Self::Screenshot,
        Self::Cabinet,
        Self::Marquee,
        Self::Flyer,
        Self::Icon,
        Self::SystemImage,
    ];

    fn directory_name(self) -> &'static str {
        match self {
            Self::Screenshot => "snap",
            Self::Cabinet => "cabinets",
            Self::Marquee => "marquees",
            Self::Flyer => "flyers",
            Self::Icon => "icons",
            Self::SystemImage => "systems",
        }
    }

    fn asset_token(self) -> &'static str {
        match self {
            Self::Screenshot => "screenshot",
            Self::Cabinet => "cabinet",
            Self::Marquee => "marquee",
            Self::Flyer => "flyer",
            Self::Icon => "icon",
            Self::SystemImage => "systemImage",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkProvenanceKind {
    LocalFile,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkProvenance {
    pub kind: ArtworkProvenanceKind,
    /// Stable configured-root ordinal. The root path itself is intentionally
    /// not included in the public descriptor.
    pub root_index: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkDescriptor {
    pub schema_version: u32,
    pub asset_id: String,
    pub machine: String,
    pub kind: ArtworkKind,
    pub mime_type: String,
    pub bytes: u64,
    pub provenance: ArtworkProvenance,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkSlot {
    pub kind: ArtworkKind,
    pub asset: Option<ArtworkDescriptor>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineArtwork {
    pub schema_version: u32,
    pub machine: String,
    pub slots: Vec<ArtworkSlot>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkConfiguration {
    pub schema_version: u32,
    pub roots: Vec<PlatformPath>,
    pub validations: Vec<PathValidation>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetArtworkRootsRequest {
    pub roots: Vec<PlatformPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtworkSettingsV1 {
    schema_version: u32,
    #[serde(default)]
    roots: Vec<PlatformPath>,
}

impl Default for ArtworkSettingsV1 {
    fn default() -> Self {
        Self {
            schema_version: ARTWORK_SCHEMA_VERSION,
            roots: Vec::new(),
        }
    }
}

fn artwork_settings_path(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|root| root.join("artwork-settings.json"))
        .map_err(|error| {
            AppError::new(
                "ARTWORK_CONFIG_ROOT_UNAVAILABLE",
                "The platform application configuration directory is unavailable for artwork settings.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

fn load_artwork_settings(path: &Path) -> AppResult<ArtworkSettingsV1> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(ArtworkSettingsV1::default())
        }
        Err(error) => {
            return Err(AppError::new(
                "ARTWORK_CONFIG_READ_FAILED",
                "Artwork settings could not be read.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    };

    let value: Value = serde_json::from_str(&contents).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_INVALID_JSON",
            "Artwork settings are not valid JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    let version = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::new(
                "ARTWORK_CONFIG_SCHEMA_VERSION_MISSING",
                "Artwork settings have no valid schema version.",
            )
        })?;
    if version != u64::from(ARTWORK_SCHEMA_VERSION) {
        return Err(AppError::new(
            "ARTWORK_CONFIG_SCHEMA_UNSUPPORTED",
            format!("Artwork settings schema version {version} is not supported."),
        )
        .with_details(serde_json::json!({
            "supportedVersion": ARTWORK_SCHEMA_VERSION,
            "foundVersion": version,
        })));
    }

    let settings: ArtworkSettingsV1 = serde_json::from_value(value).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_SCHEMA_INVALID",
            "Artwork settings do not match the supported schema.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    validate_artwork_roots(&settings.roots)?;
    Ok(settings)
}

fn persist_artwork_settings(path: &Path, settings: &ArtworkSettingsV1) -> AppResult<()> {
    validate_artwork_roots(&settings.roots)?;

    match fs::metadata(path) {
        Ok(_) => {
            // Never silently replace a malformed or future-version document.
            load_artwork_settings(path)?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(AppError::new(
                "ARTWORK_CONFIG_READ_FAILED",
                "Artwork settings could not be inspected before persistence.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    }

    let mut encoded = serde_json::to_vec_pretty(settings).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_SERIALIZE_FAILED",
            "Artwork settings could not be serialized.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    encoded.push(b'\n');

    let parent = path.parent().ok_or_else(|| {
        AppError::new(
            "ARTWORK_CONFIG_PATH_INVALID",
            "Artwork settings path has no parent directory.",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_DIRECTORY_CREATE_FAILED",
            "Artwork settings directory could not be created.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    let mut temp = Builder::new()
        .prefix(".mame-tauri-artwork-")
        .tempfile_in(parent)
        .map_err(artwork_write_error)?;
    temp.write_all(&encoded).map_err(artwork_write_error)?;
    temp.as_file().sync_all().map_err(artwork_write_error)?;
    let persisted = temp
        .persist(path)
        .map_err(|error| artwork_write_error(error.error))?;
    persisted.sync_all().map_err(artwork_write_error)?;

    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(artwork_write_error)?;

    Ok(())
}

fn artwork_write_error(error: io::Error) -> AppError {
    AppError::new(
        "ARTWORK_CONFIG_ATOMIC_WRITE_FAILED",
        "Artwork settings could not be atomically replaced.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

fn validate_artwork_roots(roots: &[PlatformPath]) -> AppResult<()> {
    if roots.len() > MAX_ARTWORK_ROOTS {
        return Err(AppError::new(
            "ARTWORK_ROOT_LIMIT_EXCEEDED",
            format!("At most {MAX_ARTWORK_ROOTS} artwork roots may be configured."),
        ));
    }

    for (index, root) in roots.iter().enumerate() {
        let path = root.as_path();
        if !path.is_absolute() {
            return Err(AppError::new(
                "ARTWORK_ROOT_NOT_ABSOLUTE",
                "Artwork roots must be absolute paths.",
            )
            .with_details(serde_json::json!({ "rootIndex": index })));
        }
        if path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(AppError::new(
                "ARTWORK_ROOT_TRAVERSAL",
                "Artwork roots must not contain parent-directory traversal.",
            )
            .with_details(serde_json::json!({ "rootIndex": index })));
        }
        if roots[..index].iter().any(|candidate| candidate == root) {
            return Err(AppError::new(
                "ARTWORK_ROOT_DUPLICATE",
                "Artwork roots must not contain duplicate paths.",
            )
            .with_details(serde_json::json!({ "rootIndex": index })));
        }
    }

    Ok(())
}

fn validate_artwork_root(path: &PlatformPath) -> PathValidation {
    let filesystem_path = path.as_path();
    let metadata = match fs::metadata(filesystem_path) {
        Ok(metadata) => metadata,
        Err(error) => return artwork_validation_from_io_error(path, &error),
    };

    if !metadata.is_dir() {
        return PathValidation {
            path: path.clone(),
            status: PathValidationStatus::NotDirectory,
            message: Some("Configured artwork root is not a directory.".to_owned()),
        };
    }

    match fs::read_dir(filesystem_path) {
        Ok(_) => PathValidation {
            path: path.clone(),
            status: PathValidationStatus::Accessible,
            message: None,
        },
        Err(error) => artwork_validation_from_io_error(path, &error),
    }
}

fn artwork_validation_from_io_error(path: &PlatformPath, error: &io::Error) -> PathValidation {
    let status = match error.kind() {
        ErrorKind::NotFound => PathValidationStatus::Missing,
        ErrorKind::PermissionDenied => PathValidationStatus::PermissionDenied,
        _ => PathValidationStatus::Unreadable,
    };
    PathValidation {
        path: path.clone(),
        status,
        message: Some(error.to_string()),
    }
}

fn artwork_configuration(settings: ArtworkSettingsV1) -> ArtworkConfiguration {
    let validations = settings.roots.iter().map(validate_artwork_root).collect();
    ArtworkConfiguration {
        schema_version: ARTWORK_SCHEMA_VERSION,
        roots: settings.roots,
        validations,
    }
}

fn apply_artwork_roots(path: &Path, roots: Vec<PlatformPath>) -> AppResult<ArtworkConfiguration> {
    validate_artwork_roots(&roots)?;
    let settings = ArtworkSettingsV1 {
        schema_version: ARTWORK_SCHEMA_VERSION,
        roots,
    };
    persist_artwork_settings(path, &settings)?;
    Ok(artwork_configuration(settings))
}

#[tauri::command]
pub fn get_artwork_configuration(app: AppHandle) -> AppResult<ArtworkConfiguration> {
    Ok(artwork_configuration(load_artwork_settings(
        &artwork_settings_path(&app)?,
    )?))
}

#[tauri::command]
pub fn set_artwork_configuration(
    request: SetArtworkRootsRequest,
    app: AppHandle,
) -> AppResult<ArtworkConfiguration> {
    apply_artwork_roots(&artwork_settings_path(&app)?, request.roots)
}

#[tauri::command]
pub async fn pick_artwork_directory(app: AppHandle) -> AppResult<Option<PlatformPath>> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let selected = app
            .dialog()
            .file()
            .set_title("Choose local artwork root")
            .blocking_pick_folder();

        match selected {
            Some(path) => path
                .into_path()
                .map(PlatformPath::new)
                .map(Some)
                .map_err(|error| {
                    AppError::new(
                        "ARTWORK_DIRECTORY_INVALID",
                        "The selected artwork directory could not be represented as a platform path.",
                    )
                    .with_details(serde_json::json!({ "cause": error.to_string() }))
                }),
            None => Ok(None),
        }
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        Err(AppError::new(
            "ARTWORK_DIRECTORY_PICKER_UNAVAILABLE",
            "Artwork directory selection is not supported on this platform.",
        ))
    }
}

#[tauri::command]
pub fn discover_machine_artwork(machine: String, app: AppHandle) -> AppResult<MachineArtwork> {
    let settings = load_artwork_settings(&artwork_settings_path(&app)?)?;
    discover_machine_artwork_from_roots(&machine, &settings.roots)
}

fn discover_machine_artwork_from_roots(
    machine: &str,
    roots: &[PlatformPath],
) -> AppResult<MachineArtwork> {
    validate_short_identifier("machine", machine)?;
    validate_artwork_roots(roots)?;

    let slots = ArtworkKind::ALL
        .into_iter()
        .map(|kind| ArtworkSlot {
            kind,
            asset: find_local_artwork(machine, kind, roots),
        })
        .collect();

    Ok(MachineArtwork {
        schema_version: ARTWORK_SCHEMA_VERSION,
        machine: machine.to_owned(),
        slots,
    })
}

fn find_local_artwork(
    machine: &str,
    kind: ArtworkKind,
    roots: &[PlatformPath],
) -> Option<ArtworkDescriptor> {
    for (index, root) in roots.iter().enumerate() {
        let Ok(root_index) = u32::try_from(index) else {
            break;
        };
        let canonical_root = match fs::canonicalize(root.as_path()) {
            Ok(root) => root,
            Err(_) => continue,
        };

        for (extension, mime_type) in ARTWORK_EXTENSIONS {
            let candidate = root
                .as_path()
                .join(kind.directory_name())
                .join(format!("{machine}.{extension}"));
            let canonical_candidate = match fs::canonicalize(&candidate) {
                Ok(candidate) => candidate,
                Err(_) => continue,
            };

            if !canonical_candidate.starts_with(&canonical_root) {
                continue;
            }

            let metadata = match fs::metadata(&canonical_candidate) {
                Ok(metadata) if metadata.is_file() && metadata.len() <= MAX_ARTWORK_FILE_BYTES => {
                    metadata
                }
                _ => continue,
            };

            return Some(ArtworkDescriptor {
                schema_version: ARTWORK_SCHEMA_VERSION,
                asset_id: format!(
                    "local:{root_index}:{machine}:{}:{extension}",
                    kind.asset_token()
                ),
                machine: machine.to_owned(),
                kind,
                mime_type: mime_type.to_owned(),
                bytes: metadata.len(),
                provenance: ArtworkProvenance {
                    kind: ArtworkProvenanceKind::LocalFile,
                    root_index,
                },
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::{PathValidationStatus, PlatformPath};

    use super::{
        apply_artwork_roots, discover_machine_artwork_from_roots, ArtworkKind, ArtworkProvenance,
        ArtworkProvenanceKind,
    };

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-artwork-{label}-{nonce}"))
    }

    #[test]
    fn artwork_model_serializes_all_required_mt801_categories() {
        let kinds = [
            ArtworkKind::Screenshot,
            ArtworkKind::Cabinet,
            ArtworkKind::Marquee,
            ArtworkKind::Flyer,
            ArtworkKind::Icon,
            ArtworkKind::SystemImage,
        ];

        assert_eq!(
            serde_json::to_value(kinds).expect("serialize artwork kinds"),
            serde_json::json!([
                "screenshot",
                "cabinet",
                "marquee",
                "flyer",
                "icon",
                "systemImage"
            ])
        );
    }

    #[test]
    fn local_provenance_exposes_root_ordinal_without_a_host_path() {
        let provenance = ArtworkProvenance {
            kind: ArtworkProvenanceKind::LocalFile,
            root_index: 2,
        };

        assert_eq!(
            serde_json::to_value(provenance).expect("serialize artwork provenance"),
            serde_json::json!({ "kind": "localFile", "rootIndex": 2 })
        );
    }

    #[test]
    fn artwork_roots_persist_in_order_and_report_validation() {
        let root = temp_root("config");
        fs::create_dir_all(&root).expect("create temporary root");
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(&first).expect("create first artwork root");
        fs::create_dir_all(&second).expect("create second artwork root");
        let settings_path = root.join("artwork-settings.json");

        let configuration = apply_artwork_roots(
            &settings_path,
            vec![PlatformPath::new(&second), PlatformPath::new(&first)],
        )
        .expect("persist artwork roots");

        assert_eq!(configuration.roots[0].as_path(), second);
        assert_eq!(configuration.roots[1].as_path(), first);
        assert!(configuration
            .validations
            .iter()
            .all(|validation| validation.status == PathValidationStatus::Accessible));

        let persisted = fs::read_to_string(&settings_path).expect("read artwork settings");
        assert!(persisted.contains("\"schemaVersion\": 1"));
        assert!(persisted.contains("\"roots\""));
        fs::remove_dir_all(root).expect("remove temporary root");
    }

    #[test]
    fn discovery_prefers_first_root_and_returns_explicit_missing_slots() {
        let root = temp_root("discovery");
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(first.join("snap")).expect("create first snap directory");
        fs::create_dir_all(second.join("snap")).expect("create second snap directory");
        fs::create_dir_all(second.join("cabinets")).expect("create cabinet directory");
        fs::write(first.join("snap/pacman.png"), b"first").expect("write first screenshot");
        fs::write(second.join("snap/pacman.png"), b"second").expect("write second screenshot");
        fs::write(second.join("cabinets/pacman.webp"), b"cabinet").expect("write cabinet");

        let discovered = discover_machine_artwork_from_roots(
            "pacman",
            &[PlatformPath::new(&first), PlatformPath::new(&second)],
        )
        .expect("discover local artwork");

        assert_eq!(discovered.slots.len(), 6);
        let screenshot = discovered
            .slots
            .iter()
            .find(|slot| slot.kind == ArtworkKind::Screenshot)
            .and_then(|slot| slot.asset.as_ref())
            .expect("screenshot must be present");
        assert_eq!(screenshot.provenance.root_index, 0);
        assert_eq!(screenshot.bytes, 5);
        let cabinet = discovered
            .slots
            .iter()
            .find(|slot| slot.kind == ArtworkKind::Cabinet)
            .and_then(|slot| slot.asset.as_ref())
            .expect("cabinet must be present");
        assert_eq!(cabinet.provenance.root_index, 1);
        assert_eq!(cabinet.mime_type, "image/webp");
        assert!(discovered
            .slots
            .iter()
            .find(|slot| slot.kind == ArtworkKind::Flyer)
            .expect("flyer slot must exist")
            .asset
            .is_none());

        fs::remove_dir_all(root).expect("remove temporary root");
    }

    #[test]
    fn discovery_rejects_machine_path_syntax() {
        let error = discover_machine_artwork_from_roots("../pacman", &[])
            .expect_err("path syntax must be rejected");
        assert_eq!(error.code, "MAME_IDENTIFIER_INVALID");
    }

    #[cfg(unix)]
    #[test]
    fn discovery_rejects_category_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = temp_root("symlink");
        let artwork_root = root.join("artwork");
        let outside = root.join("outside");
        fs::create_dir_all(&artwork_root).expect("create artwork root");
        fs::create_dir_all(&outside).expect("create outside directory");
        fs::write(outside.join("pacman.png"), b"outside").expect("write outside artwork");
        symlink(&outside, artwork_root.join("snap")).expect("create category symlink");

        let discovered =
            discover_machine_artwork_from_roots("pacman", &[PlatformPath::new(&artwork_root)])
                .expect("discover local artwork");
        let screenshot = discovered
            .slots
            .iter()
            .find(|slot| slot.kind == ArtworkKind::Screenshot)
            .expect("screenshot slot must exist");
        assert!(screenshot.asset.is_none());

        fs::remove_dir_all(root).expect("remove temporary root");
    }
}
