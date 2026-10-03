//! Continue saved orders on the next faction turn, pausing for real encounters.

use super::*;
use crate::state::movement::MovementPlan;

pub(super) fn save_remainder(
    campaign: &mut StrategicCampaign,
    order: &MoveOrder,
    moved: &mut MovementOutcome,
) {
    campaign
        .movement_plans
        .retain(|plan| !plan.armies.iter().any(|army| order.armies.contains(army)));
    if moved
        .stop
        .as_ref()
        .is_some_and(|stop| matches!(stop.reason, MovementBlock::InsufficientMovement { .. }))
        && order
            .path
            .windows(2)
            .all(|pair| campaign.world.connected_route(pair[0], pair[1]).is_some())
    {
        moved.planned_destination = order.path.last().copied();
        campaign.movement_plans.push(MovementPlan {
            armies: moved.armies.clone(),
            path: order.path[moved.path.len() - 1..].to_vec(),
        });
    }
}

pub(in crate::engine) fn cancel_plan(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    army: ArmyId,
) -> Result<(), RuleError> {
    let force = campaign
        .armies
        .get(&army)
        .ok_or(RuleError::UnknownArmy { army })?;
    if force.faction != owner {
        return Err(RuleError::ArmyNotOwned { army });
    }
    campaign
        .movement_plans
        .retain(|plan| !plan.armies.contains(&army));
    Ok(())
}

pub(in crate::engine) fn resume_plans(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut super::super::ActionOutcome,
) -> Result<(), RuleError> {
    campaign.reconcile_movement_plans();
    if matches!(
        campaign.phase,
        crate::state::CampaignPhase::NpcTurn { paused: true, .. }
    ) {
        return Ok(());
    }
    let owner = campaign.active_faction();
    let plans = campaign.movement_plans.clone();
    for plan in plans {
        if campaign.pending_battle.is_some() || campaign.diplomacy.is_blocked() {
            break;
        }
        if campaign
            .armies
            .get(&plan.armies[0])
            .is_none_or(|army| army.faction != owner)
            || !campaign.movement_plans.contains(&plan)
        {
            continue;
        }
        cancel_plan(campaign, owner, plan.armies[0])?;
        let order = MoveOrder {
            armies: plan.armies.clone(),
            path: plan.path.clone(),
        };
        let actor = if owner == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(owner)
        };
        match super::super::apply(campaign, data, actor, Command::Move(order)) {
            Ok(continued) => {
                if let Some(moved) = &continued.movement {
                    outcome.continued_movements.push(moved.clone());
                }
                super::super::actions::merge_outcome(outcome, continued);
            }
            Err(RuleError::MovementBlocked { site, reason }) => {
                let blocked = MovementOutcome {
                    requested_destination: plan.path.last().copied(),
                    armies: plan.armies,
                    path: vec![plan.path[0]],
                    spent: 0,
                    battle: None,
                    planned_destination: None,
                    stop: Some(MovementStop { site, reason }),
                };
                crate::engine::notifications::collect_blocked_continuation(
                    campaign, data, &blocked,
                )
                .map_err(RuleError::InvalidState)?;
                outcome.continued_movements.push(blocked);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
