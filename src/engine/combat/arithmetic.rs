//! Shared exact simultaneous arithmetic for sovereign formations and local threats.

use super::*;
use crate::state::threat::ThreatId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CombatantId {
    Formation(FormationId),
    Threat(ThreatId),
}

struct Combatant {
    id: CombatantId,
    headcount: u32,
    attack: u32,
    resistance: u32,
    troop: Option<crate::data::economy::TroopKind>,
    leadership: u32,
    veterancy: u32,
}

pub(super) fn exchanges(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) -> Result<(), RuleError> {
    let starts = [
        power(&formation_roster(campaign, data, &report.attacker))?,
        power(&defender_roster(campaign, data, report))?,
    ];
    for number in 0..data.combat.max_exchanges {
        let a = formation_roster(campaign, data, &report.attacker);
        let d = defender_roster(campaign, data, report);
        let leadership = leadership_snapshot(campaign, data, report);
        let wall = effective_wall(data, report, &a);
        let mut losses = calculate_losses(
            data,
            &a,
            &d,
            number,
            report.terrain_permille,
            wall,
            &mut report.counters,
        )?;
        losses.extend(calculate_losses(
            data,
            &d,
            &a,
            number,
            1000,
            1000,
            &mut report.counters,
        )?);
        losses.sort_by_key(|(id, _)| *id);
        let mut formation_losses = Vec::new();
        let mut threat_losses = 0;
        for (id, amount) in losses {
            match id {
                CombatantId::Formation(id) => {
                    campaign
                        .formations
                        .get_mut(&id)
                        .expect("participant")
                        .headcount -= amount;
                    formation_losses.push(FormationLoss {
                        formation: id,
                        amount,
                    });
                }
                CombatantId::Threat(id) => {
                    campaign
                        .threats
                        .get_mut(&id)
                        .expect("participant")
                        .headcount -= amount;
                    threat_losses = amount;
                }
            }
        }
        report.exchanges.push(BattleExchange {
            wall_permille: wall,
            number: number + 1,
            losses: formation_losses,
            threat_losses,
            leadership,
        });
        let remaining = [
            power(&formation_roster(campaign, data, &report.attacker))?,
            power(&defender_roster(campaign, data, report))?,
        ];
        if let Some((outcome, reason)) = result(
            data,
            starts,
            remaining,
            number + 1 == data.combat.max_exchanges,
        )? {
            report.outcome = outcome;
            report.reason = reason;
            break;
        }
    }
    Ok(())
}

fn formation_roster(
    campaign: &StrategicCampaign,
    data: &GameData,
    side: &BattleSideReport,
) -> Vec<Combatant> {
    side.armies
        .iter()
        .flat_map(|army| {
            let leadership = campaign
                .army_leadership_permille(army.id, data)
                .expect("participant");
            army.formations.iter().filter_map(move |snapshot| {
                let formation = &campaign.formations[&snapshot.id];
                (formation.headcount > 0).then(|| Combatant {
                    id: CombatantId::Formation(formation.id),
                    headcount: formation.headcount,
                    attack: data.troops.formations[&formation.kind].attack,
                    resistance: data.troops.formations[&formation.kind].resistance,
                    troop: Some(formation.kind),
                    leadership,
                    veterancy: formation.service.tier.permille(&data.progression),
                })
            })
        })
        .collect()
}

