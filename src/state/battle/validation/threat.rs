//! Historical local defenders retain only their real identity and immutable receipt.

use super::*;
use crate::data::economy::Resources;

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Result<(), String> {
    let BattleDefender::Threat(threat) = &report.defender else {
        return ensure(
            report
                .exchanges
                .iter()
                .all(|exchange| exchange.threat_losses == 0),
            "faction encounter has threat losses",
        );
    };
    if let Some(simulation) = &report.simulation {
        return validate_simulated_threat(campaign, data, report, threat, simulation);
    }
    let definition = &data.threats.definitions[&threat.kind];
    ensure(
        threat.id.0 > 0
            && threat.id < campaign.next_ids.threat
            && threat.name == definition.name
            && threat.start > 0
            && threat.start <= definition.headcount
            && threat.attack == definition.attack
            && threat.resistance == definition.resistance
            && report.context == BattleContext::Field
            && report.counters.is_empty(),
        "invalid historical threat definition or context",
    )?;
    ensure(
        u64::from(threat.start)
            == u64::from(threat.end)
                + u64::from(threat.combat_losses)
                + u64::from(threat.encirclement_losses),
        "invalid threat casualty totals",
    )?;
    let mut remaining = threat.start;
    for exchange in &report.exchanges {
        ensure(
            exchange.threat_losses > 0 && exchange.threat_losses <= remaining,
            "invalid threat exchange loss",
        )?;
        remaining -= exchange.threat_losses;
    }
    ensure(
        remaining == threat.start - threat.combat_losses,
        "threat exchanges disagree with combat losses",
    )?;
    let won = report.outcome == BattleOutcome::AttackerVictory;
    let mutual = report.outcome == BattleOutcome::MutualDestruction;
    let zero = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    ensure(
        if won {
            threat.end == 0
                && threat.encirclement_losses == remaining
                && threat.payout == definition.reward
        } else if mutual {
            threat.end == 0
                && remaining == 0
                && threat.encirclement_losses == 0
                && threat.payout == zero
        } else {
            threat.end > 0 && threat.encirclement_losses == 0 && threat.payout == zero
        },
        "threat outcome or single-use payout is inconsistent",
    )
}

fn validate_simulated_threat(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
    threat: &ThreatSideReport,
    simulation: &crate::state::battle::simulation::BattleResolution,
) -> Result<(), String> {
    use crate::state::battle::simulation::BattleUnitId;
    let definition = &data.threats.definitions[&threat.kind];
    let final_count = simulation
        .units
        .iter()
        .find(|unit| unit.id == BattleUnitId::Threat(threat.id))
        .map(|unit| unit.headcount)
        .ok_or("campaign.battles: missing immutable threat unit")?;
    let damage_total = simulation.events.iter().fold(0_u64, |total, event| {
        if let crate::state::battle::simulation::BattleEvent::Damage {
            target: BattleUnitId::Threat(id),
            amount,
            ..
        } = event
        {
            if *id == threat.id {
                total + u64::from(*amount)
            } else {
                total
            }
        } else {
            total
        }
    });
    let zero = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    ensure(
        threat.id.0 > 0
            && threat.id < campaign.next_ids.threat
            && threat.name == definition.name
            && threat.start > 0
            && threat.start <= definition.headcount
            && threat.attack == definition.attack
            && threat.resistance == definition.resistance
            && report.context == BattleContext::Field
            && report.counters.is_empty()
            && damage_total == u64::from(threat.combat_losses)
            && threat.start == final_count.saturating_add(threat.combat_losses)
            && u64::from(threat.start)
                == u64::from(threat.end)
                    + u64::from(threat.combat_losses)
                    + u64::from(threat.encirclement_losses)
            && match report.outcome {
                BattleOutcome::AttackerVictory => {
                    threat.end == 0
                        && threat.encirclement_losses == final_count
                        && threat.payout == definition.reward
                }
                BattleOutcome::MutualDestruction => {
                    threat.end == 0
                        && final_count == 0
                        && threat.encirclement_losses == 0
                        && threat.payout == zero
                }
                BattleOutcome::DefenderVictory | BattleOutcome::Stalemate => {
                    threat.end == final_count
                        && threat.encirclement_losses == 0
                        && threat.payout == zero
                }
            },
        "invalid immutable threat receipt or single-use reward",
    )
}
