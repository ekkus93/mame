use std::path::Path;

use serde::Serialize;
use tauri::AppHandle;

use crate::{
    config::{load_settings, settings_path},
    diagnostics,
    errors::{AppError, AppResult},
    mame::{configured_external_source, inspect_executable, MameExecutableIdentity},
    metadata::{
        self, CatalogRepository, MetadataFreshness, MetadataGenerationSummary,
        MetadataRefreshResult,
    },
    storage,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameUiConfiguredMame {
    pub version: String,
    pub build: Option<String>,
    pub raw_version_line: String,
}

impl From<&MameExecutableIdentity> for MameUiConfiguredMame {
    fn from(identity: &MameExecutableIdentity) -> Self {
        Self {
            version: identity.version.clone(),
            build: identity.build.clone(),
            raw_version_line: identity.raw_version_line.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MameUiGenerationSummary {
    pub generation_id: i64,
    pub mame_version: String,
    pub imported_at_epoch_ms: u64,
    pub machine_count: u64,
}

impl From<&MetadataGenerationSummary> for MameUiGenerationSummary {
    fn from(generation: &MetadataGenerationSummary) -> Self {
        Self {
            generation_id: generation.generation_id,
            mame_version: generation.mame_version.clone(),
            imported_at_epoch_ms: generation.imported_at_epoch_ms,
            machine_count: generation.machine_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MameUiBootstrapStatus {
    NotConfigured,
    ExecutableUnavailable {
        error_code: String,
        error_message: String,
    },
    MetadataMissing {
        mame: MameUiConfiguredMame,
    },
    MetadataStale {
        mame: MameUiConfiguredMame,
        active_generation: MameUiGenerationSummary,
    },
    Ready {
        mame: MameUiConfiguredMame,
        generation: MameUiGenerationSummary,
    },
}

#[tauri::command]
pub async fn get_mame_ui_bootstrap_status(app: AppHandle) -> AppResult<MameUiBootstrapStatus> {
    let settings_path = settings_path(&app)?;
    let catalog_path = storage::catalog_path(&app)?;

    tauri::async_runtime::spawn_blocking(move || bootstrap_status(&settings_path, &catalog_path))
        .await
        .map_err(bootstrap_worker_error)?
}

#[tauri::command]
pub async fn refresh_configured_mame_metadata(app: AppHandle) -> AppResult<MetadataRefreshResult> {
    let settings = load_settings(&settings_path(&app)?)?;
    let source =
        configured_external_source(settings.mame_executable.as_deref())?.ok_or_else(|| {
            AppError::new(
                "MAME_NOT_CONFIGURED",
                "Configure a MAME executable before importing machine metadata.",
            )
        })?;
    let catalog_path = storage::catalog_path(&app)?;

    diagnostics::record(
        "info",
        "metadata.refresh",
        "Configured-MAME metadata refresh requested from the MAME UI bootstrap flow.",
        serde_json::json!({ "source": "configuredExternal" }),
    );

    let result = tauri::async_runtime::spawn_blocking(move || {
        metadata::refresh_catalog(source, &catalog_path)
    })
    .await
    .map_err(bootstrap_worker_error)??;

    diagnostics::record(
        "info",
        "metadata.refresh",
        "Configured-MAME metadata refresh completed from the MAME UI bootstrap flow.",
        serde_json::json!({
            "generationId": result.generation.generation_id,
            "machineCount": result.generation.machine_count
        }),
    );
    Ok(result)
}

fn bootstrap_status(settings_path: &Path, catalog_path: &Path) -> AppResult<MameUiBootstrapStatus> {
    let settings = load_settings(settings_path)?;
    let Some(source) = configured_external_source(settings.mame_executable.as_deref())? else {
        return Ok(MameUiBootstrapStatus::NotConfigured);
    };

    let identity = match inspect_executable(source) {
        Ok(identity) => identity,
        Err(error) => {
            return Ok(MameUiBootstrapStatus::ExecutableUnavailable {
                error_code: error.code,
                error_message: error.message,
            })
        }
    };
    let mame = MameUiConfiguredMame::from(&identity);
    let status = CatalogRepository::open(catalog_path)?.metadata_status(identity)?;

    match (status.freshness, status.active_generation.as_ref()) {
        (MetadataFreshness::Fresh, Some(generation)) => Ok(MameUiBootstrapStatus::Ready {
            mame,
            generation: MameUiGenerationSummary::from(generation),
        }),
        (MetadataFreshness::Stale, Some(generation)) => Ok(MameUiBootstrapStatus::MetadataStale {
            mame,
            active_generation: MameUiGenerationSummary::from(generation),
        }),
        _ => Ok(MameUiBootstrapStatus::MetadataMissing { mame }),
    }
}

fn bootstrap_worker_error(error: impl std::fmt::Display) -> AppError {
    AppError::new(
        "MAME_UI_BOOTSTRAP_WORKER_FAILED",
        "The MAME UI readiness worker did not complete normally.",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{bootstrap_status, MameUiBootstrapStatus};
    #[cfg(unix)]
    use crate::{
        mame::MameExecutableSource,
        metadata::{refresh_catalog, CatalogRepository, CloneFilter, MachineQuery, MachineSort},
    };

    #[test]
    fn missing_settings_are_explicitly_not_configured() {
        let root = temp_root("not-configured");
        let status = bootstrap_status(&root.join("settings.json"), &root.join("catalog.sqlite3"))
            .expect("missing settings should use safe defaults");
        assert_eq!(status, MameUiBootstrapStatus::NotConfigured);
    }

    #[test]
    fn unusable_configured_executable_is_a_readiness_state_not_a_catalog_error() {
        let root = temp_root("unavailable");
        fs::create_dir_all(&root).expect("create root");
        fs::write(
            root.join("settings.json"),
            r#"{"schemaVersion":2,"mameExecutable":"/definitely/missing/mame"}"#,
        )
        .expect("write settings");

        let status = bootstrap_status(&root.join("settings.json"), &root.join("catalog.sqlite3"))
            .expect("unavailable executable should be represented as status");
        assert!(matches!(
            status,
            MameUiBootstrapStatus::ExecutableUnavailable { .. }
        ));
        fs::remove_dir_all(root).expect("remove root");
    }

    #[cfg(unix)]
    #[test]
    fn persisted_configuration_imports_and_populates_first_catalog_query() {
        let root = temp_root("configured-import-ready");
        fs::create_dir_all(&root).expect("create root");
        let executable = root.join("fake-mame");
        write_fake_mame(&executable);
        let settings = serde_json::json!({
            "schemaVersion": 2,
            "mameExecutable": executable.to_string_lossy().into_owned()
        });
        let settings_path = root.join("settings.json");
        let catalog_path = root.join("catalog.sqlite3");
        fs::write(
            &settings_path,
            serde_json::to_vec(&settings).expect("encode settings"),
        )
        .expect("write settings");

        let before = bootstrap_status(&settings_path, &catalog_path)
            .expect("configured executable should be inspectable");
        assert!(matches!(
            before,
            MameUiBootstrapStatus::MetadataMissing { .. }
        ));

        let imported = refresh_catalog(MameExecutableSource::external(&executable), &catalog_path)
            .expect("representative metadata import should succeed");
        assert_eq!(imported.generation.machine_count, 4);

        let after = bootstrap_status(&settings_path, &catalog_path)
            .expect("fresh imported generation should be ready");
        assert!(matches!(
            after,
            MameUiBootstrapStatus::Ready {
                ref generation,
                ..
            } if generation.machine_count == 4
        ));

        let first_page = CatalogRepository::open(&catalog_path)
            .expect("open populated catalog")
            .query_machines(&MachineQuery {
                text: None,
                manufacturer: None,
                year: None,
                driver_status: None,
                clone_filter: CloneFilter::All,
                sort: MachineSort::DescriptionAsc,
                include_devices: false,
                limit: 100,
                offset: 0,
            })
            .expect("first unfiltered query should succeed");
        assert_eq!(first_page.total, 4);
        assert!(!first_page.items.is_empty());

        fs::remove_dir_all(root).expect("remove root");
    }

    #[cfg(unix)]
    fn write_fake_mame(path: &PathBuf) {
        use std::os::unix::fs::PermissionsExt;

        let fixture = include_str!("../../tests/fixtures/listxml-representative.xml");
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = '-noreadconfig' ] && [ \"$2\" = '-version' ]; then\n  printf '%s\\n' '0.288 test-fixture'\n  exit 0\nfi\nif [ \"$1\" = '-noreadconfig' ] && [ \"$2\" = '-listxml' ]; then\n  cat <<'MAME_XML'\n{fixture}\nMAME_XML\n  exit 0\nfi\nexit 99\n"
        );
        fs::write(path, script).expect("write fake MAME");
        let mut permissions = fs::metadata(path)
            .expect("fake MAME metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("mark fake MAME executable");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "mame-tauri-pcr-bootstrap-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