fn defender_roster(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Vec<Combatant> {
    match &report.defender {
        BattleDefender::Faction(side) => formation_roster(campaign, data, side),
        BattleDefender::Threat(side) => {
            let threat = &campaign.threats[&side.id];
            if threat.headcount == 0 {
                return Vec::new();
            }
            vec![Combatant {
                id: CombatantId::Threat(side.id),
                headcount: threat.headcount,
                attack: side.attack,
                resistance: side.resistance,
                troop: None,
                leadership: data.threats.leadership_permille,
                veterancy: 1000,
            }]
        }
    }
}

fn result(
    data: &GameData,
    starts: [u128; 2],
    remaining: [u128; 2],
    limit: bool,
) -> Result<Option<(BattleOutcome, BattleEndReason)>, RuleError> {
    if remaining.contains(&0) {
        let outcome = match remaining {
            [0, 0] => BattleOutcome::MutualDestruction,
            [0, _] => BattleOutcome::DefenderVictory,
            _ => BattleOutcome::AttackerVictory,
        };
        return Ok(Some((outcome, BattleEndReason::Annihilation)));
    }
    let routed = [
        mul(remaining[0], 1000)? <= mul(starts[0], data.combat.rout_permille.into())?,
        mul(remaining[1], 1000)? <= mul(starts[1], data.combat.rout_permille.into())?,
    ];
    if routed.iter().any(|value| *value) {
        return Ok(Some((
            if routed[0] {
                BattleOutcome::DefenderVictory
            } else {
                BattleOutcome::AttackerVictory
            },
            BattleEndReason::Rout,
        )));
    }
    if !limit {
        return Ok(None);
    }
    let left = mul(mul(remaining[0], starts[1])?, 1000)?;
    let right = mul(mul(remaining[1], starts[0])?, 1000)?;
    let margin = mul(
        mul(starts[0], starts[1])?,
        data.combat.victory_margin_permille.into(),
    )?;
    let outcome = if left > right && left - right >= margin {
        BattleOutcome::AttackerVictory
    } else if right > left && right - left >= margin {
        BattleOutcome::DefenderVictory
    } else {
        BattleOutcome::Stalemate
    };
    Ok(Some((outcome, BattleEndReason::ExchangeLimit)))
}

fn leadership_snapshot(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Vec<ArmyLeadership> {
    let mut leadership: Vec<_> = report
        .faction_sides()
        .flat_map(|side| &side.armies)
        .filter(|army| {
            army.formations
                .iter()
                .any(|formation| campaign.formations[&formation.id].headcount > 0)
        })
        .map(|army| ArmyLeadership {
            army: army.id,
            permille: campaign
                .army_leadership_permille(army.id, data)
                .expect("participant"),
        })
        .collect();
    leadership.sort_by_key(|entry| entry.army);
    leadership
}

fn power(roster: &[Combatant]) -> Result<u128, RuleError> {
    roster.iter().try_fold(0_u128, |sum, unit| {
        add(sum, mul(unit.headcount.into(), unit.attack.into())?)
    })
}

fn calculate_losses(
    data: &GameData,
    sources: &[Combatant],
    targets: &[Combatant],
    exchange: u32,
    terrain: u32,
    wall: u32,
    counters: &mut Vec<CounterUse>,
) -> Result<Vec<(CombatantId, u32)>, RuleError> {
    let mut assigned = BTreeMap::<CombatantId, u128>::new();
    for (index, source) in sources.iter().enumerate() {
        let target = &targets[(index + exchange as usize) % targets.len()];
        let counter = match (source.troop, target.troop) {
            (Some(source), Some(target)) => {
                let permille = data.combat.counter(source, target);
                if permille != 1000
                    && !counters
                        .iter()
                        .any(|entry| entry.source == source && entry.target == target)
                {
                    counters.push(CounterUse {
                        source,
                        target,
                        permille,
                    });
                }
                permille
            }
            _ => 1000,
        };
        let attack = mul(
            mul(
                mul(source.headcount.into(), source.attack.into())?,
                source.leadership.into(),
            )?,
            counter.into(),
        )?;
        let attack = mul(attack, source.veterancy.into())?;
        let total = assigned.entry(target.id).or_default();
        *total = add(*total, attack)?;
    }
    assigned
        .into_iter()
        .map(|(id, attack)| {
            let target = targets
                .iter()
                .find(|target| target.id == id)
                .expect("assigned target");
            let divisor = mul(
                mul(
                    mul(1000, data.combat.casualty_divisor.into())?,
                    target.resistance.into(),
                )?,
                terrain.into(),
            )?;
            let divisor = mul(divisor, target.veterancy.into())?;
            let amount = (mul(attack, 1000)? / mul(divisor, wall.into())?)
                .max(u128::from(attack > 0))
                .min(target.headcount.into()) as u32;
            Ok((id, amount))
        })
        .collect()
}

fn mul(left: u128, right: u128) -> Result<u128, RuleError> {
    left.checked_mul(right).ok_or(RuleError::Overflow {
        field: "combat arithmetic",
    })
}
fn add(left: u128, right: u128) -> Result<u128, RuleError> {
    left.checked_add(right).ok_or(RuleError::Overflow {
        field: "combat arithmetic",
    })
}

fn effective_wall(data: &GameData, report: &BattleReport, attackers: &[Combatant]) -> u32 {
    if !matches!(report.context, BattleContext::Assault { .. }) {
        return 1000;
    }
    if attackers
        .iter()
        .any(|unit| unit.troop == Some(crate::data::economy::TroopKind::SiegeEngines))
    {
        report
            .wall_permille
            .saturating_sub(data.siege.engine_wall_reduction_permille)
            .max(data.siege.wall_minimum_permille)
    } else {
        report.wall_permille
    }
}
