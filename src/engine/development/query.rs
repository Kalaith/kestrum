//! Own-settlement projections expose reasons and costs without foreign population data.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevelopmentCause {
    pub label: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevelopmentView {
    pub site: SiteId,
    pub population: u32,
    pub capacity: u32,
    pub habitation: Habitation,
    pub maximum_habitation: Habitation,
    pub pressure: i32,
    pub contribution: Option<i32>,
    pub causes: Vec<DevelopmentCause>,
    pub safe: Option<bool>,
    pub supplied: bool,
    pub ruined: bool,
    pub displaced: u32,
    pub ruin_streak: u32,
    pub occupation: u32,
    pub structural_damage: u32,
    pub fort_damage: u32,
    pub growth_blocked: Option<String>,
    pub civil_identity: Vec<CivilIdentity>,
    pub resettle_targets: Vec<SiteId>,
    pub resettle_cost: Resources,
    pub capital_cost: Resources,
    pub capital_blocked: Option<String>,
    pub headquarters_cost: Resources,
    pub headquarters_blocked: Option<String>,
    pub headquarters_available_round: u32,
}

pub fn development_view(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    site: SiteId,
) -> Option<DevelopmentView> {
    let location = commands::owned(campaign, observer, site).ok()?;
    let state = &campaign.world.development[&site];
    let c = conditions::at(campaign, data, location);
    let known = safety_observed(campaign, observer, site);
    let rules = &data.development;
    let snapshot = snapshot(campaign, data);
    let resettle_targets = if state.displaced == 0 {
        Vec::new()
    } else {
        migration::destinations(campaign, data, &snapshot, site)
            .into_iter()
            .filter(|id| safety_observed(campaign, observer, *id))
            .filter(|id| {
                let target = campaign.world.site(*id).expect("destination");
                campaign.world.population[id] < population_capacity(data, target)
            })
            .collect()
    };
    Some(DevelopmentView {
        site,
        population: campaign.world.population[&site],
        capacity: population_capacity(data, location),
        habitation: location.habitation,
        maximum_habitation: maximum_habitation(data, location),
        pressure: state.pressure,
        contribution: known.then(|| {
            if state.ruined {
                0
            } else {
                conditions::contribution(data, &c)
            }
        }),
        causes: if !known || state.ruined {
            Vec::new()
        } else {
            conditions::causes(data, &c)
        },
        safe: known.then_some(c.safe),
        supplied: c.supplied,
        ruined: state.ruined,
        displaced: state.displaced,
        ruin_streak: state.ruin_streak,
        occupation: c.occupation,
        structural_damage: c.damage,
        fort_damage: c.fort_damage,
        growth_blocked: growth_blocked(campaign, data, location, &c),
        civil_identity: conditions::identities(campaign, data, location, &c),
        resettle_targets,
        resettle_cost: rules.population.resettle_cost,
        capital_cost: rules.administration.capital_cost,
        capital_blocked: if known {
            commands::capital_check(campaign, data, observer, site)
                .err()
                .map(|error| error.to_string())
        } else {
            Some("Safety here is not currently observed.".into())
        },
        headquarters_cost: rules.administration.headquarters_cost,
        headquarters_blocked: commands::headquarters_check(campaign, data, observer, site)
            .err()
            .map(|error| error.to_string()),
        headquarters_available_round: campaign.factions[&observer]
            .last_hq_relocation
            .map_or(0, |round| {
                round.saturating_add(rules.administration.headquarters_cooldown)
            }),
    })
}

fn growth_blocked(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: &Site,
    c: &conditions::LocalConditions,
) -> Option<String> {
    let reason = if c.ruined {
        "Reclaim these ruins with an Outpost order."
    } else if c.habitation == Habitation::Unsettled {
        "An Outpost order is required to establish habitation."
    } else if c.habitation >= maximum_habitation(data, site) {
        "This site has reached its terrain's habitation limit."
    } else if next_tier(c.habitation).is_some_and(|next| {
        campaign.world.population[&site.id] < data.construction.population.minimum[&next]
    }) {
        "Population is below the next habitation threshold."
    } else if campaign.world.development[&site.id].pressure
        < data.development.pressure.upgrade_threshold
    {
        "More development pressure is needed for the next tier."
    } else {
        return None;
    };
    Some(reason.into())
}

fn safety_observed(campaign: &StrategicCampaign, observer: FactionId, site: SiteId) -> bool {
    let visible: BTreeSet<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == observer && !army.is_empty())
        .flat_map(|army| std::iter::once(army.site).chain(campaign.world.adjacent_sites(army.site)))
        .collect();
    std::iter::once(site)
        .chain(campaign.world.adjacent_sites(site))
        .all(|id| visible.contains(&id))
}
