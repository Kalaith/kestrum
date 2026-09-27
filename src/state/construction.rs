//! Prepaid physical work and its terminal receipt; completed work never reopens.

mod validation;

use super::{military::ArmyId, StrategicCampaign};
use crate::data::{
    economy::Resources,
    world::{Facility, FactionId, RouteId, SiteId},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ConstructionTarget {
    Site(SiteId),
    Route(RouteId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "facility",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ConstructionKind {
    Outpost,
    Road,
    RoadRepair,
    Fort,
    Facility(Facility),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Focus {
    Growth,
    Fortification,
    TroopTraining,
    Gold,
    Wood,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionPause {
    BuilderMissing,
    BuilderAway,
    SupplyLost,
    Contested,
    Combat,
    NoSettlers,
    Threat,
    Ruined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationReason {
    Player,
    ControlLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConstructionStatus {
    Active,
    Paused {
        reason: ConstructionPause,
    },
    Completed {
        completed_rounds: u32,
    },
    Cancelled {
        completed_rounds: u32,
        reason: CancellationReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionOrder {
    pub id: OrderId,
    pub owner: FactionId,
    pub kind: ConstructionKind,
    pub target: ConstructionTarget,
    pub builder: Option<ArmyId>,
    pub created_round: u32,
    pub progress: u32,
    pub required_steps: u32,
    pub paid: Resources,
    pub status: ConstructionStatus,
    pub last_progress_round: Option<u32>,
}

impl ConstructionOrder {
    pub fn is_open(&self) -> bool {
        matches!(
            self.status,
            ConstructionStatus::Active | ConstructionStatus::Paused { .. }
        )
    }
}

impl StrategicCampaign {
    pub(crate) fn initialize_population(
        &mut self,
        rules: &crate::data::construction::ConstructionRules,
    ) {
        let headquarters: Vec<_> = self
            .factions
            .values()
            .map(|faction| faction.headquarters)
            .collect();
        self.world.population = self
            .world
            .sites
            .iter()
            .map(|site| {
                (
                    site.id,
                    if headquarters.contains(&site.id) {
                        rules.population.headquarters
                    } else {
                        rules.population.minimum[&site.habitation]
                    },
                )
            })
            .collect();
    }

    pub(crate) fn trim_terminal_orders(&mut self) {
        let mut latest = BTreeMap::new();
        for order in self.construction.values().filter(|order| !order.is_open()) {
            latest.insert(order.target, order.id);
        }
        self.construction
            .retain(|id, order| order.is_open() || latest.get(&order.target) == Some(id));
    }

    pub(crate) fn cancel_lost_construction(&mut self) {
        let lost: Vec<_> = self
            .construction
            .values()
            .filter(|order| {
                order.is_open() && !self.owns_construction_target(order.owner, order.target)
            })
            .map(|order| order.id)
            .collect();
        for id in lost {
            let order = self.construction.get_mut(&id).expect("existing order");
            order.status = ConstructionStatus::Cancelled {
                completed_rounds: self.completed_rounds,
                reason: CancellationReason::ControlLost,
            };
        }
        self.trim_terminal_orders();
    }

    pub(crate) fn owns_construction_target(
        &self,
        owner: FactionId,
        target: ConstructionTarget,
    ) -> bool {
        match target {
            ConstructionTarget::Site(id) => self
                .world
                .site(id)
                .is_some_and(|site| site.controller == Some(owner)),
            ConstructionTarget::Route(id) => self.world.route(id).is_some_and(|route| {
                [route.from, route.to].iter().all(|id| {
                    self.world
                        .site(*id)
                        .is_some_and(|site| site.controller == Some(owner))
                })
            }),
        }
    }
}

impl fmt::Display for ConstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Outpost => "Outpost",
            Self::Road => "Road",
            Self::RoadRepair => "Road repair",
            Self::Fort => "Fort",
            Self::Facility(Facility::TrainingGround) => "Training ground",
            Self::Facility(Facility::Stable) => "Stable",
            Self::Facility(Facility::Infirmary) => "Infirmary",
            Self::Facility(Facility::Workshop) => "Workshop",
            Self::Facility(Facility::Temple) => "Temple",
        })
    }
}
impl fmt::Display for ConstructionPause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BuilderMissing => "Assign a replacement builder",
            Self::BuilderAway => "Builder is away from the work",
            Self::SupplyLost => "Supply connection is broken",
            Self::Contested => "The work site is contested",
            Self::Combat => "Combat interrupted this season's work",
            Self::NoSettlers => "No settlers available",
            Self::Threat => "Clear the local threat before work resumes",
            Self::Ruined => "Reclaim these ruins before other work resumes",
        })
    }
}
impl fmt::Display for CancellationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Player => "Cancelled by owner",
            Self::ControlLost => "Control was lost",
        })
    }
}
