//! Historical references validate against counters, never against live enemy rosters.

mod context;
mod threat;

use super::*;
use crate::{
    data::GameData,
    state::{
        people::{PersonCombatOutcome, PersonStatus},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

impl StrategicCampaign {
    pub(crate) fn validate_battles(&self, data: &GameData) -> Result<(), String> {
        ensure(
            self.next_ids.battle.0
                > self
                    .battles
                    .keys()
                    .map(|id| id.0)
                    .chain(
                        self.pending_battle
                            .iter()
                            .map(|pending| pending.report.id.0),
                    )
                    .max()
                    .unwrap_or(0),
            "counter must exceed all battle IDs",
        )?;
        validate_pending(self)?;
        for (site, occupation) in &self.world.occupation {
            ensure(
                self.world.site(*site).is_some() && *occupation <= 100,
                "invalid occupation site/value",
            )?;
        }
        let mut sequence = 0;
        for (id, report) in &self.battles {
            validate_header(self, data, *id, report, sequence)?;
            sequence = report.sequence;
            let roster = validate_roster(self, data, report)?;
            validate_exchanges(data, report, &roster)?;
            validate_result(self, data, report)?;
            validate_people(report)?;
            threat::validate(self, data, report)?;
        }
        Ok(())
    }
}

struct Roster<'a> {
    armies: BTreeSet<ArmyId>,
    formations: BTreeMap<FormationId, &'a BattleFormationReport>,
}

fn validate_header(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: BattleId,
    report: &BattleReport,
    sequence: u64,
) -> Result<(), String> {
    ensure(
        id == report.id
            && id.0 > 0
            && report.sequence > sequence
            && report.sequence <= campaign.accepted_sequence
            && report.completed_rounds <= campaign.completed_rounds,
        "invalid identity, sequence or date",
    )?;
    ensure(
        !report.site_name.trim().is_empty()
            && Some(report.attacker.faction) != report.defender.faction()
            && campaign.factions.contains_key(&report.attacker.faction)
            && report
                .defender
                .faction()
                .is_none_or(|faction| campaign.factions.contains_key(&faction))
            && report
                .control_before
                .is_none_or(|id| campaign.factions.contains_key(&id))
            && report
                .control_after
                .is_none_or(|id| campaign.factions.contains_key(&id))
            && report.structural_damage_added <= 100
            && report.occupation_after <= 100
            && (1..=10_000).contains(&report.terrain_permille),
        "invalid location, faction or effects",
    )?;
    context::validate_context(campaign, data, report)?;
    if report.simulation.is_none() {
        ensure(
            !report.exchanges.is_empty()
                && report.exchanges.len() <= data.combat.max_exchanges as usize,
            "invalid exchange count",
        )?;
    }
    Ok(())
}

fn validate_roster<'a>(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &'a BattleReport,
) -> Result<Roster<'a>, String> {
    let mut identities = RosterBuilder::default();
    for side in report.faction_sides() {
        ensure(
            !side.name.trim().is_empty()
                && !side.armies.is_empty()
                && side.armies.windows(2).all(|pair| pair[0].id < pair[1].id),
            "invalid side roster",
        )?;
        for army in &side.armies {
            validate_army(campaign, data, report, side, army, &mut identities)?;
        }
    }
    Ok(Roster {
        armies: identities.armies,
        formations: identities.formations,
    })
}

#[derive(Default)]
struct RosterBuilder<'a> {
    armies: BTreeSet<ArmyId>,
    people: BTreeMap<PersonId, FactionId>,
    formations: BTreeMap<FormationId, &'a BattleFormationReport>,
}

