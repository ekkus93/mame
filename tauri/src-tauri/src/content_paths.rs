use std::path::PathBuf;

use serde::Serialize;

use crate::config::{
    validate_content_path, ContentPathKind, ContentPathsV1, PathValidation, PlatformPath,
};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EffectiveContentPathSource {
    Configured,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveContentPathEntry {
    pub kind: ContentPathKind,
    pub source: EffectiveContentPathSource,
    pub validation: PathValidation,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveContentPaths {
    pub schema_version: u32,
    pub entries: Vec<EffectiveContentPathEntry>,
}

impl EffectiveContentPaths {
    pub fn configured_paths(&self, kind: ContentPathKind) -> impl Iterator<Item = &PlatformPath> {
        self.entries
            .iter()
            .filter(move |entry| entry.kind == kind)
            .map(|entry| &entry.validation.path)
    }

    pub fn media_search_paths(&self) -> Vec<&PlatformPath> {
        [
            ContentPathKind::Rom,
            ContentPathKind::Software,
            ContentPathKind::Chd,
        ]
        .into_iter()
        .flat_map(|kind| self.configured_paths(kind))
        .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn effective_content_paths(configured: &ContentPathsV1) -> EffectiveContentPaths {
    let mut entries = Vec::new();
    append_configured_paths(&mut entries, ContentPathKind::Rom, &configured.rom_paths);
    append_configured_paths(
        &mut entries,
        ContentPathKind::Software,
        &configured.software_paths,
    );
    append_configured_paths(&mut entries, ContentPathKind::Chd, &configured.chd_paths);

    EffectiveContentPaths {
        schema_version: 1,
        entries,
    }
}

fn append_configured_paths(
    entries: &mut Vec<EffectiveContentPathEntry>,
    kind: ContentPathKind,
    paths: &[PlatformPath],
) {
    entries.extend(paths.iter().map(|path| EffectiveContentPathEntry {
        kind,
        source: EffectiveContentPathSource::Configured,
        validation: validate_content_path(path),
    }));
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::{ContentPathKind, ContentPathsV1, PathValidationStatus, PlatformPath};

    use super::effective_content_paths;

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("mame-effective-paths-{label}-{nonce}"))
    }

    #[test]
    fn empty_settings_report_empty_effective_paths() {
        let effective = effective_content_paths(&ContentPathsV1::default());
        assert!(effective.is_empty());
        assert!(effective.media_search_paths().is_empty());
    }

    #[test]
    fn configured_paths_preserve_media_group_order() {
        let root = temp_root("order");
        let rom_a = root.join("rom-a");
        let rom_b = root.join("rom-b");
        let software = root.join("software");
        let chd = root.join("chd");
        for path in [&rom_a, &rom_b, &software, &chd] {
            fs::create_dir_all(path).expect("create configured media directory");
        }

        let configured = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(&rom_a), PlatformPath::new(&rom_b)],
            software_paths: vec![PlatformPath::new(&software)],
            chd_paths: vec![PlatformPath::new(&chd)],
        };
        let effective = effective_content_paths(&configured);

        assert_eq!(effective.entries.len(), 4);
        assert_eq!(effective.entries[0].kind, ContentPathKind::Rom);
        assert_eq!(effective.entries[1].kind, ContentPathKind::Rom);
        assert_eq!(effective.entries[2].kind, ContentPathKind::Software);
        assert_eq!(effective.entries[3].kind, ContentPathKind::Chd);
        assert_eq!(effective.media_search_paths()[0].as_path(), rom_a);
        assert_eq!(effective.media_search_paths()[1].as_path(), rom_b);
        assert_eq!(effective.media_search_paths()[2].as_path(), software);
        assert_eq!(effective.media_search_paths()[3].as_path(), chd);
        assert!(effective
            .entries
            .iter()
            .all(|entry| entry.validation.status == PathValidationStatus::Accessible));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn missing_configured_paths_remain_visible_with_validation_status() {
        let missing = temp_root("missing").join("roms");
        let configured = ContentPathsV1 {
            rom_paths: vec![PlatformPath::new(&missing)],
            software_paths: Vec::new(),
            chd_paths: Vec::new(),
        };
        let effective = effective_content_paths(&configured);

        assert_eq!(effective.entries.len(), 1);
        assert_eq!(effective.entries[0].kind, ContentPathKind::Rom);
        assert_eq!(effective.entries[0].validation.path.as_path(), missing);
        assert_eq!(
            effective.entries[0].validation.status,
            PathValidationStatus::Missing
        );
    }
}
