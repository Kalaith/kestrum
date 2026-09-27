//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
mod combat;
pub(crate) mod construction;
mod economy;
mod evidence;
pub(crate) mod history;
pub(crate) mod knowledge;
mod movement;
mod person_combat;
mod projection;
mod recovery;
mod recruitment;
mod retreat;
mod round;
pub(crate) mod siege;
mod transfer;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use combat::battle_reports;
pub use construction::{
    construction_options, construction_refund, ConstructionBlock, ConstructionOption,
};
pub use history::{history_page, HistoryFilter, HistoryPage};
pub use knowledge::{
    hostile_presence, known_people, person_knowledge, KnownPeoplePage, PersonKnowledge,
    KNOWLEDGE_PAGE_SIZE,
};
pub use movement::{
    army_remaining, formation_remaining, movement_preview, person_remaining, route_cost, MoveOrder,
    MovementBlock, MovementEncounter, MovementOutcome, MovementPreview, MovementStop, RouteStep,
};
pub use person_combat::{resolve_person_combat, PersonCombatContext, PersonCombatSide};
pub use projection::{project, SiegeRole, VisibleCampaign, VisibleFaction, VisibleSiege};
pub use recovery::{recovery_preview, RecoveryPreview};
pub use recruitment::{recruit_options, RecruitOption, RecruitmentResult};
pub use transfer::person_site;

pub use siege::{reconcile_sieges, siege_view, SiegeActionOption, SiegeView};
