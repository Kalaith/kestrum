//! Invite, teach and deploy named people through the normal career rules.

use super::issue;
use kestrum::{
    data::{
        world::{Facility, PersonClass, SiteId},
        GameData,
    },
    engine::{self, Command},
    state::{
        construction::{ConstructionKind, ConstructionTarget},
        people::PersonAssignment,
        StrategicCampaign,
    },
};

pub(super) fn develop(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    home: SiteId,
) -> Result<(), String> {
    let living = campaign
        .people
        .values()
        .filter(|person| person.faction == campaign.player && person.is_alive())
        .count();
    if living < 10 {
        try_order(campaign, data, Command::InviteApprentice { site: home })?;
    }
    let garrison = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && army.site == home)
        .map(|army| army.id)
        .min();
    if let Some(builder) = garrison {
        try_order(
            campaign,
            data,
            Command::StartConstruction {
                target: ConstructionTarget::Site(home),
                kind: ConstructionKind::Facility(Facility::TrainingGround),
                builder,
            },
        )?;
    }
    let people: Vec<_> = campaign
        .people
        .values()
        .filter(|person| person.faction == campaign.player && person.is_alive())
        .map(|person| person.id)
        .collect();
    for person in people {
        if campaign.people[&person].class == PersonClass::Recruit {
            if let Ok(options) = engine::mentorship_options(campaign, data, person) {
                if let Some(option) = options.into_iter().find(|option| option.eligible) {
                    try_order(
                        campaign,
                        data,
                        Command::StartMentorship {
                            mentor: option.mentor,
                            learner: person,
                            discipline: option.discipline,
                        },
                    )?;
                }
            }
            if let Ok(options) = engine::career_options(campaign, data, person) {
                if let Some(option) = options.into_iter().find(|option| option.eligible) {
                    try_order(
                        campaign,
                        data,
                        Command::TrainPerson {
                            person,
                            class: option.class,
                            site: home,
                        },
                    )?;
                }
            }
        }
        if campaign.people[&person].career.course.is_none()
            && campaign.people[&person].assignment == (PersonAssignment::Site { site: home })
        {
            let mut available = campaign.available_person_formation(campaign.player, home);
            let faction = &campaign.factions[&campaign.player];
            let affordable = faction.last_economy.as_ref().is_none_or(|statement| {
                statement.income.gold - statement.upkeep_due
                    >= data.economy.formations[&kestrum::data::economy::TroopKind::Warriors]
                        .upkeep_gold
            });
            if available.is_none() && affordable {
                let receiving = campaign
                    .armies
                    .values()
                    .filter(|army| army.faction == campaign.player && army.site == home)
                    .filter(|army| army.first_empty_slot().is_some())
                    .min_by_key(|army| (army.formation_ids().count(), army.id))
                    .map(|army| army.id);
                try_order(
                    campaign,
                    data,
                    Command::Recruit {
                        site: home,
                        army: receiving,
                        kind: kestrum::data::economy::TroopKind::Warriors,
                    },
                )?;
                available = campaign.available_person_formation(campaign.player, home);
            }
            if let Some(formation) = available {
                try_order(
                    campaign,
                    data,
                    Command::TransferPerson {
                        person,
                        to_formation: formation,
                    },
                )?;
                let army = campaign
                    .armies
                    .values()
                    .find(|army| army.formation_ids().any(|id| id == formation))
                    .expect("deployed formation")
                    .id;
                if campaign.armies[&army].commander.is_none() {
                    try_order(
                        campaign,
                        data,
                        Command::SetCommander {
                            army,
                            person: Some(person),
                        },
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn try_order(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    command: Command,
) -> Result<(), String> {
    if engine::preview(campaign, data, engine::Actor::Player, command.clone()).is_ok() {
        issue(campaign, data, command)?;
    }
    Ok(())
}
