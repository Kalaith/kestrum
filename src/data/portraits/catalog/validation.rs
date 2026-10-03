//! Semantic catalog validation kept separate from candidate allocation.

use super::{find_by_id, ids, FeatureEntry, PortraitCatalog, PORTRAIT_CATALOG_SOURCE};
use crate::data::portraits::*;
use std::collections::BTreeSet;

impl PortraitCatalog {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PORTRAIT_CATALOG_SCHEMA_VERSION {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: unsupported schema version {}",
                self.schema_version
            ));
        }
        if self.catalog_revision == 0 || self.allocation_revision == 0 {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: catalog and allocation revisions must be positive"
            ));
        }
        if !self
            .supported_catalog_revisions
            .contains(&self.catalog_revision)
            || !self
                .supported_allocation_revisions
                .contains(&self.allocation_revision)
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: current revisions must appear in their supported lists"
            ));
        }
        self.validate_revisions()?;
        self.validate_rig()?;
        self.validate_features()?;
        self.validate_geometry()?;
        self.validate_assets()?;
        Ok(())
    }

    fn validate_revisions(&self) -> Result<(), String> {
        for (label, revisions) in [
            ("catalog", &self.supported_catalog_revisions),
            ("allocation", &self.supported_allocation_revisions),
        ] {
            if revisions.is_empty() || revisions.contains(&0) {
                return Err(format!(
                    "{PORTRAIT_CATALOG_SOURCE}: supported {label} revisions must be positive"
                ));
            }
            let unique: BTreeSet<_> = revisions.iter().copied().collect();
            if unique.len() != revisions.len() {
                return Err(format!(
                    "{PORTRAIT_CATALOG_SOURCE}: duplicate supported {label} revision"
                ));
            }
        }
        Ok(())
    }

    fn validate_rig(&self) -> Result<(), String> {
        if self.rig.id.trim().is_empty()
            || self.rig.visual_key.trim().is_empty()
            || self.rig.width != 512
            || self.rig.height != 512
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: rig must have IDs and a shared 512x512 canvas"
            ));
        }
        let [left, top, right, bottom] = self.rig.safe_rect;
        if left >= right || top >= bottom || right > self.rig.width || bottom > self.rig.height {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: rig safe rectangle is outside its canvas"
            ));
        }
        if self.rig.anchors.is_empty()
            || self.rig.anchors.iter().any(|(name, [x, y])| {
                name.trim().is_empty() || *x >= self.rig.width || *y >= self.rig.height
            })
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: rig anchors must be named and inside the canvas"
            ));
        }
        Ok(())
    }

    fn validate_features(&self) -> Result<(), String> {
        validate_features(&self.faces, "face", |item| {
            !item.silhouette_group.trim().is_empty()
        })?;
        validate_features(&self.noses, "nose", |_| true)?;
        validate_features(&self.eyes, "eyes", |_| true)?;
        validate_features(&self.hair, "hair", |item| {
            !item.silhouette_group.trim().is_empty()
        })?;
        validate_palettes(&self.skin_palettes, "skin", false)?;
        validate_palettes(&self.hair_palettes, "hair", true)?;
        validate_palettes(&self.eye_palettes, "eye", false)?;
        if self.hair.iter().filter(|item| item.is_bald).count() != 1 {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: catalog must define exactly one bald hair style"
            ));
        }
        Ok(())
    }

    fn validate_geometry(&self) -> Result<(), String> {
        if self.geometry.is_empty() {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: geometry compatibility table is empty"
            ));
        }
        let faces = ids(&self.faces);
        let noses = ids(&self.noses);
        let eyes = ids(&self.eyes);
        let hair = ids(&self.hair);
        let bald_id = self
            .hair
            .iter()
            .find(|item| item.is_bald)
            .map(|item| item.id.as_str())
            .ok_or_else(|| format!("{PORTRAIT_CATALOG_SOURCE}: bald hair feature is missing"))?;
        let mut tuples = BTreeSet::new();
        let mut face_coverage = BTreeSet::new();
        for tuple in &self.geometry {
            if !faces.contains(tuple.face_id.as_str())
                || !noses.contains(tuple.nose_id.as_str())
                || !eyes.contains(tuple.eyes_id.as_str())
                || !hair.contains(tuple.hair_id.as_str())
            {
                return Err(format!(
                    "{PORTRAIT_CATALOG_SOURCE}: compatibility tuple references an unknown feature"
                ));
            }
            let hair_feature = find_by_id(&self.hair, &tuple.hair_id, "hair")?;
            if hair_feature.is_bald != (tuple.hair_id == bald_id) {
                return Err(format!(
                    "{PORTRAIT_CATALOG_SOURCE}: bald compatibility must name the catalog's bald style"
                ));
            }
            if !tuples.insert((
                tuple.face_id.as_str(),
                tuple.nose_id.as_str(),
                tuple.eyes_id.as_str(),
                tuple.hair_id.as_str(),
            )) {
                return Err(format!(
                    "{PORTRAIT_CATALOG_SOURCE}: duplicate geometry compatibility tuple"
                ));
            }
            face_coverage.insert(tuple.face_id.as_str());
        }
        if face_coverage.len() != self.faces.len() {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: every face must have compatible geometry"
            ));
        }
        Ok(())
    }

    fn validate_assets(&self) -> Result<(), String> {
        for feature in &self.faces {
            if let Some(assets) = &feature.assets {
                validate_shaded_assets(assets)?;
            }
        }
        for feature in &self.noses {
            for (face, assets) in &feature.fits {
                ensure_known_face(self, face)?;
                validate_masked_assets(assets)?;
            }
        }
        for feature in &self.eyes {
            for (face, assets) in &feature.fits {
                ensure_known_face(self, face)?;
                for path in [
                    &assets.whites,
                    &assets.iris_source,
                    &assets.iris_mask,
                    &assets.ink,
                ] {
                    validate_asset_path(path)?;
                }
            }
        }
        for feature in &self.hair {
            for (face, assets) in &feature.fits {
                ensure_known_face(self, face)?;
                validate_shaded_assets(&assets.rear)?;
                validate_shaded_assets(&assets.front)?;
            }
        }
        if let Some(assets) = &self.supporting {
            validate_shaded_assets(&assets.neck)?;
            validate_shaded_assets(&assets.shoulders)?;
            for path in [
                &assets.child_silhouette,
                &assets.unknown_silhouette,
                &assets.adult_fallback,
            ] {
                validate_asset_path(path)?;
            }
        }
        Ok(())
    }
}

