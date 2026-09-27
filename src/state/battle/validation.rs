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
            self.next_ids.battle.0 > self.battles.keys().map(|id| id.0).max().unwrap_or(0),
            "counter must exceed all battle IDs",
        )?;
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
    ensure(
        !report.exchanges.is_empty()
            && report.exchanges.len() <= data.combat.max_exchanges as usize,
        "invalid exchange count",
    )?;
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
    context::validate_outcome(campaign, report)?;
    ensure(
        report.reason != BattleEndReason::ExchangeLimit
            || report.exchanges.len() == data.combat.max_exchanges as usize,
        "limit result ended too early",
    )?;
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
