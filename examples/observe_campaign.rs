//! Replay a production observer campaign without opening a window or writing saves.

use std::{collections::BTreeMap, time::Instant};

use kestrum::{
    data::{
        economy::Habitation, generation::ProductionSetup, rules::Emblem, world::FactionId, GameData,
    },
    engine,
    state::StrategicCampaign,
};

#[derive(Default)]
struct Activity {
    actions: BTreeMap<String, u64>,
    last_active_round: Option<u32>,
    peak_armies: usize,
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
        if step.outcome.round_completed && campaign.completed_rounds.is_multiple_of(20) {
            report(&campaign, &activity);
        }
    }
    campaign.validate(&data)?;
    report(&campaign, &activity);
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
    }
}
