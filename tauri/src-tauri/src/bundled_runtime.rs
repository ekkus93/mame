use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use crate::{
    errors::{AppError, AppResult},
    mame::MameExecutableSource,
};

pub const BUNDLED_RUNTIME_DIR: &str = "mame-runtime";
pub const BUNDLED_HASH_DIR: &str = "hash";
pub const BUNDLED_BGFX_DIR: &str = "bgfx";
pub const BUNDLED_LICENSE_DIR: &str = "licenses";

#[cfg(target_os = "windows")]
pub const BUNDLED_MAME_EXECUTABLE: &str = "mame.exe";
#[cfg(not(target_os = "windows"))]
pub const BUNDLED_MAME_EXECUTABLE: &str = "mame";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledRuntimeLayout {
    pub resource_dir: PathBuf,
    pub root: PathBuf,
    pub executable: PathBuf,
    pub hash_dir: PathBuf,
    pub bgfx_dir: PathBuf,
    pub copying: PathBuf,
    pub legal_dir: PathBuf,
}

impl BundledRuntimeLayout {
    pub fn from_resource_dir(resource_dir: impl AsRef<Path>) -> Self {
        let resource_dir = resource_dir.as_ref().to_path_buf();
        let root = resource_dir.join(BUNDLED_RUNTIME_DIR);
        Self {
            executable: root.join("bin").join(BUNDLED_MAME_EXECUTABLE),
            hash_dir: root.join(BUNDLED_HASH_DIR),
            bgfx_dir: root.join(BUNDLED_BGFX_DIR),
            copying: root.join(BUNDLED_LICENSE_DIR).join("COPYING"),
            legal_dir: root.join(BUNDLED_LICENSE_DIR).join("legal"),
            resource_dir,
            root,
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        let resource_dir = canonical_directory(
            &self.resource_dir,
            "resourceDir",
            "MAME_BUNDLED_RESOURCE_DIR_INVALID",
        )?;
        let root = canonical_directory(
            &self.root,
            "runtimeRoot",
            "MAME_BUNDLED_RUNTIME_MISSING",
        )?;
        ensure_contained(&resource_dir, &root, "runtimeRoot")?;

        let executable = canonical_regular_file(
            &self.executable,
            "executable",
            "MAME_BUNDLED_EXECUTABLE_MISSING",
        )?;
        ensure_contained(&root, &executable, "executable")?;
        ensure_executable(&executable)?;

        let hash_dir = canonical_directory(
            &self.hash_dir,
            "hash",
            "MAME_BUNDLED_HASH_MISSING",
        )?;
        ensure_contained(&root, &hash_dir, "hash")?;
        if !directory_has_extension(&hash_dir, "xml").map_err(|error| {
            invalid_component(
                "MAME_BUNDLED_HASH_INVALID",
                "The bundled MAME hash directory could not be inspected.",
                "hash",
                &hash_dir,
                Some(error),
            )
        })? {
            return Err(invalid_component(
                "MAME_BUNDLED_HASH_EMPTY",
                "The bundled MAME hash directory does not contain software-list XML files.",
                "hash",
                &hash_dir,
                None,
            ));
        }

        let bgfx_dir = canonical_directory(
            &self.bgfx_dir,
            "bgfx",
            "MAME_BUNDLED_BGFX_MISSING",
        )?;
        ensure_contained(&root, &bgfx_dir, "bgfx")?;
        if !directory_has_regular_file_recursive(&bgfx_dir).map_err(|error| {
            invalid_component(
                "MAME_BUNDLED_BGFX_INVALID",
                "The bundled MAME BGFX resources could not be inspected.",
                "bgfx",
                &bgfx_dir,
                Some(error),
            )
        })? {
            return Err(invalid_component(
                "MAME_BUNDLED_BGFX_EMPTY",
                "The bundled MAME BGFX directory does not contain runtime resources.",
                "bgfx",
                &bgfx_dir,
                None,
            ));
        }

        let copying = canonical_regular_file(
            &self.copying,
            "COPYING",
            "MAME_BUNDLED_COPYING_MISSING",
        )?;
        ensure_contained(&root, &copying, "COPYING")?;

        let legal_dir = canonical_directory(
            &self.legal_dir,
            "legal",
            "MAME_BUNDLED_LEGAL_MISSING",
        )?;
        ensure_contained(&root, &legal_dir, "legal")?;
        if !directory_has_regular_file_recursive(&legal_dir).map_err(|error| {
            invalid_component(
                "MAME_BUNDLED_LEGAL_INVALID",
                "The bundled MAME legal directory could not be inspected.",
                "legal",
                &legal_dir,
                Some(error),
            )
        })? {
            return Err(invalid_component(
                "MAME_BUNDLED_LEGAL_EMPTY",
                "The bundled MAME legal directory is empty.",
                "legal",
                &legal_dir,
                None,
            ));
        }

        Ok(())
    }

    pub fn executable_source(&self) -> AppResult<MameExecutableSource> {
        self.validate()?;
        Ok(MameExecutableSource::bundled(self.executable.clone()))
    }
}

fn canonical_directory(path: &Path, component: &str, code: &str) -> AppResult<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        invalid_component(
            code,
            "A required bundled MAME runtime directory is unavailable.",
            component,
            path,
            Some(error),
        )
    })?;
    let metadata = fs::metadata(&canonical).map_err(|error| {
        invalid_component(
            code,
            "A required bundled MAME runtime directory could not be inspected.",
            component,
            &canonical,
            Some(error),
        )
    })?;
    if !metadata.is_dir() {
        return Err(invalid_component(
            code,
            "A required bundled MAME runtime path is not a directory.",
            component,
            &canonical,
            None,
        ));
    }
    Ok(canonical)
}

