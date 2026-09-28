use std::path::Path;

use crate::{
    bundled_runtime::BundledRuntimeLayout,
    config::SettingsV2,
    errors::AppResult,
    mame::{configured_external_source, MameExecutableSource},
};

pub fn resolve_effective_mame_source(
    settings: &SettingsV2,
    resource_dir: &Path,
) -> AppResult<MameExecutableSource> {
    if let Some(source) = configured_external_source(settings.mame_executable.as_deref())? {
        return Ok(source);
    }

    BundledRuntimeLayout::from_resource_dir(resource_dir).executable_source()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::resolve_effective_mame_source;
    use crate::{
        bundled_runtime::BundledRuntimeLayout,
        config::SettingsV2,
        mame::{MameExecutableSourceKind, MameExecutableTrust},
    };

    #[test]
    fn default_settings_resolve_to_bundled_runtime() {
        let temp = tempdir().expect("tempdir");
        let resource_dir = temp.path().join("resources");
        fs::create_dir_all(&resource_dir).expect("resource dir");
        let layout = write_fixture_runtime(&resource_dir);

        let source = resolve_effective_mame_source(&SettingsV2::default(), &resource_dir)
            .expect("default settings must resolve bundled runtime");

        assert_eq!(source.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(source.trust(), MameExecutableTrust::QualifiedBundled);
        assert_eq!(source.path(), layout.executable.as_path());
    }

    #[test]
    fn explicit_external_override_does_not_require_bundled_layout() {
        let settings = SettingsV2 {
            mame_executable: Some("/opt/custom-mame/mame".to_owned()),
            ..SettingsV2::default()
        };

        let source = resolve_effective_mame_source(&settings, std::path::Path::new("/unused"))
            .expect("external override does not require bundled layout");

        assert_eq!(source.kind(), MameExecutableSourceKind::External);
        assert_eq!(source.path(), std::path::Path::new("/opt/custom-mame/mame"));
    }

    #[test]
    fn empty_external_override_is_rejected_before_bundled_fallback() {
        let settings = SettingsV2 {
            mame_executable: Some("   ".to_owned()),
            ..SettingsV2::default()
        };

        let error = resolve_effective_mame_source(&settings, std::path::Path::new("/unused"))
            .expect_err("blank override is an invalid explicit override");

        assert_eq!(error.code, "MAME_EXECUTABLE_PATH_EMPTY");
    }

    fn write_fixture_runtime(resource_dir: &std::path::Path) -> BundledRuntimeLayout {
        let layout = BundledRuntimeLayout::from_resource_dir(resource_dir);
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
        fs::write(layout.hash_dir.join("fixture.xml"), b"<softwarelist/>").expect("hash");
        fs::write(
            layout.bgfx_dir.join("shaders").join("fixture.bin"),
            b"shader",
        )
        .expect("bgfx");
        fs::write(&layout.copying, b"license").expect("copying");
        fs::write(layout.legal_dir.join("GPL-2.0"), b"license").expect("legal");
        layout
    }
}
