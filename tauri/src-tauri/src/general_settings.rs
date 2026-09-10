use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::DialogExt;

use crate::{
    config::{load_settings, settings_path, LaunchPreferencesV1, SettingsV2},
    config_persistence::persist_settings,
    errors::{AppError, AppResult},
    mame::{inspect_executable, MameExecutableSource},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub schema_version: u32,
    pub mame_executable: Option<String>,
    pub launch_preferences: LaunchPreferencesV1,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMameExecutableRequest {
    pub mame_executable: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetLaunchPreferencesRequest {
    pub launch_preferences: LaunchPreferencesV1,
}

impl From<SettingsV2> for GeneralSettings {
    fn from(settings: SettingsV2) -> Self {
        Self {
            schema_version: 1,
            mame_executable: settings.mame_executable,
            launch_preferences: settings.launch_preferences,
        }
    }
}

fn load_general_settings(path: &Path) -> AppResult<GeneralSettings> {
    load_settings(path).map(GeneralSettings::from)
}

fn apply_launch_preferences(
    path: &Path,
    launch_preferences: LaunchPreferencesV1,
) -> AppResult<GeneralSettings> {
    let mut settings = load_settings(path)?;
    settings.launch_preferences = launch_preferences;
    persist_settings(path, &settings)?;
    Ok(GeneralSettings::from(settings))
}

fn apply_mame_executable(
    path: &Path,
    mame_executable: Option<String>,
) -> AppResult<GeneralSettings> {
    let mame_executable = mame_executable.filter(|value| !value.is_empty());
    if let Some(executable) = mame_executable.as_deref() {
        inspect_executable(MameExecutableSource::external(executable))?;
    }

    let mut settings = load_settings(path)?;
    settings.mame_executable = mame_executable;
    persist_settings(path, &settings)?;
    Ok(GeneralSettings::from(settings))
}

#[tauri::command]
pub fn get_general_settings(app: AppHandle) -> AppResult<GeneralSettings> {
    load_general_settings(&settings_path(&app)?)
}

#[tauri::command]
pub fn set_general_mame_executable(
    request: SetMameExecutableRequest,
    app: AppHandle,
) -> AppResult<GeneralSettings> {
    apply_mame_executable(&settings_path(&app)?, request.mame_executable)
}

#[tauri::command]
pub fn set_general_launch_preferences(
    request: SetLaunchPreferencesRequest,
    app: AppHandle,
) -> AppResult<GeneralSettings> {
    apply_launch_preferences(&settings_path(&app)?, request.launch_preferences)
}

#[tauri::command]
pub async fn pick_mame_executable(app: AppHandle) -> AppResult<Option<String>> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let selected = app
            .dialog()
            .file()
            .set_title("Choose MAME executable")
            .blocking_pick_file();

        match selected {
            Some(path) => {
                let path = path.into_path().map_err(|error| {
                    AppError::new(
                        "MAME_EXECUTABLE_SELECTION_INVALID",
                        "The selected MAME executable could not be represented as a platform path.",
                    )
                    .with_details(serde_json::json!({ "cause": error.to_string() }))
                })?;
                path.into_os_string().into_string().map(Some).map_err(|_| {
                    AppError::new(
                        "MAME_EXECUTABLE_SELECTION_NON_UTF8",
                        "The selected MAME executable path cannot be stored by the current settings schema.",
                    )
                })
            }
            None => Ok(None),
        }
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = app;
        Err(AppError::new(
            "MAME_EXECUTABLE_PICKER_UNAVAILABLE",
            "Executable selection is not supported on this platform.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::{
        AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference,
    };

    use super::{apply_launch_preferences, load_general_settings};

    #[test]
    fn launch_preferences_round_trip_through_general_settings() {
        let root = temp_root("launch-preferences");
        fs::create_dir_all(&root).expect("create settings root");
        let path = root.join("settings.json");
        let preferences = LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::OpenGl,
            audio: AudioPreference::Auto,
        };

        let saved = apply_launch_preferences(&path, preferences.clone())
            .expect("persist launch preferences");
        assert_eq!(saved.launch_preferences, preferences);
        assert_eq!(
            load_general_settings(&path)
                .expect("reload general settings")
                .launch_preferences,
            preferences
        );

        fs::remove_dir_all(root).expect("remove settings root");
    }

    #[test]
    fn general_settings_load_defaults_when_file_is_absent() {
        let path = temp_root("missing").join("settings.json");
        let settings = load_general_settings(&path).expect("missing settings use defaults");
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.mame_executable, None);
        assert_eq!(settings.launch_preferences, LaunchPreferencesV1::default());
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-mt604-{label}-{nonce}"))
    }
}
