//! Resolves catalog IDs to a complete, validated set of authored layer paths.

use crate::data::portraits::{
    AppearanceDescriptor, AppearanceSignature, EyeFeature, EyeFitAssets, FaceFeature, HairFeature,
    HairFitAssets, MaskedArtAssets, NoseFeature, PaletteRamp, PortraitCatalog, ShadedArtAssets,
    SupportingArtAssets, APPEARANCE_SCHEMA_VERSION, BALD_HAIR_PALETTE_ID,
};
use std::collections::BTreeSet;

pub(super) const PORTRAIT_SIZE: u16 = super::compose::PORTRAIT_SIZE;

#[derive(Clone)]
pub(super) struct PortraitAssetManifest {
    pub catalog: PortraitCatalog,
    pub paths: Vec<String>,
    pub enabled: bool,
}

pub(super) struct ResolvedAppearance<'a> {
    pub face: &'a ShadedArtAssets,
    pub nose: &'a MaskedArtAssets,
    pub eyes: &'a EyeFitAssets,
    pub hair: Option<&'a HairFitAssets>,
    pub skin: &'a PaletteRamp,
    pub hair_palette: &'a PaletteRamp,
    pub eye_palette: &'a PaletteRamp,
}

impl PortraitAssetManifest {
    pub fn from_catalog(catalog: &PortraitCatalog) -> Result<Self, String> {
        catalog.validate()?;
        let enabled = has_art(catalog);
        if !enabled {
            return Ok(Self {
                catalog: catalog.clone(),
                paths: Vec::new(),
                enabled: false,
            });
        }
        validate_art_coverage(catalog)?;
        let supporting = catalog
            .supporting
            .as_ref()
            .ok_or_else(|| "portrait catalog: art requires supporting assets".to_owned())?;
        let mut paths = BTreeSet::new();
        add_shaded_paths(&mut paths, &supporting.neck);
        add_shaded_paths(&mut paths, &supporting.shoulders);
        paths.extend([
            supporting.adult_fallback.clone(),
            supporting.child_silhouette.clone(),
            supporting.unknown_silhouette.clone(),
        ]);
        for face in &catalog.faces {
            if let Some(assets) = &face.assets {
                add_shaded_paths(&mut paths, assets);
            }
        }
        for nose in &catalog.noses {
            for assets in nose.fits.values() {
                add_masked_paths(&mut paths, assets);
            }
        }
        for eyes in &catalog.eyes {
            for assets in eyes.fits.values() {
                paths.extend([
                    assets.whites.clone(),
                    assets.iris_source.clone(),
                    assets.iris_mask.clone(),
                    assets.ink.clone(),
                ]);
            }
        }
        for hair in &catalog.hair {
            for assets in hair.fits.values() {
                add_shaded_paths(&mut paths, &assets.rear);
                add_shaded_paths(&mut paths, &assets.front);
            }
        }
        if paths.len() > 4096 {
            return Err("portrait catalog: art manifest exceeds 4096 unique files".into());
        }
        for path in &paths {
            if !path.ends_with(".png") {
                return Err(format!(
                    "portrait catalog: rendered layer '{path}' must be a PNG"
                ));
            }
        }
        Ok(Self {
            catalog: catalog.clone(),
            paths: paths.into_iter().collect(),
            enabled: true,
        })
    }

    pub fn resolve(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<ResolvedAppearance<'_>, String> {
        if !self.enabled {
            return Err("portrait catalog has identity metadata but no art".into());
        }
        let catalog = &self.catalog;
        if appearance.schema_version != APPEARANCE_SCHEMA_VERSION
            || !catalog
                .supported_catalog_revisions
                .contains(&appearance.catalog_revision)
            || !catalog
                .supported_allocation_revisions
                .contains(&appearance.allocation_revision)
        {
            return Err("portrait descriptor uses an unsupported revision".into());
        }
        if appearance.rig_id != catalog.rig.id {
            return Err("portrait descriptor uses an unknown rig".into());
        }
        let face = by_id(&catalog.faces, &appearance.face_id, "face")?;
        let nose = by_id(&catalog.noses, &appearance.nose_id, "nose")?;
        let eyes = by_id(&catalog.eyes, &appearance.eyes_id, "eyes")?;
        let hair = by_id(&catalog.hair, &appearance.hair_id, "hair")?;
        let skin_palette = by_id(
            &catalog.skin_palettes,
            &appearance.skin_palette_id,
            "skin palette",
        )?;
        let hair_palette = by_id(
            &catalog.hair_palettes,
            &appearance.hair_palette_id,
            "hair palette",
        )?;
        let eye_palette = by_id(
            &catalog.eye_palettes,
            &appearance.eye_palette_id,
            "eye palette",
        )?;
        if !catalog.geometry.iter().any(|tuple| {
            tuple.face_id == appearance.face_id
                && tuple.nose_id == appearance.nose_id
                && tuple.eyes_id == appearance.eyes_id
                && tuple.hair_id == appearance.hair_id
        }) {
            return Err("portrait descriptor has an incompatible feature combination".into());
        }
        let bald = hair.is_bald;
        if bald != (appearance.hair_palette_id == BALD_HAIR_PALETTE_ID) {
            return Err("portrait descriptor has an invalid bald hair palette".into());
        }
        let signature = AppearanceSignature {
            rig_visual_key: catalog.rig.visual_key.clone(),
            face_visual_key: face.visual_key.clone(),
            nose_visual_key: nose.visual_key.clone(),
            eyes_visual_key: eyes.visual_key.clone(),
            hair_visual_key: hair.visual_key.clone(),
            skin_visual_key: skin_palette.visual_key.clone(),
            hair_palette_visual_key: (!bald).then(|| hair_palette.visual_key.clone()),
            eye_palette_visual_key: eye_palette.visual_key.clone(),
        };
        if signature.canonical_key() != appearance.signature {
            return Err("portrait descriptor canonical signature does not match its IDs".into());
        }
        let face_assets = face
            .assets
            .as_ref()
            .ok_or_else(|| format!("portrait catalog: missing face fit '{}'", face.id))?;
        let nose_assets = nose.fits.get(&face.id).ok_or_else(|| {
            format!(
                "portrait catalog: missing nose fit '{}' for '{}'",
                nose.id, face.id
            )
        })?;
        let eyes_assets = eyes.fits.get(&face.id).ok_or_else(|| {
            format!(
                "portrait catalog: missing eye fit '{}' for '{}'",
                eyes.id, face.id
            )
        })?;
        let hair_assets = if bald {
            None
        } else {
            Some(hair.fits.get(&face.id).ok_or_else(|| {
                format!(
                    "portrait catalog: missing hair fit '{}' for '{}'",
                    hair.id, face.id
                )
            })?)
        };
        Ok(ResolvedAppearance {
            face: face_assets,
            nose: nose_assets,
            eyes: eyes_assets,
            hair: hair_assets,
            skin: skin_palette,
            hair_palette,
            eye_palette,
        })
    }
}

