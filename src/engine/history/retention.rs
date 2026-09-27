//! A battle report is the payload of its history entry, not a second budget item.

use super::*;
use crate::data::GameData;
use std::collections::BTreeSet;

pub(crate) fn prune(campaign: &mut StrategicCampaign, data: &GameData) {
    prune_with_rules(campaign, &data.history);
}

pub(crate) fn prune_with_rules(
    campaign: &mut StrategicCampaign,
    rules: &crate::data::progression::HistoryRules,
) {
    let now = campaign.completed_rounds;
    let pending: BTreeSet<_> = campaign.pending_facts.iter().map(|fact| fact.id).collect();
    campaign.history.events.retain(|_, entry| {
        entry.source_fact.is_some_and(|id| pending.contains(&id))
            || now.saturating_sub(entry.completed_rounds) <= rules.detail_max_age_rounds
    });
    let mut ordered: Vec<_> = campaign
        .history
        .events
        .values()
        .map(|entry| (entry.completed_rounds, entry.id))
        .collect();
    ordered.sort();
    let remove = ordered.len().saturating_sub(rules.detail_max_entries);
    for (_, id) in ordered
        .into_iter()
        .filter(|(_, id)| {
            !campaign.history.events[id]
                .source_fact
                .is_some_and(|id| pending.contains(&id))
        })
        .take(remove)
        .collect::<Vec<_>>()
    {
        campaign.history.events.remove(&id);
    }
    campaign.history.refresh_war_links();
    let retained: BTreeSet<_> = campaign
        .history
        .events
        .values()
        .filter_map(|entry| match entry.kind {
            HistoryKind::Battle { battle, .. } => Some(battle),
            _ => None,
        })
        .collect();
    campaign.battles.retain(|id, _| retained.contains(id));
    super::departed::prune(campaign, rules);
    campaign
        .history
        .person_last_reminded
        .retain(|id, _| campaign.people.contains_key(id));
    campaign
        .history
        .site_last_reminded
        .retain(|id, _| campaign.world.site(*id).is_some());
    campaign
        .history
        .site_notables
        .retain(|id, _| campaign.world.site(*id).is_some());
    campaign
        .history
        .army_notables
        .retain(|id, _| campaign.armies.contains_key(id));
    campaign
        .history
        .person_notables
        .retain(|id, _| campaign.people.contains_key(id));
    for entries in campaign
        .history
        .site_notables
        .values_mut()
        .chain(campaign.history.army_notables.values_mut())
        .chain(campaign.history.person_notables.values_mut())
    {
        entries.retain(|entry| {
            now.saturating_sub(entry.completed_rounds) <= rules.notable_max_age_rounds
        });
        entries.sort_by_key(|entry| (entry.completed_rounds, entry.id));
        let remove = entries.len().saturating_sub(rules.notable_max_entries);
        entries.drain(..remove);
    }
}
