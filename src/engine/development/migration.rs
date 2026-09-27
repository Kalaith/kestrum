//! Frozen friendly paths and fixed arrival budgets prevent sequential population reuse.

use super::*;
use std::collections::VecDeque;

pub(super) fn destinations(
    campaign: &StrategicCampaign,
    data: &GameData,
    snapshot: &DevelopmentSnapshot,
    from: SiteId,
) -> Vec<SiteId> {
    let Some(owner) = snapshot.conditions.get(&from).and_then(|c| c.owner) else {
        return Vec::new();
    };
    if !campaign.world.is_secure(from, owner) || campaign.active_threat(from).is_some() {
        return Vec::new();
    }
    let mut seen = BTreeSet::from([from]);
    let mut queue = VecDeque::from([(from, 0)]);
    let mut result = Vec::new();
    while let Some((site, distance)) = queue.pop_front() {
        if site != from {
            let c = &snapshot.conditions[&site];
            if c.safe && !c.ruined && c.habitation != Habitation::Unsettled {
                result.push((distance, site));
            }
        }
        if distance == data.development.population.migration_max_edges {
            continue;
        }
        for next in campaign.world.adjacent_sites(site) {
            if campaign.world.is_secure(next, owner)
                && campaign.active_threat(next).is_none()
                && seen.insert(next)
            {
                queue.push_back((next, distance + 1));
            }
        }
    }
    result.sort();
    result.into_iter().map(|(_, site)| site).collect()
}

pub(super) fn apply(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    snapshot: &DevelopmentSnapshot,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let mut room: BTreeMap<_, _> = campaign
        .world
        .sites
        .iter()
        .map(|site| {
            (
                site.id,
                population_capacity(data, site).saturating_sub(campaign.world.population[&site.id]),
            )
        })
        .collect();
    let departures: Vec<_> = campaign
        .world
        .development
        .iter()
        .filter(|(_, state)| state.displaced > 0)
        .map(|(id, state)| (*id, state.displaced))
        .collect();
    let mut moves = Vec::new();
    for (from, mut remaining) in departures {
        for to in destinations(campaign, data, snapshot, from) {
            let amount = remaining.min(room[&to]);
            if amount == 0 {
                continue;
            }
            remaining -= amount;
            *room.get_mut(&to).expect("destination budget") -= amount;
            moves.push((from, to, amount));
            if remaining == 0 {
                break;
            }
        }
    }
    // The move plan reads no arrivals. Applying it cannot change a later decision.
    for (from, to, amount) in moves {
        transfer(campaign, from, to, amount)?;
        let owner = snapshot.conditions[&from].owner.expect("friendly path");
        record(
            campaign,
            outcome,
            DevelopmentReceipt::PopulationMoved {
                owner,
                from,
                to,
                amount,
                resettled: false,
            },
        )?;
    }
    Ok(())
}

pub(super) fn transfer(
    campaign: &mut StrategicCampaign,
    from: SiteId,
    to: SiteId,
    amount: u32,
) -> Result<(), RuleError> {
    let destination =
        campaign.world.population[&to]
            .checked_add(amount)
            .ok_or(RuleError::Overflow {
                field: "population migration",
            })?;
    *campaign
        .world
        .population
        .get_mut(&from)
        .expect("source population") -= amount;
    campaign
        .world
        .development
        .get_mut(&from)
        .expect("source development")
        .displaced -= amount;
    campaign.world.population.insert(to, destination);
    Ok(())
}
