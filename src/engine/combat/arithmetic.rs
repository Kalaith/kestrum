//! One final integer division per casualty target; no rounded intermediate strength.

use super::*;

pub(super) fn exchanges(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) -> Result<(), RuleError> {
    let attacker = roster(&report.attacker);
    let defender = roster(&report.defender);
    let starts = [
        power(campaign, data, &attacker)?,
        power(campaign, data, &defender)?,
    ];
    for number in 0..data.combat.max_exchanges {
        let leadership = leadership_snapshot(campaign, data, report);
        let a: Vec<_> = attacker
            .iter()
            .copied()
            .filter(|id| campaign.formations[id].headcount > 0)
            .collect();
        let d: Vec<_> = defender
            .iter()
            .copied()
            .filter(|id| campaign.formations[id].headcount > 0)
            .collect();
        let mut losses = calculate_losses(
            campaign,
            data,
            AttackWave {
                sources: &a,
                targets: &d,
                side: &report.attacker,
                exchange: number,
                terrain: report.terrain_permille,
            },
            &mut report.counters,
        )?;
        losses.extend(calculate_losses(
            campaign,
            data,
            AttackWave {
                sources: &d,
                targets: &a,
                side: &report.defender,
                exchange: number,
                terrain: 1000,
            },
            &mut report.counters,
        )?);
        losses.sort_by_key(|loss| loss.formation);
        for loss in &losses {
            campaign
                .formations
                .get_mut(&loss.formation)
                .expect("roster member")
                .headcount -= loss.amount;
        }
        report.exchanges.push(BattleExchange {
            number: number + 1,
            losses,
            leadership,
        });
        let remaining = [
            power(campaign, data, &attacker)?,
            power(campaign, data, &defender)?,
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
        let outcome = if routed[0] {
            BattleOutcome::DefenderVictory
        } else {
            BattleOutcome::AttackerVictory
        };
        return Ok(Some((outcome, BattleEndReason::Rout)));
    }
    if !limit {
        return Ok(None);
    }
    // Cross multiplication compares exact remaining fractions, without rounding.
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
        .attacker
        .armies
        .iter()
        .chain(&report.defender.armies)
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

fn roster(side: &BattleSideReport) -> Vec<FormationId> {
    side.armies
        .iter()
        .flat_map(|army| army.formations.iter().map(|formation| formation.id))
        .collect()
}

fn power(
    campaign: &StrategicCampaign,
    data: &GameData,
    ids: &[FormationId],
) -> Result<u128, RuleError> {
    ids.iter().try_fold(0_u128, |sum, id| {
        let formation = &campaign.formations[id];
        add(
            sum,
            mul(
                formation.headcount.into(),
                data.troops.formations[&formation.kind].attack.into(),
            )?,
        )
    })
}

struct AttackWave<'a> {
    sources: &'a [FormationId],
    targets: &'a [FormationId],
    side: &'a BattleSideReport,
    exchange: u32,
    terrain: u32,
}

fn calculate_losses(
    campaign: &StrategicCampaign,
    data: &GameData,
    wave: AttackWave<'_>,
    counters: &mut Vec<CounterUse>,
) -> Result<Vec<FormationLoss>, RuleError> {
    let AttackWave {
        sources,
        targets,
        side,
        exchange,
        terrain,
    } = wave;
    let mut assigned = BTreeMap::<FormationId, u128>::new();
    for (index, id) in sources.iter().enumerate() {
        let source = &campaign.formations[id];
        let target = &campaign.formations[&targets[(index + exchange as usize) % targets.len()]];
        let army = side
            .armies
            .iter()
            .find(|army| army.formations.iter().any(|formation| formation.id == *id))
            .expect("source army");
        let leadership = campaign
            .army_leadership_permille(army.id, data)
            .expect("source army");
        let counter = data.combat.counter(source.kind, target.kind);
        if counter != 1000
            && !counters
                .iter()
                .any(|entry| entry.source == source.kind && entry.target == target.kind)
        {
            counters.push(CounterUse {
                source: source.kind,
                target: target.kind,
                permille: counter,
            });
        }
        // Ordinary veterancy is 1000 until earned progression is introduced. Its
        // numerator/denominator cancel exactly; future multipliers belong here.
        let attack = mul(
            mul(
                mul(
                    source.headcount.into(),
                    data.troops.formations[&source.kind].attack.into(),
                )?,
                leadership.into(),
            )?,
            counter.into(),
        )?;
        let total = assigned.entry(target.id).or_default();
        *total = add(*total, attack)?;
    }
    assigned
        .into_iter()
        .map(|(id, attack)| {
            let target = &campaign.formations[&id];
            // Attack denominator 10^6; resistance denominator 10^3. Cancel only
            // their common factor, preserving e.g. Rider forest resistance 19.8.
            let divisor = mul(
                mul(
                    mul(1000, data.combat.casualty_divisor.into())?,
                    data.troops.formations[&target.kind].resistance.into(),
                )?,
                terrain.into(),
            )?;
            let amount = (attack / divisor)
                .max(u128::from(attack > 0))
                .min(target.headcount.into()) as u32;
            Ok(FormationLoss {
                formation: id,
                amount,
            })
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