fn validate_art_coverage(catalog: &PortraitCatalog) -> Result<(), String> {
    if !has_art(catalog) {
        return Ok(());
    }
    let supporting: &SupportingArtAssets = catalog.supporting.as_ref().ok_or_else(|| {
        "portrait catalog: art requires neck, shoulders and fallback assets".to_owned()
    })?;
    for (label, fallback) in [
        ("adult", &supporting.adult_fallback),
        ("child", &supporting.child_silhouette),
        ("unknown", &supporting.unknown_silhouette),
    ] {
        if fallback.trim().is_empty() {
            return Err(format!("portrait catalog: missing {label} fallback asset"));
        }
    }
    for tuple in &catalog.geometry {
        let face = by_id(&catalog.faces, &tuple.face_id, "face")?;
        let nose = by_id(&catalog.noses, &tuple.nose_id, "nose")?;
        let eyes = by_id(&catalog.eyes, &tuple.eyes_id, "eyes")?;
        let hair = by_id(&catalog.hair, &tuple.hair_id, "hair")?;
        let face_assets = face
            .assets
            .as_ref()
            .ok_or_else(|| format!("portrait catalog: face '{}' has no authored fit", face.id))?;
        if face_assets.ink.is_none() {
            return Err(format!(
                "portrait catalog: face '{}' lacks mouth/ink",
                face.id
            ));
        }
        if !nose.fits.contains_key(&face.id) {
            return Err(format!(
                "portrait catalog: nose '{}' lacks fit for '{}'",
                nose.id, face.id
            ));
        }
        if !eyes.fits.contains_key(&face.id) {
            return Err(format!(
                "portrait catalog: eyes '{}' lack fit for '{}'",
                eyes.id, face.id
            ));
        }
        if !hair.is_bald && !hair.fits.contains_key(&face.id) {
            return Err(format!(
                "portrait catalog: hair '{}' lacks fit for '{}'",
                hair.id, face.id
            ));
        }
    }
    Ok(())
}

fn has_art(catalog: &PortraitCatalog) -> bool {
    catalog.supporting.is_some()
        || catalog.faces.iter().any(|feature| feature.assets.is_some())
        || catalog.noses.iter().any(|feature| !feature.fits.is_empty())
        || catalog.eyes.iter().any(|feature| !feature.fits.is_empty())
        || catalog.hair.iter().any(|feature| !feature.fits.is_empty())
}

fn add_shaded_paths(paths: &mut BTreeSet<String>, assets: &ShadedArtAssets) {
    add_masked_paths(paths, &assets.base);
    if let Some(shadow) = &assets.shadow {
        add_masked_paths(paths, shadow);
    }
    if let Some(highlight) = &assets.highlight {
        add_masked_paths(paths, highlight);
    }
    if let Some(ink) = &assets.ink {
        paths.insert(ink.clone());
    }
}

fn add_masked_paths(paths: &mut BTreeSet<String>, assets: &MaskedArtAssets) {
    paths.insert(assets.source.clone());
    paths.insert(assets.mask.clone());
}

fn by_id<'a, T>(items: &'a [T], id: &str, kind: &str) -> Result<&'a T, String>
where
    T: FeatureId,
{
    items
        .iter()
        .find(|item| item.id() == id)
        .ok_or_else(|| format!("portrait catalog: unknown {kind} ID '{id}'"))
}

trait FeatureId {
    fn id(&self) -> &str;
}

impl FeatureId for FaceFeature {
    fn id(&self) -> &str {
        &self.id
    }
}
impl FeatureId for NoseFeature {
    fn id(&self) -> &str {
        &self.id
    }
}
impl FeatureId for EyeFeature {
    fn id(&self) -> &str {
        &self.id
    }
}
impl FeatureId for HairFeature {
    fn id(&self) -> &str {
        &self.id
    }
}
impl FeatureId for PaletteRamp {
    fn id(&self) -> &str {
        &self.id
    }
}
