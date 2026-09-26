//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
mod economy;
mod projection;
mod recruitment;
mod round;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use projection::{project, VisibleCampaign, VisibleFaction};
pub use recruitment::{recruit_options, RecruitOption, RecruitmentResult};
