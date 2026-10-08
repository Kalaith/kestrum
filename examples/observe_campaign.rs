//! Replay a production observer campaign without opening a window or writing saves.

use std::{collections::BTreeMap, time::Instant};

use kestrum::{
    data::{
        economy::Habitation, generation::ProductionSetup, rules::Emblem, world::FactionId, GameData,
    },
    engine,
    state::{
        evidence::{EvidenceKind, SeasonService},
        military::FormationId,
        people::PersonAssignment,
        relationships::FamilyOrigin,
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
    candidate_seasons: u64,
    occupied_candidates: u64,
    emerged_people: u64,
    chance_misses: u64,
    zero_chance: u64,
    zero_xp_candidates: u64,
    non_independent_candidates: u64,
    no_candidate_seasons: u64,
    chance_sum: u64,
    chance_count: u64,
    chance_min: Option<u64>,
    chance_max: Option<u64>,
    last_source: Option<LastSource>,
}

#[derive(Clone, Copy)]
struct LastSource {
    round: u32,
    formation: FormationId,
    service_xp: u32,
    adult_roster: u64,
    chance: Option<u64>,
    occupied: bool,
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
        let step = engine::advance_observer_without_diagnostics(&mut campaign, &data)
            .map_err(|error| format!("round={before_round} faction={faction:?}: {error}"))?;
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
    Ok(())
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
        let active = activity
            .get(&faction.id)
            .and_then(|entry| entry.last_active_round);
        let peak = activity
            .get(&faction.id)
            .map_or(0, |entry| entry.peak_armies);
        println!("  #{} {} {:?}: armies={} supplied={} peak={} sites={} cities={} gold={} last_action={active:?}",
            faction.id.0, faction.name, faction.status, armies.len(), supplied, peak, sites.len(), cities, faction.resources.gold);
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
    let average = (totals.chance_count > 0).then(|| totals.chance_sum / totals.chance_count);
    let latest = totals.last_source.map_or_else(
        || "none".to_owned(),
        |source| {
            format!(
                "round={} formation={} xp={} roster={} chance={:?} occupied={} emerged={}",
                source.round,
                source.formation.0,
                source.service_xp,
                source.adult_roster,
                source.chance,
                source.occupied,
                source.emerged
            )
        },
    );
    println!("    old emergence: formation seasons={} eligible={} selected={} occupied candidates={} emerged={} chance misses={} zero chance={} zero-xp selections={} non-independent selections={} no-candidate seasons={} chance min/avg/max={:?}/{average:?}/{:?}; latest={latest}",
        totals.formation_seasons,
        totals.eligible_formation_seasons,
        totals.candidate_seasons,
        totals.occupied_candidates,
        totals.emerged_people,
        totals.chance_misses,
        totals.zero_chance,
        totals.zero_xp_candidates,
        totals.non_independent_candidates,
        totals.no_candidate_seasons,
        totals.chance_min,
        totals.chance_max);
}

fn record_formation_opportunities(
    campaign: &StrategicCampaign,
    data: &GameData,
    activity: &mut BTreeMap<FactionId, Activity>,
) {
    for faction in campaign.factions.keys().copied() {
        let seasons = campaign
            .formations
            .values()
            .filter(|formation| formation.faction == faction)
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
        let entry = activity.entry(faction).or_default();
        entry.emergence.formation_seasons += seasons.len() as u64;
        entry.emergence.eligible_formation_seasons += seasons
            .iter()
            .filter(|(formation, season)| {
                formation.headcount > 0 && season.xp > 0 && campaign.is_independent(faction)
            })
            .count() as u64;
        let selected = selected_service_source(campaign, faction);
        let Some((formation_id, service)) = selected else {
            entry.emergence.no_candidate_seasons += 1;
            continue;
        };
        entry.emergence.candidate_seasons += 1;
        let occupied = campaign.people.values().any(|person| {
            person.assignment
                == (PersonAssignment::Formation {
                    formation: formation_id,
                })
                && !person.career.emergence.as_ref().is_some_and(|record| {
                    record.completed_rounds == campaign.completed_rounds
                        && record.source_formation == formation_id
                })
        });
        let emerged = campaign.people.values().any(|person| {
            person.career.emergence.as_ref().is_some_and(|record| {
                record.completed_rounds == campaign.completed_rounds
                    && record.source_formation == formation_id
            })
        });
        let adults = emergence_roster_count(campaign, faction);
        let chance = if service.xp > 0 && campaign.is_independent(faction) {
            Some(emergence_chance(
                campaign,
                data,
                formation_id,
                &service,
                adults,
            ))
        } else {
            None
        };
        if occupied {
            entry.emergence.occupied_candidates += 1;
        }
        if emerged {
            entry.emergence.emerged_people += 1;
        } else if chance.is_some() {
            entry.emergence.chance_misses += 1;
        }
        if service.xp == 0 {
            entry.emergence.zero_xp_candidates += 1;
        }
        if !campaign.is_independent(faction) {
            entry.emergence.non_independent_candidates += 1;
        }
        if let Some(chance) = chance {
            entry.emergence.chance_count += 1;
            entry.emergence.chance_sum += chance;
            entry.emergence.chance_min = Some(
                entry
                    .emergence
                    .chance_min
                    .map_or(chance, |old| old.min(chance)),
            );
            entry.emergence.chance_max = Some(
                entry
                    .emergence
                    .chance_max
                    .map_or(chance, |old| old.max(chance)),
            );
            if chance == 0 {
                entry.emergence.zero_chance += 1;
            }
        }
        entry.emergence.last_source = Some(LastSource {
            round: campaign.completed_rounds,
            formation: formation_id,
            service_xp: service.xp,
            adult_roster: adults,
            chance,
            occupied,
            emerged,
        });
    }
}

fn selected_service_source(
    campaign: &StrategicCampaign,
    faction: FactionId,
) -> Option<(FormationId, SeasonService)> {
    campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction && formation.headcount > 0)
        .filter_map(|formation| {
            let season = formation.service.recent.last()?;
            (season.completed_rounds.saturating_add(1) == campaign.completed_rounds)
                .then_some((formation, season))
        })
        .max_by_key(|(formation, season)| {
            (
                season.xp,
                formation.service.tier,
                std::cmp::Reverse(formation.id),
            )
        })
        .map(|(formation, season)| (formation.id, season.clone()))
}

