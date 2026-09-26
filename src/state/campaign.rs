//! Authoritative campaign identity, phases, resources, and serializable RNG streams.

use crate::data::{
    economy::Resources,
    rules::Emblem,
    world::{FactionId, MarkerId, Relation, RouteId, SiteId},
    GameData,
};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use super::world::CampaignWorld;

pub const STRATEGIC_VERSION: u32 = 2;

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
    pub faction: FactionId,
    pub marker: MarkerId,
    pub site: SiteId,
    pub route: RouteId,
    pub fact: FactId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DomainFactKind {
    FactionPassed { faction: FactionId },
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
    pub version: u32,
    pub content_version: u32,
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
    pub world: CampaignWorld,
    pub relations: Vec<Relation>,
    pub accepted_sequence: u64,
    pub consumed_sequence: u64,
    pub pending_facts: Vec<DomainFact>,
}

impl StrategicCampaign {
    pub fn new(data: &GameData) -> Result<Self, String> {
        data.validate()?;
        let scenario = &data.scenario;
        let factions = scenario
            .factions
            .iter()
            .map(|setup| {
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
                    },
                )
            })
            .collect();
        let mut campaign = Self {
            version: STRATEGIC_VERSION,
            content_version: scenario.content_version,
            campaign_id: CampaignId(scenario.seed),
            seed: scenario.seed,
            rng: RandomStreams::new(scenario.seed),
            next_ids: NextIds {
                faction: FactionId(next(scenario.factions.iter().map(|f| f.id.0))?),
                marker: MarkerId(next(scenario.markers.iter().map(|m| m.id.0))?),
                site: SiteId(next(scenario.sites.iter().map(|s| s.id.0))?),
                route: RouteId(next(scenario.routes.iter().map(|r| r.id.0))?),
                fact: FactId(1),
            },
            completed_rounds: 0,
            player: scenario.player,
            phase: CampaignPhase::PlayerTurn,
            round_order: Vec::new(),
            acted: BTreeSet::new(),
            factions,
            world: CampaignWorld::from_scenario(scenario),
            relations: scenario.relations.clone(),
            accepted_sequence: 0,
            consumed_sequence: 0,
            pending_facts: Vec::new(),
        };
        for relation in &mut campaign.relations {
            relation.factions.sort();
        }
        campaign.relations.sort_by_key(|relation| relation.factions);
        campaign.round_order = campaign.independent_order();
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
