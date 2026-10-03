//! Per-validation catalog lookup tables and canonical appearance identity.

use super::{feature_map, PortraitCatalog};
use crate::data::portraits::{
    AppearanceDescriptor, AppearanceSignature, EyeFeature, FaceFeature, HairFeature, NoseFeature,
    PaletteRamp, APPEARANCE_SCHEMA_VERSION, BALD_HAIR_PALETTE_ID,
};
use std::collections::{BTreeMap, BTreeSet};

/// Per-call lookup tables for validating campaign descriptors and reservations.
pub struct PortraitCatalogIndex<'a> {
    catalog: &'a PortraitCatalog,
    faces: BTreeMap<&'a str, &'a FaceFeature>,
    noses: BTreeMap<&'a str, &'a NoseFeature>,
    eyes: BTreeMap<&'a str, &'a EyeFeature>,
    hair: BTreeMap<&'a str, &'a HairFeature>,
    skin_palettes: BTreeMap<&'a str, &'a PaletteRamp>,
    hair_palettes: BTreeMap<&'a str, &'a PaletteRamp>,
    eye_palettes: BTreeMap<&'a str, &'a PaletteRamp>,
    compatible_ids: BTreeSet<(&'a str, &'a str, &'a str, &'a str)>,
    compatible_visuals: BTreeSet<(&'a str, &'a str, &'a str, &'a str)>,
}

impl PortraitCatalog {
    pub fn validation_index(&self) -> Result<PortraitCatalogIndex<'_>, String> {
        let faces = feature_map(&self.faces);
        let noses = feature_map(&self.noses);
        let eyes = feature_map(&self.eyes);
        let hair = feature_map(&self.hair);
        let skin_palettes = feature_map(&self.skin_palettes);
        let hair_palettes = feature_map(&self.hair_palettes);
        let eye_palettes = feature_map(&self.eye_palettes);
        let mut compatible_ids = BTreeSet::new();
        let mut compatible_visuals = BTreeSet::new();
        for tuple in &self.geometry {
            let face = faces
                .get(tuple.face_id.as_str())
                .ok_or_else(|| "portrait catalog index: unknown face in geometry".to_owned())?;
            let nose = noses
                .get(tuple.nose_id.as_str())
                .ok_or_else(|| "portrait catalog index: unknown nose in geometry".to_owned())?;
            let eyes_entry = eyes
                .get(tuple.eyes_id.as_str())
                .ok_or_else(|| "portrait catalog index: unknown eyes in geometry".to_owned())?;
            let hair_entry = hair
                .get(tuple.hair_id.as_str())
                .ok_or_else(|| "portrait catalog index: unknown hair in geometry".to_owned())?;
            compatible_ids.insert((
                face.id.as_str(),
                nose.id.as_str(),
                eyes_entry.id.as_str(),
                hair_entry.id.as_str(),
            ));
            compatible_visuals.insert((
                face.visual_key.as_str(),
                nose.visual_key.as_str(),
                eyes_entry.visual_key.as_str(),
                hair_entry.visual_key.as_str(),
            ));
        }
        Ok(PortraitCatalogIndex {
            catalog: self,
            faces,
            noses,
            eyes,
            hair,
            skin_palettes,
            hair_palettes,
            eye_palettes,
            compatible_ids,
            compatible_visuals,
        })
    }

    pub fn validate_descriptor(&self, appearance: &AppearanceDescriptor) -> Result<(), String> {
        self.validation_index()?.validate_descriptor(appearance)
    }

    pub fn signature_for(&self, appearance: &AppearanceDescriptor) -> Result<String, String> {
        self.validation_index()?.signature_for(appearance)
    }

    pub fn signature_parts(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<AppearanceSignature, String> {
        self.validation_index()?.signature_parts(appearance)
    }
}

