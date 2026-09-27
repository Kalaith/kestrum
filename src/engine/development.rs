//! Frozen local conditions with simultaneous civilian changes at the seasonal boundary.

mod commands;
mod conditions;
mod migration;
mod progress;
mod query;

pub(crate) use commands::execute;
pub(crate) use progress::resolve;
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
    conditions: BTreeMap<SiteId, conditions::LocalConditions>,
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
