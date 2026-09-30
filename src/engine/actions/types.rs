//! Command inputs, results and user-facing rule failures.

use super::super::{MoveOrder, MovementBlock, MovementOutcome, RecruitmentResult};
use crate::{
    data::{
        economy::{Resources, TroopKind},
        progression::FormationSpecialization,
        world::{Facility, FactionId, PersonClass, SiteId},
    },
    state::{
        battle::BattleId,
        campaign::DomainFact,
        construction::{ConstructionKind, ConstructionTarget, Focus, OrderId},
        military::{ArmyId, FormationId},
        people::PersonId,
        relationships::{HouseholdId, LegacyCategory, SuccessorLink},
    },
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Player,
    Npc(FactionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Player acceptance commits the saved pending encounter exactly once.
    StartPendingBattle,
    SetFormationTactics {
        formation: FormationId,
        tactics: crate::data::battle_tactics::TroopTactics,
    },
    SetBattleLeader {
        formation: FormationId,
        leader: Option<PersonId>,
    },
    SwapFormationSlots {
        army: ArmyId,
        first: u8,
        second: u8,
    },
    DeclareWar {
        faction: FactionId,
    },
    OfferPeace {
        faction: FactionId,
    },
    RespondPeace {
        proposer: FactionId,
        accept: bool,
    },
    ResolveDefeat {
        faction: FactionId,
        resolution: crate::state::diplomacy::DefeatResolution,
    },
    ClearThreat {
        armies: Vec<ArmyId>,
        threat: crate::state::threat::ThreatId,
    },
    Resettle {
        from: SiteId,
        to: SiteId,
    },
    RenameSite {
        site: SiteId,
        name: String,
    },
    MoveCapital {
        site: SiteId,
    },
    RelocateHeadquarters {
        site: SiteId,
    },
    Siege(crate::state::siege::SiegeOrder),
    StartConstruction {
        target: ConstructionTarget,
        kind: ConstructionKind,
        builder: ArmyId,
    },
    CancelConstruction {
        order: OrderId,
    },
    ReassignBuilder {
        order: OrderId,
        builder: ArmyId,
    },
    SetFocus {
        site: SiteId,
        focus: Focus,
    },
    EndTurn,
    SetNpcPaused(bool),
    StepNpc,
    Recruit {
        site: SiteId,
        army: Option<ArmyId>,
        kind: TroopKind,
    },
    Disband {
        formation: FormationId,
    },
    Move(MoveOrder),
    TransferFormation {
        formation: FormationId,
        to_army: ArmyId,
        to_slot: usize,
    },
    TransferPerson {
        person: PersonId,
        to_formation: FormationId,
    },
    SplitArmy {
        formation: FormationId,
    },
    SetCommander {
        army: ArmyId,
        person: Option<PersonId>,
    },
    TrainPerson {
        person: PersonId,
        class: PersonClass,
        site: SiteId,
    },
    PracticeRiding {
        person: PersonId,
        site: SiteId,
    },
    CancelPersonCourse {
        person: PersonId,
    },
    RecoverPersonAtSite {
        person: PersonId,
        site: SiteId,
    },
    RetirePerson {
        person: PersonId,
        site: SiteId,
    },
    AppointGovernor {
        person: PersonId,
        site: SiteId,
    },
    StartMentorship {
        mentor: PersonId,
        learner: PersonId,
        discipline: crate::data::progression::TrainingDiscipline,
    },
    EndMentorship {
        learner: PersonId,
    },
    FormHousehold {
        first: PersonId,
        second: PersonId,
        site: SiteId,
    },
    EndHousehold {
        household: HouseholdId,
    },
    SetHouseholdChildraising {
        household: HouseholdId,
        enabled: bool,
    },
    AdoptWard {
        guardian: PersonId,
        site: SiteId,
    },
    AssignTrainee {
        person: PersonId,
        site: SiteId,
    },
    EnterService {
        person: PersonId,
        formation: Option<FormationId>,
    },
    InviteApprentice {
        site: SiteId,
    },
    DesignateSuccessor {
        predecessor: PersonId,
        successor: PersonId,
        category: LegacyCategory,
        link: SuccessorLink,
    },
    TransferLegacyItem {
        item: crate::state::legacy::LegacyItemId,
        to: PersonId,
    },
    SpecializeFormation {
        formation: FormationId,
        specialization: FormationSpecialization,
        site: SiteId,
    },
    CancelFormationCourse {
        formation: FormationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleError {
    Diplomacy(String),
    Development(String),
    Progression(String),
    Legacy(String),
    Threat(String),
    Siege(String),
    Construction {
        reason: crate::engine::ConstructionBlock,
    },
    NoCampaign,
    LegacyReadOnly,
    PlayObstructed,
    UnknownActor,
    WrongActor,
    NotYourTurn {
        active: FactionId,
    },
    NotNpcPhase,
    NpcPaused,
    BattlePending,
    NoPendingBattle,
    PauseRequired,
    PauseUnchanged,
    UnknownSite {
        site: SiteId,
    },
    SiteNotOwned {
        site: SiteId,
    },
    SiteContested {
        site: SiteId,
    },
    SiteUnsupplied {
        site: SiteId,
    },
    RecruitingSiteRequired {
        site: SiteId,
    },
    Deficit {
        faction: FactionId,
    },
    MissingFacility {
        site: SiteId,
        facility: Facility,
    },
    FacilityDamaged {
        site: SiteId,
    },
    MissingHorses {
        site: SiteId,
    },
    FortRequired {
        site: SiteId,
    },
    UnknownArmy {
        army: ArmyId,
    },
    ArmyNotOwned {
        army: ArmyId,
    },
    ArmyElsewhere {
        army: ArmyId,
        site: SiteId,
    },
    ArmyFull {
        army: ArmyId,
    },
    UnknownFormation {
        formation: FormationId,
    },
    FormationNotOwned {
        formation: FormationId,
    },
    InsufficientResources {
        required: Resources,
        available: Resources,
    },
    Overflow {
        field: &'static str,
    },
    InvalidState(String),
    InvalidArmyGroup,
    InvalidRoute,
    NotColocated,
    InvalidSlot,
    SlotOccupied,
    TransferUnchanged,
    TransferPauseRequired,
    UnknownPerson {
        person: PersonId,
    },
    PersonNotOwned {
        person: PersonId,
    },
    MovementBlocked {
        site: SiteId,
        reason: MovementBlock,
    },
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diplomacy(reason)
            | Self::Development(reason)
            | Self::Progression(reason)
            | Self::Legacy(reason)
            | Self::Threat(reason) => formatter.write_str(reason),
            Self::Siege(reason) => formatter.write_str(reason),
            Self::Construction { reason } => fmt::Display::fmt(reason, formatter),
            Self::InvalidArmyGroup => formatter.write_str("Choose one or more distinct armies."),
            Self::InvalidRoute => formatter.write_str(
                "Choose a connected physical route beginning at the armies' current site.",
            ),
            Self::NotColocated => formatter.write_str(
                "Every participating army and person must share the same physical site.",
            ),
            Self::InvalidSlot => formatter.write_str("Choose one of the six formation slots."),
            Self::SlotOccupied => formatter.write_str("The receiving slot is occupied."),
            Self::TransferUnchanged => {
                formatter.write_str("That person already serves in this formation.")
            }
            Self::TransferPauseRequired => {
                formatter.write_str("Pause NPC phases before transferring troops or people.")
            }
            Self::UnknownPerson { .. } => formatter.write_str("That person is unavailable."),
            Self::PersonNotOwned { .. } => {
                formatter.write_str("You can transfer only your own people.")
            }
            Self::MovementBlocked { reason, .. } => fmt::Display::fmt(reason, formatter),
            Self::NoCampaign => formatter.write_str("Start or load a campaign first."),
            Self::LegacyReadOnly => formatter.write_str(
                "This empty-atlas save is read-only. Start a strategic campaign to play.",
            ),
            Self::PlayObstructed => {
                formatter.write_str("Close the open panel and return to the campaign first.")
            }
            Self::UnknownActor => formatter.write_str("That faction is unavailable."),
            Self::WrongActor => formatter.write_str("That faction cannot issue this command."),
            Self::NotYourTurn { .. } => formatter.write_str("Wait for your faction's turn."),
            Self::NotNpcPhase => formatter.write_str("NPC phases have already finished."),
            Self::NpcPaused => formatter.write_str("NPC phases are paused. Tap STEP or RESUME."),
            Self::BattlePending => {
                formatter.write_str("Start the pending battle before issuing another order.")
            }
            Self::NoPendingBattle => formatter.write_str("There is no battle waiting to start."),
            Self::PauseRequired => {
                formatter.write_str("Pause NPC phases before taking a single step.")
            }
            Self::PauseUnchanged => {
                formatter.write_str("NPC progression is already in that state.")
            }
            Self::UnknownSite { .. } => formatter.write_str("That physical site is unavailable."),
            Self::SiteNotOwned { .. } => {
                formatter.write_str("Recruit at a site your faction controls.")
            }
            Self::SiteContested { .. } => {
                formatter.write_str("Recruitment is blocked at a contested site.")
            }
            Self::SiteUnsupplied { .. } => formatter
                .write_str("Recruitment needs a friendly supply path to your headquarters."),
            Self::RecruitingSiteRequired { .. } => {
                formatter.write_str("Recruitment requires an Outpost or larger settlement.")
            }
            Self::Deficit { .. } => formatter.write_str(
                "An upkeep shortfall blocks recruitment until a later season fully pays upkeep.",
            ),
            Self::MissingFacility { facility, .. } => write!(
                formatter,
                "Recruitment requires a functional local {}.",
                facility_name(*facility)
            ),
            Self::FacilityDamaged { .. } => {
                formatter.write_str("Structural damage has disabled this site's facilities.")
            }
            Self::MissingHorses { .. } => formatter.write_str("Riders require local horse access."),
            Self::FortRequired { .. } => {
                formatter.write_str("Siege Engines require a local Fort and Workshop.")
            }
            Self::UnknownArmy { .. } => formatter.write_str("That army is unavailable."),
            Self::ArmyNotOwned { .. } => formatter.write_str("Choose one of your own armies."),
            Self::ArmyElsewhere { .. } => {
                formatter.write_str("The receiving army must be at the recruiting site.")
            }
            Self::ArmyFull { .. } => formatter
                .write_str("This army has all six formation slots filled. Choose a new army."),
            Self::UnknownFormation { .. } => formatter.write_str("That formation is unavailable."),
            Self::FormationNotOwned { .. } => {
                formatter.write_str("You can disband only your own formations.")
            }
            Self::InsufficientResources {
                required,
                available,
            } => resource_shortage(formatter, *required, *available),
            Self::Overflow { field } => write!(
                formatter,
                "The campaign cannot advance: {field} is exhausted."
            ),
            Self::InvalidState(reason) => {
                write!(formatter, "The campaign could not be validated: {reason}")
            }
        }
    }
}

fn facility_name(facility: Facility) -> &'static str {
    match facility {
        Facility::TrainingGround => "training ground",
        Facility::Stable => "stable",
        Facility::Infirmary => "infirmary",
        Facility::Workshop => "workshop",
        Facility::Temple => "temple",
    }
}

