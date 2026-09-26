//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
mod economy;
mod movement;
mod projection;
mod recovery;
mod recruitment;
mod round;
mod transfer;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use movement::{
    army_remaining, formation_remaining, movement_preview, person_remaining, route_cost, MoveOrder,
    MovementBlock, MovementOutcome, MovementPreview, MovementStop, RouteStep,
};
pub use projection::{project, VisibleCampaign, VisibleFaction};
pub use recovery::{recovery_preview, RecoveryPreview};
pub use recruitment::{recruit_options, RecruitOption, RecruitmentResult};
pub use transfer::person_site;
