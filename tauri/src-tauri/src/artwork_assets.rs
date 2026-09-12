//! Bounded, path-blind reads for local artwork selected by MT-802 discovery.
//!
//! The WebView never supplies or receives a host filesystem path. Asset IDs are
//! parsed as untrusted input and re-authorized against the current persisted
//! artwork-root configuration on every read.

use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    config::PlatformPath,
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
};

const ARTWORK_SETTINGS_SCHEMA_VERSION: u32 = 1;
const MAX_ARTWORK_ROOTS: usize = 32;
const MAX_ARTWORK_PREVIEW_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkAssetPayload {
    pub schema_version: u32,
    pub asset_id: String,
    pub mime_type: String,
    pub bytes: u64,
    pub data_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtworkSettingsWire {
    schema_version: u32,
    #[serde(default)]
    roots: Vec<PlatformPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedAssetId<'a> {
    root_index: usize,
    machine: &'a str,
    directory: &'static str,
    mime_type: &'static str,
    extension: &'static str,
}

#[tauri::command]
pub fn read_artwork_asset(asset_id: String, app: AppHandle) -> AppResult<ArtworkAssetPayload> {
    let roots = load_artwork_roots(&artwork_settings_path(&app)?)?;
    read_artwork_asset_from_roots(&asset_id, &roots)
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

fn load_artwork_roots(path: &Path) -> AppResult<Vec<PlatformPath>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(AppError::new(
                "ARTWORK_ASSET_UNAVAILABLE",
                "The requested artwork asset is no longer available.",
            ))
        }
        Err(error) => {
            return Err(AppError::new(
                "ARTWORK_CONFIG_READ_FAILED",
                "Artwork settings could not be read while authorizing the asset.",
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
    let settings: ArtworkSettingsWire = serde_json::from_value(value).map_err(|error| {
        AppError::new(
            "ARTWORK_CONFIG_SCHEMA_INVALID",
            "Artwork settings do not match the supported schema.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    if settings.schema_version != ARTWORK_SETTINGS_SCHEMA_VERSION {
        return Err(AppError::new(
            "ARTWORK_CONFIG_SCHEMA_UNSUPPORTED",
            "Artwork settings use an unsupported schema version.",
        )
        .with_details(serde_json::json!({
            "supportedVersion": ARTWORK_SETTINGS_SCHEMA_VERSION,
            "foundVersion": settings.schema_version,
        })));
    }
    if settings.roots.len() > MAX_ARTWORK_ROOTS {
        return Err(AppError::new(
            "ARTWORK_ROOT_LIMIT_EXCEEDED",
            format!("At most {MAX_ARTWORK_ROOTS} artwork roots may be configured."),
        ));
    }

    Ok(settings.roots)
}

fn read_artwork_asset_from_roots(
    asset_id: &str,
    roots: &[PlatformPath],
) -> AppResult<ArtworkAssetPayload> {
    let parsed = parse_asset_id(asset_id)?;
    let root = roots.get(parsed.root_index).ok_or_else(stale_asset_error)?;
    validate_root(root)?;

    let canonical_root = fs::canonicalize(root.as_path()).map_err(|_| stale_asset_error())?;
    let candidate = root
        .as_path()
        .join(parsed.directory)
        .join(format!("{}.{}", parsed.machine, parsed.extension));
    let canonical_candidate = fs::canonicalize(candidate).map_err(|_| stale_asset_error())?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(AppError::new(
            "ARTWORK_ASSET_OUTSIDE_ROOT",
            "The requested artwork asset resolves outside its configured root.",
        ));
    }

    let metadata = fs::metadata(&canonical_candidate).map_err(|_| stale_asset_error())?;
    if !metadata.is_file() {
        return Err(stale_asset_error());
    }
    if metadata.len() > MAX_ARTWORK_PREVIEW_BYTES {
        return Err(AppError::new(
            "ARTWORK_ASSET_TOO_LARGE",
            format!(
                "Artwork previews are limited to {MAX_ARTWORK_PREVIEW_BYTES} bytes per asset."
            ),
        )
        .with_details(serde_json::json!({
            "assetId": asset_id,
            "bytes": metadata.len(),
            "maxBytes": MAX_ARTWORK_PREVIEW_BYTES,
        })));
    }

    let bytes = fs::read(&canonical_candidate).map_err(|error| {
        AppError::new(
            "ARTWORK_ASSET_READ_FAILED",
            "The requested artwork asset could not be read.",
        )
        .with_details(serde_json::json!({
            "assetId": asset_id,
            "cause": error.to_string(),
        }))
    })?;
    let byte_count = u64::try_from(bytes.len()).map_err(|_| {
        AppError::new(
            "ARTWORK_ASSET_TOO_LARGE",
            "The requested artwork asset is too large to represent safely.",
        )
    })?;
    if byte_count > MAX_ARTWORK_PREVIEW_BYTES {
        return Err(AppError::new(
            "ARTWORK_ASSET_TOO_LARGE",
            "The requested artwork asset exceeded the preview limit while being read.",
        ));
    }

    let encoded = encode_base64(&bytes);
    Ok(ArtworkAssetPayload {
        schema_version: ARTWORK_SETTINGS_SCHEMA_VERSION,
        asset_id: asset_id.to_owned(),
        mime_type: parsed.mime_type.to_owned(),
        bytes: byte_count,
        data_url: format!("data:{};base64,{encoded}", parsed.mime_type),
    })
}

fn parse_asset_id(asset_id: &str) -> AppResult<ParsedAssetId<'_>> {
    let parts = asset_id.split(':').collect::<Vec<_>>();
    if parts.len() != 5 || parts[0] != "local" {
        return Err(invalid_asset_id());
    }

    let root_index = parts[1].parse::<usize>().map_err(|_| invalid_asset_id())?;
    let machine = parts[2];
    validate_short_identifier("machine", machine).map_err(|_| invalid_asset_id())?;

    let directory = match parts[3] {
        "screenshot" => "snap",
        "cabinet" => "cabinets",
        "marquee" => "marquees",
        "flyer" => "flyers",
        "icon" => "icons",
        "systemImage" => "systems",
        _ => return Err(invalid_asset_id()),
    };
    let (extension, mime_type) = match parts[4] {
        "png" => ("png", "image/png"),
        "jpg" => ("jpg", "image/jpeg"),
        "jpeg" => ("jpeg", "image/jpeg"),
        "webp" => ("webp", "image/webp"),
        _ => return Err(invalid_asset_id()),
    };

    Ok(ParsedAssetId {
        root_index,
        machine,
        directory,
        mime_type,
        extension,
    })
}

fn validate_root(root: &PlatformPath) -> AppResult<()> {
    let path = root.as_path();
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AppError::new(
            "ARTWORK_ROOT_INVALID",
            "The configured artwork root is not safe for asset reads.",
        ));
    }
    Ok(())
}

fn invalid_asset_id() -> AppError {
    AppError::new(
        "ARTWORK_ASSET_ID_INVALID",
        "The requested artwork asset identifier is invalid.",
    )
}

fn stale_asset_error() -> AppError {
    AppError::new(
        "ARTWORK_ASSET_UNAVAILABLE",
        "The requested artwork asset is no longer available under the current artwork configuration.",
    )
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b11) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b11_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::PlatformPath;

    use super::{encode_base64, parse_asset_id, read_artwork_asset_from_roots};

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-artwork-preview-{label}-{nonce}"))
    }

    #[test]
    fn base64_encoder_handles_padding_boundaries() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
    }

    #[test]
    fn asset_id_parser_rejects_path_and_extension_injection() {
        assert_eq!(
            parse_asset_id("local:0:../pacman:screenshot:png")
                .expect_err("path syntax must fail")
                .code,
            "ARTWORK_ASSET_ID_INVALID"
        );
        assert_eq!(
            parse_asset_id("local:0:pacman:screenshot:svg")
                .expect_err("unsupported extension must fail")
                .code,
            "ARTWORK_ASSET_ID_INVALID"
        );
        assert_eq!(
            parse_asset_id("local:0:pacman:screenshot:png:extra")
                .expect_err("extra components must fail")
                .code,
            "ARTWORK_ASSET_ID_INVALID"
        );
    }

    #[test]
    fn asset_read_reauthorizes_and_returns_bounded_data_url() {
        let root = temp_root("read");
        fs::create_dir_all(root.join("snap")).expect("create snap directory");
        fs::write(root.join("snap/pacman.png"), b"foo").expect("write preview");

        let payload = read_artwork_asset_from_roots(
            "local:0:pacman:screenshot:png",
            &[PlatformPath::new(&root)],
        )
        .expect("read artwork asset");

        assert_eq!(payload.mime_type, "image/png");
        assert_eq!(payload.bytes, 3);
        assert_eq!(payload.data_url, "data:image/png;base64,Zm9v");
        fs::remove_dir_all(root).expect("remove temporary root");
    }

    #[test]
    fn stale_root_index_is_rejected_without_path_disclosure() {
        let error = read_artwork_asset_from_roots("local:4:pacman:screenshot:png", &[])
            .expect_err("stale root index must fail");
        assert_eq!(error.code, "ARTWORK_ASSET_UNAVAILABLE");
        assert!(!error.message.contains('/'));
        assert!(!error.message.contains('\\'));
    }

    #[cfg(unix)]
    #[test]
    fn asset_read_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = temp_root("escape");
        let artwork_root = root.join("artwork");
        let outside = root.join("outside");
        fs::create_dir_all(&artwork_root).expect("create artwork root");
        fs::create_dir_all(&outside).expect("create outside root");
        fs::write(outside.join("pacman.png"), b"foo").expect("write outside preview");
        symlink(&outside, artwork_root.join("snap")).expect("create snap symlink");

        let error = read_artwork_asset_from_roots(
            "local:0:pacman:screenshot:png",
            &[PlatformPath::new(&artwork_root)],
        )
        .expect_err("symlink escape must fail");
        assert_eq!(error.code, "ARTWORK_ASSET_OUTSIDE_ROOT");
        fs::remove_dir_all(root).expect("remove temporary root");
    }
}