fn validate_army<'a>(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
    side: &BattleSideReport,
    army: &'a BattleArmyReport,
    identities: &mut RosterBuilder<'a>,
) -> Result<(), String> {
    ensure(
        army.id.0 > 0
            && army.id < campaign.next_ids.army
            && identities.armies.insert(army.id)
            && !army.name.trim().is_empty()
            && army.leadership_permille > 0
            && army
                .final_site
                .is_none_or(|site| campaign.world.site(site).is_some())
            && !army.formations.is_empty()
            && army.formations.len() <= 6
            && army
                .formations
                .windows(2)
                .all(|pair| pair[0].slot < pair[1].slot),
        "invalid army snapshot",
    )?;
    let mut end = 0_u64;
    for formation in &army.formations {
        ensure(
            formation.id.0 > 0
                && formation.id < campaign.next_ids.formation
                && formation.slot < 6
                && formation.start > 0
                && formation.start <= data.economy.formations[&formation.kind].capacity
                && [
                    data.progression.ordinary_permille,
                    data.progression.seasoned_permille,
                    data.progression.veteran_permille,
                ]
                .contains(&formation.veterancy_permille)
                && u64::from(formation.start)
                    == u64::from(formation.end)
                        + u64::from(formation.combat_losses)
                        + u64::from(formation.encirclement_losses)
                && identities
                    .formations
                    .insert(formation.id, formation)
                    .is_none(),
            "invalid formation snapshot/losses",
        )?;
        end += u64::from(formation.end);
    }
    ensure(
        army.final_site.is_some() == (end > 0),
        "destroyed army has a final location",
    )?;
    ensure(
        army.people.windows(2).all(|pair| pair[0].id < pair[1].id),
        "unordered person snapshots",
    )?;
    for person in &army.people {
        ensure(
            person.id.0 > 0
                && person.id < campaign.next_ids.person
                && !person.name.trim().is_empty()
                && identities.people.insert(person.id, side.faction).is_none(),
            "invalid witnessed person",
        )?;
        ensure(
            army.formations
                .iter()
                .any(|entry| entry.id == person.starting_formation),
            "person starting formation is outside witnessed army",
        )?;
        validate_person_snapshot(campaign, data, report, person)?;
    }
    if let Some(commander) = &army.commander {
        ensure(
            army.people
                .iter()
                .any(|person| person.id == commander.id && person.name == commander.name),
            "commander was not witnessed in the army",
        )?;
    }
    Ok(())
}

fn validate_person_snapshot(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
    person: &BattlePersonReport,
) -> Result<(), String> {
    match person.starting_status {
        Some(PersonStatus::Dead { .. } | PersonStatus::Displaced { .. }) => {
            return Err("battle: a dead or displaced person cannot enter combat".into())
        }
        Some(PersonStatus::Wounded {
            since_round,
            remaining_steps,
        }) => ensure(
            since_round <= report.completed_rounds
                && (1..=data.combat.wound_recovery_steps).contains(&remaining_steps)
                && person.status != PersonStatus::Fit,
            "invalid starting injury or unsupported healing during combat",
        )?,
        Some(PersonStatus::Fit) | None => {}
    }
    match person.assignment {
        PersonAssignment::Formation { formation } => ensure(
            formation.0 > 0
                && formation < campaign.next_ids.formation
                && !matches!(person.status, PersonStatus::Dead { .. }),
            "invalid witnessed assignment",
        )?,
        PersonAssignment::Site { site } => ensure(
            campaign.world.site(site).is_some()
                && !matches!(person.status, PersonStatus::Dead { .. }),
            "invalid witnessed site",
        )?,
        PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. } => {
            return Err("battle: a dependent cannot be a field participant".into())
        }
        PersonAssignment::Dead => ensure(
            matches!(person.status, PersonStatus::Dead { .. }),
            "dead assignment mismatch",
        )?,
    }
    match person.status {
        PersonStatus::Displaced { .. } => {
            return Err("battle: displacement is recorded after the encounter".into())
        }
        PersonStatus::Wounded {
            since_round,
            remaining_steps,
        } => ensure(
            since_round <= report.completed_rounds
                && (1..=data.combat.wound_recovery_steps).contains(&remaining_steps),
            "invalid historical wound",
        )?,
        PersonStatus::Dead {
            completed_rounds,
            site,
        } => ensure(
            completed_rounds == report.completed_rounds && site == report.site,
            "invalid historical death",
        )?,
        PersonStatus::Fit => {}
    }
    Ok(())
}

