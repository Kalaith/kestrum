//! Compare the scripted review kingdom with its rivals through the opening decades.
//!
//! Usage: `opening_report [rounds] [interval]`. Uses the midgame review script
//! (seed 88, four factions) and prints faction land, income and forces.

#[path = "prepare_midgame/campaign.rs"]
#[allow(dead_code)] // This report drives `play` rather than the fixed midgame save.
mod campaign;

use kestrum::{
    data::{economy::Habitation, world::DiplomaticState, GameData},
    state::StrategicCampaign,
};

fn argument(index: usize, default: u32) -> Result<u32, String> {
    std::env::args().nth(index).map_or(Ok(default), |value| {
        value
            .parse()
            .map_err(|_| format!("Invalid argument {index}: {value}"))
    })
}

fn main() -> Result<(), String> {
    let rounds = argument(1, 100)?;
    let interval = argument(2, 10)?.max(1);
    let data = GameData::load()?;
    let mut reported = None;
    let result = campaign::play(&data, rounds, |campaign| {
        let round = campaign.completed_rounds;
        if round.is_multiple_of(interval) && reported != Some(round) {
            reported = Some(round);
            report(campaign);
        }
    });
    match result {
        Ok(campaign) => {
            report(&campaign);
            Ok(())
        }
        Err(error) => {
            println!("Script stopped: {error}");
            Ok(())
        }
    }
}

fn report(campaign: &StrategicCampaign) {
    let neutral = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none())
        .count();
    println!(
        "Round {} (neutral sites {neutral})",
        campaign.completed_rounds
    );
    for faction in campaign.factions.values() {
        let owned: Vec<_> = campaign
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(faction.id))
            .collect();
        let settled = owned
            .iter()
            .filter(|site| site.habitation >= Habitation::Outpost)
            .count();
        let cities = owned
            .iter()
            .filter(|site| site.habitation >= Habitation::City)
            .count();
        let supplied = campaign.supplied_sites(faction.id);
        let supplied_settled = owned
            .iter()
            .filter(|site| site.habitation >= Habitation::Outpost && supplied.contains(&site.id))
            .count();
        let armies = campaign
            .armies
            .values()
            .filter(|army| army.faction == faction.id)
            .count();
        let formations = campaign
            .formations
            .values()
            .filter(|formation| formation.faction == faction.id && formation.headcount > 0)
            .count();
        let at_war_with_player = faction.id != campaign.player
            && campaign.relations.iter().any(|relation| {
                relation.state == DiplomaticState::War
                    && relation.factions.contains(&faction.id)
                    && relation.factions.contains(&campaign.player)
            });
        let income = faction.last_economy.as_ref();
        let headquarters = campaign
            .world
            .site(faction.headquarters)
            .and_then(|site| site.controller)
            .map_or("unheld".to_owned(), |owner| {
                if owner == faction.id {
                    "held".to_owned()
                } else {
                    format!("lost to {}", owner.0)
                }
            });
        println!(
            "  {:<2} {:<16} {:?}{} hq={headquarters} sites={} settled={} supplied_settled={} cities={} armies={} formations={} gold={} income={} upkeep={}",
            faction.id.0,
            faction.name,
            faction.status,
            if faction.id == campaign.player {
                " (player)"
            } else if at_war_with_player {
                " (war)"
            } else {
                ""
            },
            owned.len(),
            settled,
            supplied_settled,
            cities,
            armies,
            formations,
            faction.resources.gold,
            income.map_or(0, |statement| statement.income.gold),
            income.map_or(0, |statement| statement.upkeep_due),
        );
    }
}
