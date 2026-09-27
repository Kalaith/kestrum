//! A local defender enters the same exchanges and real attacker consequence path.

use super::*;
use crate::{
    data::economy::Resources,
    state::threat::{ThreatId, ThreatStatus},
};

pub(crate) fn resolve_threat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    attackers: &[ArmyId],
    origin: SiteId,
    threat: ThreatId,
) -> Result<BattleId, RuleError> {
    let mut report = report(campaign, data, attackers, origin, threat)?;
    campaign.next_ids.battle = BattleId(report.id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "battle identifiers",
    })?);
    let mut people = PersonCombatContext {
        site: report.site,
        sides: vec![person_side(campaign, data, &report.attacker)],
    };
    arithmetic::exchanges(campaign, data, &mut report)?;
    record_losses(campaign, data, &mut report);
    finish_threat(campaign, data, &mut report)?;
    if matches!(
        report.outcome,
        BattleOutcome::DefenderVictory | BattleOutcome::Stalemate
    ) {
        withdraw(
            campaign,
            &mut report.attacker,
            report.site,
            Some(origin),
            None,
        );
    }
    people.sides[0].refuges =
        retreat::refuges(campaign, report.attacker.faction, report.site, None);
    report.person_events = resolve_person_combat(campaign, data, &people)?;
    cleanup(campaign);
    finish_side(campaign, &mut report.attacker);
    apply_site_result(campaign, data, &mut report);
    let id = report.id;
    campaign.battles.insert(id, report);
    Ok(id)
}

fn report(
    campaign: &StrategicCampaign,
    data: &GameData,
    attackers: &[ArmyId],
    origin: SiteId,
    id: ThreatId,
) -> Result<BattleReport, RuleError> {
    let threat = &campaign.threats[&id];
    let definition = &data.threats.definitions[&threat.kind];
    let site = campaign.world.site(threat.site).expect("threat site");
    Ok(BattleReport {
        context: BattleContext::Field,
        wall_permille: 1000,
        fort_damage_added: 0,
        road_damage: None,
        id: campaign.next_ids.battle,
        completed_rounds: campaign.completed_rounds,
        sequence: campaign.accepted_sequence,
        site: site.id,
        site_name: site.name.clone(),
        origin,
        outcome: BattleOutcome::Stalemate,
        reason: BattleEndReason::ExchangeLimit,
        exchanges: Vec::new(),
        attacker: snapshot(campaign, data, attackers)?,
        defender: BattleDefender::Threat(ThreatSideReport {
            id,
            kind: threat.kind,
            name: definition.name.clone(),
            start: threat.headcount,
            end: threat.headcount,
            combat_losses: 0,
            encirclement_losses: 0,
            attack: definition.attack,
            resistance: definition.resistance,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        }),
        terrain_permille: data.combat.terrain(site),
        counters: Vec::new(),
        control_before: site.controller,
        control_after: site.controller,
        structural_damage_added: 0,
        occupation_after: campaign
            .world
            .occupation
            .get(&site.id)
            .copied()
            .unwrap_or(0),
        person_events: Vec::new(),
    })
}

fn finish_threat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) -> Result<(), RuleError> {
    let BattleDefender::Threat(side) = &mut report.defender else {
        unreachable!("threat report")
    };
    let threat = campaign.threats.get_mut(&side.id).expect("participant");
    side.combat_losses = side.start - threat.headcount;
    if report.outcome == BattleOutcome::AttackerVictory {
        side.encirclement_losses = threat.headcount;
        threat.headcount = 0;
        side.payout = data.threats.definitions[&side.kind].reward;
        let resources = &mut campaign
            .factions
            .get_mut(&report.attacker.faction)
            .expect("attacker")
            .resources;
        resources.gold =
            resources
                .gold
                .checked_add(side.payout.gold)
                .ok_or(RuleError::Overflow {
                    field: "threat Gold reward",
                })?;
        resources.wood =
            resources
                .wood
                .checked_add(side.payout.wood)
                .ok_or(RuleError::Overflow {
                    field: "threat Wood reward",
                })?;
        resources.stone =
            resources
                .stone
                .checked_add(side.payout.stone)
                .ok_or(RuleError::Overflow {
                    field: "threat Stone reward",
                })?;
    }
    side.end = threat.headcount;
    if threat.headcount == 0 {
        threat.status = ThreatStatus::Cleared {
            round: campaign.completed_rounds,
            by: report.attacker.faction,
            payout: side.payout,
        };
    }
    Ok(())
}
