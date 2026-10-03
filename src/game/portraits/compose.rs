//! Layer ordering and tint choices for one completed human bust.

use super::{
    manifest::PortraitAssetManifest,
    source_lru::{SourceLru, SourceMetrics},
};
use crate::data::portraits::{
    AppearanceDescriptor, MaskedArtAssets, PaletteRamp, PortraitCatalog, ShadedArtAssets,
};
use macroquad::prelude::{Color, Image};
use macroquad_toolkit::image_composition::{alpha_over, downsample_alpha_aware, tint_with_mask};
use std::collections::BTreeMap;

pub(super) const THUMBNAIL_SIZE: u16 = 128;
pub(super) const DETAIL_SIZE: u16 = 256;
pub(super) const PORTRAIT_SIZE: u16 = 512;
const SHOULDER_TINT: [u8; 3] = [84, 101, 108];
const SHOULDER_SHADOW: [u8; 3] = [39, 51, 58];
const SHOULDER_HIGHLIGHT: [u8; 3] = [153, 164, 157];

/// CPU compositor shared by the game cache and headless integration tests.
///
/// The catalog and its immutable path index are validated once at construction;
/// encoded PNG bytes remain owned by the frame-bounded game cache.
pub struct PortraitCompositor {
    manifest: PortraitAssetManifest,
    sources: SourceLru,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PortraitSourceMetrics {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub entries: usize,
    pub hits: u64,
    pub decodes: u64,
    pub evictions: u64,
}

impl PortraitCompositor {
    pub fn new(catalog: &PortraitCatalog) -> Result<Self, String> {
        Ok(Self {
            manifest: PortraitAssetManifest::from_catalog(catalog)?,
            sources: SourceLru::default(),
        })
    }

    /// Paths required by the validated catalog, including optional fallback art.
    pub fn asset_paths(&self) -> &[String] {
        &self.manifest.paths
    }

    pub fn enabled(&self) -> bool {
        self.manifest.enabled
    }

    /// Composite a descriptor using the cache's real source decoder and bounded LRU.
    pub fn compose(
        &mut self,
        compressed: &BTreeMap<String, Vec<u8>>,
        appearance: &AppearanceDescriptor,
        target_size: u16,
    ) -> Result<Image, String> {
        compose(
            &self.manifest,
            &mut self.sources,
            compressed,
            appearance,
            target_size,
        )
    }

    pub fn source_metrics(&self) -> PortraitSourceMetrics {
        let metrics: SourceMetrics = self.sources.metrics();
        PortraitSourceMetrics {
            bytes: metrics.bytes,
            peak_bytes: metrics.peak_bytes,
            entries: metrics.entries,
            hits: metrics.hits,
            decodes: metrics.decodes,
            evictions: metrics.evictions,
        }
    }

    /// Clear decoded source memory without changing the saved descriptor or compressed assets.
    pub fn reset_decoded_sources(&mut self) {
        self.sources.clear();
    }

