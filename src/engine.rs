//! Pure strategic commands, stable phase boundaries, and observer-safe projection.

mod actions;
pub mod ai;
mod battle_preparation;
pub mod battle_sim;
mod combat;
pub(crate) mod construction;
pub(crate) mod development;
pub mod diplomacy;
mod economy;
mod evidence;
pub(crate) mod exploration;
pub use exploration::{explored_sites, known_factions, project_map};
pub(crate) mod history;
pub(crate) mod knowledge;
mod legacy;
mod lifecycle;
mod mentorship;
mod movement;
mod notices;
mod person_combat;
pub(crate) mod progression;
mod projection;
mod recovery;
mod recruitment;
mod retreat;
mod round;
pub(crate) mod siege;
mod succession;
pub use succession::{household_option, HouseholdAction, HouseholdOption, HouseholdSelection};
pub(crate) mod threats;
mod transfer;
pub use notices::action_notices;
pub use progression::course_gold_cost;

pub use actions::{
    advance_npc, apply, preview, ActionOutcome, ActionPreview, Actor, Command, RuleError,
};
pub use battle_sim::resolve_battle;
pub use combat::battle_reports;
pub use construction::{
    construction_options, construction_refund, ConstructionBlock, ConstructionOption,
};
pub use diplomacy::{diplomacy_view, DiplomacyFactionView, DiplomacyView};
pub use history::{history_page, HistoryFilter, HistoryPage};
pub use knowledge::{
    hostile_presence, known_people, person_knowledge, KnownPeoplePage, PersonKnowledge,
    KNOWLEDGE_PAGE_SIZE,
};
pub use mentorship::{mentorship_options, mentorship_status_text, MentorshipOption};
pub use movement::{
    army_remaining, formation_remaining, map_movement_preview, movement_preview, person_remaining,
    route_cost, MoveOrder, MovementBlock, MovementEncounter, MovementOutcome, MovementPreview,
    MovementStop, RouteStep,
};
pub use person_combat::{resolve_person_combat, PersonCombatContext, PersonCombatSide};
pub use progression::{
    career_options, specialization_options, CareerOption, CareerRequirement, SpecializationOption,
};
pub use projection::{project, SiegeRole, VisibleCampaign, VisibleFaction, VisibleSiege};
pub use recovery::{recovery_preview, RecoveryPreview};
pub use recruitment::{recruit_options, RecruitOption, RecruitmentResult};
pub use transfer::person_site;

pub use development::{development_view, DevelopmentCause, DevelopmentView};
pub use siege::{reconcile_sieges, siege_view, SiegeActionOption, SiegeView};
pub use threats::{
    threat_preview, threat_view, visible_threats, ThreatArmyOption, ThreatPreview, ThreatView,
    VisibleThreat,
};
