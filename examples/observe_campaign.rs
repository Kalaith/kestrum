//! Replay a production observer campaign without opening a window or writing saves.
//!
//! Usage: `observe_campaign [seed] [rounds] [factions] [trace faction id]`.

use std::{collections::BTreeMap, time::Instant};

use kestrum::{
    data::{
        economy::Habitation, generation::ProductionSetup, rules::Emblem, world::FactionId, GameData,
    },
    engine,
    state::{
        military::FormationId, people::PersonAssignment, relationships::FamilyOrigin,
        StrategicCampaign,
    },
};

#[derive(Default)]
struct Activity {
    actions: BTreeMap<String, u64>,
    last_active_round: Option<u32>,
    peak_armies: usize,
    emergence: EmergenceTotals,
}

#[derive(Default)]
struct EmergenceTotals {
    formation_seasons: u64,
    eligible_formation_seasons: u64,
    staffed_slots: u64,
    below_threshold: u64,
    threshold_reached: u64,
    emerged_people: u64,
    no_living_troops: u64,
    last_source: Option<LastSource>,
}

#[derive(Clone, Copy)]
struct LastSource {
    round: u32,
    formation: FormationId,
    service_xp: u32,
    qualifying_encounters: usize,
    vacancy_progress: u8,
    vacancy_threshold: u8,
    staffed: bool,
    emerged: bool,
}

fn argument<T: std::str::FromStr>(index: usize, default: T) -> Result<T, String> {
    std::env::args().nth(index).map_or(Ok(default), |value| {
        value
            .parse()
            .map_err(|_| format!("Invalid argument {index}: {value}"))
    })
}

fn main() -> Result<(), String> {
    let seed = argument(1, 260_926_u64)?;
    let rounds = argument(2, 160_u32)?;
    let factions = argument(3, 4_usize)?;
    // Optional: print this faction's planner diagnostics when it passes every tenth round.
    let trace = argument(4, 0_u32)?;
    let data = GameData::load()?;
    let mut campaign = StrategicCampaign::new_production_observer(
        &data,
        &ProductionSetup {
            kingdom_name: "Rose".into(),
            emblem: Emblem::Rose,
            factions,
            seed,
        },
    )?;
    let mut activity = BTreeMap::<FactionId, Activity>::new();
    let start = Instant::now();
    let mut steps = 0_u64;
    let mut last_reported_round = 0;
    println!("Observer replay: seed={seed} factions={factions} round_cap={rounds}");
    while campaign.completed_rounds < rounds && !campaign.observer_finished() {
        let faction = campaign.active_faction();
        let before_round = campaign.completed_rounds;
        let traced = faction.0 == trace && before_round.is_multiple_of(10);
        let step = if traced {
            engine::advance_observer_diagnosed(&mut campaign, &data)
        } else {
            engine::advance_observer_without_diagnostics(&mut campaign, &data)
        }
        .map_err(|error| format!("round={before_round} faction={faction:?}: {error}"))?;
        if traced && step.action.starts_with("EndTurn") {
            println!("Trace round {before_round}:");
            for line in &step.diagnostics {
                println!("    {line}");
            }
        }
        steps += 1;
        let entry = activity.entry(faction).or_default();
        let action = step
            .action
            .split([' ', '(', '{'])
            .next()
            .unwrap_or("unknown");
        *entry.actions.entry(action.to_owned()).or_default() += 1;
        if action != "EndTurn" {
            entry.last_active_round = Some(before_round);
        }
        entry.peak_armies = entry.peak_armies.max(
            campaign
                .armies
                .values()
                .filter(|army| army.faction == faction)
                .count(),
        );
        if step.outcome.round_completed {
            record_formation_opportunities(&campaign, &data, &mut activity);
            if campaign.completed_rounds.is_multiple_of(40) {
                report(&campaign, &activity);
                last_reported_round = campaign.completed_rounds;
            }
        }
    }
    campaign.validate(&data)?;
    if last_reported_round != campaign.completed_rounds {
        report(&campaign, &activity);
    }
    println!(
        "Completed {steps} steps in {:?}; finished={}",
        start.elapsed(),
        campaign.observer_finished()
    );
    for (faction, entry) in activity {
        println!("Faction {} actions: {:?}", faction.0, entry.actions);
    }
    report_hero_service_detail(&campaign, &data);
    Ok(())
}

