//! Ordinary two-faction encounters: one bounded, atomic result and saved receipt.

mod arithmetic;

use super::{resolve_person_combat, retreat, PersonCombatContext, PersonCombatSide, RuleError};
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        battle::*,
        military::{ArmyId, FormationId},
        people::PersonAssignment,
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// Reports reveal only snapshots witnessed by an actual participating faction.
pub fn battle_reports(campaign: &StrategicCampaign, observer: FactionId) -> Vec<BattleReport> {
    campaign
        .battles
        .values()
        .filter(|report| report.attacker.faction == observer || report.defender.faction == observer)
        .cloned()
        .collect()
}

pub(super) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    attackers: &[ArmyId],
    defenders: &[ArmyId],
    origin: SiteId,
    site: SiteId,
) -> Result<BattleId, RuleError> {
    let mut report = new_report(campaign, data, attackers, defenders, origin, site)?;
    if !retreat::hostile(campaign, report.attacker.faction, report.defender.faction) {
        return Err(RuleError::InvalidState(
            "Encounter participants are not hostile.".into(),
        ));
    }
    campaign.next_ids.battle = BattleId(report.id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "battle identifiers",
    })?);
    let mut context = PersonCombatContext {
        site,
        sides: [
            person_side(campaign, data, &report.attacker),
            person_side(campaign, data, &report.defender),
        ],
    };
    arithmetic::exchanges(campaign, data, &mut report)?;
    for side in [&mut report.attacker, &mut report.defender] {
        for army in &mut side.armies {
            for formation in &mut army.formations {
                let actual = campaign
                    .formations
                    .get_mut(&formation.id)
                    .expect("participant");
                formation.combat_losses = formation.start - actual.headcount;
                actual.movement_spent = data.economy.formations[&actual.kind].movement_allowance;
            }
        }
    }
    match report.outcome {
        BattleOutcome::AttackerVictory => {
            withdraw(campaign, &mut report.defender, site, None, Some(origin))
        }
        BattleOutcome::DefenderVictory | BattleOutcome::Stalemate => {
            withdraw(campaign, &mut report.attacker, site, Some(origin), None)
        }
        BattleOutcome::MutualDestruction => {}
    }
    context.sides[0].refuges = retreat::refuges(campaign, report.attacker.faction, site, None);
    context.sides[1].refuges =
        retreat::refuges(campaign, report.defender.faction, site, Some(origin));
    report.person_events = resolve_person_combat(campaign, data, &context)?;
    cleanup(campaign);
    finish_side(campaign, &mut report.attacker);
    finish_side(campaign, &mut report.defender);
    apply_site_result(campaign, data, &mut report);
    super::knowledge::observe_battle(&mut campaign.knowledge, &report);
    let id = report.id;
    campaign.battles.insert(id, report);
    Ok(id)
}

fn apply_site_result(campaign: &mut StrategicCampaign, data: &GameData, report: &mut BattleReport) {
    let site = report.site;
    let before_damage = campaign.world.structural_damage(site);
    if campaign.world.site(site).expect("target").habitation != Habitation::Unsettled {
        damage(campaign, site, data.combat.field_damage);
    }
    let controller = match report.outcome {
        BattleOutcome::AttackerVictory => Some(report.attacker.faction),
        BattleOutcome::DefenderVictory | BattleOutcome::Stalemate => report.control_before,
        BattleOutcome::MutualDestruction => None,
    };
    if let Some(owner) = controller {
        capture(campaign, data, site, owner);
    } else {
        campaign
            .world
            .sites
            .iter_mut()
            .find(|entry| entry.id == site)
            .expect("target")
            .controller = None;
        campaign.reconcile_region_control();
    }
    report.control_after = controller;
    report.structural_damage_added = campaign.world.structural_damage(site) - before_damage;
    report.occupation_after = campaign.world.occupation.get(&site).copied().unwrap_or(0);
}

fn new_report(
    campaign: &StrategicCampaign,
    data: &GameData,
    attackers: &[ArmyId],
    defenders: &[ArmyId],
    origin: SiteId,
    site: SiteId,
) -> Result<BattleReport, RuleError> {
    let target = campaign
        .world
        .site(site)
        .ok_or(RuleError::UnknownSite { site })?;
    Ok(BattleReport {
        id: campaign.next_ids.battle,
        completed_rounds: campaign.completed_rounds,
        sequence: campaign.accepted_sequence,
        site,
        site_name: target.name.clone(),
        origin,
        outcome: BattleOutcome::Stalemate,
        reason: BattleEndReason::ExchangeLimit,
        exchanges: Vec::new(),
        attacker: snapshot(campaign, data, attackers)?,
        defender: snapshot(campaign, data, defenders)?,
        terrain_permille: data.combat.terrain(target),
        counters: Vec::new(),
        control_before: target.controller,
        control_after: target.controller,
        structural_damage_added: 0,
        occupation_after: campaign.world.occupation.get(&site).copied().unwrap_or(0),
        person_events: Vec::new(),
    })
}

