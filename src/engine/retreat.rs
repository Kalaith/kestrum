//! Shared physical retreat eligibility; a retreat never initiates an encounter.

use crate::{
    data::{
        economy::Habitation,
        world::{DiplomaticState, FactionId, SiteId},
    },
    state::StrategicCampaign,
};

pub(super) fn hostile(campaign: &StrategicCampaign, first: FactionId, second: FactionId) -> bool {
    first != second
        && campaign.relations.iter().any(|relation| {
            relation.factions.contains(&first)
                && relation.factions.contains(&second)
                && relation.state == DiplomaticState::War
        })
}

pub(super) fn eligible(
    campaign: &StrategicCampaign,
    faction: FactionId,
    battle: SiteId,
    excluded: Option<SiteId>,
) -> Vec<SiteId> {
    campaign
        .world
        .adjacent_sites(battle)
        .into_iter()
        .filter(|id| {
            Some(*id) != excluded
                && campaign.world.site(*id).is_some_and(|site| {
                    site.controller.is_none_or(|owner| owner == faction)
                        && !campaign.world.contested_sites.contains(id)
                })
                && !campaign.sieges.contains_key(id)
                && !campaign.armies.values().any(|army| {
                    army.site == *id
                        && hostile(campaign, faction, army.faction)
                        && army
                            .formation_ids()
                            .any(|id| campaign.formations[&id].headcount > 0)
                })
        })
        .collect()
}

pub(super) fn destination(
    campaign: &StrategicCampaign,
    faction: FactionId,
    battle: SiteId,
    preferred: Option<SiteId>,
    excluded: Option<SiteId>,
) -> Option<SiteId> {
    let mut sites = eligible(campaign, faction, battle, excluded);
    let headquarters = campaign.factions.get(&faction)?.headquarters;
    let supplied = campaign.world.supplied_sites(faction, headquarters);
    sites.sort_by_key(|site| (Some(*site) != preferred, !supplied.contains(site), *site));
    sites.first().copied()
}

pub(super) fn refuges(
    campaign: &StrategicCampaign,
    faction: FactionId,
    battle: SiteId,
    excluded: Option<SiteId>,
) -> Vec<SiteId> {
    eligible(campaign, faction, battle, excluded)
        .into_iter()
        .filter(|id| {
            campaign.world.site(*id).is_some_and(|site| {
                site.controller == Some(faction) && site.habitation != Habitation::Unsettled
            })
        })
        .collect()
}
