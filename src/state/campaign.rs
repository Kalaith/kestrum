//! Authoritative campaign identity, phases, resources, and serializable RNG streams.

mod compatibility;

use crate::data::{
    economy::{Resources, TroopKind},
    rules::Emblem,
    world::{FactionId, MarkerId, Relation, RouteId, ScenarioKind, SiteId, WorldLayout},
    GameData,
};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    battle::{BattleId, BattleReport, PendingBattle},
    history::{CampaignHistory, HistoryId},
    knowledge::CampaignKnowledge,
    legacy::{LegacyItem, LegacyItemId},
    military::{Army, ArmyId, EconomyStatement, Formation, FormationId, RecoveryStatement},
    people::{Person, PersonId},
    relationships::{Household, HouseholdId, PersonFamily, SuccessorRegister},
    world::CampaignWorld,
};

pub const STRATEGIC_VERSION: u32 = 2;

fn default_scenario_kind() -> ScenarioKind {
    ScenarioKind::RosemarchPrototype
}

fn first_household_id() -> HouseholdId {
    HouseholdId(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CampaignId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FactId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionStatus {
    Independent,
    Eliminated,
    Vassal { sovereign: FactionId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub emblem: Emblem,
    pub status: FactionStatus,
    pub resources: Resources,
    pub deficit: bool,
    pub headquarters: SiteId,
    pub capital: SiteId,
    pub last_hq_relocation: Option<u32>,
    #[serde(default)]
    pub last_economy: Option<EconomyStatement>,
    #[serde(default)]
    pub last_recovery: Option<RecoveryStatement>,
    /// How this kingdom plays when a sovereign planner controls it.
    #[serde(default)]
    pub personality: crate::data::ai::AiPersonality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CampaignPhase {
    PlayerTurn,
    NpcTurn { faction: FactionId, paused: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RandomStreams {
    pub generation: SeededRng,
    pub combat: SeededRng,
    pub development: SeededRng,
    pub people: SeededRng,
}

impl RandomStreams {
    pub fn new(seed: u64) -> Self {
        let mut root = SeededRng::new(seed);
        Self {
            generation: SeededRng::new(root.next_u64()),
            combat: SeededRng::new(root.next_u64()),
            development: SeededRng::new(root.next_u64()),
            people: SeededRng::new(root.next_u64()),
        }
    }

    pub fn states(&self) -> [u64; 4] {
        [
            self.generation.state(),
            self.combat.state(),
            self.development.state(),
            self.people.state(),
        ]
    }
}

impl PartialEq for RandomStreams {
    fn eq(&self, other: &Self) -> bool {
        self.states() == other.states()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NextIds {
    pub threat: super::threat::ThreatId,
    pub siege: super::siege::SiegeId,
    pub order: super::construction::OrderId,
    pub history: HistoryId,
    pub battle: BattleId,
    pub faction: FactionId,
    pub marker: MarkerId,
    pub site: SiteId,
    pub route: RouteId,
    pub fact: FactId,
    pub army: ArmyId,
    pub formation: FormationId,
    pub person: PersonId,
    #[serde(default = "first_household_id")]
    pub household: HouseholdId,
    #[serde(default = "first_legacy_item_id")]
    pub legacy_item: LegacyItemId,
}

fn first_legacy_item_id() -> LegacyItemId {
    LegacyItemId(1)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DomainFactKind {
    DiplomacyChanged {
        receipt: super::diplomacy::DiplomacyReceipt,
    },
    DevelopmentChanged {
        receipt: super::development::DevelopmentReceipt,
    },
    SiegeChanged {
        siege: super::siege::Siege,
        change: super::siege::SiegeChange,
    },
    ConstructionChanged {
        order: super::construction::ConstructionOrder,
    },
    FocusChanged {
        faction: FactionId,
        site: SiteId,
        focus: super::construction::Focus,
    },
    BattleResolved {
        battle: BattleId,
        #[serde(default)]
        movement: Option<super::evidence::MovementService>,
    },
    FactionPassed {
        faction: FactionId,
    },
    FormationRecruited {
        faction: FactionId,
        army: ArmyId,
        formation: FormationId,
        site: SiteId,
        troop: TroopKind,
    },
    FormationDisbanded {
        faction: FactionId,
        army: ArmyId,
        formation: FormationId,
        site: SiteId,
        troop: TroopKind,
    },
    ArmiesMoved {
        faction: FactionId,
        armies: Vec<ArmyId>,
        path: Vec<SiteId>,
        spent: u32,
        #[serde(default)]
        movement: Option<super::evidence::MovementService>,
    },
    FormationTransferred {
        faction: FactionId,
        formation: FormationId,
        from_army: ArmyId,
        to_army: ArmyId,
        site: SiteId,
    },
    PersonTransferred {
        faction: FactionId,
        person: PersonId,
        to_formation: FormationId,
        site: SiteId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainFact {
    pub id: FactId,
    pub sequence: u64,
    pub completed_rounds: u32,
    pub kind: DomainFactKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategicCampaign {
    pub appearance_registry: super::appearance::AppearanceRegistry,
    #[serde(default)]
    pub notifications: super::notifications::NotificationInbox,
    /// All independent factions, including the original player faction, are AI controlled.
    #[serde(default)]
    pub observer_mode: bool,
    #[serde(default)]
    pub tutorial: super::tutorial::TutorialProgress,
    pub diplomacy: super::diplomacy::CampaignDiplomacy,
    pub ai: super::ai::AiState,
    pub threats: BTreeMap<super::threat::ThreatId, super::threat::Threat>,
    #[serde(default)]
    pub initial_threats: BTreeMap<SiteId, crate::data::threats::ThreatKind>,
    pub sieges: BTreeMap<SiteId, super::siege::Siege>,
    pub construction:
        BTreeMap<super::construction::OrderId, super::construction::ConstructionOrder>,
    pub history: CampaignHistory,
    pub knowledge: CampaignKnowledge,
    pub battles: BTreeMap<BattleId, BattleReport>,
    #[serde(default)]
    pub pending_battle: Option<PendingBattle>,
    pub version: u32,
    #[serde(default)]
    pub roster_version: u32,
    pub content_version: u32,
    #[serde(default = "default_scenario_kind")]
    pub scenario_kind: ScenarioKind,
    pub campaign_id: CampaignId,
    pub seed: u64,
    pub rng: RandomStreams,
    pub next_ids: NextIds,
    pub completed_rounds: u32,
    pub player: FactionId,
    pub phase: CampaignPhase,
    pub round_order: Vec<FactionId>,
    pub acted: BTreeSet<FactionId>,
    pub factions: BTreeMap<FactionId, Faction>,
    pub armies: BTreeMap<ArmyId, Army>,
    #[serde(default)]
    pub movement_plans: Vec<super::movement::MovementPlan>,
    pub formations: BTreeMap<FormationId, Formation>,
    pub people: BTreeMap<PersonId, Person>,
    #[serde(default)]
    pub households: BTreeMap<HouseholdId, Household>,
    #[serde(default)]
    pub families: BTreeMap<PersonId, PersonFamily>,
    #[serde(default)]
    pub successors: SuccessorRegister,
    #[serde(default)]
    pub apprentice_last_invited_year: BTreeMap<FactionId, u32>,
    #[serde(default)]
    pub mentorships: BTreeMap<PersonId, super::mentorship::Mentorship>,
    #[serde(default)]
    pub legacy_items: BTreeMap<LegacyItemId, LegacyItem>,
    #[serde(default)]
    pub battle_templates: Vec<super::battle_plans::BattlePlanTemplate>,
    pub world: CampaignWorld,
    pub relations: Vec<Relation>,
    pub accepted_sequence: u64,
    pub consumed_sequence: u64,
    /// Completed boundary plus its automatic travel; later player actions invalidate it.
    #[serde(default)]
    pub round_checkpoint_sequence: Option<u64>,
    pub pending_facts: Vec<DomainFact>,
}

impl StrategicCampaign {
    pub fn new(data: &GameData) -> Result<Self, String> {
        data.validate()?;
        Self::from_scenario(data, &data.scenario, &data.threats.initial, None)
    }

    /// Creates the authored scenario with every faction controlled by the AI.
    pub fn new_observer(data: &GameData) -> Result<Self, String> {
        let mut campaign = Self::new(data)?;
        campaign.enable_observer_mode();
        campaign.validate(data)?;
        Ok(campaign)
    }

    pub fn new_production(
        data: &GameData,
        setup: &crate::data::generation::ProductionSetup,
    ) -> Result<Self, String> {
        data.validate()?;
        let generated = data.production_layout.generate(data, setup)?;
        Self::from_scenario(
            data,
            &generated.scenario,
            &generated.initial_threats,
            Some(&generated.topology),
        )
    }

    /// Creates a generated world for spectator play, with no human-controlled faction.
    pub fn new_production_observer(
        data: &GameData,
        setup: &crate::data::generation::ProductionSetup,
    ) -> Result<Self, String> {
        let mut campaign = Self::new_production(data, setup)?;
        campaign.enable_observer_mode();
        campaign.validate(data)?;
        Ok(campaign)
    }

    fn enable_observer_mode(&mut self) {
        self.observer_mode = true;
        self.tutorial.dismiss();
        self.phase = CampaignPhase::NpcTurn {
            faction: self.player,
            paused: false,
        };
        self.ai.factions.extend(
            self.independent_order()
                .into_iter()
                .map(|faction| (faction, Default::default())),
        );
    }

    pub fn is_observer(&self) -> bool {
        self.observer_mode
    }

    pub fn observer_finished(&self) -> bool {
        self.observer_mode
            && (self.diplomacy.ending.is_some()
                || (self.pending_battle.is_none()
                    && !self.diplomacy.has_pending_decision()
                    && self
                        .factions
                        .keys()
                        .filter(|faction| self.is_independent(**faction))
                        .count()
                        <= 1))
    }

    fn from_scenario(
        data: &GameData,
        scenario: &crate::data::world::Scenario,
        initial_threats: &[crate::data::threats::InitialThreat],
        production_topology: Option<&WorldLayout>,
    ) -> Result<Self, String> {
        let factions = scenario
            .factions
            .iter()
            .enumerate()
            .map(|(index, setup)| {
                (
                    setup.id,
                    Faction {
                        id: setup.id,
                        name: setup.name.clone(),
                        emblem: setup.emblem,
                        status: FactionStatus::Independent,
                        resources: setup.resources.resolve(&data.economy),
                        deficit: false,
                        headquarters: setup.headquarters,
                        capital: setup.capital,
                        last_hq_relocation: None,
                        last_economy: None,
                        last_recovery: None,
                        // The authored prototype keeps its tuned, even-tempered rivals.
                        personality: if scenario.kind == ScenarioKind::Production {
                            crate::data::ai::AiPersonality::for_campaign(scenario.seed, index)
                        } else {
                            Default::default()
                        },
                    },
                )
            })
            .collect();
        let mut campaign = Self {
            appearance_registry: super::appearance::AppearanceRegistry::for_campaign_seed(
                scenario.seed,
                data.portraits.catalog_revision,
                data.portraits.allocation_revision,
            ),
            notifications: Default::default(),
            observer_mode: false,
            tutorial: super::tutorial::TutorialProgress::new(),
            diplomacy: Default::default(),
            ai: Default::default(),
            threats: super::threat::initialize_threats(data, scenario, initial_threats)?,
            initial_threats: initial_threats
                .iter()
                .map(|threat| (threat.site, threat.kind))
                .collect(),
            sieges: BTreeMap::new(),
            construction: BTreeMap::new(),
            history: CampaignHistory::default(),
            knowledge: CampaignKnowledge::default(),
            battles: BTreeMap::new(),
            pending_battle: None,
            version: STRATEGIC_VERSION,
            roster_version: 1,
            content_version: scenario.content_version,
            scenario_kind: scenario.kind,
            campaign_id: CampaignId(scenario.seed),
            seed: scenario.seed,
            rng: RandomStreams::new(scenario.seed),
            next_ids: NextIds {
                threat: super::threat::ThreatId(1),
                siege: super::siege::SiegeId(1),
                order: super::construction::OrderId(1),
                history: HistoryId(1),
                battle: BattleId(1),
                faction: FactionId(next(scenario.factions.iter().map(|f| f.id.0))?),
                marker: MarkerId(next(scenario.markers.iter().map(|m| m.id.0))?),
                site: SiteId(next(scenario.sites.iter().map(|s| s.id.0))?),
                route: RouteId(next(scenario.routes.iter().map(|r| r.id.0))?),
                fact: FactId(1),
                army: ArmyId(1),
                formation: FormationId(1),
                person: PersonId(1),
                household: HouseholdId(1),
                legacy_item: LegacyItemId(1),
            },
            completed_rounds: 0,
            player: scenario.player,
            phase: CampaignPhase::PlayerTurn,
            round_order: Vec::new(),
            acted: BTreeSet::new(),
            factions,
            armies: BTreeMap::new(),
            movement_plans: Vec::new(),
            formations: BTreeMap::new(),
            people: BTreeMap::new(),
            households: BTreeMap::new(),
            families: BTreeMap::new(),
            successors: BTreeMap::new(),
            apprentice_last_invited_year: BTreeMap::new(),
            mentorships: BTreeMap::new(),
            legacy_items: BTreeMap::new(),
            battle_templates: Vec::new(),
            world: CampaignWorld::from_scenario(scenario),
            relations: scenario.relations.clone(),
            accepted_sequence: 0,
            consumed_sequence: 0,
            round_checkpoint_sequence: None,
            pending_facts: Vec::new(),
        };
        for relation in &mut campaign.relations {
            relation.factions.sort();
        }
        if scenario.kind == ScenarioKind::Production {
            let topology =
                production_topology.ok_or("production campaign is missing its derived topology")?;
            campaign.world.layout_revision = topology.layout_revision;
            campaign.world.atlas_paths = topology.atlas_paths.clone();
        }
        campaign.relations.sort_by_key(|relation| relation.factions);
        campaign.initialize_diplomacy();
        campaign.round_order = campaign.independent_order();
        campaign.instantiate_starting_military(data, scenario)?;
        campaign.initialize_population(&data.construction);
        campaign.initialize_development(data)?;
        campaign.initialize_legacy_items()?;
        crate::engine::exploration::observe(&mut campaign);
        campaign.next_ids.threat =
            super::threat::ThreatId(next(campaign.threats.keys().map(|id| id.0))?);
        campaign.reconcile_region_control();
        campaign.validate(data)?;
        Ok(campaign)
    }

    pub fn active_faction(&self) -> FactionId {
        match self.phase {
            CampaignPhase::PlayerTurn => self.player,
            CampaignPhase::NpcTurn { faction, .. } => faction,
        }
    }

    pub fn is_independent(&self, faction: FactionId) -> bool {
        self.factions
            .get(&faction)
            .is_some_and(|state| state.status == FactionStatus::Independent)
            && !self
                .diplomacy
                .pending_defeats
                .iter()
                .any(|pending| pending.faction == faction)
    }

    pub fn independent_order(&self) -> Vec<FactionId> {
        std::iter::once(self.player)
            .chain(
                self.factions
                    .keys()
                    .copied()
                    .filter(|id| *id != self.player),
            )
            .filter(|id| self.is_independent(*id))
            .collect()
    }

    pub fn season_index(&self) -> usize {
        (self.completed_rounds % 4) as usize
    }

    pub fn year(&self, start: u32) -> u32 {
        start.saturating_add(self.completed_rounds / 4)
    }
}

fn next(ids: impl Iterator<Item = u32>) -> Result<u32, String> {
    ids.max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "campaign.next_ids: identifier space exhausted".into())
}
