//! Administration and voluntary refugee movement use the same transactional command path.

use super::*;
use crate::data::world::{Facility, MarkerLocation};

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: Command,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    match command {
        Command::RenameSite { site, name } => rename(campaign, owner, site, name, outcome),
        Command::Resettle { from, to } => resettle(campaign, data, owner, from, to, outcome),
        Command::DevelopCity { site } => develop_city(campaign, data, owner, site, outcome),
        Command::MoveCapital { site } => move_capital(campaign, data, owner, site, outcome),
        Command::RelocateHeadquarters { site } => relocate(campaign, data, owner, site, outcome),
        _ => Err(blocked("Choose a settlement administration order.")),
    }
}

pub(super) fn owned(
    campaign: &StrategicCampaign,
    owner: FactionId,
    id: SiteId,
) -> Result<&Site, RuleError> {
    campaign
        .world
        .site(id)
        .filter(|site| site.controller == Some(owner))
        .ok_or_else(|| blocked("Choose a physical site you control."))
}

fn pay(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    required: Resources,
) -> Result<(), RuleError> {
    can_pay(campaign, owner, required)?;
    let balance = &mut campaign.factions.get_mut(&owner).expect("owner").resources;
    balance.gold -= required.gold;
    balance.wood -= required.wood;
    balance.stone -= required.stone;
    Ok(())
}

fn can_pay(
    campaign: &StrategicCampaign,
    owner: FactionId,
    required: Resources,
) -> Result<(), RuleError> {
    let available = campaign
        .factions
        .get(&owner)
        .ok_or(RuleError::UnknownActor)?
        .resources;
    if available.gold < required.gold
        || available.wood < required.wood
        || available.stone < required.stone
    {
        return Err(RuleError::InsufficientResources {
            required,
            available,
        });
    }
    Ok(())
}

pub(crate) fn city_development_check(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
) -> Result<(), RuleError> {
    let location = owned(campaign, owner, site)?;
    let rules = &data.development.city_development;
    if campaign.site_is_ruined(site) {
        return Err(blocked(&rules.messages.ruined));
    }
    if location.habitation < rules.minimum_habitation {
        return Err(blocked(&rules.messages.minimum_habitation));
    }
    if location.habitation >= Habitation::City {
        return Err(blocked(&rules.messages.already_city));
    }
    if campaign.world.contested_sites.contains(&site) || campaign.sieges.contains_key(&site) {
        return Err(blocked(&rules.messages.contested));
    }
    let local = conditions::at(campaign, data, location);
    if !local.safe {
        return Err(blocked(&rules.messages.unsafe_site));
    }
    if !local.supplied {
        return Err(blocked(&rules.messages.unsupplied));
    }
    if local.occupation >= data.development.conditions.occupation_threshold {
        return Err(blocked(&rules.messages.occupation));
    }
    if local.damage >= data.economy.facility_failure_damage {
        return Err(blocked(&rules.messages.damaged));
    }
    if campaign
        .world
        .adjacent_sites(site)
        .into_iter()
        .any(|neighbor| {
            campaign
                .world
                .site(neighbor)
                .is_some_and(|neighbor| neighbor.habitation >= Habitation::City)
        })
    {
        return Err(blocked(&rules.messages.adjacent_city));
    }
    can_pay(campaign, owner, rules.cost)
}

fn develop_city(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    city_development_check(campaign, data, owner, site)?;
    let cost = data.development.city_development.cost;
    let from = owned(campaign, owner, site)?.habitation;
    pay(campaign, owner, cost)?;
    campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .expect("validated site")
        .habitation = Habitation::City;
    record(
        campaign,
        outcome,
        DevelopmentReceipt::HabitationChanged {
            site,
            owner: Some(owner),
            from,
            to: Habitation::City,
        },
    )
}

fn rename(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    site: SiteId,
    name: String,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let current = owned(campaign, owner, site)?;
    let name = name.trim().to_owned();
    if !crate::state::development::valid_name(&name) {
        return Err(blocked("Use 1â€“40 characters without control characters."));
    }
    if current.name == name {
        return Err(blocked("This site already has that name."));
    }
    let old_name = current.name.clone();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .expect("site")
        .name = name.clone();
    for marker in &mut campaign.world.markers {
        if matches!(marker.location,MarkerLocation::Site{site:id} if id==site) {
            marker.name = name.clone();
        }
    }
    record(
        campaign,
        outcome,
        DevelopmentReceipt::SiteRenamed {
            owner,
            site,
            old_name,
            new_name: name,
        },
    )
}