    pub(super) fn validate_appearance(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<(), String> {
        self.manifest.resolve(appearance).map(|_| ())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn base_source_path(
        &self,
        appearance: &AppearanceDescriptor,
    ) -> Result<&str, String> {
        Ok(&self.manifest.resolve(appearance)?.face.base.source)
    }
}

pub(super) fn compose(
    manifest: &PortraitAssetManifest,
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    appearance: &AppearanceDescriptor,
    target_size: u16,
) -> Result<Image, String> {
    if !matches!(target_size, THUMBNAIL_SIZE | DETAIL_SIZE) {
        return Err("portrait composite requested an unsupported output tier".into());
    }
    let parts = manifest.resolve(appearance)?;
    let supporting = manifest
        .catalog
        .supporting
        .as_ref()
        .ok_or_else(|| "portrait catalog has no supporting layers".to_owned())?;
    let mut canvas = transparent_canvas();

    if let Some(hair) = parts.hair {
        shaded(
            &mut canvas,
            sources,
            compressed,
            &hair.rear,
            parts.hair_palette,
        )?;
    }
    shaded_colors(
        &mut canvas,
        sources,
        compressed,
        &supporting.shoulders,
        SHOULDER_TINT,
        SHOULDER_SHADOW,
        SHOULDER_HIGHLIGHT,
    )?;
    if let Some(ink) = &supporting.shoulders.ink {
        direct(sources, compressed, ink, &mut canvas)?;
    }
    shaded(
        &mut canvas,
        sources,
        compressed,
        &supporting.neck,
        parts.skin,
    )?;
    shaded_layers(&mut canvas, sources, compressed, parts.face, parts.skin)?;
    direct(sources, compressed, &parts.eyes.whites, &mut canvas)?;
    masked(
        &mut canvas,
        sources,
        compressed,
        &MaskedArtAssets {
            source: parts.eyes.iris_source.clone(),
            mask: parts.eyes.iris_mask.clone(),
        },
        color(parts.eye_palette.base),
    )?;
    direct(sources, compressed, &parts.eyes.ink, &mut canvas)?;
    masked(
        &mut canvas,
        sources,
        compressed,
        parts.nose,
        color(parts.skin.base),
    )?;
    if let Some(ink) = &parts.face.ink {
        direct(sources, compressed, ink, &mut canvas)?;
    }
    if let Some(hair) = parts.hair {
        shaded(
            &mut canvas,
            sources,
            compressed,
            &hair.front,
            parts.hair_palette,
        )?;
    }
    downsample_alpha_aware(&canvas, target_size, target_size)
}

fn shaded(
    canvas: &mut Image,
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    assets: &ShadedArtAssets,
    ramp: &PaletteRamp,
) -> Result<(), String> {
    shaded_layers(canvas, sources, compressed, assets, ramp)?;
    if let Some(ink) = &assets.ink {
        direct(sources, compressed, ink, canvas)?;
    }
    Ok(())
}

fn shaded_layers(
    canvas: &mut Image,
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    assets: &ShadedArtAssets,
    ramp: &PaletteRamp,
) -> Result<(), String> {
    shaded_colors(
        canvas,
        sources,
        compressed,
        assets,
        ramp.base,
        ramp.shadow,
        ramp.highlight,
    )
}

fn shaded_colors(
    canvas: &mut Image,
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    assets: &ShadedArtAssets,
    base: [u8; 3],
    shadow_tint: [u8; 3],
    highlight_tint: [u8; 3],
) -> Result<(), String> {
    masked(canvas, sources, compressed, &assets.base, color(base))?;
    if let Some(shadow) = &assets.shadow {
        masked(canvas, sources, compressed, shadow, color(shadow_tint))?;
    }
    if let Some(highlight) = &assets.highlight {
        masked(
            canvas,
            sources,
            compressed,
            highlight,
            color(highlight_tint),
        )?;
    }
    Ok(())
}

fn masked(
    canvas: &mut Image,
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    assets: &MaskedArtAssets,
    tint: Color,
) -> Result<(), String> {
    sources.with_pair(compressed, &assets.source, &assets.mask, |source, mask| {
        overlay_masked(canvas, source, mask, tint)
    })
}

fn direct(
    sources: &mut SourceLru,
    compressed: &BTreeMap<String, Vec<u8>>,
    path: &str,
    canvas: &mut Image,
) -> Result<(), String> {
    sources.with_image(compressed, path, |layer| overlay_direct(canvas, layer))
}

fn overlay_masked(
    canvas: &mut Image,
    source: &Image,
    mask: &Image,
    tint: Color,
) -> Result<(), String> {
    let layer = tint_with_mask(source, mask, tint)?;
    alpha_over(canvas, &layer)
}

fn overlay_direct(canvas: &mut Image, layer: &Image) -> Result<(), String> {
    alpha_over(canvas, layer)
}

fn transparent_canvas() -> Image {
    Image::gen_image_color(PORTRAIT_SIZE, PORTRAIT_SIZE, Color::new(0.0, 0.0, 0.0, 0.0))
}

fn color(rgb: [u8; 3]) -> Color {
    Color::new(
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
        1.0,
    )
}