impl PortraitCatalogIndex<'_> {
    pub fn validate_descriptor(&self, appearance: &AppearanceDescriptor) -> Result<(), String> {
        if appearance.schema_version != APPEARANCE_SCHEMA_VERSION {
            return Err(format!(
                "appearance: unsupported descriptor schema {}",
                appearance.schema_version
            ));
        }
        if !self
            .catalog
            .supported_catalog_revisions
            .contains(&appearance.catalog_revision)
        {
            return Err(format!(
                "appearance: unsupported catalog revision {}",
                appearance.catalog_revision
            ));
        }
        if !self
            .catalog
            .supported_allocation_revisions
            .contains(&appearance.allocation_revision)
        {
            return Err(format!(
                "appearance: unsupported allocation revision {}",
                appearance.allocation_revision
            ));
        }
        let signature = self.signature_for(appearance)?;
        if signature != appearance.signature {
            return Err(
                "appearance: stored canonical signature does not match its feature IDs".into(),
            );
        }
        Ok(())
    }

    pub fn validate_signature(&self, signature: &AppearanceSignature) -> Result<(), String> {
        let nonempty = [
            signature.rig_visual_key.as_str(),
            signature.face_visual_key.as_str(),
            signature.nose_visual_key.as_str(),
            signature.eyes_visual_key.as_str(),
            signature.hair_visual_key.as_str(),
            signature.skin_visual_key.as_str(),
            signature.eye_palette_visual_key.as_str(),
        ]
        .into_iter()
        .all(|value| !value.trim().is_empty());
        let non_bald_hair_palette =
            signature
                .hair_palette_visual_key
                .as_deref()
                .is_some_and(|visual_key| {
                    visual_key != "none"
                        && self
                            .hair_palettes
                            .values()
                            .any(|palette| palette.visual_key == visual_key)
                });
        let bald_hair = self
            .hair
            .values()
            .any(|feature| feature.visual_key == signature.hair_visual_key && feature.is_bald);
        let hair_palette_matches = if bald_hair {
            signature.hair_palette_visual_key.is_none()
        } else {
            non_bald_hair_palette
        };
        if !nonempty
            || signature.rig_visual_key != self.catalog.rig.visual_key
            || !self
                .faces
                .values()
                .any(|feature| feature.visual_key == signature.face_visual_key)
            || !self
                .noses
                .values()
                .any(|feature| feature.visual_key == signature.nose_visual_key)
            || !self
                .eyes
                .values()
                .any(|feature| feature.visual_key == signature.eyes_visual_key)
            || !self
                .hair
                .values()
                .any(|feature| feature.visual_key == signature.hair_visual_key)
            || !self
                .skin_palettes
                .values()
                .any(|palette| palette.visual_key == signature.skin_visual_key)
            || !self
                .eye_palettes
                .values()
                .any(|palette| palette.visual_key == signature.eye_palette_visual_key)
            || !hair_palette_matches
            || !self.compatible_visuals.contains(&(
                signature.face_visual_key.as_str(),
                signature.nose_visual_key.as_str(),
                signature.eyes_visual_key.as_str(),
                signature.hair_visual_key.as_str(),
            ))
        {
            return Err(
                "appearance_registry: signature references unsupported or incompatible content"
                    .into(),
            );
        }
        Ok(())
    }

    pub fn signature_for(&self, appearance: &AppearanceDescriptor) -> Result<String, String> {
        Ok(self.signature_parts(appearance)?.canonical_key())
    }

    pub fn signature_parts(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<AppearanceSignature, String> {
        let face = self
            .faces
            .get(appearance.face_id.as_str())
            .copied()
            .ok_or_else(|| format!("appearance: unknown face ID '{}'", appearance.face_id))?;
        let nose = self
            .noses
            .get(appearance.nose_id.as_str())
            .copied()
            .ok_or_else(|| format!("appearance: unknown nose ID '{}'", appearance.nose_id))?;
        let eyes = self
            .eyes
            .get(appearance.eyes_id.as_str())
            .copied()
            .ok_or_else(|| format!("appearance: unknown eyes ID '{}'", appearance.eyes_id))?;
        let hair = self
            .hair
            .get(appearance.hair_id.as_str())
            .copied()
            .ok_or_else(|| format!("appearance: unknown hair ID '{}'", appearance.hair_id))?;
        let skin = self
            .skin_palettes
            .get(appearance.skin_palette_id.as_str())
            .copied()
            .ok_or_else(|| "appearance: unknown skin palette ID".to_owned())?;
        let hair_palette = self
            .hair_palettes
            .get(appearance.hair_palette_id.as_str())
            .copied()
            .ok_or_else(|| "appearance: unknown hair palette ID".to_owned())?;
        let eye_palette = self
            .eye_palettes
            .get(appearance.eye_palette_id.as_str())
            .copied()
            .ok_or_else(|| "appearance: unknown eye palette ID".to_owned())?;
        if appearance.rig_id != self.catalog.rig.id {
            return Err("appearance: unknown portrait rig".into());
        }
        if !self.compatible_ids.contains(&(
            appearance.face_id.as_str(),
            appearance.nose_id.as_str(),
            appearance.eyes_id.as_str(),
            appearance.hair_id.as_str(),
        )) {
            return Err("appearance: feature tuple is not compatible".into());
        }
        if hair.is_bald != (hair_palette.id == BALD_HAIR_PALETTE_ID) {
            return Err("appearance: bald hair must use the canonical none palette".into());
        }
        Ok(AppearanceSignature {
            rig_visual_key: self.catalog.rig.visual_key.clone(),
            face_visual_key: face.visual_key.clone(),
            nose_visual_key: nose.visual_key.clone(),
            eyes_visual_key: eyes.visual_key.clone(),
            hair_visual_key: hair.visual_key.clone(),
            skin_visual_key: skin.visual_key.clone(),
            hair_palette_visual_key: (!hair.is_bald).then(|| hair_palette.visual_key.clone()),
            eye_palette_visual_key: eye_palette.visual_key.clone(),
        })
    }

    pub fn distinctiveness(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<[String; 4], String> {
        let parts = self.signature_parts(appearance)?;
        let face = self.faces[appearance.face_id.as_str()];
        let hair = self.hair[appearance.hair_id.as_str()];
        let hair_color = self.hair_palettes[appearance.hair_palette_id.as_str()];
        let skin = self.skin_palettes[appearance.skin_palette_id.as_str()];
        Ok([
            face.silhouette_group.clone(),
            hair.silhouette_group.clone(),
            if parts.hair_palette_visual_key.is_none() {
                "none".to_owned()
            } else {
                hair_color.value_group.clone()
            },
            skin.value_group.clone(),
        ])
    }
}
