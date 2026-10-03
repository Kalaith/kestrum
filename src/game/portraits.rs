//! Bounded, observer-safe portrait rendering shared by roster and history UI.

#[path = "portraits/cache.rs"]
mod cache;
#[path = "portraits/compose.rs"]
mod compose;
#[path = "portraits/manifest.rs"]
mod manifest;
#[path = "portraits/queue.rs"]
mod queue;
#[path = "portraits/source_lru.rs"]
mod source_lru;

pub use cache::{PortraitCache, PortraitCacheMetrics, PortraitDraw, PortraitFallback};
pub use compose::{PortraitCompositor, PortraitSourceMetrics};
pub use queue::{PortraitJob, PortraitRequestQueue, PortraitRequestStatus, PENDING_JOB_LIMIT};
