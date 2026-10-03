//! Versioned feature catalogs and deterministic candidate enumeration.

mod index;
mod validation;

pub use index::PortraitCatalogIndex;

use super::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PORTRAIT_CATALOG_SOURCE: &str = "assets/data/portrait_catalog.json";
const MAX_CANDIDATE_COUNT: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortraitCatalog {
    pub schema_version: u32,
    pub catalog_revision: u32,
    pub allocation_revision: u32,
    pub supported_catalog_revisions: Vec<u32>,
    pub supported_allocation_revisions: Vec<u32>,
    pub rig: PortraitRig,
    pub faces: Vec<FaceFeature>,
    pub noses: Vec<NoseFeature>,
    pub eyes: Vec<EyeFeature>,
    pub hair: Vec<HairFeature>,
    pub skin_palettes: Vec<PaletteRamp>,
    pub hair_palettes: Vec<PaletteRamp>,
    pub eye_palettes: Vec<PaletteRamp>,
    pub geometry: Vec<GeometryTuple>,
    #[serde(default)]
    pub supporting: Option<SupportingArtAssets>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CandidateGeometry<'a> {
    pub face: &'a FaceFeature,
    pub nose: &'a NoseFeature,
    pub eyes: &'a EyeFeature,
    pub hair: &'a HairFeature,
}

pub(crate) struct PortraitCandidateSpace<'a> {
    catalog: &'a PortraitCatalog,
    geometry: Vec<CandidateGeometry<'a>>,
    geometry_ends: Vec<usize>,
    skin_palettes: Vec<&'a PaletteRamp>,
    eye_palettes: Vec<&'a PaletteRamp>,
    hair_palettes: Vec<Vec<&'a PaletteRamp>>,
    candidate_count: usize,
}

impl PortraitCandidateSpace<'_> {
    pub(crate) fn len(&self) -> usize {
        self.candidate_count
    }

    pub(crate) fn candidate_at(&self, index: usize) -> Result<AppearanceDescriptor, String> {
        if index >= self.candidate_count {
            return Err("portrait candidate index is outside the catalog's legal space".into());
        }
        let geometry_index = self.geometry_ends.partition_point(|end| *end <= index);
        let range_start = geometry_index
            .checked_sub(1)
            .and_then(|previous| self.geometry_ends.get(previous).copied())
            .unwrap_or(0);
        let local = index - range_start;
        let tuple = self.geometry[geometry_index];
        let hair_palette_choices = &self.hair_palettes[geometry_index];
        let hair_index = local % hair_palette_choices.len();
        let eye_index = (local / hair_palette_choices.len()) % self.eye_palettes.len();
        let skin_index = local / (hair_palette_choices.len() * self.eye_palettes.len());
        let hair_palette = hair_palette_choices[hair_index];
        let eye_palette = self.eye_palettes[eye_index];
        let skin_palette = self.skin_palettes[skin_index];
        let signature = AppearanceSignature {
            rig_visual_key: self.catalog.rig.visual_key.clone(),
            face_visual_key: tuple.face.visual_key.clone(),
            nose_visual_key: tuple.nose.visual_key.clone(),
            eyes_visual_key: tuple.eyes.visual_key.clone(),
            hair_visual_key: tuple.hair.visual_key.clone(),
            skin_visual_key: skin_palette.visual_key.clone(),
            hair_palette_visual_key: (!tuple.hair.is_bald).then(|| hair_palette.visual_key.clone()),
            eye_palette_visual_key: eye_palette.visual_key.clone(),
        };
        Ok(AppearanceDescriptor {
            schema_version: APPEARANCE_SCHEMA_VERSION,
            catalog_revision: self.catalog.catalog_revision,
            allocation_revision: self.catalog.allocation_revision,
            rig_id: self.catalog.rig.id.clone(),
            face_id: tuple.face.id.clone(),
            nose_id: tuple.nose.id.clone(),
            eyes_id: tuple.eyes.id.clone(),
            hair_id: tuple.hair.id.clone(),
            skin_palette_id: skin_palette.id.clone(),
            hair_palette_id: hair_palette.id.clone(),
            eye_palette_id: eye_palette.id.clone(),
            signature: signature.canonical_key(),
        })
    }
}

