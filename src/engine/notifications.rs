//! Accepted campaign transactions become durable, observer-safe event receipts.

mod collect;
mod forecast;
mod projection;

pub use collect::{baseline_current_conditions, collect};
pub use projection::project;
