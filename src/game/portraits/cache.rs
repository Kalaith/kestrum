//! Frame-bounded source, composition, and GPU texture caches.

#[cfg(not(target_arch = "wasm32"))]
use super::queue::PortraitRequestStatus;
use super::{
    compose::{DETAIL_SIZE, PORTRAIT_SIZE, THUMBNAIL_SIZE},
    queue::{PortraitKey, PortraitRequestQueue},
    source_lru::DECODED_SOURCE_BUDGET,
};
use crate::data::portraits::{AppearanceDescriptor, PortraitCatalog};
use macroquad::prelude::{
    draw_texture_ex, vec2, DrawTextureParams, FilterMode, Rect, Texture2D, WHITE,
};
use macroquad_toolkit::{
    assets::{decode_image_bytes, AssetManager},
    image_composition::{downsample_alpha_aware, upload_texture},
};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, VecDeque},
};

pub const COMPRESSED_SOURCE_BUDGET: usize = 64 * 1024 * 1024;
pub const THUMBNAIL_TEXTURE_COUNT: usize = 128;
pub const DETAIL_TEXTURE_COUNT: usize = 32;
const COMPOSITIONS_PER_FRAME: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PortraitFallback {
    Adult,
    Child,
    Unknown,
}

impl PortraitFallback {
    /// Return the fallback kind selected for saved snapshot metadata.
    pub fn for_snapshot(
        age_years: Option<u32>,
        child_threshold: u32,
        has_appearance: bool,
    ) -> Option<Self> {
        match age_years {
            None => Some(Self::Unknown),
            Some(age) if age < child_threshold => Some(Self::Child),
            Some(_) if !has_appearance => Some(Self::Unknown),
            Some(_) => None,
        }
    }

