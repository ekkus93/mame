//! Typed artwork metadata shared by local discovery and the frontend.
//!
//! Artwork is presentation data, not a privileged filesystem API. Public
//! descriptors therefore identify the logical artwork asset and its provenance
//! without exposing a configured root or canonical host path to the WebView.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkKind {
    Screenshot,
    Cabinet,
    Marquee,
    Flyer,
    Icon,
    SystemImage,
}

impl ArtworkKind {
    pub(crate) const ALL: [Self; 6] = [
        Self::Screenshot,
        Self::Cabinet,
        Self::Marquee,
        Self::Flyer,
        Self::Icon,
        Self::SystemImage,
    ];

    pub(crate) fn directory_name(self) -> &'static str {
        match self {
            Self::Screenshot => "snap",
            Self::Cabinet => "cabinets",
            Self::Marquee => "marquees",
            Self::Flyer => "flyers",
            Self::Icon => "icons",
            Self::SystemImage => "systems",
        }
    }

    pub(crate) fn asset_token(self) -> &'static str {
        match self {
            Self::Screenshot => "screenshot",
            Self::Cabinet => "cabinet",
            Self::Marquee => "marquee",
            Self::Flyer => "flyer",
            Self::Icon => "icon",
            Self::SystemImage => "systemImage",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkProvenanceKind {
    LocalFile,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkProvenance {
    pub kind: ArtworkProvenanceKind,
    /// Stable configured-root ordinal. The root path itself is intentionally
    /// not included in the public descriptor.
    pub root_index: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkDescriptor {
    pub schema_version: u32,
    pub asset_id: String,
    pub machine: String,
    pub kind: ArtworkKind,
    pub mime_type: String,
    pub bytes: u64,
    pub provenance: ArtworkProvenance,
}

pub(crate) fn local_asset_id(machine: &str, kind: ArtworkKind, root_index: u32) -> String {
    format!("local:{root_index}:{machine}:{}", kind.asset_token())
}

#[cfg(test)]
mod tests {
    use super::{local_asset_id, ArtworkKind};

    #[test]
    fn artwork_model_covers_all_required_mt801_categories() {
        assert_eq!(
            ArtworkKind::ALL.map(ArtworkKind::asset_token),
            [
                "screenshot",
                "cabinet",
                "marquee",
                "flyer",
                "icon",
                "systemImage"
            ]
        );
    }

    #[test]
    fn local_asset_identity_contains_no_filesystem_path() {
        let id = local_asset_id("pacman", ArtworkKind::Screenshot, 2);
        assert_eq!(id, "local:2:pacman:screenshot");
        assert!(!id.contains('/'));
        assert!(!id.contains('\\'));
    }

    #[test]
    fn conventional_local_directory_names_are_explicit() {
        assert_eq!(ArtworkKind::Screenshot.directory_name(), "snap");
        assert_eq!(ArtworkKind::Cabinet.directory_name(), "cabinets");
        assert_eq!(ArtworkKind::Marquee.directory_name(), "marquees");
        assert_eq!(ArtworkKind::Flyer.directory_name(), "flyers");
        assert_eq!(ArtworkKind::Icon.directory_name(), "icons");
        assert_eq!(ArtworkKind::SystemImage.directory_name(), "systems");
    }
}
