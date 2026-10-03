//! Bounded portrait request scheduling and negative-cache policy.

use crate::data::portraits::AppearanceDescriptor;
use std::collections::{HashSet, VecDeque};

pub const PENDING_JOB_LIMIT: usize = 128;
pub(super) const FAILURE_CACHE_LIMIT: usize = 128;
const RENDER_REVISION: u16 = 1;
const THUMBNAIL_SIZE: u16 = 128;
const DETAIL_SIZE: u16 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortraitRequestStatus {
    Enqueued,
    AlreadyPending,
    PreviouslyFailed,
    AtCapacity,
    UnsupportedSize,
}

/// A queued composition accepted by [`PortraitRequestQueue`].
pub struct PortraitJob {
    key: PortraitKey,
    appearance: AppearanceDescriptor,
    size: u16,
}

impl PortraitJob {
    pub fn appearance(&self) -> &AppearanceDescriptor {
        &self.appearance
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub(super) fn key(&self) -> &PortraitKey {
        &self.key
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(super) struct PortraitKey {
    schema: u32,
    catalog_revision: u32,
    allocation_revision: u32,
    render_revision: u16,
    pub(super) size: u16,
    rig: String,
    face: String,
    nose: String,
    eyes: String,
    hair: String,
    skin: String,
    hair_palette: String,
    eye_palette: String,
    signature: String,
}

impl PortraitKey {
    pub(super) fn new(appearance: &AppearanceDescriptor, size: u16) -> Self {
        Self {
            schema: appearance.schema_version,
            catalog_revision: appearance.catalog_revision,
            allocation_revision: appearance.allocation_revision,
            render_revision: RENDER_REVISION,
            size,
            rig: appearance.rig_id.clone(),
            face: appearance.face_id.clone(),
            nose: appearance.nose_id.clone(),
            eyes: appearance.eyes_id.clone(),
            hair: appearance.hair_id.clone(),
            skin: appearance.skin_palette_id.clone(),
            hair_palette: appearance.hair_palette_id.clone(),
            eye_palette: appearance.eye_palette_id.clone(),
            signature: appearance.signature.clone(),
        }
    }
}

/// FIFO policy used by the renderer; callers validate descriptors before requesting work.
#[derive(Default)]
pub struct PortraitRequestQueue {
    jobs: VecDeque<PortraitJob>,
    pending: HashSet<PortraitKey>,
    failures: HashSet<PortraitKey>,
    failure_order: VecDeque<PortraitKey>,
}

impl PortraitRequestQueue {
    pub fn request(
        &mut self,
        appearance: AppearanceDescriptor,
        size: u16,
    ) -> PortraitRequestStatus {
        if !matches!(size, THUMBNAIL_SIZE | DETAIL_SIZE) {
            return PortraitRequestStatus::UnsupportedSize;
        }
        let key = PortraitKey::new(&appearance, size);
        if self.failures.contains(&key) {
            return PortraitRequestStatus::PreviouslyFailed;
        }
        if self.pending.contains(&key) {
            return PortraitRequestStatus::AlreadyPending;
        }
        if self.jobs.len() >= PENDING_JOB_LIMIT {
            return PortraitRequestStatus::AtCapacity;
        }
        self.pending.insert(key.clone());
        self.jobs.push_back(PortraitJob {
            key,
            appearance,
            size,
        });
        PortraitRequestStatus::Enqueued
    }

    pub fn take_next(&mut self) -> Option<PortraitJob> {
        let job = self.jobs.pop_front()?;
        self.pending.remove(&job.key);
        Some(job)
    }

    pub fn has_failure(&self, appearance: &AppearanceDescriptor, size: u16) -> bool {
        self.failures.contains(&PortraitKey::new(appearance, size))
    }

    pub fn is_pending(&self, appearance: &AppearanceDescriptor, size: u16) -> bool {
        self.pending.contains(&PortraitKey::new(appearance, size))
    }

    pub fn remember_failure(&mut self, appearance: &AppearanceDescriptor, size: u16) {
        let key = PortraitKey::new(appearance, size);
        if self.failures.insert(key.clone()) {
            self.failure_order.push_back(key);
        }
        while self.failure_order.len() > FAILURE_CACHE_LIMIT {
            if let Some(oldest) = self.failure_order.pop_front() {
                self.failures.remove(&oldest);
            }
        }
    }

    pub fn pending_len(&self) -> usize {
        self.jobs.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    pub fn is_full(&self) -> bool {
        self.jobs.len() >= PENDING_JOB_LIMIT
    }

    pub fn clear(&mut self) {
        self.jobs.clear();
        self.pending.clear();
        self.failures.clear();
        self.failure_order.clear();
    }
}
