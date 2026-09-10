use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    config::{
        load_settings, settings_path, AudioPreference, LaunchPreferencesV1, RendererPreference,
        WindowPreference,
    },
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
    storage::{self, open_catalog_connection},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineLaunchSettings {
    pub schema_version: u32,
    pub short_name: String,
    pub overrides: LaunchPreferencesV1,
    pub effective: LaunchPreferencesV1,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineLaunchSettingsRequest {
    pub short_name: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetMachineLaunchSettingsRequest {
    pub short_name: String,
    pub overrides: LaunchPreferencesV1,
}

#[tauri::command]
pub fn get_machine_launch_settings(
    request: MachineLaunchSettingsRequest,
    app: AppHandle,
) -> AppResult<MachineLaunchSettings> {
    let short_name = validated_short_name(request.short_name)?;
    let settings = load_settings(&settings_path(&app)?)?;
    let catalog_path = storage::catalog_path(&app)?;
    let overrides = load_machine_overrides(&catalog_path, &short_name)?;
    Ok(machine_launch_settings(
        short_name,
        overrides,
        &settings.launch_preferences,
    ))
}

#[tauri::command]
pub fn set_machine_launch_settings(
    request: SetMachineLaunchSettingsRequest,
    app: AppHandle,
) -> AppResult<MachineLaunchSettings> {
    let short_name = validated_short_name(request.short_name)?;
    let settings = load_settings(&settings_path(&app)?)?;
    let catalog_path = storage::catalog_path(&app)?;
    let mut connection = open_catalog_connection(&catalog_path)?;

    if is_fully_inherited(&request.overrides) {
        delete_machine_overrides(&mut connection, &short_name)?;
    } else {
        save_machine_overrides(&mut connection, &short_name, &request.overrides)?;
    }

    let overrides = load_machine_overrides_with_connection(&connection, &short_name)?;
    Ok(machine_launch_settings(
        short_name,
        overrides,
        &settings.launch_preferences,
    ))
}

#[tauri::command]
pub fn reset_machine_launch_settings(
    request: MachineLaunchSettingsRequest,
    app: AppHandle,
) -> AppResult<MachineLaunchSettings> {
    let short_name = validated_short_name(request.short_name)?;
    let settings = load_settings(&settings_path(&app)?)?;
    let catalog_path = storage::catalog_path(&app)?;
    let mut connection = open_catalog_connection(&catalog_path)?;
    delete_machine_overrides(&mut connection, &short_name)?;
    Ok(machine_launch_settings(
        short_name,
        LaunchPreferencesV1::default(),
        &settings.launch_preferences,
    ))
}

pub(crate) fn effective_launch_preferences(
    catalog_path: &Path,
    general: &LaunchPreferencesV1,
    short_name: &str,
) -> AppResult<LaunchPreferencesV1> {
    let overrides = load_machine_overrides(catalog_path, short_name)?;
    Ok(resolve_effective_preferences(general, &overrides))
}

fn machine_launch_settings(
    short_name: String,
    overrides: LaunchPreferencesV1,
    general: &LaunchPreferencesV1,
) -> MachineLaunchSettings {
    let effective = resolve_effective_preferences(general, &overrides);
    MachineLaunchSettings {
        schema_version: 1,
        short_name,
        overrides,
        effective,
    }
}

fn resolve_effective_preferences(
    general: &LaunchPreferencesV1,
    overrides: &LaunchPreferencesV1,
) -> LaunchPreferencesV1 {
    LaunchPreferencesV1 {
        window_mode: if overrides.window_mode == WindowPreference::Inherit {
            general.window_mode
        } else {
            overrides.window_mode
        },
        renderer: if overrides.renderer == RendererPreference::Inherit {
            general.renderer
        } else {
            overrides.renderer
        },
        audio: if overrides.audio == AudioPreference::Inherit {
            general.audio
        } else {
            overrides.audio
        },
    }
}

fn is_fully_inherited(preferences: &LaunchPreferencesV1) -> bool {
    preferences.window_mode == WindowPreference::Inherit
        && preferences.renderer == RendererPreference::Inherit
        && preferences.audio == AudioPreference::Inherit
}

fn load_machine_overrides(path: &Path, short_name: &str) -> AppResult<LaunchPreferencesV1> {
    let connection = open_catalog_connection(path)?;
    load_machine_overrides_with_connection(&connection, short_name)
}

fn load_machine_overrides_with_connection(
    connection: &Connection,
    short_name: &str,
) -> AppResult<LaunchPreferencesV1> {
    let encoded = connection
        .query_row(
            "SELECT preferences_json FROM machine_launch_preferences WHERE machine_short_name = ?1",
            [short_name],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(machine_settings_database_error)?;

    match encoded {
        Some(encoded) => serde_json::from_str(&encoded).map_err(|error| {
            AppError::new(
                "MACHINE_SETTINGS_INVALID",
                "Stored per-machine launch settings are invalid.",
            )
            .with_details(serde_json::json!({
                "shortName": short_name,
                "cause": error.to_string()
            }))
        }),
        None => Ok(LaunchPreferencesV1::default()),
    }
}

fn save_machine_overrides(
    connection: &mut Connection,
    short_name: &str,
    overrides: &LaunchPreferencesV1,
) -> AppResult<()> {
    let encoded = serde_json::to_string(overrides).map_err(|error| {
        AppError::new(
            "MACHINE_SETTINGS_SERIALIZE_FAILED",
            "Per-machine launch settings could not be serialized.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    connection
        .execute(
            "INSERT INTO machine_launch_preferences(machine_short_name, preferences_json) VALUES (?1, ?2)\n             ON CONFLICT(machine_short_name) DO UPDATE SET preferences_json = excluded.preferences_json",
            (short_name, encoded),
        )
        .map_err(machine_settings_database_error)?;
    Ok(())
}

fn delete_machine_overrides(connection: &mut Connection, short_name: &str) -> AppResult<()> {
    connection
        .execute(
            "DELETE FROM machine_launch_preferences WHERE machine_short_name = ?1",
            [short_name],
        )
        .map_err(machine_settings_database_error)?;
    Ok(())
}

fn validated_short_name(short_name: String) -> AppResult<String> {
    let short_name = short_name.trim();
    validate_short_identifier("machine", short_name)?;
    Ok(short_name.to_owned())
}

fn machine_settings_database_error(error: rusqlite::Error) -> AppError {
    AppError::new(
        "MACHINE_SETTINGS_DATABASE_FAILED",
        "The per-machine settings database operation failed.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{AudioPreference, LaunchPreferencesV1, RendererPreference, WindowPreference},
        storage::open_catalog_memory,
    };

    use super::{
        delete_machine_overrides, is_fully_inherited, load_machine_overrides_with_connection,
        resolve_effective_preferences, save_machine_overrides,
    };

    fn general_preferences() -> LaunchPreferencesV1 {
        LaunchPreferencesV1 {
            window_mode: WindowPreference::Windowed,
            renderer: RendererPreference::Bgfx,
            audio: AudioPreference::Auto,
        }
    }

    #[test]
    fn machine_overrides_persist_and_reload() {
        let mut connection = open_catalog_memory().expect("catalog schema");
        let overrides = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Inherit,
            audio: AudioPreference::Disabled,
        };

        save_machine_overrides(&mut connection, "pacman", &overrides).expect("save overrides");
        assert_eq!(
            load_machine_overrides_with_connection(&connection, "pacman")
                .expect("reload overrides"),
            overrides
        );
    }

    #[test]
    fn reset_deletes_the_persisted_override_row() {
        let mut connection = open_catalog_memory().expect("catalog schema");
        let overrides = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Software,
            audio: AudioPreference::Disabled,
        };
        save_machine_overrides(&mut connection, "pacman", &overrides).expect("save overrides");

        delete_machine_overrides(&mut connection, "pacman").expect("reset overrides");

        assert_eq!(
            load_machine_overrides_with_connection(&connection, "pacman")
                .expect("load reset overrides"),
            LaunchPreferencesV1::default()
        );
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM machine_launch_preferences WHERE machine_short_name = 'pacman'",
                [],
                |row| row.get(0),
            )
            .expect("count override rows");
        assert_eq!(count, 0);
    }

    #[test]
    fn effective_values_use_machine_override_then_general_setting() {
        let general = general_preferences();
        let overrides = LaunchPreferencesV1 {
            window_mode: WindowPreference::Fullscreen,
            renderer: RendererPreference::Inherit,
            audio: AudioPreference::Disabled,
        };

        assert_eq!(
            resolve_effective_preferences(&general, &overrides),
            LaunchPreferencesV1 {
                window_mode: WindowPreference::Fullscreen,
                renderer: RendererPreference::Bgfx,
                audio: AudioPreference::Disabled,
            }
        );
    }

    #[test]
    fn fully_inherited_machine_settings_are_storage_empty() {
        assert!(is_fully_inherited(&LaunchPreferencesV1::default()));
        assert!(!is_fully_inherited(&LaunchPreferencesV1 {
            window_mode: WindowPreference::Inherit,
            renderer: RendererPreference::Auto,
            audio: AudioPreference::Inherit,
        }));
    }
}
