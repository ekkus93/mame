//! Local-only, bounded artwork discovery for the Tauri frontend.
//!
//! Artwork roots are application-owned configuration and are deliberately
//! separate from MAME content/search paths. The WebView never receives broad
//! filesystem access: callers supply only a validated machine short name, and
//! Rust maps that identifier onto fixed artwork categories and extensions.

use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tempfile::Builder;

use crate::{
    config::{settings_path, validate_content_path, PathValidation, PathValidationStatus, PlatformPath},
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
};

const ARTWORK_SCHEMA_VERSION: u32 = 1;
const MAX_ARTWORK_ROOTS: usize = 16;
const MAX_ARTWORK_ITEM_BYTES: u64 = 1024 * 1024;
const MAX_ARTWORK_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const ARTWORK_EXTENSIONS: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
];
const ARTWORK_LAYOUTS: &[(ArtworkKind, &str)] = &[
    (ArtworkKind::Screenshot, "snap"),
    (ArtworkKind::Cabinet, "cabinets"),
    (ArtworkKind::Marquee, "marquees"),
    (ArtworkKind::Flyer, "flyers"),
    (ArtworkKind::Icon, "icons"),
    (ArtworkKind::SystemImage, "systems"),
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkKind {
    Screenshot,
    Cabinet,
    Marquee,
    Flyer,
    Icon,
    SystemImage,
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineArtworkRequest {
    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkItem {
    pub kind: ArtworkKind,
    pub source: &'static str,
    pub root_index: usize,
    pub relative_path: String,
    pub mime_type: &'static str,
    pub bytes: u64,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkWarning {
    pub code: &'static str,
    pub message: String,
    pub kind: Option<ArtworkKind>,
    pub root_index: usize,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineArtworkResult {
    pub schema_version: u32,
    pub short_name: String,
    pub items: Vec<ArtworkItem>,
    pub warnings: Vec<ArtworkWarning>,
}

#[tauri::command]
pub fn get_artwork_configuration(app: AppHandle) -> AppResult<ArtworkConfiguration> {
    let settings = load_artwork_settings(&artwork_settings_path(&app)?)?;
    Ok(configuration_for(settings))
}

#[tauri::command]
pub fn set_artwork_configuration(
    request: SetArtworkRootsRequest,
    app: AppHandle,
) -> AppResult<ArtworkConfiguration> {
    validate_root_count(&request.roots)?;
    let settings = ArtworkSettingsV1 {
        schema_version: ARTWORK_SCHEMA_VERSION,
        roots: request.roots,
    };
    persist_artwork_settings(&artwork_settings_path(&app)?, &settings)?;
    Ok(configuration_for(settings))
}

#[tauri::command]
pub async fn get_machine_artwork(
    request: MachineArtworkRequest,
    app: AppHandle,
) -> AppResult<MachineArtworkResult> {
    validate_short_identifier("machine", &request.short_name)?;
    let path = artwork_settings_path(&app)?;
    let short_name = request.short_name;
    tauri::async_runtime::spawn_blocking(move || {
        let settings = load_artwork_settings(&path)?;
        discover_machine_artwork(&settings, &short_name)
    })
    .await
    .map_err(|error| {
        AppError::new(
            "ARTWORK_DISCOVERY_TASK_FAILED",
            "Local artwork discovery could not complete.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?
}

fn artwork_settings_path(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(settings_path(app)?.with_file_name("artwork.json"))
}

fn configuration_for(settings: ArtworkSettingsV1) -> ArtworkConfiguration {
    let validations = settings.roots.iter().map(validate_content_path).collect();
    ArtworkConfiguration {
        schema_version: ARTWORK_SCHEMA_VERSION,
        roots: settings.roots,
        validations,
    }
}

fn validate_root_count(roots: &[PlatformPath]) -> AppResult<()> {
    if roots.len() > MAX_ARTWORK_ROOTS {
        return Err(AppError::new(
            "ARTWORK_ROOT_LIMIT_EXCEEDED",
            "Too many local artwork roots are configured.",
        )
        .with_details(serde_json::json!({
            "maximum": MAX_ARTWORK_ROOTS,
            "provided": roots.len()
        })));
    }
    Ok(())
}

fn load_artwork_settings(path: &Path) -> AppResult<ArtworkSettingsV1> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(ArtworkSettingsV1::default()),
        Err(error) => {
            return Err(AppError::new(
                "ARTWORK_CONFIG_READ_FAILED",
                "The local artwork configuration could not be read.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    };

    let settings: ArtworkSettingsV1 = serde_json::from_slice(&bytes).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_INVALID",
            "The local artwork configuration is not valid schema-v1 JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    if settings.schema_version != ARTWORK_SCHEMA_VERSION {
        return Err(AppError::new(
            "ARTWORK_CONFIG_SCHEMA_UNSUPPORTED",
            "The local artwork configuration schema is not supported.",
        )
        .with_details(serde_json::json!({
            "supportedVersion": ARTWORK_SCHEMA_VERSION,
            "foundVersion": settings.schema_version
        })));
    }
    validate_root_count(&settings.roots)?;
    Ok(settings)
}

fn persist_artwork_settings(path: &Path, settings: &ArtworkSettingsV1) -> AppResult<()> {
    validate_root_count(&settings.roots)?;
    let mut encoded = serde_json::to_vec_pretty(settings).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_SERIALIZE_FAILED",
            "The local artwork configuration could not be serialized.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;
    encoded.push(b'\n');

    let parent = path.parent().ok_or_else(|| {
        AppError::new(
            "ARTWORK_CONFIG_PATH_INVALID",
            "The local artwork configuration path has no parent directory.",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| artwork_write_error(path, error))?;
    let mut temp = Builder::new()
        .prefix(".mame-tauri-artwork-")
        .tempfile_in(parent)
        .map_err(|error| artwork_write_error(path, error))?;
    temp.write_all(&encoded)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|error| artwork_write_error(path, error))?;
    let persisted = temp
        .persist(path)
        .map_err(|error| artwork_write_error(path, error.error))?;
    persisted
        .sync_all()
        .map_err(|error| artwork_write_error(path, error))?;
    sync_parent(parent).map_err(|error| artwork_write_error(path, error))?;
    Ok(())
}

fn artwork_write_error(path: &Path, error: io::Error) -> AppError {
    AppError::new(
        "ARTWORK_CONFIG_WRITE_FAILED",
        "The local artwork configuration could not be atomically persisted.",
    )
    .with_details(serde_json::json!({
        "path": path,
        "cause": error.to_string()
    }))
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> io::Result<()> {
    Ok(())
}

fn discover_machine_artwork(
    settings: &ArtworkSettingsV1,
    short_name: &str,
) -> AppResult<MachineArtworkResult> {
    validate_short_identifier("machine", short_name)?;
    let mut warnings = Vec::new();
    let mut roots = Vec::new();

    for (root_index, root) in settings.roots.iter().enumerate() {
        let validation = validate_content_path(root);
        if validation.status != PathValidationStatus::Accessible {
            warnings.push(ArtworkWarning {
                code: "ARTWORK_ROOT_UNAVAILABLE",
                message: validation
                    .message
                    .unwrap_or_else(|| "Configured artwork root is unavailable.".to_owned()),
                kind: None,
                root_index,
                relative_path: None,
            });
            continue;
        }
        match fs::canonicalize(root.as_path()) {
            Ok(canonical) => roots.push((root_index, canonical)),
            Err(error) => warnings.push(ArtworkWarning {
                code: "ARTWORK_ROOT_CANONICALIZE_FAILED",
                message: error.to_string(),
                kind: None,
                root_index,
                relative_path: None,
            }),
        }
    }

    let mut items = Vec::new();
    let mut response_bytes = 0_u64;
    for &(kind, directory) in ARTWORK_LAYOUTS {
        if response_bytes >= MAX_ARTWORK_RESPONSE_BYTES {
            break;
        }
        'root_search: for (root_index, root) in &roots {
            for &(extension, mime_type) in ARTWORK_EXTENSIONS {
                let relative_path = format!("{directory}/{short_name}.{extension}");
                let candidate = root.join(&relative_path);
                if !candidate.exists() {
                    continue;
                }

                let canonical = match fs::canonicalize(&candidate) {
                    Ok(path) => path,
                    Err(error) => {
                        warnings.push(candidate_warning(
                            "ARTWORK_FILE_UNAVAILABLE",
                            error.to_string(),
                            kind,
                            *root_index,
                            relative_path,
                        ));
                        continue;
                    }
                };
                if !canonical.starts_with(root) {
                    warnings.push(candidate_warning(
                        "ARTWORK_PATH_ESCAPE",
                        "Artwork candidate resolves outside its configured root.".to_owned(),
                        kind,
                        *root_index,
                        relative_path,
                    ));
                    continue;
                }

                let metadata = match fs::metadata(&canonical) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        warnings.push(candidate_warning(
                            "ARTWORK_FILE_UNAVAILABLE",
                            error.to_string(),
                            kind,
                            *root_index,
                            relative_path,
                        ));
                        continue;
                    }
                };
                if !metadata.is_file() {
                    warnings.push(candidate_warning(
                        "ARTWORK_NOT_REGULAR_FILE",
                        "Artwork candidate is not a regular file.".to_owned(),
                        kind,
                        *root_index,
                        relative_path,
                    ));
                    continue;
                }
                if metadata.len() > MAX_ARTWORK_ITEM_BYTES {
                    warnings.push(candidate_warning(
                        "ARTWORK_ITEM_TOO_LARGE",
                        format!(
                            "Artwork candidate is {} bytes; the per-item limit is {} bytes.",
                            metadata.len(),
                            MAX_ARTWORK_ITEM_BYTES
                        ),
                        kind,
                        *root_index,
                        relative_path,
                    ));
                    continue;
                }
                if response_bytes.saturating_add(metadata.len()) > MAX_ARTWORK_RESPONSE_BYTES {
                    warnings.push(candidate_warning(
                        "ARTWORK_RESPONSE_LIMIT_REACHED",
                        "Artwork response byte budget was reached before this item could be included."
                            .to_owned(),
                        kind,
                        *root_index,
                        relative_path,
                    ));
                    continue;
                }

                let bytes = match fs::read(&canonical) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        warnings.push(candidate_warning(
                            "ARTWORK_FILE_READ_FAILED",
                            error.to_string(),
                            kind,
                            *root_index,
                            relative_path,
                        ));
                        continue;
                    }
                };
                let byte_count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
                if byte_count > MAX_ARTWORK_ITEM_BYTES
                    || response_bytes.saturating_add(byte_count) > MAX_ARTWORK_RESPONSE_BYTES
                {
                    warnings.push(candidate_warning(
                        "ARTWORK_FILE_GREW_DURING_READ",
                        "Artwork candidate exceeded a configured byte limit while it was being read."
                            .to_owned(),
                        kind,
                        *root_index,
                        relative_path,
                    ));
                    continue;
                }

                response_bytes += byte_count;
                items.push(ArtworkItem {
                    kind,
                    source: "local",
                    root_index: *root_index,
                    relative_path,
                    mime_type,
                    bytes: byte_count,
                    data_base64: base64_encode(&bytes),
                });
                break 'root_search;
            }
        }
    }

    Ok(MachineArtworkResult {
        schema_version: 1,
        short_name: short_name.to_owned(),
        items,
        warnings,
    })
}

fn candidate_warning(
    code: &'static str,
    message: String,
    kind: ArtworkKind,
    root_index: usize,
    relative_path: String,
) -> ArtworkWarning {
    ArtworkWarning {
        code,
        message,
        kind: Some(kind),
        root_index,
        relative_path: Some(relative_path),
    }
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    let (chunks, remainder) = input.as_chunks::<3>();
    for chunk in chunks {
        let value = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        output.push(TABLE[(value & 0x3f) as usize] as char);
    }
    match remainder {
        [a] => {
            let value = u32::from(*a) << 16;
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            output.push('=');
            output.push('=');
        }
        [a, b] => {
            let value = (u32::from(*a) << 16) | (u32::from(*b) << 8);
            output.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            output.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
            output.push('=');
        }
        [] => {}
        _ => unreachable!("as_chunks remainder is shorter than three bytes"),
    }
    output
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        base64_encode, discover_machine_artwork, load_artwork_settings, persist_artwork_settings,
        ArtworkKind, ArtworkSettingsV1, PlatformPath, ARTWORK_SCHEMA_VERSION,
    };

    #[test]
    fn base64_encoder_is_standard_and_padded() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn missing_artwork_config_uses_empty_local_only_defaults() {
        let root = temp_root("missing-config");
        let settings = load_artwork_settings(&root.join("artwork.json")).expect("load defaults");
        assert_eq!(settings.schema_version, ARTWORK_SCHEMA_VERSION);
        assert!(settings.roots.is_empty());
    }

    #[test]
    fn artwork_config_round_trips_order_atomically() {
        let root = temp_root("config-round-trip");
        fs::create_dir_all(&root).expect("create temp root");
        let path = root.join("artwork.json");
        let settings = ArtworkSettingsV1 {
            schema_version: ARTWORK_SCHEMA_VERSION,
            roots: vec![PlatformPath::new("/art/primary"), PlatformPath::new("/art/fallback")],
        };
        persist_artwork_settings(&path, &settings).expect("persist artwork config");
        assert_eq!(load_artwork_settings(&path).expect("reload artwork config"), settings);
        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn discovery_returns_one_bounded_local_item_per_kind_with_relative_provenance() {
        let root = temp_root("discover");
        let snap = root.join("snap");
        fs::create_dir_all(&snap).expect("create snap directory");
        fs::write(snap.join("pacman.png"), b"fake-png").expect("write fixture artwork");
        let settings = ArtworkSettingsV1 {
            schema_version: ARTWORK_SCHEMA_VERSION,
            roots: vec![PlatformPath::new(root.clone())],
        };

        let result = discover_machine_artwork(&settings, "pacman").expect("discover artwork");
        assert_eq!(result.items.len(), 1);
        let item = &result.items[0];
        assert_eq!(item.kind, ArtworkKind::Screenshot);
        assert_eq!(item.source, "local");
        assert_eq!(item.root_index, 0);
        assert_eq!(item.relative_path, "snap/pacman.png");
        assert_eq!(item.mime_type, "image/png");
        assert_eq!(item.data_base64, base64_encode(b"fake-png"));
        assert!(result.warnings.is_empty());

        fs::remove_dir_all(root).expect("remove temp root");
    }

    #[cfg(unix)]
    #[test]
    fn discovery_rejects_symlink_escape_from_configured_root() {
        use std::os::unix::fs::symlink;

        let root = temp_root("symlink-root");
        let outside = temp_root("symlink-outside");
        fs::create_dir_all(root.join("snap")).expect("create snap directory");
        fs::create_dir_all(&outside).expect("create outside directory");
        let outside_file = outside.join("pacman.png");
        fs::write(&outside_file, b"outside").expect("write outside artwork");
        symlink(&outside_file, root.join("snap/pacman.png")).expect("create escaping symlink");
        let settings = ArtworkSettingsV1 {
            schema_version: ARTWORK_SCHEMA_VERSION,
            roots: vec![PlatformPath::new(root.clone())],
        };

        let result = discover_machine_artwork(&settings, "pacman").expect("discover artwork");
        assert!(result.items.is_empty());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.code == "ARTWORK_PATH_ESCAPE"));

        fs::remove_dir_all(root).expect("remove root");
        fs::remove_dir_all(outside).expect("remove outside root");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-artwork-{label}-{nonce}"))
    }
}
