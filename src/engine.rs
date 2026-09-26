//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
mod projection;
mod round;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use projection::{project, VisibleCampaign, VisibleFaction};
