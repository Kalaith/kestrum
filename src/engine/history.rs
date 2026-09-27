//! Immutable narrative insertion, observer-filtered paging, and seasonal retention.

mod departed;
mod records;
mod retention;
pub(crate) use records::{record_facts, record_veterancy, restore_battle_history};
pub(crate) use retention::{prune, prune_with_rules};

use crate::{
    data::world::FactionId,
    state::{history::*, StrategicCampaign},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HistoryFilter {
    pub subject: Option<HistorySubject>,
    pub kind: Option<HistoryKindFilter>,
    pub from_round: Option<u32>,
    pub to_round: Option<u32>,
    pub page: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPage {
    pub entries: Vec<HistoryRecord>,
    pub notables: Vec<NotableSummary>,
    pub page: usize,
    pub total_pages: usize,
    pub total_entries: usize,
}

pub fn history_page(
    campaign: &StrategicCampaign,
    observer: FactionId,
    filter: &HistoryFilter,
) -> HistoryPage {
    let subject_allowed = match filter.subject {
        Some(HistorySubject::Formation(id)) => campaign
            .formations
            .get(&id)
            .is_some_and(|formation| formation.faction == observer),
        _ => campaign.factions.contains_key(&observer),
    };
    let mut entries: Vec<_> = campaign
        .history
        .events
        .values()
        .filter(|entry| {
            subject_allowed
                && entry.visible_to.contains(&observer)
                && filter.subject.is_none_or(|subject| entry.concerns(subject))
                && filter.kind.is_none_or(|kind| entry.kind.category() == kind)
                && filter
                    .from_round
                    .is_none_or(|date| entry.completed_rounds >= date)
                && filter
                    .to_round
                    .is_none_or(|date| entry.completed_rounds <= date)
        })
        .collect();
    entries.sort_by_key(|entry| std::cmp::Reverse((entry.completed_rounds, entry.id)));
    let total_entries = entries.len();
    let total_pages = total_entries.div_ceil(50).max(1);
    let page = filter.page.min(total_pages - 1);
    let entries = entries
        .into_iter()
        .skip(page * 50)
        .take(50)
        .cloned()
        .collect();
    let notables = notables(campaign, observer, filter);
    HistoryPage {
        entries,
        notables,
        page,
        total_pages,
        total_entries,
    }
}

fn notables(
    campaign: &StrategicCampaign,
    observer: FactionId,
    filter: &HistoryFilter,
) -> Vec<NotableSummary> {
    let list = match filter.subject {
        Some(HistorySubject::Site(id)) => campaign.history.site_notables.get(&id),
        Some(HistorySubject::Army(id))
            if campaign
                .armies
                .get(&id)
                .is_some_and(|army| army.faction == observer) =>
        {
            campaign.history.army_notables.get(&id)
        }
        Some(HistorySubject::Person(id))
            if campaign
                .people
                .get(&id)
                .is_some_and(|person| person.faction == observer) =>
        {
            campaign.history.person_notables.get(&id)
        }
        _ => None,
    };
    let mut result: Vec<_> = list
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.visible_to.contains(&observer)
                && filter.kind.is_none_or(|kind| entry.kind.category() == kind)
                && filter
                    .from_round
                    .is_none_or(|date| entry.completed_rounds >= date)
                && filter
                    .to_round
                    .is_none_or(|date| entry.completed_rounds <= date)
        })
        .cloned()
        .collect();
    result.sort_by_key(|entry| std::cmp::Reverse((entry.completed_rounds, entry.id)));
    result
}