fn validate_exchanges(
    data: &GameData,
    report: &BattleReport,
    roster: &Roster<'_>,
) -> Result<(), String> {
    if let Some(simulation) = &report.simulation {
        return validate_simulation(data, report, roster, simulation);
    }
    let mut remaining: BTreeMap<_, _> = roster
        .formations
        .iter()
        .map(|(id, formation)| (*id, formation.start))
        .collect();
    for (index, exchange) in report.exchanges.iter().enumerate() {
        context::validate_wall(data, report, exchange, &remaining)?;
        ensure(
            !exchange.leadership.is_empty()
                && exchange
                    .leadership
                    .windows(2)
                    .all(|pair| pair[0].army < pair[1].army)
                && exchange
                    .leadership
                    .iter()
                    .all(|entry| roster.armies.contains(&entry.army) && entry.permille > 0),
            "invalid exchange leadership",
        )?;
        ensure(
            exchange.number as usize == index + 1
                && !exchange.losses.is_empty()
                && exchange
                    .losses
                    .windows(2)
                    .all(|pair| pair[0].formation < pair[1].formation),
            "invalid exchange ordering",
        )?;
        for loss in &exchange.losses {
            let count = remaining
                .get_mut(&loss.formation)
                .ok_or("campaign.battles: loss has unknown participant")?;
            ensure(
                loss.amount > 0 && loss.amount <= *count,
                "invalid exchange loss",
            )?;
            *count -= loss.amount;
        }
    }
    for (id, formation) in &roster.formations {
        ensure(
            remaining[id] == formation.start - formation.combat_losses,
            "exchange losses disagree with total",
        )?;
    }
    Ok(())
}