pub(super) fn capital_check(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
) -> Result<(), RuleError> {
    let location = owned(campaign, owner, site)?;
    let c = conditions::at(campaign, data, location);
    if campaign.factions[&owner].capital == site {
        return Err(blocked("This site is already the capital."));
    }
    if c.ruined || location.habitation < data.development.administration.capital_minimum {
        return Err(blocked(
            "The capital needs a functioning Town or larger settlement.",
        ));
    }
    if !c.safe || !c.supplied {
        return Err(blocked(
            "The capital needs a safe site connected to headquarters.",
        ));
    }
    if c.occupation >= data.development.administration.capital_max_occupation {
        return Err(blocked(
            "Occupation pressure is too high to move the capital here.",
        ));
    }
    Ok(())
}

fn move_capital(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    capital_check(campaign, data, owner, site)?;
    pay(
        campaign,
        owner,
        data.development.administration.capital_cost,
    )?;
    let faction = campaign.factions.get_mut(&owner).expect("owner");
    let from = faction.capital;
    faction.capital = site;
    record(
        campaign,
        outcome,
        DevelopmentReceipt::CapitalMoved {
            owner,
            from,
            to: site,
        },
    )
}

pub(super) fn headquarters_check(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
) -> Result<(), RuleError> {
    let location = owned(campaign, owner, site)?;
    let c = conditions::at(campaign, data, location);
    let faction = &campaign.factions[&owner];
    if faction.headquarters == site {
        return Err(blocked("This site is already headquarters."));
    }
    if c.ruined || location.habitation < data.development.administration.headquarters_minimum {
        return Err(blocked(
            "Headquarters needs a functioning Village or larger settlement.",
        ));
    }
    if !c.garrison {
        return Err(blocked(
            "A friendly army must be present to relocate headquarters.",
        ));
    }
    if campaign.world.contested_sites.contains(&site) || campaign.active_threat(site).is_some() {
        return Err(blocked(
            "Headquarters cannot relocate into a siege or local threat.",
        ));
    }
    if !location.facilities.contains(&Facility::TrainingGround)
        || c.damage >= data.economy.facility_failure_damage
    {
        return Err(blocked("Headquarters needs a functional Training Ground."));
    }
    if faction.last_hq_relocation.is_some_and(|round| {
        u64::from(campaign.completed_rounds)
            < u64::from(round) + u64::from(data.development.administration.headquarters_cooldown)
    }) {
        return Err(blocked("Headquarters relocation is still on cooldown."));
    }
    Ok(())
}

fn relocate(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    headquarters_check(campaign, data, owner, site)?;
    pay(
        campaign,
        owner,
        data.development.administration.headquarters_cost,
    )?;
    let faction = campaign.factions.get_mut(&owner).expect("owner");
    let from = faction.headquarters;
    faction.headquarters = site;
    faction.last_hq_relocation = Some(campaign.completed_rounds);
    campaign.reconcile_region_control();
    record(
        campaign,
        outcome,
        DevelopmentReceipt::HeadquartersMoved {
            owner,
            from,
            to: site,
        },
    )
}

fn resettle(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    from: SiteId,
    to: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    owned(campaign, owner, from)?;
    let destination = owned(campaign, owner, to)?;
    let snapshot = snapshot(campaign, data);
    if !migration::destinations(campaign, data, &snapshot, from).contains(&to) {
        return Err(blocked(
            "Choose a safe friendly inhabited destination within the migration range.",
        ));
    }
    let amount = campaign.world.development[&from]
        .displaced
        .min(data.development.population.resettle_limit)
        .min(population_capacity(data, destination).saturating_sub(campaign.world.population[&to]));
    if amount == 0 {
        return Err(blocked(
            "This move needs displaced people and free destination capacity.",
        ));
    }
    pay(campaign, owner, data.development.population.resettle_cost)?;
    migration::transfer(campaign, from, to, amount)?;
    record(
        campaign,
        outcome,
        DevelopmentReceipt::PopulationMoved {
            owner,
            from,
            to,
            amount,
            resettled: true,
        },
    )
}
