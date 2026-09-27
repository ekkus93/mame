//! Authoritative MAME runtime selection for installed and developer configurations.
//!
//! Normal installed operation uses the package-owned bundled runtime. A persisted
//! external executable is an explicit advanced override. Frontend callers never
//! get to self-assert bundled trust for an arbitrary path.

use std::path::Path;

use tauri::{AppHandle, Manager, Runtime};

use crate::{
    bundled_runtime::BundledRuntimeLayout,
    config::{load_settings, settings_path},
    errors::{AppError, AppResult},
    mame::{
        configured_external_source, inspect_executable, MameExecutableIdentity,
        MameExecutableSource,
    },
};

pub fn source_from_preference(
    external_override: Option<&str>,
    resource_dir: &Path,
) -> AppResult<MameExecutableSource> {
    if let Some(source) = configured_external_source(external_override)? {
        return Ok(source);
    }

    BundledRuntimeLayout::from_resource_dir(resource_dir)
        .executable_source()
        .map_err(|error| {
            AppError::new(
                "MAME_BUNDLED_RUNTIME_UNAVAILABLE",
                "The MAME runtime installed with this application is missing or invalid.",
            )
            .with_details(serde_json::json!({
                "causeCode": error.code,
                "causeMessage": error.message,
                "causeDetails": error.details
            }))
        })
}

pub fn effective_mame_source<R: Runtime>(app: &AppHandle<R>) -> AppResult<MameExecutableSource> {
    let settings = load_settings(&settings_path(app)?)?;
    let resource_dir = app.path().resource_dir().map_err(|error| {
        AppError::new(
            "MAME_BUNDLED_RESOURCE_DIR_UNAVAILABLE",
            "The application resource directory containing the bundled MAME runtime is unavailable.",
        )
        .with_details(serde_json::json!({ "cause": error.to_string() }))
    })?;

    source_from_preference(settings.mame_executable.as_deref(), &resource_dir)
}

pub fn effective_mame_identity<R: Runtime>(
    app: &AppHandle<R>,
) -> AppResult<MameExecutableIdentity> {
    inspect_executable(effective_mame_source(app)?)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::source_from_preference;
    use crate::{
        bundled_runtime::{BUNDLED_MAME_EXECUTABLE, BUNDLED_RUNTIME_DIR},
        mame::{MameExecutableSourceKind, MameExecutableTrust},
    };

    fn valid_resource_root() -> tempfile::TempDir {
        let temp = tempdir().expect("tempdir");
        let runtime = temp.path().join(BUNDLED_RUNTIME_DIR);
        fs::create_dir_all(runtime.join("bin")).expect("bin");
        fs::create_dir_all(runtime.join("hash")).expect("hash");
        fs::create_dir_all(runtime.join("bgfx")).expect("bgfx");
        fs::create_dir_all(runtime.join("licenses/legal")).expect("legal");
        fs::write(
            runtime.join("bin").join(BUNDLED_MAME_EXECUTABLE),
            b"fixture",
        )
        .expect("mame");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = runtime.join("bin").join(BUNDLED_MAME_EXECUTABLE);
            let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(executable, permissions).expect("permissions");
        }
        fs::write(runtime.join("hash/fixture.xml"), b"<softwarelist/>").expect("hash");
        fs::write(runtime.join("bgfx/fixture.bin"), b"bgfx").expect("bgfx");
        fs::write(runtime.join("licenses/COPYING"), b"copying").expect("copying");
        fs::write(runtime.join("licenses/legal/GPL-2.0"), b"legal").expect("legal");
        temp
    }

    #[test]
    fn no_override_resolves_package_owned_bundled_runtime() {
        let temp = valid_resource_root();
        let source = source_from_preference(None, temp.path()).expect("bundled source");
        assert_eq!(source.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(source.trust(), MameExecutableTrust::QualifiedBundled);
        assert!(source.path().starts_with(temp.path()));
    }

    #[test]
    fn explicit_external_override_wins_without_needing_bundled_layout() {
        let temp = tempdir().expect("tempdir");
        let source =
            source_from_preference(Some("/opt/custom/mame"), temp.path()).expect("external source");
        assert_eq!(source.kind(), MameExecutableSourceKind::External);
        assert_eq!(source.trust(), MameExecutableTrust::UserConfigured);
    }

    #[test]
    fn missing_bundled_runtime_is_reported_as_package_failure() {
        let temp = tempdir().expect("tempdir");
        let error = source_from_preference(None, temp.path()).expect_err("missing runtime");
        assert_eq!(error.code, "MAME_BUNDLED_RUNTIME_UNAVAILABLE");
    }
}