    /// Adult portrait work in progress uses the neutral UI silhouette.
    pub fn uses_authored_fallback(self) -> bool {
        self != Self::Adult
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortraitDraw {
    Ready,
    Fallback {
        kind: PortraitFallback,
        asset_drawn: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PortraitCacheMetrics {
    pub compressed_bytes: usize,
    pub compressed_budget: usize,
    pub decoded_source_bytes: usize,
    pub decoded_source_peak_bytes: usize,
    pub decoded_source_budget: usize,
    pub decoded_source_entries: usize,
    pub decoded_source_hits: u64,
    pub decoded_source_decodes: u64,
    pub decoded_source_evictions: u64,
    pub thumbnail_texture_bytes: usize,
    pub thumbnail_texture_entries: usize,
    pub detail_texture_bytes: usize,
    pub detail_texture_entries: usize,
    pub fallback_texture_bytes: usize,
    pub pending_jobs: usize,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub peak_estimated_cpu_bytes: usize,
    pub last_job_micros: u64,
    pub peak_job_micros: u64,
}

pub struct PortraitCache {
    compositor: super::compose::PortraitCompositor,
    compressed: BTreeMap<String, Vec<u8>>,
    compressed_bytes: usize,
    ready: RefCell<ReadyTextures>,
    fallbacks: HashMap<PortraitFallback, Texture2D>,
    queue: RefCell<PortraitRequestQueue>,
    reset_requested: Cell<bool>,
    asset_errors: Vec<String>,
    runtime_errors: RefCell<VecDeque<String>>,
    completed_jobs: u64,
    failed_jobs: u64,
    peak_estimated_cpu_bytes: usize,
    last_job_micros: u64,
    peak_job_micros: u64,
}

impl PortraitCache {
    pub async fn load(catalog: &PortraitCatalog, assets: &AssetManager) -> Result<Self, String> {
        let compositor = super::compose::PortraitCompositor::new(catalog)?;
        let mut compressed = BTreeMap::new();
        let mut compressed_bytes = 0_usize;
        let mut asset_errors = Vec::new();
        let mut fallbacks = HashMap::new();
        for path in compositor.asset_paths() {
            let encoded = match assets.load_bytes(path).await {
                Ok(bytes) => bytes,
                Err(error) => {
                    asset_errors.push(error);
                    continue;
                }
            };
            let image = match decode_image_bytes(&encoded, None) {
                Ok(image) => image,
                Err(error) => {
                    asset_errors.push(format!("portrait asset '{path}': {error}"));
                    continue;
                }
            };
            if image.width != PORTRAIT_SIZE || image.height != PORTRAIT_SIZE {
                asset_errors.push(format!(
                    "portrait asset '{path}' is {}x{}, expected {}x{}",
                    image.width, image.height, PORTRAIT_SIZE, PORTRAIT_SIZE
                ));
                continue;
            }
            if let Some(kind) =
                fallback_kind(catalog, path).filter(|fallback| fallback.uses_authored_fallback())
            {
                match downsample_alpha_aware(&image, DETAIL_SIZE, DETAIL_SIZE) {
                    Ok(small) => {
                        fallbacks.insert(kind, upload_texture(&small, FilterMode::Linear));
                    }
                    Err(error) => asset_errors.push(format!(
                        "portrait fallback '{path}' could not be prepared: {error}"
                    )),
                }
            }
            let next_total = compressed_bytes.checked_add(encoded.len());
            if next_total.is_some_and(|size| size <= COMPRESSED_SOURCE_BUDGET) {
                compressed_bytes = next_total.expect("checked compressed source size");
                compressed.insert(path.clone(), encoded);
            } else {
                asset_errors.push(format!(
                    "portrait asset '{path}' was validated but exceeds the compressed cache budget"
                ));
            }
        }
        let mut cache = Self {
            compositor,
            compressed,
            compressed_bytes,
            ready: RefCell::new(ReadyTextures::default()),
            fallbacks,
            queue: RefCell::new(PortraitRequestQueue::default()),
            reset_requested: Cell::new(false),
            asset_errors,
            runtime_errors: RefCell::new(VecDeque::new()),
            completed_jobs: 0,
            failed_jobs: 0,
            peak_estimated_cpu_bytes: 0,
            last_job_micros: 0,
            peak_job_micros: 0,
        };
        cache.update_cpu_peak();
        Ok(cache)
    }

    /// Process requests queued during the preceding frame, before drawing begins.
    pub fn begin_frame(&mut self) {
        let _ = apply_deferred_reset(
            &self.reset_requested,
            &mut self.compositor,
            self.ready.get_mut(),
            self.queue.get_mut(),
        );
        for _ in 0..COMPOSITIONS_PER_FRAME {
            let Some(job) = self.queue.get_mut().take_next() else {
                break;
            };
            let started = macroquad::time::get_time();
            match self
                .compositor
                .compose(&self.compressed, job.appearance(), job.size())
            {
                Ok(image) => {
                    let texture = upload_texture(&image, FilterMode::Linear);
                    self.ready.get_mut().insert(job.key().clone(), texture);
                    self.completed_jobs = self.completed_jobs.saturating_add(1);
                }
                Err(error) => {
                    self.failed_jobs = self.failed_jobs.saturating_add(1);
                    self.queue
                        .get_mut()
                        .remember_failure(job.appearance(), job.size());
                    self.record_runtime_error(error);
                }
            }
            let elapsed = (macroquad::time::get_time() - started).max(0.0);
            let micros = (elapsed * 1_000_000.0).min(u64::MAX as f64) as u64;
            self.last_job_micros = micros;
            self.peak_job_micros = self.peak_job_micros.max(micros);
            self.update_cpu_peak();
        }
    }

    /// Defer GPU texture destruction until the next frame, after the previous draw flushes.
    pub fn request_reset(&self) {
        self.reset_requested.set(true);
    }

    /// Draw an identity only when authorized event-time age metadata says adult.
    /// Missing age is intentionally handled by draw() as an unknown silhouette.
    pub fn draw(
        &self,
        appearance: Option<&AppearanceDescriptor>,
        age_years: Option<u32>,
        child_threshold: u32,
        bounds: Rect,
    ) -> PortraitDraw {
        if let Some(kind) =
            PortraitFallback::for_snapshot(age_years, child_threshold, appearance.is_some())
        {
            self.fallback(kind, bounds)
        } else if let Some(appearance) = appearance {
            self.draw_adult(appearance, bounds)
        } else {
            self.fallback(PortraitFallback::Unknown, bounds)
        }
    }

    /// Draw a known adult snapshot whose source context guarantees service-age eligibility.
    pub fn draw_adult(&self, appearance: &AppearanceDescriptor, bounds: Rect) -> PortraitDraw {
        let size = if bounds.w.max(bounds.h) <= 64.0 {
            THUMBNAIL_SIZE
        } else {
            DETAIL_SIZE
        };
        let key = PortraitKey::new(appearance, size);
        if self.ready.borrow_mut().draw(&key, bounds) {
            return PortraitDraw::Ready;
        }
        let kind = PortraitFallback::Adult;
        if !self.compositor.enabled() {
            return self.fallback(kind, bounds);
        }
        {
            let queue = self.queue.borrow();
            if queue.has_failure(appearance, size) {
                return self.fallback(kind, bounds);
            }
            if queue.is_pending(appearance, size) {
                return self.fallback(kind, bounds);
            }
            if queue.is_full() {
                return self.fallback(kind, bounds);
            }
        }
        if let Err(error) = self.compositor.validate_appearance(appearance) {
            self.queue.borrow_mut().remember_failure(appearance, size);
            self.record_runtime_error(error);
            return self.fallback(kind, bounds);
        }
        let _ = self.queue.borrow_mut().request(appearance.clone(), size);
        self.fallback(kind, bounds)
    }

    /// Exercise missing-layer fallback behavior only during an active hidden capture.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn capture_missing_source(
        &mut self,
        appearance: &AppearanceDescriptor,
    ) -> Result<(), String> {
        if !macroquad_toolkit::capture::capture_requested("KESTRUM")
            || !macroquad_toolkit::capture::headless::headless_requested("KESTRUM")
        {
            return Err(
                "portrait missing-source capture requires a hidden KESTRUM capture run".into(),
            );
        }
        let source_path = self.compositor.base_source_path(appearance)?.to_owned();
        self.request_reset();
        self.begin_frame();
        let reset_metrics = self.metrics();
        if reset_metrics.decoded_source_entries != 0
            || reset_metrics.thumbnail_texture_entries != 0
            || reset_metrics.detail_texture_entries != 0
            || reset_metrics.pending_jobs != 0
        {
            return Err(
                "portrait missing-source capture could not reset caches at frame boundary".into(),
            );
        }

        let encoded = self
            .compressed
            .remove(&source_path)
            .ok_or_else(|| format!("portrait capture source '{source_path}' is not available"))?;
        let old_compressed_bytes = self.compressed_bytes;
        let Some(reduced_compressed_bytes) = old_compressed_bytes.checked_sub(encoded.len()) else {
            self.compressed.insert(source_path, encoded);
            return Err("portrait capture compressed byte accounting is inconsistent".into());
        };
        self.compressed_bytes = reduced_compressed_bytes;

        let exercise = (|| {
            let initial_failures = self.failed_jobs;
            for size in [THUMBNAIL_SIZE, DETAIL_SIZE] {
                if self.queue.get_mut().request(appearance.clone(), size)
                    != PortraitRequestStatus::Enqueued
                {
                    return Err(format!(
                        "portrait capture could not enqueue the {size}px missing-source job"
                    ));
                }
                self.begin_frame();
            }
            let expected_failures = initial_failures
                .checked_add(2)
                .ok_or_else(|| "portrait capture failure counter overflow".to_owned())?;
            if self.failed_jobs != expected_failures {
                return Err(format!(
                    "portrait missing-source compositor failed {} jobs; expected 2",
                    self.failed_jobs.saturating_sub(initial_failures)
                ));
            }
            let queue = self.queue.borrow();
            if [THUMBNAIL_SIZE, DETAIL_SIZE]
                .into_iter()
                .any(|size| !queue.has_failure(appearance, size))
            {
                return Err(
                    "portrait missing-source failures were not cached for both sizes".into(),
                );
            }
            Ok(())
        })();

        self.compressed.insert(source_path, encoded);
        self.compressed_bytes = old_compressed_bytes;
        self.update_cpu_peak();
        exercise
    }

    pub fn asset_errors(&self) -> &[String] {
        &self.asset_errors
    }

    pub fn take_runtime_errors(&self) -> Vec<String> {
        self.runtime_errors.borrow_mut().drain(..).collect()
    }

    pub fn metrics(&self) -> PortraitCacheMetrics {
        let sources = self.compositor.source_metrics();
        let ready = self.ready.borrow();
        PortraitCacheMetrics {
            compressed_bytes: self.compressed_bytes,
            compressed_budget: COMPRESSED_SOURCE_BUDGET,
            decoded_source_bytes: sources.bytes,
            decoded_source_peak_bytes: sources.peak_bytes,
            decoded_source_budget: DECODED_SOURCE_BUDGET,
            decoded_source_entries: sources.entries,
            decoded_source_hits: sources.hits,
            decoded_source_decodes: sources.decodes,
            decoded_source_evictions: sources.evictions,
            thumbnail_texture_bytes: ready.thumbnails.bytes,
            thumbnail_texture_entries: ready.thumbnails.textures.len(),
            detail_texture_bytes: ready.details.bytes,
            detail_texture_entries: ready.details.textures.len(),
            fallback_texture_bytes: self.fallbacks.len() * usize::from(DETAIL_SIZE).pow(2) * 4,
            pending_jobs: self.queue.borrow().pending_len(),
            completed_jobs: self.completed_jobs,
            failed_jobs: self.failed_jobs,
            peak_estimated_cpu_bytes: self.peak_estimated_cpu_bytes,
            last_job_micros: self.last_job_micros,
            peak_job_micros: self.peak_job_micros,
        }
    }

    fn fallback(&self, kind: PortraitFallback, bounds: Rect) -> PortraitDraw {
        let asset_drawn = kind.uses_authored_fallback()
            && self.fallbacks.get(&kind).is_some_and(|texture| {
                draw_texture(texture, bounds);
                true
            });
        PortraitDraw::Fallback { kind, asset_drawn }
    }

    fn record_runtime_error(&self, error: String) {
        let mut errors = self.runtime_errors.borrow_mut();
        if errors.len() == super::queue::FAILURE_CACHE_LIMIT {
            errors.pop_front();
        }
        errors.push_back(error);
    }

    fn update_cpu_peak(&mut self) {
        let decoded =
            self.compositor.source_metrics().peak_bytes + usize::from(PORTRAIT_SIZE).pow(2) * 4;
        let canvas_and_layer = usize::from(PORTRAIT_SIZE).pow(2) * 4 * 2;
        let output = usize::from(DETAIL_SIZE).pow(2) * 4;
        let estimate = self.compressed_bytes + decoded + canvas_and_layer + output;
        self.peak_estimated_cpu_bytes = self.peak_estimated_cpu_bytes.max(estimate);
    }
}

fn apply_deferred_reset(
    requested: &Cell<bool>,
    compositor: &mut super::compose::PortraitCompositor,
    ready: &mut ReadyTextures,
    queue: &mut PortraitRequestQueue,
) -> bool {
    if !requested.replace(false) {
        return false;
    }
    compositor.reset_decoded_sources();
    *ready = ReadyTextures::default();
    queue.clear();
    true
}

#[derive(Default)]
struct ReadyTextures {
    thumbnails: TextureLru,
    details: TextureLru,
}

impl ReadyTextures {
    fn draw(&mut self, key: &PortraitKey, bounds: Rect) -> bool {
        if key.size == THUMBNAIL_SIZE {
            self.thumbnails.draw(key, bounds)
        } else {
            self.details.draw(key, bounds)
        }
    }

    fn insert(&mut self, key: PortraitKey, texture: Texture2D) {
        if key.size == THUMBNAIL_SIZE {
            self.thumbnails
                .insert(key, texture, THUMBNAIL_TEXTURE_COUNT);
        } else {
            self.details.insert(key, texture, DETAIL_TEXTURE_COUNT);
        }
    }
}

#[derive(Default)]
struct TextureLru {
    textures: HashMap<PortraitKey, Texture2D>,
    recency: VecDeque<PortraitKey>,
    bytes: usize,
}

impl TextureLru {
    fn draw(&mut self, key: &PortraitKey, bounds: Rect) -> bool {
        if !self.textures.contains_key(key) {
            return false;
        }
        if let Some(index) = self.recency.iter().position(|entry| entry == key) {
            self.recency.remove(index);
        }
        self.recency.push_back(key.clone());
        if let Some(texture) = self.textures.get(key) {
            draw_texture(texture, bounds);
            true
        } else {
            false
        }
    }

    fn insert(&mut self, key: PortraitKey, texture: Texture2D, capacity: usize) {
        if self.textures.contains_key(&key) {
            return;
        }
        while self.textures.len() >= capacity {
            let Some(oldest) = self.recency.pop_front() else {
                break;
            };
            if self.textures.remove(&oldest).is_some() {
                self.bytes = self.bytes.saturating_sub(texture_bytes(oldest.size));
            }
        }
        self.bytes = self.bytes.saturating_add(texture_bytes(key.size));
        self.recency.push_back(key.clone());
        self.textures.insert(key, texture);
    }
}

fn draw_texture(texture: &Texture2D, bounds: Rect) {
    draw_texture_ex(
        texture,
        bounds.x,
        bounds.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(bounds.w, bounds.h)),
            ..Default::default()
        },
    );
}

fn texture_bytes(size: u16) -> usize {
    usize::from(size).pow(2) * 4
}

fn fallback_kind(catalog: &PortraitCatalog, path: &str) -> Option<PortraitFallback> {
    let supporting = catalog.supporting.as_ref()?;
    if path == supporting.adult_fallback {
        Some(PortraitFallback::Adult)
    } else if path == supporting.child_silhouette {
        Some(PortraitFallback::Child)
    } else if path == supporting.unknown_silhouette {
        Some(PortraitFallback::Unknown)
    } else {
        None
    }
}
