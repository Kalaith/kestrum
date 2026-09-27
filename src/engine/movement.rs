//! Physical route previews and per-member movement; no region-marker shortcuts.

mod execute;
mod path;
mod service;

use super::{actions::validate_command, Actor, Command, RuleError};
use crate::{
    data::{
        world::{DiplomaticState, FactionId, MilitaryLayer, Route, RouteId, SiteId},
        GameData,
    },
    state::{
        battle::BattleId,
        military::{ArmyId, FormationId},
        people::PersonId,
        StrategicCampaign,
    },
};
use std::{collections::BTreeSet, fmt};

pub(super) use execute::{execute, spend_edge};
pub(super) use service::service_snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOrder {
    pub armies: Vec<ArmyId>,
    /// Includes the origin and every confirmed physical destination in order.
    pub path: Vec<SiteId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteStep {
    pub route: RouteId,
    pub from: SiteId,
    pub to: SiteId,
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementBlock {
    InsufficientMovement { required: u32, remaining: u32 },
    PeaceBoundary,
    EncounterUnavailable,
    Contested,
    SiegeExitRequired,
    RouteUnavailable,
    ThreatRequiresClear,
}

impl fmt::Display for MovementBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThreatRequiresClear => {
                f.write_str("A local threat blocks entry. Use Clear Threat to attack it.")
            }
            Self::InsufficientMovement {
                required,
                remaining,
            } => write!(
                f,
                "The next edge costs {required}; this group has {remaining} movement left."
            ),
            Self::PeaceBoundary => {
                f.write_str("Peace grants no military access to this foreign site.")
            }
            Self::EncounterUnavailable => {
                f.write_str("This encounter involves incompatible military sides.")
            }
            Self::SiegeExitRequired => f.write_str("Use Withdraw or Escape to leave this siege."),
            Self::Contested => f.write_str("A contested site requires an encounter before entry."),
            Self::RouteUnavailable => f.write_str("The next physical route is unavailable."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementStop {
    pub site: SiteId,
    pub reason: MovementBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementPreview {
    pub encounter: Option<MovementEncounter>,
    pub order: MoveOrder,
    pub steps: Vec<RouteStep>,
    pub total_cost: u32,
    pub remaining: u32,
    pub reachable_steps: usize,
    pub reachable_site: SiteId,
    pub stop: Option<MovementStop>,
    pub supplied_after: bool,
    pub uncertain_contact: bool,
    /// Observed at the current positions; future travel never grants remote sight.
    pub observed_hostile_sites: BTreeSet<SiteId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementEncounter {
    EstablishSiege,
    JoinBesiegers,
    Relief,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementOutcome {
    pub battle: Option<BattleId>,
    pub armies: Vec<ArmyId>,
    pub path: Vec<SiteId>,
    pub spent: u32,
    pub stop: Option<MovementStop>,
}

pub fn army_remaining(
    campaign: &StrategicCampaign,
    data: &GameData,
    army: ArmyId,
) -> Result<u32, RuleError> {
    campaign
        .army_movement_remaining(army, data)
        .ok_or(RuleError::UnknownArmy { army })
}

pub fn formation_remaining(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: FormationId,
) -> Result<u32, RuleError> {
    let formation = campaign
        .formations
        .get(&id)
        .ok_or(RuleError::UnknownFormation { formation: id })?;
    let allowance = formation.movement_allowance(data);
    Ok(allowance.saturating_sub(formation.movement_spent))
}

pub fn person_remaining(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: PersonId,
) -> Result<u32, RuleError> {
    let person = campaign
        .people
        .get(&id)
        .ok_or(RuleError::UnknownPerson { person: id })?;
    if !person.is_alive() {
        return Err(RuleError::UnknownPerson { person: id });
    }
    Ok(data
        .rules
        .leadership
        .officer_movement_allowance
        .saturating_sub(person.movement_spent))
}

pub fn route_cost(route: &Route, data: &GameData) -> u32 {
    let rules = &data.rules.movement;
    let bonus = if route.road.improved && route.road.damage < rules.road_disabled_damage {
        rules.road_bonus
    } else {
        0
    };
    route
        .terrain_cost
        .saturating_sub(bonus)
        .max(rules.minimum_edge_cost)
}

pub fn movement_preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    armies: &[ArmyId],
    destination: SiteId,
) -> Result<MovementPreview, RuleError> {
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    let (origin, _) = validate_group(campaign, data, observer, armies)?;
    if campaign.world.site(destination).is_none() {
        return Err(RuleError::UnknownSite { site: destination });
    }
    let path = path::shortest(campaign, data, observer, origin, destination, true)
        .or_else(|| path::shortest(campaign, data, observer, origin, destination, false))
        .ok_or(RuleError::InvalidRoute)?;
    let mut armies = armies.to_vec();
    armies.sort();
    preview_order(campaign, data, observer, &MoveOrder { armies, path })
}

pub(super) fn preview_order(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    order: &MoveOrder,
) -> Result<MovementPreview, RuleError> {
    data.economy.validate().map_err(RuleError::InvalidState)?;
    data.rules.validate().map_err(RuleError::InvalidState)?;
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    let actor = if observer == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(observer)
    };
    validate_command(campaign, actor, &Command::Move(order.clone()))?;
    let (origin, remaining) = validate_order(campaign, data, observer, order)?;
    let mut steps = Vec::new();
    let mut total_cost = 0_u32;
    let mut reachable_steps = 0;
    let mut reachable_site = origin;
    let mut left = remaining;
    let mut stop = None;
    let threats = super::threats::visible_threats(campaign, observer);
    for pair in order.path.windows(2) {
        let [from, to] = [pair[0], pair[1]];
        let Some(route) = campaign.world.connected_route(from, to) else {
            stop.get_or_insert(MovementStop {
                site: to,
                reason: MovementBlock::RouteUnavailable,
            });
            break;
        };
        let cost = route_cost(route, data);
        total_cost = total_cost.checked_add(cost).ok_or(RuleError::Overflow {
            field: "route cost",
        })?;
        steps.push(RouteStep {
            route: route.id,
            from,
            to,
            cost,
        });
        if stop.is_some() {
            continue;
        }
        let reason = public_block(campaign, observer, to)
            .or_else(|| {
                threats
                    .iter()
                    .any(|threat| threat.site == to)
                    .then_some(MovementBlock::ThreatRequiresClear)
            })
            .or_else(|| {
                (cost > left).then_some(MovementBlock::InsufficientMovement {
                    required: cost,
                    remaining: left,
                })
            });
        if let Some(reason) = reason {
            stop = Some(MovementStop { site: to, reason });
        } else {
            left -= cost;
            reachable_steps += 1;
            reachable_site = to;
        }
    }
    Ok(MovementPreview {
        encounter: order
            .path
            .iter()
            .skip(1)
            .find_map(|site| encounter(campaign, observer, *site)),
        order: order.clone(),
        steps,
        total_cost,
        remaining,
        reachable_steps,
        reachable_site,
        stop,
        supplied_after: forecast_supply(
            campaign,
            observer,
            &order.path[..=reachable_steps],
            &threats,
        ),
        uncertain_contact: order.path.iter().skip(1).any(|site| {
            campaign
                .world
                .site(*site)
                .is_some_and(|site| site.controller != Some(observer))
        }),
        observed_hostile_sites: super::hostile_presence(campaign, observer)
            .into_iter()
            .filter(|site| order.path.iter().skip(1).any(|step| step == site))
            .collect(),
    })
}

fn forecast_supply(
    campaign: &StrategicCampaign,
    observer: FactionId,
    path: &[SiteId],
    threats: &[super::threats::VisibleThreat],
) -> bool {
    // Hypothetical future ownership cannot discover distant occupants through supply.
    let mut known = campaign.clone();
    known
        .threats
        .retain(|id, _| threats.iter().any(|threat| threat.id == *id));
    for id in path.iter().skip(1) {
        known
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == *id)
            .expect("known site")
            .controller = Some(observer);
    }
    path.last()
        .is_some_and(|site| known.supplied_sites(observer).contains(site))
}

pub(super) fn validate_group(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    armies: &[ArmyId],
) -> Result<(SiteId, u32), RuleError> {
    if armies.is_empty() || armies.iter().copied().collect::<BTreeSet<_>>().len() != armies.len() {
        return Err(RuleError::InvalidArmyGroup);
    }
    let mut origin = None;
    let mut remaining = u32::MAX;
    for &id in armies {
        let army = campaign
            .armies
            .get(&id)
            .ok_or(RuleError::UnknownArmy { army: id })?;
        if army.faction != owner {
            return Err(RuleError::ArmyNotOwned { army: id });
        }
        if origin.is_some_and(|site| site != army.site) {
            return Err(RuleError::NotColocated);
        }
        origin = Some(army.site);
        remaining = remaining.min(army_remaining(campaign, data, id)?);
    }
    Ok((origin.ok_or(RuleError::InvalidArmyGroup)?, remaining))
}

fn validate_order(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    order: &MoveOrder,
) -> Result<(SiteId, u32), RuleError> {
    let (origin, remaining) = validate_group(campaign, data, owner, &order.armies)?;
    if campaign.sieges.contains_key(&origin) {
        return Err(RuleError::MovementBlocked {
            site: origin,
            reason: MovementBlock::SiegeExitRequired,
        });
    }
    if order.path.len() < 2
        || order.path.first() != Some(&origin)
        || order.path.iter().copied().collect::<BTreeSet<_>>().len() != order.path.len()
        || order
            .path
            .iter()
            .any(|site| campaign.world.site(*site).is_none())
    {
        return Err(RuleError::InvalidRoute);
    }
    Ok((origin, remaining))
}

pub(super) fn public_block(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: SiteId,
) -> Option<MovementBlock> {
    let site = campaign.world.site(site)?;
    if let Some(foreign) = site.controller.filter(|id| *id != owner) {
        let peace = campaign
            .relations
            .iter()
            .find(|relation| {
                relation.factions.contains(&owner) && relation.factions.contains(&foreign)
            })
            .is_some_and(|relation| relation.state == DiplomaticState::Peace);
        if peace {
            return Some(MovementBlock::PeaceBoundary);
        }
    }
    if campaign.world.contested_sites.contains(&site.id) && site.military == MilitaryLayer::None {
        return Some(MovementBlock::Contested);
    }

    None
}

fn encounter(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: SiteId,
) -> Option<MovementEncounter> {
    if let Some(siege) = campaign.sieges.get(&site) {
        if owner == siege.defender {
            return Some(MovementEncounter::Relief);
        }
        if owner == siege.besieger {
            return Some(MovementEncounter::JoinBesiegers);
        }
    }
    campaign
        .world
        .site(site)
        .filter(|site| site.military != MilitaryLayer::None && site.controller != Some(owner))
        .map(|_| MovementEncounter::EstablishSiege)
}
