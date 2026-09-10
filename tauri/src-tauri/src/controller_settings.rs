//! Typed controller-profile commands for MT-608.
//!
//! An assigned profile is configuration intent, not evidence that MAME is
//! currently using that mapping. Until launch integration applies these
//! profiles natively, `active_profile` is deliberately always `None`.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    controller_profiles::{
        clear_controller_profile_assignment, create_controller_profile,
        get_controller_profile_assignment_for_scope, list_controller_profiles,
        set_controller_profile_assignment, ControllerDeviceIdentity, ControllerDeviceIdentityKind,
        ControllerMappingProvenance, ControllerMappingProvenanceKind, ControllerProfile,
        ControllerProfileDraft, ControllerProfileScope,
    },
    errors::{AppError, AppResult},
    storage,
};

pub const CONTROLLER_CONFIGURATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ControllerProfileApplicationStatus {
    Unassigned,
    AssignedNotApplied,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfileConfiguration {
    pub schema_version: u32,
    pub profiles: Vec<ControllerProfile>,
    /// The profile assigned exactly at the requested scope, if any.
    pub assigned_profile: Option<ControllerProfile>,
    /// Machine scope falls back to the global assignment when no machine
    /// assignment exists. Global scope has no lower-priority fallback.
    pub effective_profile: Option<ControllerProfile>,
    pub effective_scope: Option<ControllerProfileScope>,
    /// Runtime truth, not configuration intent. This stays `None` until a
    /// native launch/runtime integration can prove the mapping was applied.
    pub active_profile: Option<ControllerProfile>,
    pub application_status: ControllerProfileApplicationStatus,
    pub status_message: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfileConfigurationRequest {
    pub scope: ControllerProfileScope,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetControllerProfileSelectionRequest {
    pub scope: ControllerProfileScope,
    pub profile_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateBrowserControllerProfileRequest {
    pub name: String,
    pub gamepad_id: String,
    pub mapping: String,
}

#[tauri::command]
pub fn get_controller_profile_configuration(
    request: ControllerProfileConfigurationRequest,
    app: AppHandle,
) -> AppResult<ControllerProfileConfiguration> {
    controller_profile_configuration(&storage::catalog_path(&app)?, request.scope)
}

#[tauri::command]
pub fn set_controller_profile_selection(
    request: SetControllerProfileSelectionRequest,
    app: AppHandle,
) -> AppResult<ControllerProfileConfiguration> {
    let catalog_path = storage::catalog_path(&app)?;
    match request.profile_id {
        Some(profile_id) => {
            set_controller_profile_assignment(&catalog_path, profile_id, request.scope.clone())?;
        }
        None => {
            clear_controller_profile_assignment(&catalog_path, request.scope.clone())?;
        }
    }
    controller_profile_configuration(&catalog_path, request.scope)
}

#[tauri::command]
pub fn create_browser_controller_profile(
    request: CreateBrowserControllerProfileRequest,
    app: AppHandle,
) -> AppResult<ControllerProfile> {
    create_browser_controller_profile_at_path(
        &storage::catalog_path(&app)?,
        request,
        current_epoch_ms()?,
    )
}

fn controller_profile_configuration(
    catalog_path: &Path,
    scope: ControllerProfileScope,
) -> AppResult<ControllerProfileConfiguration> {
    let profiles = list_controller_profiles(catalog_path)?;
    let assigned = get_controller_profile_assignment_for_scope(catalog_path, scope.clone())?;
    let assigned_profile = assigned
        .as_ref()
        .map(|assignment| profile_from_list(&profiles, assignment.profile_id))
        .transpose()?;

    let effective_assignment = if assigned.is_some() {
        assigned
    } else if matches!(scope, ControllerProfileScope::Machine { .. }) {
        get_controller_profile_assignment_for_scope(catalog_path, ControllerProfileScope::Global)?
    } else {
        None
    };
    let effective_scope = effective_assignment
        .as_ref()
        .map(|assignment| assignment.scope.clone());
    let effective_profile = effective_assignment
        .as_ref()
        .map(|assignment| profile_from_list(&profiles, assignment.profile_id))
        .transpose()?;

    let application_status = if effective_profile.is_some() {
        ControllerProfileApplicationStatus::AssignedNotApplied
    } else {
        ControllerProfileApplicationStatus::Unassigned
    };
    let status_message = match application_status {
        ControllerProfileApplicationStatus::Unassigned =>
            "No controller profile is selected for this scope.".to_owned(),
        ControllerProfileApplicationStatus::AssignedNotApplied =>
            "A controller profile is selected, but this build does not yet apply saved profiles to MAME gameplay input. It is not reported as active.".to_owned(),
    };

    Ok(ControllerProfileConfiguration {
        schema_version: CONTROLLER_CONFIGURATION_SCHEMA_VERSION,
        profiles,
        assigned_profile,
        effective_profile,
        effective_scope,
        active_profile: None,
        application_status,
        status_message,
    })
}

fn profile_from_list(
    profiles: &[ControllerProfile],
    profile_id: i64,
) -> AppResult<ControllerProfile> {
    profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .or_else(|| get_controller_profile_fallback(profile_id, profiles))
        .ok_or_else(|| {
            AppError::new(
                "CONTROLLER_PROFILE_ASSIGNMENT_DANGLING",
                "A controller profile assignment refers to a missing profile.",
            )
            .with_details(serde_json::json!({ "profileId": profile_id }))
        })
}

fn get_controller_profile_fallback(
    _profile_id: i64,
    _profiles: &[ControllerProfile],
) -> Option<ControllerProfile> {
    // All profiles are loaded in the same catalog snapshot. A missing ID is a
    // storage-integrity error; do not silently fetch/guess another profile.
    None
}

fn create_browser_controller_profile_at_path(
    catalog_path: &Path,
    request: CreateBrowserControllerProfileRequest,
    now_epoch_ms: u64,
) -> AppResult<ControllerProfile> {
    if request.mapping != "standard" {
        return Err(AppError::new(
            "CONTROLLER_GAMEPAD_MAPPING_UNSUPPORTED",
            "Only browser gamepads reporting the W3C standard mapping can be captured as supported profiles.",
        )
        .with_details(serde_json::json!({ "mapping": request.mapping })));
    }

    let requested_name = request.name.trim();
    if list_controller_profiles(catalog_path)?
        .iter()
        .any(|profile| profile.name.eq_ignore_ascii_case(requested_name))
    {
        return Err(AppError::new(
            "CONTROLLER_PROFILE_NAME_CONFLICT",
            "A controller profile with that name already exists.",
        )
        .with_details(serde_json::json!({ "name": requested_name })));
    }

    create_controller_profile(
        catalog_path,
        ControllerProfileDraft {
            name: request.name,
            target_device: ControllerDeviceIdentity {
                kind: ControllerDeviceIdentityKind::BrowserGamepadId,
                value: request.gamepad_id,
                reported_mapping: Some(request.mapping),
            },
            mapping_provenance: ControllerMappingProvenance {
                kind: ControllerMappingProvenanceKind::BrowserStandardGamepad,
                source_reference: None,
            },
        },
        now_epoch_ms,
    )
}

fn current_epoch_ms() -> AppResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                "CONTROLLER_PROFILE_CLOCK_INVALID",
                "The system clock is before the Unix epoch.",
            )
            .with_details(serde_json::json!({ "cause": error.to_string() }))
        })?;
    u64::try_from(duration.as_millis()).map_err(|_| {
        AppError::new(
            "CONTROLLER_PROFILE_TIMESTAMP_INVALID",
            "The current timestamp is outside the supported range.",
        )
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::controller_profiles::{
        create_controller_profile, set_controller_profile_assignment, ControllerDeviceIdentity,
        ControllerDeviceIdentityKind, ControllerMappingProvenance, ControllerMappingProvenanceKind,
        ControllerProfileDraft, ControllerProfileScope,
    };

    use super::{
        controller_profile_configuration, create_browser_controller_profile_at_path,
        ControllerProfileApplicationStatus, CreateBrowserControllerProfileRequest,
    };

    fn project_profile(name: &str, device: &str) -> ControllerProfileDraft {
        ControllerProfileDraft {
            name: name.to_owned(),
            target_device: ControllerDeviceIdentity {
                kind: ControllerDeviceIdentityKind::UserDefined,
                value: device.to_owned(),
                reported_mapping: None,
            },
            mapping_provenance: ControllerMappingProvenance {
                kind: ControllerMappingProvenanceKind::ProjectOwned,
                source_reference: Some(format!("project:{name}")),
            },
        }
    }

    #[test]
    fn machine_selection_overrides_global_but_is_not_claimed_active() {
        let path = temp_catalog("precedence");
        let global = create_controller_profile(&path, project_profile("Global", "pad-a"), 1)
            .expect("global profile");
        let machine = create_controller_profile(&path, project_profile("Machine", "pad-b"), 2)
            .expect("machine profile");
        set_controller_profile_assignment(&path, global.id, ControllerProfileScope::Global)
            .expect("assign global");
        set_controller_profile_assignment(
            &path,
            machine.id,
            ControllerProfileScope::Machine {
                short_name: "pacman".to_owned(),
            },
        )
        .expect("assign machine");

        let configuration = controller_profile_configuration(
            &path,
            ControllerProfileScope::Machine {
                short_name: "pacman".to_owned(),
            },
        )
        .expect("configuration");
        assert_eq!(
            configuration.assigned_profile.as_ref().map(|p| p.id),
            Some(machine.id)
        );
        assert_eq!(
            configuration.effective_profile.as_ref().map(|p| p.id),
            Some(machine.id)
        );
        assert_eq!(
            configuration.effective_scope,
            Some(ControllerProfileScope::Machine {
                short_name: "pacman".to_owned(),
            })
        );
        assert_eq!(
            configuration.application_status,
            ControllerProfileApplicationStatus::AssignedNotApplied
        );
        assert_eq!(configuration.active_profile, None);
        cleanup(&path);
    }

    #[test]
    fn machine_without_override_inherits_global_selection() {
        let path = temp_catalog("inherit-global");
        let global = create_controller_profile(&path, project_profile("Global", "pad-a"), 1)
            .expect("global profile");
        set_controller_profile_assignment(&path, global.id, ControllerProfileScope::Global)
            .expect("assign global");

        let configuration = controller_profile_configuration(
            &path,
            ControllerProfileScope::Machine {
                short_name: "galaga".to_owned(),
            },
        )
        .expect("configuration");
        assert_eq!(configuration.assigned_profile, None);
        assert_eq!(
            configuration.effective_profile.as_ref().map(|p| p.id),
            Some(global.id)
        );
        assert_eq!(
            configuration.effective_scope,
            Some(ControllerProfileScope::Global)
        );
        assert_eq!(configuration.active_profile, None);
        cleanup(&path);
    }

    #[test]
    fn selecting_another_profile_replaces_the_exact_scope_assignment() {
        let path = temp_catalog("replace");
        let first = create_controller_profile(&path, project_profile("First", "pad-a"), 1)
            .expect("first profile");
        let second = create_controller_profile(&path, project_profile("Second", "pad-b"), 2)
            .expect("second profile");
        set_controller_profile_assignment(&path, first.id, ControllerProfileScope::Global)
            .expect("first assignment");
        set_controller_profile_assignment(&path, second.id, ControllerProfileScope::Global)
            .expect("replacement assignment");

        let configuration = controller_profile_configuration(&path, ControllerProfileScope::Global)
            .expect("configuration");
        assert_eq!(
            configuration.assigned_profile.as_ref().map(|p| p.id),
            Some(second.id)
        );
        cleanup(&path);
    }

    #[test]
    fn browser_capture_rejects_nonstandard_mapping() {
        let path = temp_catalog("unsupported");
        let error = create_browser_controller_profile_at_path(
            &path,
            CreateBrowserControllerProfileRequest {
                name: "Unsupported pad".to_owned(),
                gamepad_id: "vendor pad".to_owned(),
                mapping: "".to_owned(),
            },
            1,
        )
        .expect_err("nonstandard mapping must be rejected");
        assert_eq!(error.code, "CONTROLLER_GAMEPAD_MAPPING_UNSUPPORTED");
        cleanup(&path);
    }

    fn temp_catalog(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-tauri-mt608-{label}-{nonce}.sqlite3"))
    }

    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
