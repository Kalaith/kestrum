//! Confirmed orders retain every legal traversed edge when a later edge stops.

use super::super::{combat, retreat};
use super::*;
use crate::state::people::PersonAssignment;

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    order: &MoveOrder,
) -> Result<MovementOutcome, RuleError> {
    let owner = campaign.active_faction();
    let (origin, mut remaining) = validate_order(campaign, data, owner, order)?;
    let mut armies = order.armies.clone();
    armies.sort();
    let mut outcome = MovementOutcome {
        battle: None,
        armies,
        path: vec![origin],
        spent: 0,
        stop: None,
    };
    for pair in order.path.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let route = campaign.world.connected_route(from, to);
        let cost = route.map(|route| route_cost(route, data));
        let contact = contact(campaign, owner, to);
        let reason = if let Some(cost) = cost {
            public_block(campaign, owner, to)
                .or_else(|| {
                    (cost > remaining).then_some(MovementBlock::InsufficientMovement {
                        required: cost,
                        remaining,
                    })
                })
                .or(contact.block.clone())
        } else {
            Some(MovementBlock::RouteUnavailable)
        };
        if let Some(reason) = reason {
            if outcome.path.len() == 1 {
                return Err(RuleError::MovementBlocked { site: to, reason });
            }
            outcome.stop = Some(MovementStop { site: to, reason });
            break;
        }
        let cost = cost.ok_or(RuleError::InvalidRoute)?;
        spend_edge(campaign, &outcome.armies, to, cost)?;
        remaining -= cost;
        outcome.spent = outcome.spent.checked_add(cost).ok_or(RuleError::Overflow {
            field: "movement spent",
        })?;
        outcome.path.push(to);
        if !contact.defenders.is_empty() {
            outcome.battle = Some(combat::resolve(
                campaign,
                data,
                &outcome.armies,
                &contact.defenders,
                from,
                to,
            )?);
            break;
        }
        if !contact.neutral_peaceful_stack {
            combat::capture(campaign, data, to, owner);
        }
    }
    Ok(outcome)
}

struct Contact {
    defenders: Vec<ArmyId>,
    neutral_peaceful_stack: bool,
    block: Option<MovementBlock>,
}

fn contact(campaign: &StrategicCampaign, owner: FactionId, to: SiteId) -> Contact {
    let occupants: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.site == to && army.faction != owner)
        .map(|army| army.id)
        .collect();
    let foreign: BTreeSet<_> = occupants
        .iter()
        .map(|id| campaign.armies[id].faction)
        .collect();
    let peaceful = foreign
        .iter()
        .any(|faction| !retreat::hostile(campaign, owner, *faction));
    let defenders: Vec<_> = occupants
        .iter()
        .copied()
        .filter(|id| retreat::hostile(campaign, owner, campaign.armies[id].faction))
        .collect();
    let neutral_peaceful_stack = peaceful
        && defenders.is_empty()
        && campaign
            .world
            .site(to)
            .is_some_and(|site| site.controller.is_none());
    let block = if peaceful && !defenders.is_empty() {
        Some(MovementBlock::EncounterUnavailable)
    } else if peaceful && !neutral_peaceful_stack {
        Some(MovementBlock::PeaceBoundary)
    } else if (!peaceful && foreign.len() > 1)
        || (!defenders.is_empty()
            && campaign
                .world
                .site(to)
                .is_some_and(|site| site.military != MilitaryLayer::None))
    {
        Some(MovementBlock::EncounterUnavailable)
    } else {
        None
    };
    Contact {
        defenders,
        neutral_peaceful_stack,
        block,
    }
}

fn spend_edge(
    campaign: &mut StrategicCampaign,
    armies: &[ArmyId],
    destination: SiteId,
    cost: u32,
) -> Result<(), RuleError> {
    let formations: BTreeSet<_> = armies
        .iter()
        .filter_map(|id| campaign.armies.get(id))
        .flat_map(|army| army.formation_ids())
        .collect();
    for id in armies {
        campaign
            .armies
            .get_mut(id)
            .ok_or(RuleError::UnknownArmy { army: *id })?
            .site = destination;
    }
    for id in &formations {
        let formation = campaign
            .formations
            .get_mut(id)
            .ok_or(RuleError::UnknownFormation { formation: *id })?;
        formation.movement_spent =
            formation
                .movement_spent
                .checked_add(cost)
                .ok_or(RuleError::Overflow {
                    field: "formation movement",
                })?;
    }
    for person in campaign.people.values_mut() {
        if matches!(person.assignment, PersonAssignment::Formation {formation} if formations.contains(&formation))
        {
            person.movement_spent =
                person
                    .movement_spent
                    .checked_add(cost)
                    .ok_or(RuleError::Overflow {
                        field: "person movement",
                    })?;
        }
    }
    Ok(())
}