fn resource_shortage(
    formatter: &mut fmt::Formatter<'_>,
    required: Resources,
    available: Resources,
) -> fmt::Result {
    write!(
        formatter,
        "This order needs {} Gold, {} Wood and {} Stone; available: {}, {} and {}.",
        required.gold,
        required.wood,
        required.stone,
        available.gold,
        available.wood,
        available.stone
    )
}

impl std::error::Error for RuleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutcome {
    pub life_events: Vec<crate::state::history::HistoryId>,
    pub automatic_retirements: Vec<crate::state::people::PersonId>,
    pub battle: Option<BattleId>,
    pub battle_pending: bool,
    pub accepted_sequence: u64,
    pub active_faction: FactionId,
    pub round_completed: bool,
    pub facts: Vec<DomainFact>,
    pub consumed_facts: Vec<DomainFact>,
    pub recruited: Option<RecruitmentResult>,
    pub disbanded: Option<FormationId>,
    pub movement: Option<MovementOutcome>,
    pub split_army: Option<ArmyId>,
    pub succession: Vec<crate::state::relationships::SuccessionNotice>,
    pub new_people: Vec<PersonId>,
    pub legacy_items_changed: Vec<crate::state::legacy::LegacyItemId>,
    pub anniversary_reminders: Vec<crate::state::history::AnniversarySubject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPreview {
    pub active_faction_after: FactionId,
    pub round_completed: bool,
}
