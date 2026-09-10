use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::DialogExt;

use crate::{
    config::{
        load_settings, settings_path, validate_content_path, ContentPathsV1, PathValidation,
        PlatformPath,
    },
    config_persistence::persist_settings,
    errors::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContentPathValidations {
    pub rom_paths: Vec<PathValidation>,
    pub software_paths: Vec<PathValidation>,
    pub chd_paths: Vec<PathValidation>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContentPathConfiguration {
    pub content_paths: ContentPathsV1,
    pub validations: ContentPathValidations,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetContentPathsRequest {
    pub content_paths: ContentPathsV1,
}

fn configuration_for(content_paths: ContentPathsV1) -> ContentPathConfiguration {
    let validations = ContentPathValidations {
        rom_paths: content_paths
            .rom_paths
            .iter()
            .map(validate_content_path)
            .collect(),
        software_paths: content_paths
            .software_paths
            .iter()
            .map(validate_content_path)
            .collect(),
        chd_paths: content_paths
            .chd_paths
            .iter()
            .map(validate_content_path)
            .collect(),
    };

    ContentPathConfiguration {
        content_paths,
        validations,
    }
}

fn apply_content_paths(
    path: &Path,
    content_paths: ContentPathsV1,
) -> AppResult<ContentPathConfiguration> {
    let mut settings = load_settings(path)?;
    settings.content_paths = content_paths;
    persist_settings(path, &settings)?;
    Ok(configuration_for(settings.content_paths))
}

#[tauri::command]
pub fn get_content_path_configuration(app: AppHandle) -> AppResult<ContentPathConfiguration> {
    let settings = load_settings(&settings_path(&app)?)?;
    Ok(configuration_for(settings.content_paths))
}

#[tauri::command]
pub fn set_content_path_configuration(
    request: SetContentPathsRequest,
    app: AppHandle,
) -> AppResult<ContentPathConfiguration> {
    apply_content_paths(&settings_path(&app)?, request.content_paths)
}

#[tauri::command]
pub async fn pick_content_directory(app: AppHandle) -> AppResult<Option<PlatformPath>> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let selected = app
            .dialog()
            .file()
            .set_title("Choose MAME content directory")
            .blocking_pick_folder();

        match selected {
            Some(path) => path
                .into_path()
                .map(PlatformPath::new)
                .map(Some)
                .map_err(|error| {
                    AppError::new(
                        "CONTENT_DIRECTORY_INVALID",
                        "The selected content directory could not be represented as a platform path.",
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
            "CONTENT_DIRECTORY_PICKER_UNAVAILABLE",
            "Directory selection is not supported on this platform.",
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

    use crate::config::{ContentPathsV1, PathValidationStatus, PlatformPath};

    use super::apply_content_paths;

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-{label}-{nonce}"))
    }

    #[test]
    fn persists_order_and_preserves_existing_executable_setting() {
        let root = temp_root("content-paths");
        fs::create_dir_all(&root).expect("create temporary settings root");
        let settings_path = root.join("settings.json");
        fs::write(
            &settings_path,
            r#"{"schemaVersion":2,"mameExecutable":"/opt/mame/mame","contentPaths":{"romPaths":[],"softwarePaths":[],"chdPaths":[]}}"#,
        )
        .expect("seed settings");

        let rom_one = root.join("rom-one");
        let rom_two = root.join("rom-two");
        fs::create_dir_all(&rom_one).expect("create first ROM directory");
        fs::create_dir_all(&rom_two).expect("create second ROM directory");
        let paths = ContentPathsV1 {
            rom_paths: vec![
                PlatformPath::new(rom_two.clone()),
                PlatformPath::new(rom_one.clone()),
            ],
            software_paths: vec![PlatformPath::new(root.join("missing-software"))],
            chd_paths: Vec::new(),
        };

        let configuration =
            apply_content_paths(&settings_path, paths).expect("persist content paths");
        assert_eq!(configuration.content_paths.rom_paths[0].as_path(), rom_two);
        assert_eq!(configuration.content_paths.rom_paths[1].as_path(), rom_one);
        assert_eq!(
            configuration.validations.software_paths[0].status,
            PathValidationStatus::Missing
        );

        let persisted = fs::read_to_string(&settings_path).expect("read persisted settings");
        assert!(persisted.contains("\"mameExecutable\": \"/opt/mame/mame\""));
        assert!(persisted.contains("\"romPaths\""));

        fs::remove_dir_all(root).expect("remove temporary settings root");
    }
}