fn validate_features<T>(
    entries: &[T],
    label: &str,
    has_required_groups: impl Fn(&T) -> bool,
) -> Result<(), String>
where
    T: FeatureEntry,
{
    if entries.is_empty() {
        return Err(format!("{PORTRAIT_CATALOG_SOURCE}: no {label} feature IDs"));
    }
    let mut feature_ids = BTreeSet::new();
    for entry in entries {
        if entry.id().trim().is_empty()
            || entry.visual_key().trim().is_empty()
            || !has_required_groups(entry)
            || !feature_ids.insert(entry.id())
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: invalid or duplicate {label} feature ID"
            ));
        }
    }
    Ok(())
}

fn validate_palettes(
    entries: &[PaletteRamp],
    label: &str,
    includes_none: bool,
) -> Result<(), String> {
    if entries.is_empty() {
        return Err(format!("{PORTRAIT_CATALOG_SOURCE}: no {label} palettes"));
    }
    let mut ids_seen = BTreeSet::new();
    for entry in entries {
        if entry.id.trim().is_empty()
            || entry.visual_key.trim().is_empty()
            || entry.value_group.trim().is_empty()
            || !ids_seen.insert(entry.id.as_str())
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: invalid or duplicate {label} palette"
            ));
        }
    }
    if includes_none {
        let none = entries
            .iter()
            .filter(|entry| entry.id == BALD_HAIR_PALETTE_ID)
            .collect::<Vec<_>>();
        if none.len() != 1
            || none[0].value_group != "none"
            || none[0].shadow != [0, 0, 0]
            || none[0].base != [0, 0, 0]
            || none[0].highlight != [0, 0, 0]
        {
            return Err(format!(
                "{PORTRAIT_CATALOG_SOURCE}: bald hair requires one canonical none palette"
            ));
        }
    }
    Ok(())
}

fn validate_shaded_assets(assets: &ShadedArtAssets) -> Result<(), String> {
    validate_masked_assets(&assets.base)?;
    if let Some(shadow) = &assets.shadow {
        validate_masked_assets(shadow)?;
    }
    if let Some(highlight) = &assets.highlight {
        validate_masked_assets(highlight)?;
    }
    if let Some(ink) = &assets.ink {
        validate_asset_path(ink)?;
    }
    Ok(())
}

fn validate_masked_assets(assets: &MaskedArtAssets) -> Result<(), String> {
    validate_asset_path(&assets.source)?;
    validate_asset_path(&assets.mask)
}

fn validate_asset_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty()
        || path.contains('\\')
        || path.split('/').any(|part| part == ".." || part.is_empty())
        || !path.starts_with("assets/")
    {
        return Err(format!(
            "{PORTRAIT_CATALOG_SOURCE}: invalid portrait asset path '{path}'"
        ));
    }
    Ok(())
}

fn ensure_known_face(catalog: &PortraitCatalog, face: &str) -> Result<(), String> {
    if catalog.faces.iter().any(|entry| entry.id == face) {
        Ok(())
    } else {
        Err(format!(
            "{PORTRAIT_CATALOG_SOURCE}: portrait adapter references unknown face '{face}'"
        ))
    }
}