fn report_hero_service_detail(campaign: &StrategicCampaign, data: &GameData) {
    let threshold = data.progression.recognition.personal_engagements;
    println!(
        "Personal Hero service at round {} (threshold={threshold})",
        campaign.completed_rounds
    );
    for person in campaign.people.values().filter(|person| {
        person.career.emergence.is_some()
            || person.career.hero_service_progress > 0
            || person.career.recognition.is_some()
    }) {
        let formation = match person.assignment {
            PersonAssignment::Formation { formation } => Some(formation),
            _ => None,
        };
        let army = formation.and_then(|formation| {
            campaign
                .armies
                .values()
                .find(|army| army.formation_ids().any(|id| id == formation))
                .map(|army| army.id)
        });
        let emergence_round = person
            .career
            .emergence
            .as_ref()
            .map(|record| record.completed_rounds);
        let recognition_round = person
            .career
            .recognition
            .as_ref()
            .map(|record| record.completed_rounds);
        println!(
            "  #{} faction={} name={:?} emerge={emergence_round:?} service={}/{} recognize={recognition_round:?} formation={formation:?} army={army:?} status={:?} retired={}",
            person.id.0,
            person.faction.0,
            person.name,
            person.career.hero_service_progress,
            threshold,
            person.status,
            person.career.retired
        );
    }
}

fn report(campaign: &StrategicCampaign, activity: &BTreeMap<FactionId, Activity>) {
    println!("Round {}", campaign.completed_rounds);
    for faction in campaign.factions.values() {
        let armies: Vec<_> = campaign
            .armies
            .values()
            .filter(|army| army.faction == faction.id)
            .collect();
        let supplied = armies
            .iter()
            .filter(|army| campaign.army_is_supplied(army.id))
            .count();
        let sites: Vec<_> = campaign
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(faction.id))
            .collect();
        let cities = sites
            .iter()
            .filter(|site| site.habitation >= Habitation::City)
            .count();
        // Holdings cut off from the capital's supply network.
        let supplied_sites = campaign.supplied_sites(faction.id);
        let detached = sites
            .iter()
            .filter(|site| !supplied_sites.contains(&site.id))
            .count();
        let active = activity
            .get(&faction.id)
            .and_then(|entry| entry.last_active_round);
        let peak = activity
            .get(&faction.id)
            .map_or(0, |entry| entry.peak_armies);
        println!("  #{} {} {:?} {:?}: armies={} supplied={} peak={} sites={} detached={detached} cities={} gold={} last_action={active:?}",
            faction.id.0, faction.name, faction.personality, faction.status, armies.len(), supplied, peak, sites.len(), cities, faction.resources.gold);
        report_personnel(campaign, faction.id, activity.get(&faction.id));
    }
}

fn report_personnel(campaign: &StrategicCampaign, faction: FactionId, activity: Option<&Activity>) {
    let people = campaign
        .people
        .values()
        .filter(|person| person.faction == faction)
        .collect::<Vec<_>>();
    let living_attached = people
        .iter()
        .filter(|person| {
            person.is_alive()
                && !person.career.retired
                && matches!(person.assignment, PersonAssignment::Formation { .. })
        })
        .copied()
        .collect::<Vec<_>>();
    let armies = campaign
        .armies
        .values()
        .filter(|army| army.faction == faction)
        .collect::<Vec<_>>();
    let formations = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction && formation.headcount > 0)
        .collect::<Vec<_>>();
    let apprentices = living_attached
        .iter()
        .filter(|person| person.career.emergence.is_some() && person.career.recognition.is_none())
        .count();
    let heroes = living_attached
        .iter()
        .filter(|person| person.career.recognition.is_some())
        .count();
    let apprentice_armies = armies
        .iter()
        .filter(|army| {
            living_attached.iter().any(|person| {
                person.career.emergence.is_some()
                    && person.career.recognition.is_none()
                    && matches!(person.assignment, PersonAssignment::Formation { formation }
                        if army.formation_ids().any(|id| id == formation))
            })
        })
        .count();
    let hero_armies = armies
        .iter()
        .filter(|army| {
            living_attached.iter().any(|person| {
                person.career.recognition.is_some()
                    && matches!(person.assignment, PersonAssignment::Formation { formation }
                        if army.formation_ids().any(|id| id == formation))
            })
        })
        .count();
    let unstaffed = formations
        .iter()
        .filter(|formation| {
            !living_attached.iter().any(|person| {
                person.assignment
                    == (PersonAssignment::Formation {
                        formation: formation.id,
                    })
            })
        })
        .count();
    let leaderless = armies
        .iter()
        .filter(|army| {
            army.commander
                .and_then(|id| campaign.people.get(&id))
                .is_none_or(|person| {
                    !person.is_alive()
                        || person.career.retired
                        || !matches!(person.assignment, PersonAssignment::Formation { formation }
                            if army.formation_ids().any(|id| id == formation))
                })
        })
        .count();
    let invited = people
        .iter()
        .filter(|person| {
            campaign
                .families
                .get(&person.id)
                .is_some_and(|family| family.origin == FamilyOrigin::LocalApprentice)
        })
        .count();
    let births = people
        .iter()
        .filter(|person| {
            campaign
                .families
                .get(&person.id)
                .is_some_and(|family| family.origin == FamilyOrigin::Birth)
        })
        .count();
    let emergence = activity.map(|entry| &entry.emergence);
    println!("    people total={} emerged={} invited={} births={} attached apprentices={} heroes={}; hero armies={}/{} apprentice armies={}/{}; retirees={} deaths={} unstaffed formations={} leaderless armies={}",
        people.len(),
        people.iter().filter(|person| person.career.emergence.is_some()).count(),
        invited,
        births,
        apprentices,
        heroes,
        hero_armies,
        armies.len(),
        apprentice_armies,
        armies.len(),
        people.iter().filter(|person| person.career.retired).count(),
        people.iter().filter(|person| !person.is_alive()).count(),
        unstaffed,
        leaderless);
    report_emergence_opportunities(emergence);
}

