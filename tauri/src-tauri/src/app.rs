use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::{
    bundled_runtime::BundledRuntimeLayout,
    config::{load_settings, settings_path, SettingsV2, SETTINGS_SCHEMA_VERSION},
    errors::{AppError, AppResult},
    event_names::APP_READY_EVENT,
    mame::{inspect_executable, MameExecutableIdentity, MameExecutableSource},
    sessions::query_state::RUNTIME_CONTROL_PROTOCOL_VERSION,
    storage::CATALOG_SCHEMA_VERSION,
};

pub const APP_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoRequest {
    pub protocol_version: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BuildIdentity {
    pub git_sha: Option<&'static str>,
    pub profile: &'static str,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MameVersionReport {
    NotConfigured,
    Available {
        identity: MameExecutableIdentity,
    },
    Unavailable {
        path: Option<String>,
        error_code: String,
        error_message: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoResponse {
    pub protocol_version: u32,
    pub app_version: &'static str,
    pub backend: &'static str,
    pub build: BuildIdentity,
    pub database_schema_version: i64,
    pub settings_schema_version: u32,
    pub runtime_protocol_version: u32,
    pub mame: MameVersionReport,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppReadyEventV1 {
    pub schema_version: u32,
    pub app_version: &'static str,
}

fn build_identity() -> BuildIdentity {
    BuildIdentity {
        git_sha: option_env!("GITHUB_SHA"),
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    }
}

fn build_app_info_with_mame(
    request: AppInfoRequest,
    mame: MameVersionReport,
) -> AppResult<AppInfoResponse> {
    if request.protocol_version != APP_PROTOCOL_VERSION {
        return Err(AppError::new(
            "UNSUPPORTED_PROTOCOL_VERSION",
            format!(
                "Frontend protocol version {} is not supported by backend version {}.",
                request.protocol_version, APP_PROTOCOL_VERSION
            ),
        )
        .with_details(serde_json::json!({
            "requestedVersion": request.protocol_version,
            "supportedVersion": APP_PROTOCOL_VERSION
        })));
    }

    Ok(AppInfoResponse {
        protocol_version: APP_PROTOCOL_VERSION,
        app_version: env!("CARGO_PKG_VERSION"),
        backend: "rust-tauri",
        build: build_identity(),
        database_schema_version: CATALOG_SCHEMA_VERSION,
        settings_schema_version: SETTINGS_SCHEMA_VERSION,
        runtime_protocol_version: RUNTIME_CONTROL_PROTOCOL_VERSION,
        mame,
    })
}

pub fn build_app_info(request: AppInfoRequest) -> AppResult<AppInfoResponse> {
    build_app_info_with_mame(request, MameVersionReport::NotConfigured)
}

fn unavailable_mame(path: Option<String>, error: AppError) -> MameVersionReport {
    MameVersionReport::Unavailable {
        path,
        error_code: error.code,
        error_message: error.message,
    }
}

fn effective_mame_source(
    settings: &SettingsV2,
    resource_dir: &std::path::Path,
) -> AppResult<MameExecutableSource> {
    if let Some(path) = settings.mame_executable.as_deref() {
        return Ok(MameExecutableSource::external(path));
    }

    BundledRuntimeLayout::from_resource_dir(resource_dir).executable_source()
}

fn resolve_mame_version<R: Runtime>(app: &AppHandle<R>) -> MameVersionReport {
    let path = match settings_path(app) {
        Ok(path) => path,
        Err(error) => return unavailable_mame(None, error),
    };
    let settings = match load_settings(&path) {
        Ok(settings) => settings,
        Err(error) => return unavailable_mame(None, error),
    };
    let resource_dir = match app.path().resource_dir() {
        Ok(path) => path,
        Err(error) => {
            return unavailable_mame(
                None,
                AppError::new(
                    "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE",
                    "The application resource directory could not be resolved.",
                )
                .with_details(serde_json::json!({ "cause": error.to_string() })),
            )
        }
    };

    let source = match effective_mame_source(&settings, &resource_dir) {
        Ok(source) => source,
        Err(error) => return unavailable_mame(settings.mame_executable.clone(), error),
    };
    let display_path = source.path().to_string_lossy().into_owned();

    match inspect_executable(source) {
        Ok(identity) => MameVersionReport::Available { identity },
        Err(error) => unavailable_mame(Some(display_path), error),
    }
}

#[tauri::command]
pub fn get_app_info(request: AppInfoRequest, app: AppHandle) -> AppResult<AppInfoResponse> {
    build_app_info_with_mame(request, resolve_mame_version(&app))
}

pub fn emit_ready<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let event = AppReadyEventV1 {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION"),
    };

    app.emit(APP_READY_EVENT, event).map_err(|error| {
        AppError::new(
            "APP_READY_EVENT_FAILED",
            "The backend could not emit the application-ready event.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        build_app_info, effective_mame_source, AppInfoRequest, MameVersionReport,
        APP_PROTOCOL_VERSION,
    };
    use crate::{
        bundled_runtime::BundledRuntimeLayout,
        config::{SettingsV2, SETTINGS_SCHEMA_VERSION},
        mame::{MameExecutableSourceKind, MameExecutableTrust},
        sessions::query_state::RUNTIME_CONTROL_PROTOCOL_VERSION,
        storage::CATALOG_SCHEMA_VERSION,
    };

    #[test]
    fn accepts_current_protocol_and_reports_version_contract() {
        let result = build_app_info(AppInfoRequest {
            protocol_version: APP_PROTOCOL_VERSION,
        })
        .expect("current protocol must be accepted");

        assert_eq!(result.protocol_version, APP_PROTOCOL_VERSION);
        assert_eq!(result.backend, "rust-tauri");
        assert_eq!(result.database_schema_version, CATALOG_SCHEMA_VERSION);
        assert_eq!(result.settings_schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(
            result.runtime_protocol_version,
            RUNTIME_CONTROL_PROTOCOL_VERSION
        );
        assert_eq!(result.mame, MameVersionReport::NotConfigured);
        assert!(!result.build.profile.is_empty());
        assert!(!result.build.target.is_empty());
    }

    #[test]
    fn default_settings_resolve_the_package_owned_bundled_runtime() {
        let temp = tempdir().expect("tempdir");
        let resource_dir = temp.path().join("resources");
        fs::create_dir_all(&resource_dir).expect("resource dir");
        let layout = BundledRuntimeLayout::from_resource_dir(&resource_dir);
        fs::create_dir_all(layout.executable.parent().expect("bin dir")).expect("bin dir");
        fs::create_dir_all(&layout.hash_dir).expect("hash dir");
        fs::create_dir_all(layout.bgfx_dir.join("shaders")).expect("bgfx dir");
        fs::create_dir_all(&layout.legal_dir).expect("legal dir");
        fs::write(&layout.executable, b"mame fixture").expect("executable");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&layout.executable)
                .expect("metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&layout.executable, permissions).expect("permissions");
        }
        fs::write(layout.hash_dir.join("fixture.xml"), b"<softwarelist/>")
            .expect("hash");
        fs::write(
            layout.bgfx_dir.join("shaders").join("fixture.bin"),
            b"shader",
        )
        .expect("bgfx");
        fs::write(&layout.copying, b"license").expect("copying");
        fs::write(layout.legal_dir.join("GPL-2.0"), b"license").expect("legal");

        let source = effective_mame_source(&SettingsV2::default(), &resource_dir)
            .expect("default settings must resolve bundled runtime");
        assert_eq!(source.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(source.trust(), MameExecutableTrust::QualifiedBundled);
        assert_eq!(source.path(), layout.executable.as_path());
    }

    #[test]
    fn explicit_external_setting_remains_an_override() {
        let settings = SettingsV2 {
            mame_executable: Some("/opt/custom-mame/mame".to_owned()),
            ..SettingsV2::default()
        };
        let source = effective_mame_source(&settings, std::path::Path::new("/unused"))
            .expect("external override does not require bundled layout");
        assert_eq!(source.kind(), MameExecutableSourceKind::External);
        assert_eq!(source.path(), std::path::Path::new("/opt/custom-mame/mame"));
    }

    #[test]
    fn missing_bundled_runtime_is_a_package_error_not_not_configured() {
        let temp = tempdir().expect("tempdir");
        let resource_dir = temp.path().join("resources");
        fs::create_dir_all(&resource_dir).expect("resource dir");
        let error = effective_mame_source(&SettingsV2::default(), &resource_dir)
            .expect_err("missing bundled runtime must fail closed");
        assert_eq!(error.code, "MAME_BUNDLED_RUNTIME_MISSING");
    }

    #[test]
    fn rejects_unknown_protocol() {
        let error = build_app_info(AppInfoRequest {
            protocol_version: APP_PROTOCOL_VERSION + 1,
        })
        .expect_err("unknown protocol must be rejected");

        assert_eq!(error.code, "UNSUPPORTED_PROTOCOL_VERSION");
    }
}
