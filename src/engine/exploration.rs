//! Campaign-owned discovery on the physical route graph.

use crate::{
    data::world::{FactionId, SiteId},
    state::StrategicCampaign,
};
use std::collections::BTreeSet;

/// Ownership and stationed armies reveal their location and its immediate exits.
/// Saved discoveries survive departure; inspecting and previewing never add any.
pub fn explored_sites(campaign: &StrategicCampaign, observer: FactionId) -> BTreeSet<SiteId> {
    let mut known = campaign
        .knowledge
        .explored
        .get(&observer)
        .cloned()
        .unwrap_or_default();
    let origins: BTreeSet<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(observer))
        .map(|site| site.id)
        .chain(
            campaign
                .armies
                .values()
                .filter(|army| army.faction == observer)
                .map(|army| army.site),
        )
        .collect();
    for site in origins {
        known.insert(site);
        known.extend(campaign.world.adjacent_sites(site));
    }
    known
}

pub(crate) fn observe(campaign: &mut StrategicCampaign) {
    for observer in campaign.factions.keys().copied().collect::<Vec<_>>() {
        let known = explored_sites(campaign, observer);
        campaign.knowledge.explored.insert(observer, known);
        let contacts = known_factions(campaign, observer);
        campaign.knowledge.contacts.insert(observer, contacts);
    }
}

pub(super) fn visit(campaign: &mut StrategicCampaign, observer: FactionId, site: SiteId) {
    let nearby = campaign.world.adjacent_sites(site);
    let known = campaign.knowledge.explored.entry(observer).or_default();
    known.insert(site);
    known.extend(nearby);
    let contacts = known_factions(campaign, observer);
    campaign.knowledge.contacts.insert(observer, contacts);
}

pub fn known_factions(campaign: &StrategicCampaign, observer: FactionId) -> BTreeSet<FactionId> {
    let known = explored_sites(campaign, observer);
    let mut contacts = campaign
        .knowledge
        .contacts
        .get(&observer)
        .cloned()
        .unwrap_or_default();
    contacts.insert(observer);
    contacts.extend(
        campaign
            .world
            .sites
            .iter()
            .filter(|site| known.contains(&site.id))
            .filter_map(|site| site.controller),
    );
    contacts.extend(
        campaign
            .battles
            .values()
            .filter(|battle| battle.participant_factions().any(|id| id == observer))
            .flat_map(|battle| battle.participant_factions()),
    );
    for offer in &campaign.diplomacy.pending_offers {
        if offer.recipient == observer {
            contacts.insert(offer.proposer);
        }
        if offer.proposer == observer {
            contacts.insert(offer.recipient);
        }
    }
    for defeat in &campaign.diplomacy.pending_defeats {
        if defeat.victor == observer {
            contacts.insert(defeat.faction);
        }
    }
    contacts
}

/// Only discovered geography enters the player's map, inspectors and picking.
/// The strategic simulation continues to use its own full authoritative graph.
pub fn project_map(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Result<super::VisibleCampaign, super::RuleError> {
    let mut view = super::project(campaign, observer)?;
    let known = explored_sites(campaign, observer);
    let contacts = known_factions(campaign, observer);
    view.factions
        .retain(|faction| contacts.contains(&faction.id));
    view.world.sites.retain(|site| known.contains(&site.id));
    let markers: BTreeSet<_> = view.world.sites.iter().map(|site| site.marker).collect();
    view.world
        .markers
        .retain(|marker| markers.contains(&marker.id));
    for marker in &mut view.world.markers {
        if let crate::data::world::MarkerLocation::Region {
            sites, entrances, ..
        } = &mut marker.location
        {
            sites.retain(|site| known.contains(site));
            entrances.retain(|entrance| known.contains(&entrance.site));
        }
    }
    view.world
        .routes
        .retain(|route| known.contains(&route.from) && known.contains(&route.to));
    view.world
        .atlas_paths
        .retain(|id, _| view.world.routes.iter().any(|route| route.id == *id));
    view.world.region_control.retain(|id, _| {
        campaign
            .world
            .sites
            .iter()
            .filter(|site| site.marker == *id)
            .all(|site| known.contains(&site.id))
    });
    view.world.development.retain(|id, _| known.contains(id));
    view.world.site_damage.retain(|id, _| known.contains(id));
    view.world.occupation.retain(|id, _| known.contains(id));
    view.world.founded_rounds.retain(|id, _| known.contains(id));
    view.world.contested_sites.retain(|id| known.contains(id));
    Ok(view)
}