fn report_emergence_opportunities(emergence: Option<&EmergenceTotals>) {
    let Some(totals) = emergence else {
        return;
    };
    let latest = totals.last_source.map_or_else(
        || "none".to_owned(),
        |source| {
            format!(
                "round={} formation={} xp={} qualifying={} vacancy={}/{} staffed={} emerged={}",
                source.round,
                source.formation.0,
                source.service_xp,
                source.qualifying_encounters,
                source.vacancy_progress,
                source.vacancy_threshold,
                source.staffed,
                source.emerged
            )
        },
    );
    println!("    formation emergence: service seasons={} eligible vacant={} staffed={} below threshold={} threshold reached={} emerged={} no troops={}; latest={latest}",
        totals.formation_seasons,
        totals.eligible_formation_seasons,
        totals.staffed_slots,
        totals.below_threshold,
        totals.threshold_reached,
        totals.emerged_people,
        totals.no_living_troops);
}

fn record_formation_opportunities(
    campaign: &StrategicCampaign,
    data: &GameData,
    activity: &mut BTreeMap<FactionId, Activity>,
) {
    let threshold = data.progression.emergence.vacant_slot_engagements;
    let seasons = campaign
        .formations
        .values()
        .filter_map(|formation| {
            formation
                .service
                .recent
                .last()
                .filter(|season| {
                    season.completed_rounds.saturating_add(1) == campaign.completed_rounds
                })
                .map(|season| (formation, season))
        })
        .collect::<Vec<_>>();
    for (formation, service) in seasons {
        let entry = activity.entry(formation.faction).or_default();
        entry.emergence.formation_seasons += 1;
        let staffed = campaign.formation_person(formation.id).is_some();
        let eligible = formation.headcount > 0 && !staffed;
        if eligible {
            entry.emergence.eligible_formation_seasons += 1;
        }
        if staffed {
            entry.emergence.staffed_slots += 1;
        }
        if formation.headcount == 0 {
            entry.emergence.no_living_troops += 1;
        }
        let emerged = campaign.people.values().any(|person| {
            person.career.emergence.as_ref().is_some_and(|record| {
                record.completed_rounds == campaign.completed_rounds
                    && record.source_formation == formation.id
            })
        });
        let threshold_reached = formation.service.vacancy_service_progress >= threshold || emerged;
        if threshold_reached {
            entry.emergence.threshold_reached += 1;
        } else if eligible {
            entry.emergence.below_threshold += 1;
        }
        if emerged {
            entry.emergence.emerged_people += 1;
        }
        entry.emergence.last_source = Some(LastSource {
            round: campaign.completed_rounds,
            formation: formation.id,
            service_xp: service.xp,
            qualifying_encounters: service
                .encounters
                .iter()
                .filter(|encounter| encounter.meaningful)
                .count(),
            vacancy_progress: formation.service.vacancy_service_progress,
            vacancy_threshold: threshold,
            staffed,
            emerged,
        });
    }
}
