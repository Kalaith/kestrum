//! NPC doctrine and deployment decisions use only the current visible encounter.

use super::{apply_doctrine_snapshot, RuleError, StrategicCampaign};
use crate::state::battle::{BattleReport, BattleSideReport};
use crate::{
    data::{world::FactionId, GameData},
    engine::battle_strategy::{plan_rival_battle, ObservedTroopPosition},
};

pub(super) fn prepare_rivals(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) -> Result<(), RuleError> {
    if report.defender.faction_side().is_none() {
        return Ok(());
    }
    let attacker_positions = positions(&report.attacker);
    let defender_positions = report
        .defender
        .faction_side()
        .map(positions)
        .unwrap_or_default();
    let mut plans = Vec::new();
    if report.attacker.faction != campaign.player {
        plans.push((
            report.attacker.faction,
            plan_rival_battle(&attacker_positions, &defender_positions),
        ));
    }
    if let Some(defender) = report
        .defender
        .faction_side()
        .filter(|side| side.faction != campaign.player)
    {
        plans.push((
            defender.faction,
            plan_rival_battle(&defender_positions, &attacker_positions),
        ));
    }
    for (faction, plan) in plans {
        let side = side_mut(report, faction);
        for (army_index, army) in side.armies.iter_mut().enumerate() {
            let army_index = u8::try_from(army_index)
                .map_err(|_| RuleError::InvalidState("Too many battle armies.".into()))?;
            apply_doctrine_snapshot(campaign, data, army.id, plan.doctrine);
            army.battle_doctrine = Some(plan.doctrine);
            army.ai_prepared = true;
            for formation in &mut army.formations {
                if let Some(movement) = plan.slot_moves.iter().find(|movement| {
                    movement.army_index == army_index && movement.from == formation.slot as u8
                }) {
                    formation.slot = usize::from(movement.to);
                }
            }
            army.formations.sort_by_key(|formation| formation.slot);
        }
    }
    Ok(())
}

fn positions(side: &BattleSideReport) -> Vec<ObservedTroopPosition> {
    side.armies
        .iter()
        .enumerate()
        .flat_map(|(army_index, army)| {
            let army_index = u8::try_from(army_index).expect("battle army limit is bounded");
            army.formations
                .iter()
                .map(move |formation| ObservedTroopPosition {
                    army_index,
                    kind: formation.kind,
                    slot: u8::try_from(formation.slot).expect("validated formation slot"),
                })
        })
        .collect()
}

fn side_mut(report: &mut BattleReport, faction: FactionId) -> &mut BattleSideReport {
    if report.attacker.faction == faction {
        &mut report.attacker
    } else {
        report
            .defender
            .faction_side_mut()
            .filter(|side| side.faction == faction)
            .expect("plan belongs to a faction side")
    }
}
