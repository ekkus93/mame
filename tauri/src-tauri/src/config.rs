use std::{
    ffi::OsString,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use tauri::{Manager, Runtime};

use crate::errors::{AppError, AppResult};

pub const SETTINGS_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformPath(PathBuf);

impl PlatformPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl From<PathBuf> for PlatformPath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&Path> for PlatformPath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncodedPathRef<'a> {
    encoding: &'static str,
    data: &'a str,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlatformPathWire {
    Utf8(String),
    Encoded { encoding: String, data: String },
}

impl Serialize for PlatformPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(path) = self.0.to_str() {
            return serializer.serialize_str(path);
        }

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;

            let data = encode_hex(self.0.as_os_str().as_bytes());
            EncodedPathRef {
                encoding: "unixBytesHex",
                data: &data,
            }
            .serialize(serializer)
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            let bytes = self
                .0
                .as_os_str()
                .encode_wide()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>();
            let data = encode_hex(&bytes);
            EncodedPathRef {
                encoding: "windowsWideHex",
                data: &data,
            }
            .serialize(serializer)
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err(serde::ser::Error::custom(
                "non-UTF-8 paths are unsupported on this platform",
            ))
        }
    }
}

impl<'de> Deserialize<'de> for PlatformPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match PlatformPathWire::deserialize(deserializer)? {
            PlatformPathWire::Utf8(path) => Ok(Self(PathBuf::from(path))),
            PlatformPathWire::Encoded { encoding, data } => {
                let bytes = decode_hex(&data).map_err(de::Error::custom)?;

                #[cfg(unix)]
                if encoding == "unixBytesHex" {
                    use std::os::unix::ffi::OsStringExt;
                    return Ok(Self(PathBuf::from(OsString::from_vec(bytes))));
                }

                #[cfg(windows)]
                if encoding == "windowsWideHex" {
                    use std::os::windows::ffi::OsStringExt;

                    let (wide_bytes, remainder) = bytes.as_slice().as_chunks::<2>();
                    if !remainder.is_empty() {
                        return Err(de::Error::custom(
                            "windowsWideHex path data must contain complete UTF-16 code units",
                        ));
                    }
                    let wide = wide_bytes
                        .iter()
                        .map(|chunk| u16::from_le_bytes(*chunk))
                        .collect::<Vec<_>>();
                    return Ok(Self(PathBuf::from(OsString::from_wide(&wide))));
                }

                Err(de::Error::custom(format!(
                    "path encoding {encoding:?} is not supported on this platform"
                )))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentPathsV1 {
    #[serde(default)]
    pub rom_paths: Vec<PlatformPath>,
    #[serde(default)]
    pub software_paths: Vec<PlatformPath>,
    #[serde(default)]
    pub chd_paths: Vec<PlatformPath>,
}

impl ContentPathsV1 {
    pub fn paths(&self, kind: ContentPathKind) -> &[PlatformPath] {
        match kind {
            ContentPathKind::Rom => &self.rom_paths,
            ContentPathKind::Software => &self.software_paths,
            ContentPathKind::Chd => &self.chd_paths,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContentPathKind {
    Rom,
    Software,
    Chd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsV2 {
    pub schema_version: u32,
    pub mame_executable: Option<String>,
    #[serde(default)]
    pub content_paths: ContentPathsV1,
}

impl Default for SettingsV2 {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            mame_executable: None,
            content_paths: ContentPathsV1::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsV1Wire {
    mame_executable: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PathValidationStatus {
    Accessible,
    Missing,
    NotDirectory,
    PermissionDenied,
    Unreadable,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PathValidation {
    pub path: PlatformPath,
    pub status: PathValidationStatus,
    pub message: Option<String>,
}

pub fn validate_content_path(path: &PlatformPath) -> PathValidation {
    let filesystem_path = path.as_path();
    let metadata = match fs::metadata(filesystem_path) {
        Ok(metadata) => metadata,
        Err(error) => return validation_from_io_error(path, &error),
    };

    if !metadata.is_dir() {
        return PathValidation {
            path: path.clone(),
            status: PathValidationStatus::NotDirectory,
            message: Some("Configured content path is not a directory.".to_owned()),
        };
    }

    match fs::read_dir(filesystem_path) {
        Ok(_) => PathValidation {
            path: path.clone(),
            status: PathValidationStatus::Accessible,
            message: None,
        },
        Err(error) => validation_from_io_error(path, &error),
    }
}

fn validation_from_io_error(path: &PlatformPath, error: &io::Error) -> PathValidation {
    PathValidation {
        path: path.clone(),
        status: path_error_status(error),
        message: Some(error.to_string()),
    }
}

fn path_error_status(error: &io::Error) -> PathValidationStatus {
    match error.kind() {
        ErrorKind::NotFound => PathValidationStatus::Missing,
        ErrorKind::PermissionDenied => PathValidationStatus::PermissionDenied,
        _ => PathValidationStatus::Unreadable,
    }
}

pub fn settings_path<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<PathBuf> {
    app.path()
        .app_config_dir()
        .map(|root| root.join("settings.json"))
        .map_err(|error| {
            AppError::new(
                "CONFIG_ROOT_UNAVAILABLE",
                "The platform application configuration directory is unavailable.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })
}

pub fn load_settings(path: &Path) -> AppResult<SettingsV2> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(SettingsV2::default()),
        Err(error) => {
            return Err(AppError::new(
                "CONFIG_READ_FAILED",
                "The application settings file could not be read.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() })))
        }
    };

    parse_settings_json(&contents)
}

pub fn parse_settings_json(contents: &str) -> AppResult<SettingsV2> {
    let value: Value = serde_json::from_str(contents).map_err(|error| {
        AppError::new(
            "CONFIG_INVALID_JSON",
            "The application settings file is not valid JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    migrate_settings(value)
}

fn migrate_settings(value: Value) -> AppResult<SettingsV2> {
    let version = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::new(
                "CONFIG_SCHEMA_VERSION_MISSING",
                "The application settings file has no valid schema version.",
            )
        })?;

    match version {
        1 => {
            let legacy: SettingsV1Wire = serde_json::from_value(value).map_err(|error| {
                AppError::new(
                    "CONFIG_SCHEMA_INVALID",
                    "The application settings file does not match schema version 1.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() }))
            })?;
            Ok(SettingsV2 {
                schema_version: SETTINGS_SCHEMA_VERSION,
                mame_executable: legacy.mame_executable,
                content_paths: ContentPathsV1::default(),
            })
        }
        2 => serde_json::from_value(value).map_err(|error| {
            AppError::new(
                "CONFIG_SCHEMA_INVALID",
                "The application settings file does not match schema version 2.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        }),
        unsupported => Err(AppError::new(
            "CONFIG_SCHEMA_UNSUPPORTED",
            format!("Settings schema version {unsupported} is not supported."),
        )
        .with_details(serde_json::json!({
            "supportedVersion": SETTINGS_SCHEMA_VERSION,
            "foundVersion": unsupported
        }))),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn decode_hex(encoded: &str) -> Result<Vec<u8>, &'static str> {
    let (pairs, remainder) = encoded.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err("encoded path data must contain complete bytes");
    }

    pairs
        .iter()
        .map(|pair| {
            let high = decode_hex_digit(pair[0])?;
            let low = decode_hex_digit(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_hex_digit(value: u8) -> Result<u8, &'static str> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("encoded path data contains a non-hexadecimal character"),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{self, ErrorKind},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        load_settings, parse_settings_json, path_error_status, validate_content_path,
        ContentPathKind, PathValidationStatus, PlatformPath, SettingsV2, SETTINGS_SCHEMA_VERSION,
    };

    #[test]
    fn missing_settings_use_safe_defaults() {
        let path = PathBuf::from("this-file-must-not-exist-mame-tauri-settings.json");
        assert_eq!(
            load_settings(&path).expect("missing config uses defaults"),
            SettingsV2::default()
        );
    }

    #[test]
    fn parses_current_schema_and_preserves_path_order() {
        let settings = parse_settings_json(
            r#"{
                "schemaVersion":2,
                "mameExecutable":"/opt/mame/mame",
                "contentPaths":{
                    "romPaths":["/games/roms-primary","/games/roms-fallback"],
                    "softwarePaths":["/games/software"],
                    "chdPaths":["/games/chd"]
                }
            }"#,
        )
        .expect("schema v2 must parse");

        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.mame_executable.as_deref(), Some("/opt/mame/mame"));
        let rom_paths = settings.content_paths.paths(ContentPathKind::Rom);
        assert_eq!(rom_paths.len(), 2);
        assert_eq!(rom_paths[0].as_path(), PathBuf::from("/games/roms-primary"));
        assert_eq!(
            rom_paths[1].as_path(),
            PathBuf::from("/games/roms-fallback")
        );
        assert_eq!(
            settings
                .content_paths
                .paths(ContentPathKind::Software)
                .len(),
            1
        );
        assert_eq!(settings.content_paths.paths(ContentPathKind::Chd).len(), 1);
    }

    #[test]
    fn migrates_schema_v1_without_guessing_content_paths() {
        let settings =
            parse_settings_json(r#"{"schemaVersion":1,"mameExecutable":"/opt/mame/mame"}"#)
                .expect("schema v1 must migrate");

        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.mame_executable.as_deref(), Some("/opt/mame/mame"));
        assert!(settings.content_paths.rom_paths.is_empty());
        assert!(settings.content_paths.software_paths.is_empty());
        assert!(settings.content_paths.chd_paths.is_empty());
    }

    #[test]
    fn platform_paths_round_trip_through_json() {
        let original = PlatformPath::new(PathBuf::from("relative").join("roms").join("set"));
        let json = serde_json::to_string(&original).expect("serialize platform path");
        let decoded: PlatformPath = serde_json::from_str(&json).expect("deserialize platform path");
        assert_eq!(decoded, original);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_unix_paths_round_trip_losslessly() {
        use std::os::unix::ffi::OsStringExt;

        let original = PlatformPath::new(PathBuf::from(std::ffi::OsString::from_vec(vec![
            b'r', b'o', b'm', 0xff,
        ])));
        let json = serde_json::to_string(&original).expect("serialize non-UTF-8 path");
        assert!(json.contains("unixBytesHex"));
        let decoded: PlatformPath =
            serde_json::from_str(&json).expect("deserialize non-UTF-8 path");
        assert_eq!(decoded, original);
    }

    #[test]
    fn validation_distinguishes_accessible_missing_and_non_directory_paths() {
        let temp_root = unique_temp_path();
        fs::create_dir_all(&temp_root).expect("create temporary content directory");
        let regular_file = temp_root.join("not-a-directory");
        fs::write(&regular_file, b"not content").expect("create temporary regular file");
        let missing = temp_root.join("missing");

        let accessible = validate_content_path(&PlatformPath::new(&temp_root));
        let non_directory = validate_content_path(&PlatformPath::new(&regular_file));
        let absent = validate_content_path(&PlatformPath::new(&missing));

        assert_eq!(accessible.status, PathValidationStatus::Accessible);
        assert_eq!(non_directory.status, PathValidationStatus::NotDirectory);
        assert_eq!(absent.status, PathValidationStatus::Missing);

        fs::remove_dir_all(&temp_root).expect("remove temporary content directory");
    }

    #[test]
    fn permission_errors_have_a_stable_validation_status() {
        let error = io::Error::from(ErrorKind::PermissionDenied);
        assert_eq!(
            path_error_status(&error),
            PathValidationStatus::PermissionDenied
        );
    }

    #[test]
    fn corrupt_json_is_not_silently_replaced() {
        let error = parse_settings_json("{broken").expect_err("corrupt config must fail");
        assert_eq!(error.code, "CONFIG_INVALID_JSON");
    }

    #[test]
    fn future_schema_is_rejected_explicitly() {
        let error = parse_settings_json(r#"{"schemaVersion":999}"#)
            .expect_err("future schema must not be guessed");
        assert_eq!(error.code, "CONFIG_SCHEMA_UNSUPPORTED");
    }

    fn unique_temp_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-mt501-{}-{nonce}", std::process::id()))
    }
}
