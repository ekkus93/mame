//! User-owned controller profile identity, scope, and mapping provenance.
//!
//! This module models configuration metadata only. Gameplay input remains
//! native to MAME; no controller samples are routed through Tauri IPC here.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, AppResult},
    mame::validate_short_identifier,
    storage::open_catalog_connection,
};

pub const CONTROLLER_PROFILE_SCHEMA_VERSION: u32 = 1;
const MAX_PROFILE_NAME_BYTES: usize = 120;
const MAX_DEVICE_IDENTITY_BYTES: usize = 1024;
const MAX_MAPPING_LABEL_BYTES: usize = 120;
const MAX_PROVENANCE_REFERENCE_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ControllerDeviceIdentityKind {
    /// The browser/OS-provided Gamepad API `id` string.
    ///
    /// This is intentionally not called a serial number: browsers and drivers
    /// do not guarantee that it is globally stable or uniquely identifies one
    /// physical unit.
    BrowserGamepadId,
    /// An identifier reported by a future native/MAME input-device adapter.
    MameInputDeviceId,
    /// A user-assigned identity label when the platform exposes no stronger ID.
    UserDefined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerDeviceIdentity {
    pub kind: ControllerDeviceIdentityKind,
    pub value: String,
    /// Mapping label reported with the device identity, e.g. Gamepad API
    /// `standard`. This is descriptive provenance, not proof of runtime use.
    pub reported_mapping: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ControllerMappingProvenanceKind {
    /// W3C Gamepad API `standard` mapping observed by the frontend.
    BrowserStandardGamepad,
    /// MAME core `-ctrlr <name>` controller configuration.
    MameControllerConfig,
    /// Generic OSD `controller_map` mapping file.
    MameOsdControllerMap,
    /// Mapping owned by this application rather than a MAME-owned file.
    ProjectOwned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerMappingProvenance {
    pub kind: ControllerMappingProvenanceKind,
    /// Source-specific reference such as a `-ctrlr` profile name, controller
    /// map path, or project mapping identifier. Absence is meaningful for
    /// browser-standard mappings that have no external source file.
    pub source_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfileDraft {
    pub name: String,
    pub target_device: ControllerDeviceIdentity,
    pub mapping_provenance: ControllerMappingProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfile {
    pub schema_version: u32,
    pub id: i64,
    pub name: String,
    pub target_device: ControllerDeviceIdentity,
    pub mapping_provenance: ControllerMappingProvenance,
    pub created_at_epoch_ms: i64,
    pub updated_at_epoch_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ControllerProfileScope {
    Global,
    Machine {
        #[serde(rename = "shortName")]
        short_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ControllerProfileAssignment {
    pub profile_id: i64,
    pub scope: ControllerProfileScope,
}

pub fn create_controller_profile(
    catalog_path: &Path,
    draft: ControllerProfileDraft,
    now_epoch_ms: u64,
) -> AppResult<ControllerProfile> {
    let mut connection = open_catalog_connection(catalog_path)?;
    create_controller_profile_with_connection(&mut connection, draft, now_epoch_ms)
}

pub fn update_controller_profile(
    catalog_path: &Path,
    profile_id: i64,
    draft: ControllerProfileDraft,
    now_epoch_ms: u64,
) -> AppResult<ControllerProfile> {
    let mut connection = open_catalog_connection(catalog_path)?;
    update_controller_profile_with_connection(&mut connection, profile_id, draft, now_epoch_ms)
}

pub fn get_controller_profile(
    catalog_path: &Path,
    profile_id: i64,
) -> AppResult<Option<ControllerProfile>> {
    let connection = open_catalog_connection(catalog_path)?;
    load_controller_profile_with_connection(&connection, profile_id)
}

pub fn list_controller_profiles(catalog_path: &Path) -> AppResult<Vec<ControllerProfile>> {
    let connection = open_catalog_connection(catalog_path)?;
    list_controller_profiles_with_connection(&connection)
}

pub fn set_controller_profile_assignment(
    catalog_path: &Path,
    profile_id: i64,
    scope: ControllerProfileScope,
) -> AppResult<ControllerProfileAssignment> {
    let mut connection = open_catalog_connection(catalog_path)?;
    set_controller_profile_assignment_with_connection(&mut connection, profile_id, scope)
}

pub fn list_controller_profile_assignments(
    catalog_path: &Path,
    profile_id: i64,
) -> AppResult<Vec<ControllerProfileAssignment>> {
    let connection = open_catalog_connection(catalog_path)?;
    list_controller_profile_assignments_with_connection(&connection, profile_id)
}

pub fn get_controller_profile_assignment_for_scope(
    catalog_path: &Path,
    scope: ControllerProfileScope,
) -> AppResult<Option<ControllerProfileAssignment>> {
    let connection = open_catalog_connection(catalog_path)?;
    get_controller_profile_assignment_for_scope_with_connection(&connection, scope)
}

pub fn clear_controller_profile_assignment(
    catalog_path: &Path,
    scope: ControllerProfileScope,
) -> AppResult<bool> {
    let connection = open_catalog_connection(catalog_path)?;
    let scope = validate_scope(scope)?;
    let (scope_kind, scope_key, _) = scope_storage_fields(&scope);
    connection
        .execute(
            "DELETE FROM controller_profile_assignments WHERE scope_kind = ?1 AND scope_key = ?2",
            params![scope_kind, scope_key],
        )
        .map(|changed| changed != 0)
        .map_err(controller_profile_database_error)
}

pub fn delete_controller_profile(catalog_path: &Path, profile_id: i64) -> AppResult<bool> {
    validate_profile_id(profile_id)?;
    let connection = open_catalog_connection(catalog_path)?;
    connection
        .execute(
            "DELETE FROM controller_profiles WHERE id = ?1",
            [profile_id],
        )
        .map(|changed| changed != 0)
        .map_err(controller_profile_database_error)
}

fn create_controller_profile_with_connection(
    connection: &mut Connection,
    draft: ControllerProfileDraft,
    now_epoch_ms: u64,
) -> AppResult<ControllerProfile> {
    let draft = validate_draft(draft)?;
    let now_epoch_ms = checked_epoch_ms(now_epoch_ms)?;
    let device_identity_json = serialize_json("target device identity", &draft.target_device)?;
    let mapping_provenance_json = serialize_json("mapping provenance", &draft.mapping_provenance)?;

    connection
        .execute(
            "INSERT INTO controller_profiles(name, device_identity_json, mapping_provenance_json, created_at_epoch_ms, updated_at_epoch_ms) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![draft.name, device_identity_json, mapping_provenance_json, now_epoch_ms],
        )
        .map_err(controller_profile_database_error)?;
    let profile_id = connection.last_insert_rowid();
    load_controller_profile_with_connection(connection, profile_id)?.ok_or_else(|| {
        AppError::new(
            "CONTROLLER_PROFILE_INSERT_LOST",
            "The controller profile was inserted but could not be reloaded.",
        )
    })
}

fn update_controller_profile_with_connection(
    connection: &mut Connection,
    profile_id: i64,
    draft: ControllerProfileDraft,
    now_epoch_ms: u64,
) -> AppResult<ControllerProfile> {
    validate_profile_id(profile_id)?;
    let draft = validate_draft(draft)?;
    let now_epoch_ms = checked_epoch_ms(now_epoch_ms)?;
    let device_identity_json = serialize_json("target device identity", &draft.target_device)?;
    let mapping_provenance_json = serialize_json("mapping provenance", &draft.mapping_provenance)?;

    let changed = connection
        .execute(
            "UPDATE controller_profiles SET name = ?1, device_identity_json = ?2, mapping_provenance_json = ?3, updated_at_epoch_ms = ?4 WHERE id = ?5",
            params![
                draft.name,
                device_identity_json,
                mapping_provenance_json,
                now_epoch_ms,
                profile_id
            ],
        )
        .map_err(controller_profile_database_error)?;
    if changed == 0 {
        return Err(profile_not_found(profile_id));
    }
    load_controller_profile_with_connection(connection, profile_id)?.ok_or_else(|| {
        AppError::new(
            "CONTROLLER_PROFILE_UPDATE_LOST",
            "The controller profile was updated but could not be reloaded.",
        )
    })
}

fn load_controller_profile_with_connection(
    connection: &Connection,
    profile_id: i64,
) -> AppResult<Option<ControllerProfile>> {
    validate_profile_id(profile_id)?;
    let raw = connection
        .query_row(
            "SELECT id, name, device_identity_json, mapping_provenance_json, created_at_epoch_ms, updated_at_epoch_ms FROM controller_profiles WHERE id = ?1",
            [profile_id],
            raw_profile_from_row,
        )
        .optional()
        .map_err(controller_profile_database_error)?;
    raw.map(decode_profile).transpose()
}

fn list_controller_profiles_with_connection(
    connection: &Connection,
) -> AppResult<Vec<ControllerProfile>> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, device_identity_json, mapping_provenance_json, created_at_epoch_ms, updated_at_epoch_ms FROM controller_profiles ORDER BY name COLLATE NOCASE, id",
        )
        .map_err(controller_profile_database_error)?;
    let rows = statement
        .query_map([], raw_profile_from_row)
        .map_err(controller_profile_database_error)?;
    rows.map(|row| {
        row.map_err(controller_profile_database_error)
            .and_then(decode_profile)
    })
    .collect()
}

fn set_controller_profile_assignment_with_connection(
    connection: &mut Connection,
    profile_id: i64,
    scope: ControllerProfileScope,
) -> AppResult<ControllerProfileAssignment> {
    validate_profile_id(profile_id)?;
    if load_controller_profile_with_connection(connection, profile_id)?.is_none() {
        return Err(profile_not_found(profile_id));
    }
    let scope = validate_scope(scope)?;
    let (scope_kind, scope_key, machine_short_name) = scope_storage_fields(&scope);
    let transaction = connection
        .transaction()
        .map_err(controller_profile_database_error)?;
    transaction
        .execute(
            "DELETE FROM controller_profile_assignments WHERE scope_kind = ?1 AND scope_key = ?2",
            params![scope_kind, &scope_key],
        )
        .map_err(controller_profile_database_error)?;
    transaction
        .execute(
            "INSERT INTO controller_profile_assignments(profile_id, scope_kind, scope_key, machine_short_name) VALUES (?1, ?2, ?3, ?4)",
            params![profile_id, scope_kind, &scope_key, &machine_short_name],
        )
        .map_err(controller_profile_database_error)?;
    transaction
        .commit()
        .map_err(controller_profile_database_error)?;
    Ok(ControllerProfileAssignment { profile_id, scope })
}

fn list_controller_profile_assignments_with_connection(
    connection: &Connection,
    profile_id: i64,
) -> AppResult<Vec<ControllerProfileAssignment>> {
    validate_profile_id(profile_id)?;
    let mut statement = connection
        .prepare(
            "SELECT scope_kind, machine_short_name FROM controller_profile_assignments WHERE profile_id = ?1 ORDER BY scope_kind, scope_key",
        )
        .map_err(controller_profile_database_error)?;
    let rows = statement
        .query_map([profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(controller_profile_database_error)?;

    rows.map(|row| {
        let (scope_kind, machine_short_name) = row.map_err(controller_profile_database_error)?;
        let scope = match (scope_kind.as_str(), machine_short_name) {
            ("global", None) => ControllerProfileScope::Global,
            ("machine", Some(short_name)) => ControllerProfileScope::Machine { short_name },
            _ => {
                return Err(AppError::new(
                    "CONTROLLER_PROFILE_SCOPE_INVALID",
                    "Stored controller profile assignment scope is invalid.",
                )
                .with_details(serde_json::json!({ "profileId": profile_id })))
            }
        };
        Ok(ControllerProfileAssignment { profile_id, scope })
    })
    .collect()
}

fn get_controller_profile_assignment_for_scope_with_connection(
    connection: &Connection,
    scope: ControllerProfileScope,
) -> AppResult<Option<ControllerProfileAssignment>> {
    let scope = validate_scope(scope)?;
    let (scope_kind, scope_key, _) = scope_storage_fields(&scope);
    let mut statement = connection
        .prepare(
            "SELECT profile_id FROM controller_profile_assignments WHERE scope_kind = ?1 AND scope_key = ?2 ORDER BY profile_id LIMIT 2",
        )
        .map_err(controller_profile_database_error)?;
    let profile_ids = statement
        .query_map(params![scope_kind, &scope_key], |row| row.get::<_, i64>(0))
        .map_err(controller_profile_database_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(controller_profile_database_error)?;

    if profile_ids.len() > 1 {
        return Err(AppError::new(
            "CONTROLLER_PROFILE_ASSIGNMENT_AMBIGUOUS",
            "More than one controller profile is assigned to the same scope.",
        )
        .with_details(serde_json::json!({ "scope": scope })));
    }

    Ok(profile_ids
        .first()
        .map(|profile_id| ControllerProfileAssignment {
            profile_id: *profile_id,
            scope,
        }))
}

fn validate_draft(mut draft: ControllerProfileDraft) -> AppResult<ControllerProfileDraft> {
    draft.name = bounded_trimmed(
        "CONTROLLER_PROFILE_NAME_INVALID",
        "Controller profile name",
        draft.name,
        MAX_PROFILE_NAME_BYTES,
    )?;
    draft.target_device.value = bounded_trimmed(
        "CONTROLLER_DEVICE_IDENTITY_INVALID",
        "Controller device identity",
        draft.target_device.value,
        MAX_DEVICE_IDENTITY_BYTES,
    )?;
    draft.target_device.reported_mapping = bounded_optional_trimmed(
        "CONTROLLER_DEVICE_MAPPING_LABEL_INVALID",
        "Controller reported mapping",
        draft.target_device.reported_mapping,
        MAX_MAPPING_LABEL_BYTES,
    )?;
    draft.mapping_provenance.source_reference = bounded_optional_trimmed(
        "CONTROLLER_MAPPING_PROVENANCE_INVALID",
        "Controller mapping provenance reference",
        draft.mapping_provenance.source_reference,
        MAX_PROVENANCE_REFERENCE_BYTES,
    )?;
    Ok(draft)
}

fn validate_scope(scope: ControllerProfileScope) -> AppResult<ControllerProfileScope> {
    match scope {
        ControllerProfileScope::Global => Ok(ControllerProfileScope::Global),
        ControllerProfileScope::Machine { short_name } => {
            let short_name = short_name.trim();
            validate_short_identifier("machine", short_name)?;
            Ok(ControllerProfileScope::Machine {
                short_name: short_name.to_owned(),
            })
        }
    }
}

fn scope_storage_fields(scope: &ControllerProfileScope) -> (&'static str, String, Option<String>) {
    match scope {
        ControllerProfileScope::Global => ("global", "*".to_owned(), None),
        ControllerProfileScope::Machine { short_name } => {
            ("machine", short_name.clone(), Some(short_name.clone()))
        }
    }
}

fn validate_profile_id(profile_id: i64) -> AppResult<()> {
    if profile_id <= 0 {
        return Err(AppError::new(
            "CONTROLLER_PROFILE_ID_INVALID",
            "Controller profile identity must be a positive database ID.",
        )
        .with_details(serde_json::json!({ "profileId": profile_id })));
    }
    Ok(())
}

fn bounded_trimmed(code: &str, label: &str, value: String, max_bytes: usize) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > max_bytes {
        return Err(
            AppError::new(code, format!("{label} is empty or too long.")).with_details(
                serde_json::json!({ "maxBytes": max_bytes, "actualBytes": value.len() }),
            ),
        );
    }
    Ok(value.to_owned())
}

fn bounded_optional_trimmed(
    code: &str,
    label: &str,
    value: Option<String>,
    max_bytes: usize,
) -> AppResult<Option<String>> {
    value
        .map(|value| bounded_trimmed(code, label, value, max_bytes))
        .transpose()
}

fn checked_epoch_ms(value: u64) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| {
        AppError::new(
            "CONTROLLER_PROFILE_TIMESTAMP_INVALID",
            "Controller profile timestamp is outside SQLite's supported range.",
        )
    })
}

fn serialize_json<T: Serialize>(label: &str, value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| {
        AppError::new(
            "CONTROLLER_PROFILE_SERIALIZE_FAILED",
            format!("Controller profile {label} could not be serialized."),
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

fn decode_profile(raw: RawControllerProfile) -> AppResult<ControllerProfile> {
    let target_device = serde_json::from_str(&raw.device_identity_json)
        .map_err(|error| invalid_stored_profile(raw.id, "target device identity", error))?;
    let mapping_provenance = serde_json::from_str(&raw.mapping_provenance_json)
        .map_err(|error| invalid_stored_profile(raw.id, "mapping provenance", error))?;
    Ok(ControllerProfile {
        schema_version: CONTROLLER_PROFILE_SCHEMA_VERSION,
        id: raw.id,
        name: raw.name,
        target_device,
        mapping_provenance,
        created_at_epoch_ms: raw.created_at_epoch_ms,
        updated_at_epoch_ms: raw.updated_at_epoch_ms,
    })
}

struct RawControllerProfile {
    id: i64,
    name: String,
    device_identity_json: String,
    mapping_provenance_json: String,
    created_at_epoch_ms: i64,
    updated_at_epoch_ms: i64,
}

fn raw_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawControllerProfile> {
    Ok(RawControllerProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        device_identity_json: row.get(2)?,
        mapping_provenance_json: row.get(3)?,
        created_at_epoch_ms: row.get(4)?,
        updated_at_epoch_ms: row.get(5)?,
    })
}

fn invalid_stored_profile(profile_id: i64, field: &str, error: serde_json::Error) -> AppError {
    AppError::new(
        "CONTROLLER_PROFILE_STORED_DATA_INVALID",
        "Stored controller profile data is invalid.",
    )
    .with_details(serde_json::json!({
        "profileId": profile_id,
        "field": field,
        "cause": error.to_string()
    }))
}

fn profile_not_found(profile_id: i64) -> AppError {
    AppError::new(
        "CONTROLLER_PROFILE_NOT_FOUND",
        "The requested controller profile does not exist.",
    )
    .with_details(serde_json::json!({ "profileId": profile_id }))
}

fn controller_profile_database_error(error: rusqlite::Error) -> AppError {
    AppError::new(
        "CONTROLLER_PROFILE_DATABASE_FAILED",
        "The controller profile database operation failed.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::{
        create_controller_profile_with_connection, delete_controller_profile,
        list_controller_profile_assignments_with_connection,
        list_controller_profiles_with_connection,
        set_controller_profile_assignment_with_connection, ControllerDeviceIdentity,
        ControllerDeviceIdentityKind, ControllerMappingProvenance, ControllerMappingProvenanceKind,
        ControllerProfileDraft, ControllerProfileScope, CONTROLLER_PROFILE_SCHEMA_VERSION,
    };
    use crate::storage::open_catalog_memory;

    fn browser_profile(name: &str) -> ControllerProfileDraft {
        ControllerProfileDraft {
            name: name.to_owned(),
            target_device: ControllerDeviceIdentity {
                kind: ControllerDeviceIdentityKind::BrowserGamepadId,
                value: "045e-0b13-XInput STANDARD GAMEPAD".to_owned(),
                reported_mapping: Some("standard".to_owned()),
            },
            mapping_provenance: ControllerMappingProvenance {
                kind: ControllerMappingProvenanceKind::BrowserStandardGamepad,
                source_reference: None,
            },
        }
    }

    #[test]
    fn profile_identity_target_device_and_provenance_round_trip() {
        let mut connection = open_catalog_memory().expect("catalog");
        let profile = create_controller_profile_with_connection(
            &mut connection,
            browser_profile("Arcade pad"),
            1234,
        )
        .expect("create profile");

        assert!(profile.id > 0);
        assert_eq!(profile.schema_version, CONTROLLER_PROFILE_SCHEMA_VERSION);
        assert_eq!(profile.name, "Arcade pad");
        assert_eq!(
            profile.target_device.kind,
            ControllerDeviceIdentityKind::BrowserGamepadId
        );
        assert_eq!(
            profile.target_device.reported_mapping.as_deref(),
            Some("standard")
        );
        assert_eq!(
            profile.mapping_provenance.kind,
            ControllerMappingProvenanceKind::BrowserStandardGamepad
        );
        assert_eq!(profile.created_at_epoch_ms, 1234);
        assert_eq!(profile.updated_at_epoch_ms, 1234);

        let listed = list_controller_profiles_with_connection(&connection).expect("list profiles");
        assert_eq!(listed, vec![profile]);
    }

    #[test]
    fn global_and_machine_associations_are_distinct_user_state() {
        let mut connection = open_catalog_memory().expect("catalog");
        let profile = create_controller_profile_with_connection(
            &mut connection,
            browser_profile("Primary pad"),
            1,
        )
        .expect("create profile");

        set_controller_profile_assignment_with_connection(
            &mut connection,
            profile.id,
            ControllerProfileScope::Global,
        )
        .expect("global assignment");
        set_controller_profile_assignment_with_connection(
            &mut connection,
            profile.id,
            ControllerProfileScope::Machine {
                short_name: "pacman".to_owned(),
            },
        )
        .expect("machine assignment");

        let assignments =
            list_controller_profile_assignments_with_connection(&connection, profile.id)
                .expect("list assignments");
        assert_eq!(assignments.len(), 2);
        assert!(assignments
            .iter()
            .any(|assignment| assignment.scope == ControllerProfileScope::Global));
        assert!(assignments.iter().any(|assignment| {
            assignment.scope
                == ControllerProfileScope::Machine {
                    short_name: "pacman".to_owned(),
                }
        }));

        let machine_rows: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM controller_profile_assignments WHERE scope_kind='machine' AND scope_key='pacman'",
                [],
                |row| row.get(0),
            )
            .expect("machine assignment count");
        assert_eq!(machine_rows, 1);
    }

    #[test]
    fn mame_controller_and_osd_mapping_provenance_do_not_collapse() {
        let mut connection = open_catalog_memory().expect("catalog");
        let mut ctrlr = browser_profile("MAME ctrlr");
        ctrlr.mapping_provenance = ControllerMappingProvenance {
            kind: ControllerMappingProvenanceKind::MameControllerConfig,
            source_reference: Some("xbox360".to_owned()),
        };
        let ctrlr = create_controller_profile_with_connection(&mut connection, ctrlr, 1)
            .expect("create ctrlr profile");

        let mut osd = browser_profile("OSD map");
        osd.mapping_provenance = ControllerMappingProvenance {
            kind: ControllerMappingProvenanceKind::MameOsdControllerMap,
            source_reference: Some("/etc/mame/controller.map".to_owned()),
        };
        let osd = create_controller_profile_with_connection(&mut connection, osd, 2)
            .expect("create osd profile");

        assert_ne!(ctrlr.mapping_provenance.kind, osd.mapping_provenance.kind);
        assert_eq!(
            ctrlr.mapping_provenance.source_reference.as_deref(),
            Some("xbox360")
        );
        assert_eq!(
            osd.mapping_provenance.source_reference.as_deref(),
            Some("/etc/mame/controller.map")
        );
    }

    #[test]
    fn machine_scope_does_not_require_generated_metadata_row() {
        let mut connection = open_catalog_memory().expect("catalog");
        let profile = create_controller_profile_with_connection(
            &mut connection,
            browser_profile("Future machine pad"),
            1,
        )
        .expect("create profile");

        set_controller_profile_assignment_with_connection(
            &mut connection,
            profile.id,
            ControllerProfileScope::Machine {
                short_name: "pacman".to_owned(),
            },
        )
        .expect("assignment must be independent of generated metadata");

        let machine_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM machines", [], |row| row.get(0))
            .expect("machine count");
        let assignment_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM controller_profile_assignments",
                [],
                |row| row.get(0),
            )
            .expect("assignment count");
        assert_eq!(machine_count, 0);
        assert_eq!(assignment_count, 1);
    }

    #[test]
    fn profile_delete_cascades_only_profile_owned_assignments() {
        let mut connection = open_catalog_memory().expect("catalog");
        let profile = create_controller_profile_with_connection(
            &mut connection,
            browser_profile("Disposable pad"),
            1,
        )
        .expect("create profile");
        set_controller_profile_assignment_with_connection(
            &mut connection,
            profile.id,
            ControllerProfileScope::Global,
        )
        .expect("assign profile");

        let changed = connection
            .execute(
                "DELETE FROM controller_profiles WHERE id = ?1",
                [profile.id],
            )
            .expect("delete profile");
        assert_eq!(changed, 1);
        let assignment_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM controller_profile_assignments",
                [],
                |row| row.get(0),
            )
            .expect("assignment count");
        assert_eq!(assignment_count, 0);
    }

    #[test]
    fn public_delete_rejects_nonpositive_profile_identity_before_io() {
        let error = delete_controller_profile(Path::new("/unused"), 0)
            .expect_err("invalid identity must fail before database access");
        assert_eq!(error.code, "CONTROLLER_PROFILE_ID_INVALID");
    }

    use std::path::Path;
}
