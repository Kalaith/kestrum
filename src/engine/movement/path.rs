//! Positive-cost shortest paths, with complete site-ID paths breaking equal costs.

use super::*;
use std::collections::BTreeMap;

pub(super) fn shortest(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    origin: SiteId,
    destination: SiteId,
    usable_only: bool,
) -> Option<Vec<SiteId>> {
    let mut best = BTreeMap::from([(origin, (0_u32, vec![origin]))]);
    let mut settled = BTreeSet::new();
    loop {
        let (&site, (cost, path)) = best
            .iter()
            .filter(|(site, _)| !settled.contains(*site))
            .min_by(|a, b| a.1.cmp(b.1))?;
        let (cost, path) = (*cost, path.clone());
        if site == destination {
            return Some(path);
        }
        settled.insert(site);
        for adjacent in campaign.world.adjacent_sites(site) {
            if settled.contains(&adjacent)
                || (usable_only
                    && adjacent != destination
                    && public_block(campaign, owner, adjacent).is_some())
            {
                continue;
            }
            let edge = campaign.world.connected_route(site, adjacent)?;
            let cost = cost.checked_add(route_cost(edge, data))?;
            let mut path = path.clone();
            path.push(adjacent);
            let candidate = (cost, path);
            if best.get(&adjacent).is_none_or(|known| candidate < *known) {
                best.insert(adjacent, candidate);
            }
        }
    }
}
