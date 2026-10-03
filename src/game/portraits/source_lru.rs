//! On-demand decoded PNG source images with an independent byte budget.

use macroquad::prelude::Image;
use std::collections::{BTreeMap, HashMap, VecDeque};

pub(super) const DECODED_SOURCE_BUDGET: usize = 32 * 1024 * 1024;

#[derive(Default)]
pub(super) struct SourceLru {
    images: HashMap<String, Image>,
    recency: VecDeque<String>,
    bytes: usize,
    peak_bytes: usize,
    hits: u64,
    decodes: u64,
    evictions: u64,
}

#[derive(Clone, Copy, Default)]
pub(super) struct SourceMetrics {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub entries: usize,
    pub hits: u64,
    pub decodes: u64,
    pub evictions: u64,
}

impl SourceLru {
    pub fn with_pair<T>(
        &mut self,
        compressed: &BTreeMap<String, Vec<u8>>,
        first_path: &str,
        second_path: &str,
        operation: impl FnOnce(&Image, &Image) -> Result<T, String>,
    ) -> Result<T, String> {
        self.ensure(compressed, first_path)?;
        self.touch(first_path);
        self.ensure(compressed, second_path)?;
        let first = self
            .images
            .get(first_path)
            .ok_or_else(|| format!("portrait source '{first_path}' was evicted while composing"))?;
        let second = self.images.get(second_path).ok_or_else(|| {
            format!("portrait source '{second_path}' was evicted while composing")
        })?;
        operation(first, second)
    }

    pub fn with_image<T>(
        &mut self,
        compressed: &BTreeMap<String, Vec<u8>>,
        path: &str,
        operation: impl FnOnce(&Image) -> Result<T, String>,
    ) -> Result<T, String> {
        self.ensure(compressed, path)?;
        let image = self
            .images
            .get(path)
            .ok_or_else(|| format!("portrait source '{path}' is unavailable"))?;
        operation(image)
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.recency.clear();
        self.bytes = 0;
    }

    pub fn metrics(&self) -> SourceMetrics {
        SourceMetrics {
            bytes: self.bytes,
            peak_bytes: self.peak_bytes,
            entries: self.images.len(),
            hits: self.hits,
            decodes: self.decodes,
            evictions: self.evictions,
        }
    }

    fn ensure(&mut self, compressed: &BTreeMap<String, Vec<u8>>, path: &str) -> Result<(), String> {
        if self.images.contains_key(path) {
            self.hits = self.hits.saturating_add(1);
            self.touch(path);
            return Ok(());
        }
        let bytes = compressed.get(path).ok_or_else(|| {
            format!("portrait source '{path}' is missing or exceeds the byte budget")
        })?;
        let image = macroquad_toolkit::assets::decode_image_bytes(bytes, None)
            .map_err(|error| format!("portrait source '{path}': {error}"))?;
        if image.width != super::manifest::PORTRAIT_SIZE
            || image.height != super::manifest::PORTRAIT_SIZE
        {
            return Err(format!(
                "portrait source '{path}' is {}x{}, expected {}x{}",
                image.width,
                image.height,
                super::manifest::PORTRAIT_SIZE,
                super::manifest::PORTRAIT_SIZE
            ));
        }
        self.insert_image(path.to_owned(), image)
    }

    fn insert_image(&mut self, path: String, image: Image) -> Result<(), String> {
        if image.width != super::manifest::PORTRAIT_SIZE
            || image.height != super::manifest::PORTRAIT_SIZE
        {
            return Err(format!(
                "portrait source '{path}' is {}x{}, expected {}x{}",
                image.width,
                image.height,
                super::manifest::PORTRAIT_SIZE,
                super::manifest::PORTRAIT_SIZE
            ));
        }
        let image_bytes = image.bytes.len();
        if image_bytes > DECODED_SOURCE_BUDGET {
            return Err(format!(
                "portrait source '{path}' exceeds the decoded image byte budget"
            ));
        }
        while self.bytes.saturating_add(image_bytes) > DECODED_SOURCE_BUDGET {
            let Some(oldest) = self.recency.pop_front() else {
                break;
            };
            if let Some(evicted) = self.images.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(evicted.bytes.len());
                self.evictions = self.evictions.saturating_add(1);
            }
        }
        self.bytes += image_bytes;
        self.peak_bytes = self.peak_bytes.max(self.bytes);
        self.decodes = self.decodes.saturating_add(1);
        self.images.insert(path.clone(), image);
        self.recency.push_back(path);
        Ok(())
    }

    fn touch(&mut self, path: &str) {
        if let Some(index) = self.recency.iter().position(|entry| entry == path) {
            self.recency.remove(index);
            self.recency.push_back(path.to_owned());
        }
    }
}
