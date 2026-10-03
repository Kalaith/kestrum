//! Accepted campaign transactions become durable, observer-safe event receipts.

mod attention;
mod collect;
mod forecast;
mod projection;

pub use attention::extend_attention;
pub(crate) use collect::collect_blocked_continuation;
pub use collect::{baseline_current_conditions, collect};
pub use projection::project;
