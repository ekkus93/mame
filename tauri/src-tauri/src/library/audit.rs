use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    config::{load_settings, settings_path, ContentPathsV1},
    errors::{AppError, AppResult},
    mame::{
        audit_machine, inspect_executable, load_current_machine_audit_result,
        save_machine_audit_result, MameAuditParseResult, MameExecutableIdentity,
        MameExecutableSource, StoredMachineAuditResult,
    },
    metadata::CatalogRepository,
    storage,
};

use super::{
    ensure_generation_matches_executable, launch_source_from_generation, now_epoch_ms,
    validate_machine_short_name,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineAuditRequest {
    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MachineAuditResponse {
    pub schema_version: u32,
    pub machine_short_name: String,
    pub result: MameAuditParseResult,
    pub audited_at_epoch_ms: u64,
}

#[derive(Clone)]
pub(crate) struct AuditContext {
    pub(crate) source: MameExecutableSource,
    pub(crate) identity: MameExecutableIdentity,
    pub(crate) content_paths: ContentPathsV1,
}

#[tauri::command]
pub async fn run_library_machine_audit(
    request: MachineAuditRequest,
    app: AppHandle,
) -> AppResult<MachineAuditResponse> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;
    let settings_path = settings_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        run_machine_audit(&catalog_path, &settings_path, short_name)
    })
    .await
    .map_err(audit_worker_error)?
}

#[tauri::command]
pub async fn get_library_machine_audit(
    request: MachineAuditRequest,
    app: AppHandle,
) -> AppResult<Option<MachineAuditResponse>> {
    let short_name = validate_machine_short_name(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;
    let settings_path = settings_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || {
        load_machine_audit(&catalog_path, &settings_path, short_name)
    })
    .await
    .map_err(audit_worker_error)?
}

fn run_machine_audit(
    catalog_path: &Path,
    settings_path: &Path,
    short_name: String,
) -> AppResult<MachineAuditResponse> {
    let context = resolve_audit_context(catalog_path, settings_path, &short_name)?;
    run_machine_audit_with_context(catalog_path, short_name, &context)
}

pub(crate) fn run_machine_audit_with_context(
    catalog_path: &Path,
    short_name: String,
    context: &AuditContext,
) -> AppResult<MachineAuditResponse> {
    let result = audit_machine(&context.source, &short_name, &context.content_paths)?;
    let audited_at_epoch_ms = now_epoch_ms()?;

    save_machine_audit_result(
        catalog_path,
        &short_name,
        &result,
        &context.identity,
        &context.content_paths,
        audited_at_epoch_ms,
    )?;

    Ok(MachineAuditResponse {
        schema_version: 1,
        machine_short_name: short_name,
        result,
        audited_at_epoch_ms,
    })
}

fn load_machine_audit(
    catalog_path: &Path,
    settings_path: &Path,
    short_name: String,
) -> AppResult<Option<MachineAuditResponse>> {
    let context = resolve_audit_context(catalog_path, settings_path, &short_name)?;
    load_current_machine_audit_result(
        catalog_path,
        &short_name,
        &context.identity,
        &context.content_paths,
    )
    .map(|stored| stored.map(MachineAuditResponse::from))
}

fn resolve_audit_context(
    catalog_path: &Path,
    settings_path: &Path,
    short_name: &str,
) -> AppResult<AuditContext> {
    let context = resolve_bulk_audit_context(catalog_path, settings_path)?;

    // Keep the command scoped to a machine in the active generation. A syntactically
    // valid arbitrary frontend string must not become an unconstrained MAME target.
    CatalogRepository::open(catalog_path)?.machine_detail(short_name)?;
    Ok(context)
}

pub(crate) fn resolve_bulk_audit_context(
    catalog_path: &Path,
    settings_path: &Path,
) -> AppResult<AuditContext> {
    let repository = CatalogRepository::open(catalog_path)?;
    let generation = repository.active_generation()?.ok_or_else(|| {
        AppError::new(
            "MAME_METADATA_NOT_READY",
            "No successfully imported MAME metadata generation is active.",
        )
    })?;

    let source = launch_source_from_generation(&generation)?;
    let identity = inspect_executable(source.clone())?;
    ensure_generation_matches_executable(&generation, &identity)?;
    let content_paths = load_settings(settings_path)?.content_paths;

    Ok(AuditContext {
        source,
        identity,
        content_paths,
    })
}

impl From<StoredMachineAuditResult> for MachineAuditResponse {
    fn from(stored: StoredMachineAuditResult) -> Self {
        Self {
            schema_version: 1,
            machine_short_name: stored.machine_short_name,
            result: stored.result,
            audited_at_epoch_ms: stored.audited_at_epoch_ms,
        }
    }
}

fn audit_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_AUDIT_WORKER_FAILED",
        "The background MAME audit worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use crate::mame::{parse_mame_audit_output, StoredMachineAuditResult};

    use super::MachineAuditResponse;

    #[test]
    fn stored_result_maps_to_versioned_frontend_response() {
        let stored = StoredMachineAuditResult {
            machine_short_name: "pacman".to_owned(),
            result: parse_mame_audit_output("romset pacman is good\n", "", Some(0)),
            audited_at_epoch_ms: 1234,
        };
        let response = MachineAuditResponse::from(stored);
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.machine_short_name, "pacman");
        assert_eq!(response.audited_at_epoch_ms, 1234);
    }
}