fn snapshot(
    campaign: &StrategicCampaign,
    data: &GameData,
    ids: &[ArmyId],
) -> Result<BattleSideReport, RuleError> {
    let faction = ids
        .first()
        .and_then(|id| campaign.armies.get(id))
        .ok_or(RuleError::InvalidArmyGroup)?
        .faction;
    let mut ids = ids.to_vec();
    ids.sort();
    ids.dedup();
    let armies = ids.into_iter().map(|id| {
        let army = &campaign.armies[&id];
        let formations: Vec<_> = army.slots.iter().enumerate().filter_map(|(slot,id)| id.map(|id| {
            let formation = &campaign.formations[&id];
            BattleFormationReport {id,slot,kind:formation.kind,start:formation.headcount,end:formation.headcount,
                combat_losses:0,encirclement_losses:0,veterancy_permille:formation.service.tier.permille(&data.progression)}
        })).collect();
        let people = campaign.people.values().filter(|person| person.is_alive() && matches!(person.assignment,
            PersonAssignment::Formation {formation} if formations.iter().any(|entry| entry.id == formation)))
            .map(|person| BattlePersonReport {id:person.id,starting_formation:match person.assignment {PersonAssignment::Formation {formation} => formation,_ => unreachable!("attached participant")},name:person.name.clone(),class:person.class,
                starting_status:Some(person.status),status:person.status,assignment:person.assignment}).collect();
        BattleArmyReport {id,name:army.name.clone(),leadership_permille:campaign.army_leadership_permille(id,data).expect("army"),
            commander:army.commander.and_then(|id| campaign.people.get(&id)).map(|person|
                BattleCommander {id:person.id,name:person.name.clone()}),people,formations,final_site:Some(army.site)}
    }).collect();
    Ok(BattleSideReport {
        faction,
        name: campaign.factions[&faction].name.clone(),
        armies,
    })
}

fn person_side(
    campaign: &StrategicCampaign,
    data: &GameData,
    side: &BattleSideReport,
) -> PersonCombatSide {
    PersonCombatSide {
        faction: side.faction,
        armies: side.armies.iter().map(|army| army.id).collect(),
        starting_headcounts: side
            .armies
            .iter()
            .flat_map(|army| {
                army.formations
                    .iter()
                    .map(|formation| (formation.id, formation.start))
            })
            .collect(),
        commanders: side
            .armies
            .iter()
            .filter_map(|army| army.commander.as_ref())
            .filter(|commander| {
                campaign.people[&commander.id].is_fit_for_field(
                    campaign.completed_rounds,
                    data.rules.leadership.field_min_age_years,
                )
            })
            .map(|commander| commander.id)
            .collect(),
        refuges: Vec::new(),
    }
}

fn withdraw(
    campaign: &mut StrategicCampaign,
    side: &mut BattleSideReport,
    site: SiteId,
    preferred: Option<SiteId>,
    excluded: Option<SiteId>,
) {
    let destination = retreat::destination(campaign, side.faction, site, preferred, excluded);
    for army in &mut side.armies {
        if let Some(destination) = destination {
            campaign.armies.get_mut(&army.id).expect("participant").site = destination;
        } else {
            for formation in &mut army.formations {
                let actual = campaign
                    .formations
                    .get_mut(&formation.id)
                    .expect("participant");
                formation.encirclement_losses = actual.headcount;
                actual.headcount = 0;
            }
        }
    }
}

fn cleanup(campaign: &mut StrategicCampaign) {
    let destroyed: BTreeSet<_> = campaign
        .formations
        .values()
        .filter(|formation| formation.headcount == 0)
        .map(|formation| formation.id)
        .collect();
    campaign.formations.retain(|id, _| !destroyed.contains(id));
    for army in campaign.armies.values_mut() {
        for slot in &mut army.slots {
            if slot.is_some_and(|id| destroyed.contains(&id)) {
                *slot = None;
            }
        }
    }
    campaign.armies.retain(|_, army| !army.is_empty());
}

fn finish_side(campaign: &StrategicCampaign, side: &mut BattleSideReport) {
    for army in &mut side.armies {
        army.final_site = campaign.armies.get(&army.id).map(|army| army.site);
        for formation in &mut army.formations {
            formation.end = campaign
                .formations
                .get(&formation.id)
                .map_or(0, |formation| formation.headcount);
        }
        for snapshot in &mut army.people {
            let person = &campaign.people[&snapshot.id];
            snapshot.status = person.status;
            snapshot.assignment = person.assignment;
        }
    }
}

fn damage(campaign: &mut StrategicCampaign, site: SiteId, amount: u32) {
    let damage = campaign.world.site_damage.entry(site).or_default();
    *damage = damage.saturating_add(amount).min(100);
}

/// Unopposed capture is real too: apply the same one-time hostile capture effects.
pub(super) fn capture(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    owner: FactionId,
) {
    let target = campaign.world.site(site).expect("known target");
    let hostile_capture = target
        .controller
        .is_some_and(|old| retreat::hostile(campaign, owner, old));
    if hostile_capture && target.habitation != Habitation::Unsettled {
        damage(campaign, site, data.combat.capture_damage);
        campaign
            .world
            .occupation
            .insert(site, data.combat.capture_occupation);
    }
    campaign
        .world
        .sites
        .iter_mut()
        .find(|target| target.id == site)
        .expect("known target")
        .controller = Some(owner);
    campaign.reconcile_region_control();
}
