//! Confirmed orders retain every legal traversed edge when a later edge stops.

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
        armies,
        path: vec![origin],
        spent: 0,
        stop: None,
    };
    for pair in order.path.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let route = campaign.world.connected_route(from, to);
        let cost = route.map(|route| route_cost(route, data));
        let reason = if let Some(cost) = cost {
            public_block(campaign, owner, to)
                .or_else(|| {
                    campaign
                        .armies
                        .values()
                        .any(|army| army.site == to && army.faction != owner)
                        .then_some(MovementBlock::EncounterUnavailable)
                })
                .or_else(|| {
                    (cost > remaining).then_some(MovementBlock::InsufficientMovement {
                        required: cost,
                        remaining,
                    })
                })
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
        campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == to)
            .ok_or(RuleError::UnknownSite { site: to })?
            .controller = Some(owner);
        campaign.reconcile_region_control();
        remaining -= cost;
        outcome.spent = outcome.spent.checked_add(cost).ok_or(RuleError::Overflow {
            field: "movement spent",
        })?;
        outcome.path.push(to);
    }
    Ok(outcome)
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