fn canonical_regular_file(path: &Path, component: &str, code: &str) -> AppResult<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        invalid_component(
            code,
            "A required bundled MAME runtime file is unavailable.",
            component,
            path,
            Some(error),
        )
    })?;
    let metadata = fs::metadata(&canonical).map_err(|error| {
        invalid_component(
            code,
            "A required bundled MAME runtime file could not be inspected.",
            component,
            &canonical,
            Some(error),
        )
    })?;
    if !metadata.is_file() {
        return Err(invalid_component(
            code,
            "A required bundled MAME runtime path is not a regular file.",
            component,
            &canonical,
            None,
        ));
    }
    Ok(canonical)
}

fn ensure_contained(root: &Path, candidate: &Path, component: &str) -> AppResult<()> {
    if candidate.starts_with(root) {
        return Ok(());
    }
    Err(invalid_component(
        "MAME_BUNDLED_RUNTIME_PATH_ESCAPE",
        "A bundled MAME runtime path resolves outside its package-owned resource root.",
        component,
        candidate,
        None,
    ))
}

fn ensure_executable(path: &Path) -> AppResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path)
            .map_err(|error| {
                invalid_component(
                    "MAME_BUNDLED_EXECUTABLE_INVALID",
                    "The bundled MAME executable could not be inspected.",
                    "executable",
                    path,
                    Some(error),
                )
            })?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            return Err(invalid_component(
                "MAME_BUNDLED_EXECUTABLE_NOT_EXECUTABLE",
                "The bundled MAME executable is not marked executable.",
                "executable",
                path,
                None,
            ));
        }
    }
    Ok(())
}

fn directory_has_extension(path: &Path, extension: &str) -> io::Result<bool> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn directory_has_regular_file_recursive(path: &Path) -> io::Result<bool> {
    let mut pending = vec![path.to_path_buf()];
    let mut visited = 0_usize;
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            visited += 1;
            if visited > 100_000 {
                return Ok(false);
            }
            let file_type = entry.file_type()?;
            if file_type.is_file() {
                return Ok(true);
            }
            if file_type.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Ok(false)
}

fn invalid_component(
    code: &str,
    message: &str,
    component: &str,
    path: &Path,
    cause: Option<io::Error>,
) -> AppError {
    let mut details = serde_json::json!({
        "component": component,
        "path": path,
    });
    if let Some(error) = cause {
        details["cause"] = serde_json::Value::String(error.to_string());
    }
    AppError::new(code, message).with_details(details)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{BundledRuntimeLayout, BUNDLED_MAME_EXECUTABLE};
    use crate::mame::{MameExecutableSourceKind, MameExecutableTrust};

    fn create_valid_layout() -> (tempfile::TempDir, BundledRuntimeLayout) {
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
            .expect("hash fixture");
        fs::write(layout.bgfx_dir.join("shaders").join("fixture.bin"), b"shader")
            .expect("bgfx fixture");
        fs::write(&layout.copying, b"MAME license summary").expect("COPYING");
        fs::write(layout.legal_dir.join("GPL-2.0"), b"GPL-2.0").expect("legal fixture");
        (temp, layout)
    }

    #[test]
    fn valid_installed_layout_resolves_only_the_bundled_executable() {
        let (_temp, layout) = create_valid_layout();
        layout.validate().expect("valid bundled runtime");
        let source = layout.executable_source().expect("bundled source");
        assert_eq!(source.kind(), MameExecutableSourceKind::Bundled);
        assert_eq!(source.trust(), MameExecutableTrust::QualifiedBundled);
        assert_eq!(source.path(), layout.executable.as_path());
        assert_eq!(
            layout.executable.file_name().and_then(|value| value.to_str()),
            Some(BUNDLED_MAME_EXECUTABLE)
        );
    }

    #[test]
    fn missing_hash_resources_fail_closed() {
        let (_temp, layout) = create_valid_layout();
        fs::remove_dir_all(&layout.hash_dir).expect("remove hash dir");
        let error = layout.validate().expect_err("missing hash must fail");
        assert_eq!(error.code, "MAME_BUNDLED_HASH_MISSING");
    }

    #[test]
    fn missing_license_material_fails_closed() {
        let (_temp, layout) = create_valid_layout();
        fs::remove_dir_all(&layout.legal_dir).expect("remove legal dir");
        let error = layout.validate().expect_err("missing legal dir must fail");
        assert_eq!(error.code, "MAME_BUNDLED_LEGAL_MISSING");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_runtime_component_cannot_escape_package_root() {
        use std::os::unix::fs::symlink;

        let (temp, layout) = create_valid_layout();
        fs::remove_dir_all(&layout.hash_dir).expect("remove staged hash");
        let escaped = temp.path().join("outside-hash");
        fs::create_dir_all(&escaped).expect("outside hash");
        fs::write(escaped.join("fixture.xml"), b"<softwarelist/>").expect("outside fixture");
        symlink(&escaped, &layout.hash_dir).expect("hash symlink");

        let error = layout.validate().expect_err("escaped hash must fail");
        assert_eq!(error.code, "MAME_BUNDLED_RUNTIME_PATH_ESCAPE");
    }
}
