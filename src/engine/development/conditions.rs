//! Explainable current conditions; no enemy identities are included in local views.

use super::*;
use crate::data::world::{Facility, MilitaryLayer};

#[derive(Clone)]
pub(crate) struct LocalConditions {
    pub owner: Option<FactionId>,
    pub habitation: Habitation,
    pub ruined: bool,
    pub safe: bool,
    pub supplied: bool,
    pub trade: bool,
    pub food: bool,
    pub capital: bool,
    pub garrison: bool,
    pub functional_fort: bool,
    pub lawless: bool,
    pub battle: bool,
    pub governor: bool,
    pub damage: u32,
    pub occupation: u32,
    pub fort_damage: u32,
    pub focus: Option<Focus>,
}

pub(crate) fn at(campaign: &StrategicCampaign, data: &GameData, site: &Site) -> LocalConditions {
    let battle = campaign.battles.values().any(|battle| {
        battle.site == site.id && battle.completed_rounds == campaign.completed_rounds
    });
    let owner = site.controller;
    let hostile_at = |id: SiteId| {
        campaign.armies.values().any(|army| {
            army.site == id
                && !army.is_empty()
                && owner.is_none_or(|owner| {
                    super::super::retreat::hostile(campaign, owner, army.faction)
                })
        })
    };
    let occupied = hostile_at(site.id);
    let hostile = occupied
        || campaign
            .world
            .adjacent_sites(site.id)
            .into_iter()
            .any(hostile_at);
    let safe = !hostile
        && !battle
        && campaign.active_threat(site.id).is_none()
        && !campaign.world.contested_sites.contains(&site.id);
    let supplied = owner
        .and_then(|id| campaign.factions.get(&id))
        .is_some_and(|faction| campaign.supply_path(faction.id, site.id).is_some());
    let damage = campaign.world.structural_damage(site.id);
    let fort_damage = campaign
        .world
        .fort_damage
        .get(&site.id)
        .copied()
        .unwrap_or(0);
    let garrison = owner.is_some_and(|owner| {
        campaign
            .armies
            .values()
            .any(|army| army.site == site.id && army.faction == owner && !army.is_empty())
    });
    let functional_fort = site.military == MilitaryLayer::Fort
        && fort_damage < data.development.conditions.functional_fort_damage
        && !campaign.world.contested_sites.contains(&site.id);
    let ruined = campaign.site_is_ruined(site.id);
    LocalConditions {
        owner,
        habitation: site.habitation,
        ruined,
        safe,
        supplied,
        trade: trade(campaign, data, site),
        food: matches!(
            site.geography,
            Geography::Plains | Geography::River | Geography::Valley
        ) && damage < data.development.conditions.food_damage,
        capital: owner
            .and_then(|id| campaign.factions.get(&id))
            .is_some_and(|faction| faction.capital == site.id),
        garrison,
        functional_fort,
        lawless: ruined
            && !garrison
            && !functional_fort
            && !occupied
            && !campaign
                .armies
                .values()
                .any(|army| army.site == site.id && !army.is_empty())
            && campaign.active_threat(site.id).is_none(),
        battle,
        governor: campaign.people.values().any(|person| {
            owner == Some(person.faction)
                && person.career.site_role == Some(crate::state::people::PersonSiteRole::Governor)
                && person.status == crate::state::people::PersonStatus::Fit
                && person.assignment
                    == (crate::state::people::PersonAssignment::Site { site: site.id })
        }),
        damage,
        fort_damage,
        occupation: campaign
            .world
            .occupation
            .get(&site.id)
            .copied()
            .unwrap_or(0),
        focus: campaign.world.focus.get(&site.id).copied(),
    }
}

fn trade(campaign: &StrategicCampaign, data: &GameData, site: &Site) -> bool {
    let Some(owner) = site.controller else {
        return false;
    };
    campaign.world.routes.iter().any(|route| {
        route.road.improved
            && route.road.damage < data.rules.movement.road_disabled_damage
            && route
                .other_endpoint(site.id)
                .and_then(|id| campaign.world.site(id))
                .is_some_and(|neighbor| {
                    neighbor.controller == Some(owner)
                        && neighbor.habitation != Habitation::Unsettled
                        && !campaign.site_is_ruined(neighbor.id)
                })
    })
}

pub(crate) fn causes(data: &GameData, conditions: &LocalConditions) -> Vec<DevelopmentCause> {
    let p = &data.development.pressure;
    let entries = [
        (conditions.safe, "Safe", p.safe),
        (conditions.supplied, "Connected", p.connected),
        (conditions.trade, "Trade", p.trade),
        (conditions.food, "Food", p.food),
        (conditions.capital, "Capital", p.capital),
        (
            conditions.focus == Some(Focus::Growth),
            "Growth focus",
            p.growth_focus,
        ),
        (conditions.battle, "Battle", p.battle),
        (
            conditions.governor,
            "Governor",
            data.lifecycle.governor_pressure,
        ),
        (!conditions.supplied, "Cut off", p.cut_off),
        (
            conditions.damage >= data.development.conditions.pressure_damage,
            "Damage",
            p.damaged,
        ),
        (
            conditions.occupation >= data.development.conditions.occupation_threshold,
            "Occupation",
            p.occupied,
        ),
    ];
    entries
        .into_iter()
        .filter(|(enabled, _, _)| *enabled)
        .map(|(_, label, amount)| DevelopmentCause {
            label: label.into(),
            amount,
        })
        .collect()
}

pub(crate) fn contribution(data: &GameData, conditions: &LocalConditions) -> i32 {
    causes(data, conditions)
        .iter()
        .map(|cause| cause.amount)
        .sum::<i32>()
        .clamp(
            data.development.pressure.per_round_min,
            data.development.pressure.per_round_max,
        )
}

pub(super) fn identities(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: &Site,
    conditions: &LocalConditions,
) -> Vec<CivilIdentity> {
    let mut result = Vec::new();
    if conditions.food {
        result.push(CivilIdentity::Agricultural);
    }
    if site.military != MilitaryLayer::None {
        result.push(CivilIdentity::Military);
    }
    if site.facilities.contains(&Facility::Temple)
        && !conditions.ruined
        && conditions.damage < data.economy.facility_failure_damage
    {
        result.push(CivilIdentity::Religious);
    }
    if conditions.capital
        || conditions
            .owner
            .and_then(|id| campaign.factions.get(&id))
            .is_some_and(|faction| faction.headquarters == site.id)
    {
        result.push(CivilIdentity::Administrative);
    }
    if site.habitation <= Habitation::Outpost {
        result.push(CivilIdentity::Frontier);
    }
    result
}
