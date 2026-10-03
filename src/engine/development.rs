//! Frozen local conditions with simultaneous civilian changes at the seasonal boundary.

mod commands;
pub(crate) mod conditions;
mod migration;
mod progress;
mod query;

pub(crate) use commands::execute;
pub(crate) use progress::{forecast_step, resolve};
pub(crate) use query::observed_safety_sites;
pub use query::{development_view, DevelopmentCause, DevelopmentView};

use super::{actions::record_fact, ActionOutcome, Command, RuleError};
use crate::{
    data::{
        economy::{Habitation, Resources},
        world::{FactionId, Geography, Site, SiteId, SiteTag},
        GameData,
    },
    state::{
        campaign::DomainFactKind,
        construction::Focus,
        development::{CivilIdentity, DevelopmentReceipt, SiteDevelopment},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct DevelopmentSnapshot {
    pub(crate) conditions: BTreeMap<SiteId, conditions::LocalConditions>,
}

#[derive(Clone)]
pub(crate) struct DevelopmentStepState {
    pub(crate) development: SiteDevelopment,
    pub(crate) population: u32,
    pub(crate) habitation: Habitation,
    pub(crate) damage: u32,
    pub(crate) occupation: u32,
    pub(crate) fort_damage: u32,
}

pub(crate) struct DevelopmentForecast {
    pub(crate) habitation: Habitation,
    pub(crate) ruined: bool,
}

pub(crate) fn step_state(campaign: &StrategicCampaign, id: SiteId) -> DevelopmentStepState {
    let site = campaign.world.site(id).expect("development site");
    DevelopmentStepState {
        development: campaign.world.development[&id].clone(),
        population: campaign.world.population[&id],
        habitation: site.habitation,
        damage: campaign.world.structural_damage(id),
        occupation: campaign.world.occupation.get(&id).copied().unwrap_or(0),
        fort_damage: campaign.world.fort_damage.get(&id).copied().unwrap_or(0),
    }
}

pub(crate) fn snapshot(campaign: &StrategicCampaign, data: &GameData) -> DevelopmentSnapshot {
    DevelopmentSnapshot {
        conditions: campaign
            .world
            .sites
            .iter()
            .map(|site| (site.id, conditions::at(campaign, data, site)))
            .collect(),
    }
}

pub fn population_capacity(data: &GameData, site: &Site) -> u32 {
    let cap = maximum_habitation(data, site);
    (u64::from(data.construction.population.minimum[&cap])
        * u64::from(data.development.population.capacity_percent)
        / 100)
        .min(u64::from(u32::MAX)) as u32
}

pub fn maximum_habitation(data: &GameData, site: &Site) -> Habitation {
    let geography = if site.tags.contains(&SiteTag::Pass) {
        Geography::Pass
    } else {
        site.geography
    };
    data.development.geography_caps[&geography]
}

pub(crate) fn focus_suitable(site: &Site, focus: Focus) -> bool {
    match focus {
        Focus::Wood => {
            site.geography == Geography::Forest || site.tags.contains(&SiteTag::WoodSource)
        }
        Focus::Stone => {
            site.geography == Geography::Hill || site.tags.contains(&SiteTag::StoneSource)
        }
        _ => true,
    }
}

fn blocked(message: &str) -> RuleError {
    RuleError::Development(message.to_owned())
}

fn record(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
    receipt: DevelopmentReceipt,
) -> Result<(), RuleError> {
    record_fact(
        campaign,
        outcome,
        DomainFactKind::DevelopmentChanged { receipt },
    )
}

fn next_tier(tier: Habitation) -> Option<Habitation> {
    use Habitation::*;
    match tier {
        Unsettled => None,
        Camp => Some(Outpost),
        Outpost => Some(Hamlet),
        Hamlet => Some(Village),
        Village => Some(Town),
        Town => Some(City),
        City => Some(MajorCity),
        MajorCity => None,
    }
}
fn previous_tier(tier: Habitation) -> Option<Habitation> {
    use Habitation::*;
    match tier {
        Unsettled => None,
        Camp => Some(Unsettled),
        Outpost => Some(Camp),
        Hamlet => Some(Outpost),
        Village => Some(Hamlet),
        Town => Some(Village),
        City => Some(Town),
        MajorCity => Some(City),
    }
}
