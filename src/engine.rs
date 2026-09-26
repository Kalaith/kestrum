//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
mod combat;
mod economy;
mod movement;
mod person_combat;
mod projection;
mod recovery;
mod recruitment;
mod retreat;
mod round;
mod transfer;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use combat::battle_reports;
pub use movement::{
    army_remaining, formation_remaining, movement_preview, person_remaining, route_cost, MoveOrder,
    MovementBlock, MovementOutcome, MovementPreview, MovementStop, RouteStep,
};
pub use person_combat::{resolve_person_combat, PersonCombatContext, PersonCombatSide};
pub use projection::{project, VisibleCampaign, VisibleFaction};
pub use recovery::{recovery_preview, RecoveryPreview};
pub use recruitment::{recruit_options, RecruitOption, RecruitmentResult};
pub use transfer::person_site;
