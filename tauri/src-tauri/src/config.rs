use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Manager, Runtime};

use crate::errors::{AppError, AppResult};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsV1 {
    pub schema_version: u32,
    pub mame_executable: Option<String>,
}

impl Default for SettingsV1 {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            mame_executable: None,
        }
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

pub fn load_settings(path: &Path) -> AppResult<SettingsV1> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(SettingsV1::default()),
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

pub fn parse_settings_json(contents: &str) -> AppResult<SettingsV1> {
    let value: Value = serde_json::from_str(contents).map_err(|error| {
        AppError::new(
            "CONFIG_INVALID_JSON",
            "The application settings file is not valid JSON.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    migrate_settings(value)
}

fn migrate_settings(value: Value) -> AppResult<SettingsV1> {
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
        1 => serde_json::from_value(value).map_err(|error| {
            AppError::new(
                "CONFIG_SCHEMA_INVALID",
                "The application settings file does not match schema version 1.",
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{load_settings, parse_settings_json, SettingsV1, SETTINGS_SCHEMA_VERSION};

    #[test]
    fn missing_settings_use_safe_defaults() {
        let path = PathBuf::from("this-file-must-not-exist-mame-tauri-settings.json");
        assert_eq!(
            load_settings(&path).expect("missing config uses defaults"),
            SettingsV1::default()
        );
    }

    #[test]
    fn parses_current_schema() {
        let settings =
            parse_settings_json(r#"{"schemaVersion":1,"mameExecutable":"/opt/mame/mame"}"#)
                .expect("schema v1 must parse");

        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.mame_executable.as_deref(), Some("/opt/mame/mame"));
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
}
