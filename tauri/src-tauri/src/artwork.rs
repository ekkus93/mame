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

#[cfg(test)]
mod tests {
    use super::{ArtworkKind, ArtworkProvenance, ArtworkProvenanceKind};

    #[test]
    fn artwork_model_serializes_all_required_mt801_categories() {
        let kinds = [
            ArtworkKind::Screenshot,
            ArtworkKind::Cabinet,
            ArtworkKind::Marquee,
            ArtworkKind::Flyer,
            ArtworkKind::Icon,
            ArtworkKind::SystemImage,
        ];

        assert_eq!(
            serde_json::to_value(kinds).expect("serialize artwork kinds"),
            serde_json::json!([
                "screenshot",
                "cabinet",
                "marquee",
                "flyer",
                "icon",
                "systemImage"
            ])
        );
    }

    #[test]
    fn local_provenance_exposes_root_ordinal_without_a_host_path() {
        let provenance = ArtworkProvenance {
            kind: ArtworkProvenanceKind::LocalFile,
            root_index: 2,
        };

        assert_eq!(
            serde_json::to_value(provenance).expect("serialize artwork provenance"),
            serde_json::json!({ "kind": "localFile", "rootIndex": 2 })
        );
    }
}