fn validate_result(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Result<(), String> {
    if let Some(simulation) = &report.simulation {
        let reason_matches = matches!(
            (report.reason, simulation.reason),
            (
                BattleEndReason::Annihilation,
                crate::state::battle::simulation::BattleResolutionReason::Annihilation
            ) | (
                BattleEndReason::Rout,
                crate::state::battle::simulation::BattleResolutionReason::Rout
            ) | (
                BattleEndReason::ExchangeLimit,
                crate::state::battle::simulation::BattleResolutionReason::RoundLimit
            )
        );
        ensure(
            simulation.resolver_version > 0
                && !simulation.rules_revision.trim().is_empty()
                && simulation.opening.terrain_permille == report.terrain_permille
                && simulation.outcome == report.outcome
                && reason_matches
                && !simulation.events.is_empty(),
            "invalid immutable formation resolution",
        )?;
    }
    context::validate_outcome(campaign, report)?;
    if report.simulation.is_none() {
        ensure(
            report.reason != BattleEndReason::ExchangeLimit
                || report.exchanges.len() == data.combat.max_exchanges as usize,
            "limit result ended too early",
        )?;
    }
    let mut counters = BTreeSet::new();
    for counter in &report.counters {
        ensure(
            counters.insert((counter.source, counter.target))
                && counter.permille == data.combat.counter(counter.source, counter.target)
                && counter.permille != 1000,
            "invalid observed counter",
        )?;
    }
    Ok(())
}

fn validate_pending(campaign: &StrategicCampaign) -> Result<(), String> {
    let Some(pending) = &campaign.pending_battle else {
        return Ok(());
    };
    let report = &pending.report;
    let simulation = report.simulation.as_ref();
    ensure(
        !campaign.battles.contains_key(&report.id)
            && report.id.0 > 0
            && report.id.0 < campaign.next_ids.battle.0
            && report.sequence == campaign.accepted_sequence
            && report.completed_rounds == campaign.completed_rounds
            && campaign.factions.contains_key(&pending.started_by)
            && report
                .faction_sides()
                .any(|side| side.faction == pending.started_by)
            && campaign.world.site(report.site).is_some(),
        "invalid pending battle identity or timing",
    )?;
    let simulation = simulation.ok_or("campaign.pending_battle: missing saved resolution")?;
    ensure(
        simulation.resolver_version > 0
            && !simulation.rules_revision.trim().is_empty()
            && simulation.opening.terrain_permille == report.terrain_permille
            && simulation.outcome == report.outcome
            && simulation.events.last().is_some_and(|event| {
                matches!(event, crate::state::battle::simulation::BattleEvent::BattleEnded { outcome, .. } if *outcome == report.outcome)
            }),
        "invalid pending formation resolution",
    )?;
    for army in report.faction_sides().flat_map(|side| &side.armies) {
        let actual = campaign.armies.get(&army.id);
        ensure(
            actual.is_some_and(|actual| {
                actual.site == report.site
                    && army.formations.len() == actual.formation_ids().count()
                    && army.formations.iter().all(|entry| {
                        actual.slots.get(entry.slot) == Some(&Some(entry.id))
                            && campaign.formations.get(&entry.id).is_some_and(|formation| {
                                formation.kind == entry.kind && formation.headcount == entry.start
                            })
                    })
            }),
            "pending participant roster or formation slots changed",
        )?;
    }
    for army in &simulation.opening.armies {
        if army.id.0 == u32::MAX {
            continue;
        }
        let witnessed = report
            .faction_sides()
            .flat_map(|side| &side.armies)
            .find(|entry| entry.id == army.id);
        ensure(
            witnessed.is_some_and(|entry| {
                army.slots.iter().enumerate().all(|(slot, unit)| {
                    let Some(unit) = unit else { return true };
                    entry.formations.iter().any(|formation| {
                        formation.slot == slot
                            && unit.id
                                == crate::state::battle::simulation::BattleUnitId::Formation(
                                    formation.id,
                                )
                    })
                })
            }),
            "pending resolution slots disagree with the participant roster",
        )?;
    }
    Ok(())
}

fn validate_simulation(
    data: &GameData,
    report: &BattleReport,
    roster: &Roster<'_>,
    simulation: &crate::state::battle::simulation::BattleResolution,
) -> Result<(), String> {
    use crate::state::battle::simulation::{BattleEvent, BattleSide, BattleUnitId};
    let opening: BTreeMap<_, _> = simulation
        .opening
        .armies
        .iter()
        .flat_map(|army| army.slots.iter().flatten())
        .map(|unit| (unit.id, unit.headcount))
        .collect();
    let final_units: BTreeMap<_, _> = simulation
        .units
        .iter()
        .map(|unit| (unit.id, unit))
        .collect();
    ensure(
        opening.len()
            == simulation
                .opening
                .armies
                .iter()
                .map(|army| army.slots.iter().flatten().count())
                .sum::<usize>()
            && final_units.len() == simulation.units.len()
            && opening.len() == final_units.len(),
        "duplicate or missing unit in formation receipt",
    )?;
    ensure(
        simulation.opening.armies.len()
            == report
                .faction_sides()
                .map(|side| side.armies.len())
                .sum::<usize>()
                + usize::from(matches!(report.defender, BattleDefender::Threat(_))),
        "formation receipt has an unwitnessed army board",
    )?;
    for side in std::iter::once((&report.attacker, BattleSide::Attacker)).chain(
        report
            .defender
            .faction_side()
            .map(|side| (side, BattleSide::Defender)),
    ) {
        for army in &side.0.armies {
            let input = simulation
                .opening
                .armies
                .iter()
                .find(|input| input.id == army.id)
                .ok_or("campaign.battles: missing army board")?;
            ensure(
                input.faction == side.0.faction && input.side == side.1 && input.name == army.name,
                "army board identity disagrees with witnessed roster",
            )?;
            for formation in &army.formations {
                let unit = input.slots[formation.slot]
                    .as_ref()
                    .ok_or("campaign.battles: missing formation opening slot")?;
                ensure(
                    unit.id == BattleUnitId::Formation(formation.id)
                        && unit.kind == Some(formation.kind)
                        && unit.headcount == formation.start,
                    "formation opening disagrees with witnessed roster",
                )?;
                ensure(
                    unit.leader.as_ref().map(|leader| leader.id) == formation.battle_leader,
                    "formation leader snapshot disagrees with witnessed roster",
                )?;
            }
        }
    }
    let expected_formations: BTreeSet<_> = roster.formations.keys().copied().collect();
    let actual_formations: BTreeSet<_> = opening
        .keys()
        .filter_map(|id| match id {
            BattleUnitId::Formation(id) => Some(*id),
            BattleUnitId::Threat(_) => None,
        })
        .collect();
    ensure(
        expected_formations == actual_formations,
        "formation receipt differs from witnessed participants",
    )?;
    let mut event_losses = BTreeMap::<BattleUnitId, u32>::new();
    let mut round_losses = BTreeMap::<u32, BTreeMap<FormationId, u32>>::new();
    let mut round_threat_losses = BTreeMap::<u32, u32>::new();
    let mut current_counts = opening.clone();
    let mut round = 0_u32;
    let mut ended = false;
    for (index, event) in simulation.events.iter().enumerate() {
        ensure(!ended, "event occurs after the battle ended")?;
        match event {
            BattleEvent::RoundStarted { round: next } => {
                ensure(
                    *next == round.saturating_add(1),
                    "invalid battle event round order",
                )?;
                round = *next;
            }
            BattleEvent::Damage {
                target,
                amount,
                remaining,
                ..
            } => {
                let before = current_counts
                    .get_mut(target)
                    .ok_or("campaign.battles: damage targets an unknown unit")?;
                ensure(
                    *amount > 0
                        && *amount <= *before
                        && before.saturating_sub(*amount) == *remaining,
                    "damage event has an invalid remaining count",
                )?;
                *before = *remaining;
                *event_losses.entry(*target).or_default() += amount;
                match target {
                    BattleUnitId::Formation(id) => {
                        *round_losses
                            .entry(round)
                            .or_default()
                            .entry(*id)
                            .or_default() += amount;
                    }
                    BattleUnitId::Threat(_) => {
                        *round_threat_losses.entry(round).or_default() += amount;
                    }
                }
            }
            BattleEvent::Routed {
                unit, survivors, ..
            } => ensure(
                current_counts.get(unit) == Some(survivors),
                "rout survivors disagree with prior damage",
            )?,
            BattleEvent::BattleEnded { outcome, .. } => {
                ensure(
                    index + 1 == simulation.events.len() && *outcome == report.outcome,
                    "battle end event disagrees with its receipt",
                )?;
                ended = true;
            }
            _ => {}
        }
    }
    ensure(ended && round > 0, "formation receipt did not end a battle")?;
    for unit in &simulation.units {
        ensure(
            current_counts.get(&unit.id) == Some(&unit.headcount),
            "final unit count disagrees with event playback",
        )?;
    }
    let mut exchange_rounds: Vec<_> = round_losses
        .keys()
        .chain(round_threat_losses.keys())
        .copied()
        .collect();
    exchange_rounds.sort_unstable();
    exchange_rounds.dedup();
    ensure(
        exchange_rounds.len() == report.exchanges.len()
            && report
                .exchanges
                .windows(2)
                .all(|pair| pair[0].number < pair[1].number),
        "exchange summary disagrees with event rounds",
    )?;
    let has_opening_engines = report
        .attacker
        .armies
        .iter()
        .flat_map(|army| &army.formations)
        .any(|formation| {
            formation.kind == crate::data::economy::TroopKind::SiegeEngines && formation.start > 0
        });
    let applied_wall =
        if matches!(report.context, BattleContext::Assault { .. }) && has_opening_engines {
            report
                .wall_permille
                .saturating_sub(data.siege.engine_wall_reduction_permille)
                .max(data.siege.wall_minimum_permille)
        } else {
            report.wall_permille
        };
    let mut leadership: Vec<_> = report
        .faction_sides()
        .flat_map(|side| &side.armies)
        .map(|army| ArmyLeadership {
            army: army.id,
            permille: army.leadership_permille,
        })
        .collect();
    leadership.sort_by_key(|entry| entry.army);
    for (number, exchange) in exchange_rounds.iter().zip(&report.exchanges) {
        let losses: Vec<_> = round_losses
            .remove(number)
            .unwrap_or_default()
            .into_iter()
            .map(|(formation, amount)| FormationLoss { formation, amount })
            .collect();
        ensure(
            exchange.number == *number
                && exchange.losses == losses
                && exchange.threat_losses == round_threat_losses.remove(number).unwrap_or_default()
                && exchange.wall_permille == applied_wall
                && exchange.leadership == leadership,
            "exchange summary differs from immutable battle events",
        )?;
    }
    for (id, formation) in &roster.formations {
        let unit = BattleUnitId::Formation(*id);
        let final_unit = final_units[&unit];
        ensure(
            opening.get(&unit) == Some(&formation.start)
                && event_losses.get(&unit).copied().unwrap_or_default() == formation.combat_losses
                && formation.combat_losses == formation.start.saturating_sub(final_unit.headcount)
                && formation.end.saturating_add(formation.encirclement_losses)
                    == final_unit.headcount,
            "formation losses disagree with immutable resolution",
        )?;
    }
    if let BattleDefender::Threat(threat) = &report.defender {
        let id = BattleUnitId::Threat(threat.id);
        let resolved = final_units
            .get(&id)
            .ok_or("campaign.battles: missing threat unit")?;
        ensure(
            opening.get(&id) == Some(&threat.start)
                && event_losses.get(&id).copied().unwrap_or_default() == threat.combat_losses
                && threat.combat_losses == threat.start.saturating_sub(resolved.headcount)
                && threat.end.saturating_add(threat.encirclement_losses) == resolved.headcount,
            "threat losses disagree with immutable resolution",
        )?;
    }
    Ok(())
}

fn validate_people(report: &BattleReport) -> Result<(), String> {
    let mut events = BTreeSet::new();
    for side in report.faction_sides() {
        let witnessed: BTreeMap<_, _> = side
            .armies
            .iter()
            .flat_map(|army| army.people.iter().map(|person| (person.id, person)))
            .collect();
        let surviving: BTreeSet<_> = side
            .armies
            .iter()
            .flat_map(|army| army.formations.iter())
            .filter(|formation| formation.end > 0)
            .map(|formation| formation.id)
            .collect();
        for person in witnessed.values() {
            if let PersonAssignment::Formation { formation } = person.assignment {
                ensure(
                    surviving.contains(&formation),
                    "witnessed assignment is not a surviving friendly participant",
                )?;
            }
        }
        for event in report
            .person_events
            .iter()
            .filter(|event| event.faction == side.faction)
        {
            let person = witnessed
                .get(&event.person)
                .ok_or("campaign.battles: unwitnessed event person")?;
            ensure(
                event.name == person.name && events.insert(event.person),
                "duplicate or mislabeled person outcome",
            )?;
            match event.outcome {
                PersonCombatOutcome::Died { .. } => ensure(
                    matches!(person.status, PersonStatus::Dead { .. })
                        && person.assignment == PersonAssignment::Dead,
                    "death event disagrees with person",
                )?,
                PersonCombatOutcome::Wounded { assignment, .. } => ensure(
                    matches!(person.status,PersonStatus::Wounded {since_round,..}
                    if since_round == report.completed_rounds)
                        && assignment == person.assignment,
                    "wound event disagrees with person",
                )?,
                PersonCombatOutcome::AssumedCommand { army, previous } => {
                    let army = side
                        .armies
                        .iter()
                        .find(|entry| entry.id == army)
                        .ok_or("campaign.battles: succession has foreign army")?;
                    ensure(
                        army.final_site.is_some()
                            && army
                                .commander
                                .as_ref()
                                .is_some_and(|commander| commander.id == previous)
                            && army.people.iter().any(|entry| entry.id == person.id)
                            && army.people.iter().any(|entry| {
                                entry.id == previous
                                    && matches!(entry.status, PersonStatus::Wounded { .. })
                            })
                            && person.status == PersonStatus::Fit
                            && matches!(person.assignment,PersonAssignment::Formation {formation} if army.formations.iter()
                            .any(|entry| entry.id == formation && entry.end>0)),
                        "invalid command succession",
                    )?;
                }
            }
        }
    }
    ensure(
        events.len() == report.person_events.len(),
        "outcome has a nonparticipating faction",
    )
}

fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(format!("campaign.battles: {message}"))
    }
}
