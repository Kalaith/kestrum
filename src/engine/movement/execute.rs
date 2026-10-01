//! Confirmed orders retain every legal traversed edge when a later edge stops.

use super::super::{combat, retreat};
use super::*;
use crate::state::people::PersonAssignment;

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    order: &MoveOrder,
    action: &mut super::super::ActionOutcome,
) -> Result<MovementOutcome, RuleError> {
    let owner = campaign.active_faction();
    let (origin, mut remaining) = validate_order(campaign, data, owner, order)?;
    if let Some(stop) = peace_boundary(campaign, owner, order) {
        return Err(RuleError::MovementBlocked {
            site: stop.site,
            reason: stop.reason,
        });
    }
    let mut armies = order.armies.clone();
    armies.sort();
    let mut outcome = MovementOutcome {
        planned_destination: None,
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
        let siege_entry = campaign.sieges.contains_key(&to)
            || campaign.world.site(to).is_some_and(|site| {
                site.military != MilitaryLayer::None && site.controller != Some(owner)
            });
        let contact = if siege_entry {
            Contact {
                defenders: Vec::new(),
                neutral_peaceful_stack: false,
                block: super::super::siege::admission(campaign, owner, to).err(),
            }
        } else {
            contact(campaign, owner, to)
        };
        let reason = step_block(campaign, owner, to, cost, remaining, &contact);
        if let Some(reason) = reason {
            if outcome.path.len() == 1
                && !matches!(reason, MovementBlock::InsufficientMovement { .. })
            {
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
        super::super::exploration::visit(campaign, owner, to);
        if siege_entry {
            super::super::siege::arrive(campaign, data, &outcome.armies, from, to, action)?;
            break;
        }
        if let Some(threat) = campaign.active_threat(to).map(|threat| threat.id) {
            super::super::combat::prepare_threat(
                campaign,
                data,
                &outcome.armies,
                from,
                threat,
                None,
            )?;
            break;
        }
        if !contact.defenders.is_empty() {
            super::super::combat::prepare_encounter(
                campaign,
                data,
                super::super::combat::Encounter {
                    attackers: outcome.armies.clone(),
                    defenders: contact.defenders,
                    origin: from,
                    site: to,
                    context: crate::state::battle::BattleContext::Field,
                },
            )?;
            break;
        }
        if !contact.neutral_peaceful_stack {
            combat::capture(campaign, data, to, owner);
        }
    }
    plans::save_remainder(campaign, order, &mut outcome);
    Ok(outcome)
}

fn step_block(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: SiteId,
    cost: Option<u32>,
    remaining: u32,
    contact: &Contact,
) -> Option<MovementBlock> {
    let Some(cost) = cost else {
        return Some(MovementBlock::RouteUnavailable);
    };
    public_block(campaign, owner, site)
        .or_else(|| {
            (cost > remaining).then_some(MovementBlock::InsufficientMovement {
                required: cost,
                remaining,
            })
        })
        .or(contact.block.clone())
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

pub(in crate::engine) fn spend_edge(
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