fn emergence_roster_count(campaign: &StrategicCampaign, faction: FactionId) -> u64 {
    let adults = campaign
        .people
        .values()
        .filter(|person| {
            person.faction == faction
                && person.is_alive()
                && !person.career.retired
                && person.age_years(campaign.completed_rounds) >= 18
        })
        .count() as u64;
    adults.saturating_sub(
        campaign
            .people
            .values()
            .filter(|person| {
                person.faction == faction
                    && person.is_alive()
                    && !person.career.retired
                    && person.age_years(campaign.completed_rounds) >= 18
                    && person
                        .career
                        .emergence
                        .as_ref()
                        .is_some_and(|record| record.completed_rounds == campaign.completed_rounds)
            })
            .count() as u64,
    )
}

fn emergence_chance(
    campaign: &StrategicCampaign,
    data: &GameData,
    formation: FormationId,
    service: &SeasonService,
    adults: u64,
) -> u64 {
    let rules = &data.progression.emergence;
    let curve = u64::from(rules.roster_base).saturating_mul(u64::from(rules.roster_factor))
        / (u64::from(rules.roster_square_factor)
            .saturating_mul(adults.saturating_mul(adults))
            .saturating_add(u64::from(rules.roster_factor))
            .max(1));
    let multiplier = match campaign.formations[&formation].service.tier {
        kestrum::state::evidence::Veterancy::Ordinary => 1000,
        kestrum::state::evidence::Veterancy::Seasoned => rules.seasoned_multiplier_permille,
        kestrum::state::evidence::Veterancy::Veteran => rules.veteran_multiplier_permille,
    } as u64;
    let exceptional = service
        .encounters
        .iter()
        .any(|encounter| encounter.tags.contains(&EvidenceKind::SurvivedOutnumbered))
        && service.encounters.iter().any(|encounter| {
            encounter.tags.contains(&EvidenceKind::CapturedAnchor)
                || encounter.tags.contains(&EvidenceKind::DefendedAnchor)
        });
    let chance = curve.saturating_mul(multiplier) / 1000;
    let chance = if exceptional {
        chance.saturating_mul(u64::from(rules.exceptional_multiplier_permille)) / 1000
    } else {
        chance
    };
    chance.min(u64::from(rules.maximum_chance_permille))
}