impl PortraitCatalog {
    /// Decode the immutable v1 catalog used when an old v2 save is first opened.
    pub fn load_frozen() -> Result<Self, String> {
        let catalog: Self = macroquad_toolkit::include_json!(
            "../../../assets/data/portrait_catalog_legacy_v1.json"
        )?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub(crate) fn candidate_count(&self) -> Result<usize, String> {
        Ok(self.candidate_space()?.len())
    }

    pub(crate) fn candidate_space(&self) -> Result<PortraitCandidateSpace<'_>, String> {
        let geometry = self.candidate_geometry()?;
        let skin_palettes = unique_by_visual_key(&self.skin_palettes);
        let eye_palettes = unique_by_visual_key(&self.eye_palettes);
        let unique_hair_palettes = unique_by_visual_key(&self.hair_palettes);
        let mut hair_palettes = Vec::with_capacity(geometry.len());
        let mut geometry_ends = Vec::with_capacity(geometry.len());
        let mut total = 0_usize;
        for tuple in &geometry {
            let selected_hair = if tuple.hair.is_bald {
                vec![find_by_id(
                    &self.hair_palettes,
                    BALD_HAIR_PALETTE_ID,
                    "hair palette",
                )?]
            } else {
                unique_hair_palettes
                    .iter()
                    .copied()
                    .filter(|palette| palette.id != BALD_HAIR_PALETTE_ID)
                    .collect()
            };
            let count = skin_palettes
                .len()
                .checked_mul(eye_palettes.len())
                .and_then(|count| count.checked_mul(selected_hair.len()))
                .ok_or_else(|| "portrait catalog candidate count overflow".to_owned())?;
            total = total
                .checked_add(count)
                .ok_or_else(|| "portrait catalog candidate count overflow".to_owned())?;
            geometry_ends.push(total);
            hair_palettes.push(selected_hair);
        }
        if total == 0 {
            return Err("portrait catalog contains no unique compatible appearances".into());
        }
        if total > MAX_CANDIDATE_COUNT || total > u32::MAX as usize {
            return Err("portrait catalog legal space exceeds the bounded allocator limit".into());
        }
        Ok(PortraitCandidateSpace {
            catalog: self,
            geometry,
            geometry_ends,
            skin_palettes,
            eye_palettes,
            hair_palettes,
            candidate_count: total,
        })
    }

    fn candidate_geometry(&self) -> Result<Vec<CandidateGeometry<'_>>, String> {
        let mut resolved = Vec::with_capacity(self.geometry.len());
        for tuple in &self.geometry {
            resolved.push(CandidateGeometry {
                face: find_by_id(&self.faces, &tuple.face_id, "face")?,
                nose: find_by_id(&self.noses, &tuple.nose_id, "nose")?,
                eyes: find_by_id(&self.eyes, &tuple.eyes_id, "eyes")?,
                hair: find_by_id(&self.hair, &tuple.hair_id, "hair")?,
            });
        }
        resolved.sort_by_key(|tuple| {
            (
                tuple.face.visual_key.as_str(),
                tuple.nose.visual_key.as_str(),
                tuple.eyes.visual_key.as_str(),
                tuple.hair.visual_key.as_str(),
                tuple.face.id.as_str(),
                tuple.nose.id.as_str(),
                tuple.eyes.id.as_str(),
                tuple.hair.id.as_str(),
            )
        });
        resolved.dedup_by(|left, right| {
            left.face.visual_key == right.face.visual_key
                && left.nose.visual_key == right.nose.visual_key
                && left.eyes.visual_key == right.eyes.visual_key
                && left.hair.visual_key == right.hair.visual_key
        });
        Ok(resolved)
    }
}

pub(super) trait FeatureEntry {
    fn id(&self) -> &str;
    fn visual_key(&self) -> &str;
}

impl FeatureEntry for FaceFeature {
    fn id(&self) -> &str {
        &self.id
    }
    fn visual_key(&self) -> &str {
        &self.visual_key
    }
}
impl FeatureEntry for NoseFeature {
    fn id(&self) -> &str {
        &self.id
    }
    fn visual_key(&self) -> &str {
        &self.visual_key
    }
}
impl FeatureEntry for EyeFeature {
    fn id(&self) -> &str {
        &self.id
    }
    fn visual_key(&self) -> &str {
        &self.visual_key
    }
}
impl FeatureEntry for HairFeature {
    fn id(&self) -> &str {
        &self.id
    }
    fn visual_key(&self) -> &str {
        &self.visual_key
    }
}
impl FeatureEntry for PaletteRamp {
    fn id(&self) -> &str {
        &self.id
    }
    fn visual_key(&self) -> &str {
        &self.visual_key
    }
}

pub(super) fn ids<T: FeatureEntry>(entries: &[T]) -> BTreeSet<&str> {
    entries.iter().map(FeatureEntry::id).collect()
}

pub(super) fn feature_map<T: FeatureEntry>(entries: &[T]) -> BTreeMap<&str, &T> {
    entries.iter().map(|entry| (entry.id(), entry)).collect()
}

pub(super) fn find_by_id<'a, T: FeatureEntry>(
    entries: &'a [T],
    id: &str,
    label: &str,
) -> Result<&'a T, String> {
    entries
        .iter()
        .find(|entry| entry.id() == id)
        .ok_or_else(|| format!("appearance: unknown {label} ID '{id}'"))
}

fn unique_by_visual_key<T: FeatureEntry>(entries: &[T]) -> Vec<&T> {
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by_key(|entry| (entry.visual_key(), entry.id()));
    sorted.dedup_by(|left, right| left.visual_key() == right.visual_key());
    sorted
}
