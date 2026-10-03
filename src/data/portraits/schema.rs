//! Serializable portrait IDs, descriptors, and optional authored layer references.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const APPEARANCE_SCHEMA_VERSION: u32 = 1;
pub const PORTRAIT_CATALOG_SCHEMA_VERSION: u32 = 1;
pub const BALD_HAIR_PALETTE_ID: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceDescriptor {
    pub schema_version: u32,
    pub catalog_revision: u32,
    pub allocation_revision: u32,
    pub rig_id: String,
    pub face_id: String,
    pub nose_id: String,
    pub eyes_id: String,
    pub hair_id: String,
    pub skin_palette_id: String,
    pub hair_palette_id: String,
    pub eye_palette_id: String,
    /// Full canonical visual key, not a short hash. Catalog validation recomputes it.
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceSignature {
    pub rig_visual_key: String,
    pub face_visual_key: String,
    pub nose_visual_key: String,
    pub eyes_visual_key: String,
    pub hair_visual_key: String,
    pub skin_visual_key: String,
    pub hair_palette_visual_key: Option<String>,
    pub eye_palette_visual_key: String,
}

impl AppearanceSignature {
    pub fn canonical_key(&self) -> String {
        let fields = [
            Some(self.rig_visual_key.as_str()),
            Some(self.face_visual_key.as_str()),
            Some(self.nose_visual_key.as_str()),
            Some(self.eyes_visual_key.as_str()),
            Some(self.hair_visual_key.as_str()),
            Some(self.skin_visual_key.as_str()),
            self.hair_palette_visual_key.as_deref(),
            Some(self.eye_palette_visual_key.as_str()),
        ];
        fields
            .into_iter()
            .map(|field| match field {
                Some(value) => format!("{}:{value}", value.len()),
                None => "-".to_owned(),
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortraitRig {
    pub id: String,
    pub visual_key: String,
    pub width: u16,
    pub height: u16,
    /// Left, top, right, bottom in untrimmed canvas coordinates.
    pub safe_rect: [u16; 4],
    pub anchors: BTreeMap<String, [u16; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaceFeature {
    pub id: String,
    pub visual_key: String,
    pub silhouette_group: String,
    #[serde(default)]
    pub assets: Option<ShadedArtAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoseFeature {
    pub id: String,
    pub visual_key: String,
    #[serde(default)]
    pub fits: BTreeMap<String, MaskedArtAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EyeFeature {
    pub id: String,
    pub visual_key: String,
    #[serde(default)]
    pub fits: BTreeMap<String, EyeFitAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HairFeature {
    pub id: String,
    pub visual_key: String,
    pub silhouette_group: String,
    pub is_bald: bool,
    #[serde(default)]
    pub fits: BTreeMap<String, HairFitAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteRamp {
    pub id: String,
    pub visual_key: String,
    /// Catalog-authored lightness group used by the near-duplicate policy.
    pub value_group: String,
    pub shadow: [u8; 3],
    pub base: [u8; 3],
    pub highlight: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryTuple {
    pub face_id: String,
    pub nose_id: String,
    pub eyes_id: String,
    pub hair_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadedArtAssets {
    pub base: MaskedArtAssets,
    #[serde(default)]
    pub shadow: Option<MaskedArtAssets>,
    #[serde(default)]
    pub highlight: Option<MaskedArtAssets>,
    #[serde(default)]
    pub ink: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskedArtAssets {
    pub source: String,
    pub mask: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EyeFitAssets {
    pub whites: String,
    pub iris_source: String,
    pub iris_mask: String,
    pub ink: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HairFitAssets {
    pub rear: ShadedArtAssets,
    pub front: ShadedArtAssets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportingArtAssets {
    pub neck: ShadedArtAssets,
    pub shoulders: ShadedArtAssets,
    pub child_silhouette: String,
    pub unknown_silhouette: String,
    pub adult_fallback: String,
}
