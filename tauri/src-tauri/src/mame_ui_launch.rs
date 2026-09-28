use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::{
    config::{load_settings, settings_path, LaunchPreferencesV1},
    effective_runtime::resolve_effective_mame_source,
    errors::{AppError, AppResult},
    mame::{
        get_machine_bios_choices, inspect_executable, validate_bios_identifier,
        validate_bios_selection, validate_short_identifier, BiosChoice, MameExecutableIdentity,
        MameExecutableSource,
    },
    metadata::{CatalogRepository, MetadataGenerationSummary},
    sessions::{self, SessionSnapshot, SessionSupervisor},
    storage,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchMameEmptyRequest {
    pub short_name: String,
    #[serde(default)]
    pub bios: Option<String>,
    #[serde(default)]
    pub launch_overrides: Option<LaunchPreferencesV1>,
}

#[tauri::command]
pub fn launch_mame_empty(
    request: LaunchMameEmptyRequest,
    app: AppHandle,
    supervisor: State<'_, SessionSupervisor>,
) -> AppResult<SessionSnapshot> {
    let short_name = validated_short_identifier(request.short_name)?;
    let catalog_path = storage::catalog_path(&app)?;
    let settings_path = settings_path(&app)?;
    let resource_dir = package_resource_dir(&app)?;
    let repository = CatalogRepository::open(&catalog_path)?;
    let detail = repository.machine_detail(&short_name)?;
    if !detail.can_start_empty {
        return Err(AppError::new(
            "MAME_START_EMPTY_NOT_ALLOWED",
            "The selected machine requires software/media and cannot be started empty.",
        )
        .with_details(serde_json::json!({ "shortName": short_name })));
    }

    let generation = active_generation(&repository)?;
    let source = validated_generation_source(&generation, &settings_path, &resource_dir)?;
    let bios = match request.bios {
        None => None,
        Some(value) => {
            let choices = get_machine_bios_choices(&source, &short_name)?;
            Some(validated_bios_selection(value, &choices)?)
        }
    };

    sessions::launch_mame_with_source_and_bios(
        source,
        short_name,
        None,
        bios,
        Vec::new(),
        request.launch_overrides,
        supervisor,
        app,
    )
}

fn validated_short_identifier(value: String) -> AppResult<String> {
    let value = value.trim();
    validate_short_identifier("machine", value)?;
    Ok(value.to_owned())
}

fn validated_bios_selection(value: String, choices: &[BiosChoice]) -> AppResult<String> {
    let value = value.trim();
    validate_bios_identifier(value)?;
    validate_bios_selection(choices, value)?;
    Ok(value.to_owned())
}

fn active_generation(repository: &CatalogRepository) -> AppResult<MetadataGenerationSummary> {
    repository.active_generation()?.ok_or_else(|| {
        AppError::new(
            "MAME_METADATA_NOT_READY",
            "No successfully imported MAME metadata generation is active.",
        )
    })
}

fn validated_generation_source(
    generation: &MetadataGenerationSummary,
    settings_path: &Path,
    resource_dir: &Path,
) -> AppResult<MameExecutableSource> {
    let effective_source = if generation.source_kind == "bundled" {
        let settings = load_settings(settings_path)?;
        Some(resolve_effective_mame_source(&settings, resource_dir)?)
    } else {
        None
    };
    let source = launch_source_from_generation(generation, effective_source)?;
    let identity = inspect_executable(source.clone())?;
    ensure_generation_matches_executable(generation, &identity)?;
    Ok(source)
}

fn launch_source_from_generation(
    generation: &MetadataGenerationSummary,
    effective_source: Option<MameExecutableSource>,
) -> AppResult<MameExecutableSource> {
    match (generation.source_kind.as_str(), generation.trust.as_str()) {
        ("external", "userConfigured") => {
            Ok(MameExecutableSource::external(&generation.executable_path))
        }
        ("developmentTree", "development") => Ok(MameExecutableSource::development_tree(
            &generation.executable_path,
        )),
        ("bundled", "qualifiedBundled") => effective_source.ok_or_else(|| {
            AppError::new(
                "CATALOG_BUNDLED_EXECUTABLE_RESOLUTION_REQUIRED",
                "Bundled MAME launch requires package-owned executable resolution.",
            )
        }),
        (source_kind, trust) => Err(AppError::new(
            "CATALOG_EXECUTABLE_PROVENANCE_INVALID",
            "The active catalog has an invalid executable source/trust pairing.",
        )
        .with_details(serde_json::json!({
            "sourceKind": source_kind,
            "trust": trust
        }))),
    }
}

fn ensure_generation_matches_executable(
    generation: &MetadataGenerationSummary,
    identity: &MameExecutableIdentity,
) -> AppResult<()> {
    if generation.executable_path == identity.path
        && generation.mame_version == identity.version
        && generation.mame_build == identity.build
        && generation.raw_version_line == identity.raw_version_line
    {
        return Ok(());
    }
    Err(AppError::new(
        "MAME_METADATA_STALE",
        "The selected MAME executable no longer matches the active metadata generation.",
    )
    .with_details(serde_json::json!({
        "catalogPath": generation.executable_path,
        "catalogVersion": generation.mame_version,
        "catalogBuild": generation.mame_build,
        "currentPath": identity.path,
        "currentVersion": identity.version,
        "currentBuild": identity.build
    })))
}

fn package_resource_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    app.path().resource_dir().map_err(|error| {
        AppError::new(
            "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE",
            "The application resource directory could not be resolved.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(test)]
mod tests {
    use super::{launch_source_from_generation, validated_bios_selection};
    use crate::{
        mame::{BiosChoice, MameExecutableSource, MameExecutableSourceKind, MameExecutableTrust},
        metadata::MetadataGenerationSummary,
    };

    fn choices() -> Vec<BiosChoice> {
        vec![
            BiosChoice {
                name: "us".to_owned(),
                description: "US BIOS".to_owned(),
                is_default: true,
            },
            BiosChoice {
                name: "jp".to_owned(),
                description: "Japan BIOS".to_owned(),
                is_default: false,
            },
        ]
    }

    fn generation(source_kind: &str, trust: &str) -> MetadataGenerationSummary {
        MetadataGenerationSummary {
            generation_id: 1,
            source_kind: source_kind.to_owned(),
            trust: trust.to_owned(),
            executable_path: "/opt/mame/mame".to_owned(),
            mame_version: "0.288".to_owned(),
            mame_build: Some("test-fixture".to_owned()),
            raw_version_line: "0.288 test-fixture".to_owned(),
            listxml_build: Some("0.288 test-fixture".to_owned()),
            mame_config: Some("10".to_owned()),
            generated_at_epoch_ms: 100,
            imported_at_epoch_ms: 200,
            machine_count: 4,
        }
    }

    #[test]
    fn selected_bios_must_be_identifier_safe_and_reported_by_mame() {
        assert_eq!(
            validated_bios_selection(" jp ".to_owned(), &choices()).expect("reported BIOS"),
            "jp"
        );
        assert_eq!(
            validated_bios_selection("-bios".to_owned(), &choices())
                .expect_err("option-like BIOS must fail")
                .code,
            "MAME_BIOS_IDENTIFIER_INVALID"
        );
        assert_eq!(
            validated_bios_selection("eu".to_owned(), &choices())
                .expect_err("unreported BIOS must fail")
                .code,
            "MAME_BIOS_SELECTION_INVALID"
        );
    }

    #[test]
    fn bundled_start_empty_uses_resolved_effective_runtime() {
        let source = launch_source_from_generation(
            &generation("bundled", "qualifiedBundled"),
            Some(MameExecutableSource::bundled(
                "/package/resources/mame-runtime/bin/mame",
            )),
        )
        .expect("start-empty uses backend-resolved bundled runtime");

        assert_eq!(source.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(source.trust(), MameExecutableTrust::QualifiedBundled);
    }

    #[test]
    fn bundled_start_empty_cannot_self_assert_bundled_runtime() {
        let error = launch_source_from_generation(&generation("bundled", "qualifiedBundled"), None)
            .expect_err("start-empty requires package-owned bundled resolution");
        assert_eq!(error.code, "CATALOG_BUNDLED_EXECUTABLE_RESOLUTION_REQUIRED");
    }
}
