//! Recent, unread or active receipts stay attached to known map locations.

use super::{orders, status::*, *};
use kestrum::{
    data::world::{RouteId, SiteId},
    state::notifications::{NotificationDetail, NotificationSubjectSnapshot, RenderedNotification},
};
use std::collections::BTreeMap;

pub(super) fn candidates(input: &MapStatusInput<'_>) -> Vec<StatusCandidate> {
    let Some(projection) = input.projection else {
        return Vec::new();
    };
    let ids: std::collections::BTreeSet<_> = projection.map_receipt_ids.iter().copied().collect();
    let mut receipts: Vec<_> = projection
        .recent
        .iter()
        .filter(|item| {
            ids.contains(&item.id)
                && (item.completed_rounds == input.campaign.completed_rounds
                    || !item.is_read
                    || item.is_active)
        })
        .collect();
    receipts.sort_by_key(|item| {
        (
            std::cmp::Reverse(item.is_active),
            item.priority,
            std::cmp::Reverse(item.occurred_order),
        )
    });
    let mut grouped: BTreeMap<StatusAnchor, (Vec<&RenderedNotification>, Vec2)> = BTreeMap::new();
    for item in receipts {
        let Some(location) = location(item) else {
            continue;
        };
        let anchor = match location {
            ReceiptLocation::Site(site) => {
                let Some((anchor, at)) = orders::site_anchor(input, site) else {
                    continue;
                };
                (anchor, at)
            }
            ReceiptLocation::Route(route) => {
                let Some((anchor, at)) = orders::route_anchor(input, route) else {
                    continue;
                };
                (anchor, at)
            }
        };
        let entry = grouped
            .entry(anchor.0)
            .or_insert_with(|| (Vec::new(), anchor.1));
        entry.0.push(item);
    }
    grouped
        .into_iter()
        .filter_map(|(_, (items, at))| {
            let latest = items.first()?;
            Some(StatusCandidate {
                at,
                content: StatusContent::Receipt {
                    ids: items.iter().map(|item| item.id).collect(),
                    title: latest.title.clone(),
                    category: latest.category,
                    priority: latest.priority,
                    active: latest.is_active,
                },
                intent: StatusIntent::Receipt(latest.id),
            })
        })
        .collect()
}

pub fn map_receipt_group(
    input: &MapStatusInput<'_>,
    id: kestrum::state::notifications::NotificationId,
) -> Vec<kestrum::state::notifications::NotificationId> {
    candidates(input)
        .into_iter()
        .find_map(|candidate| match candidate.content {
            StatusContent::Receipt { ids, .. } if ids.contains(&id) => Some(ids),
            _ => None,
        })
        .unwrap_or_default()
}

enum ReceiptLocation {
    Site(SiteId),
    Route(RouteId),
}

fn location(item: &RenderedNotification) -> Option<ReceiptLocation> {
    item.subject
        .as_ref()
        .and_then(subject_location)
        .or_else(|| detail_location(&item.detail))
}

fn subject_location(subject: &NotificationSubjectSnapshot) -> Option<ReceiptLocation> {
    match subject {
        NotificationSubjectSnapshot::Person(person) => person
            .site
            .as_ref()
            .map(|site| ReceiptLocation::Site(site.id))
            .or_else(|| {
                person
                    .army
                    .as_ref()
                    .and_then(|army| army.site.as_ref())
                    .map(|site| ReceiptLocation::Site(site.id))
            }),
        NotificationSubjectSnapshot::Place(place) => Some(ReceiptLocation::Site(place.id)),
        NotificationSubjectSnapshot::Army(army) => army
            .site
            .as_ref()
            .map(|site| ReceiptLocation::Site(site.id)),
        NotificationSubjectSnapshot::Route(route) => Some(ReceiptLocation::Route(route.id)),
        NotificationSubjectSnapshot::Construction(work) => work
            .site
            .as_ref()
            .map(|site| ReceiptLocation::Site(site.id))
            .or_else(|| {
                work.route
                    .as_ref()
                    .map(|route| ReceiptLocation::Route(route.id))
            }),
        NotificationSubjectSnapshot::Battle(battle) => Some(ReceiptLocation::Site(battle.site.id)),
        NotificationSubjectSnapshot::Faction { .. }
        | NotificationSubjectSnapshot::History { .. } => None,
    }
}

fn detail_location(detail: &NotificationDetail) -> Option<ReceiptLocation> {
    match detail {
        NotificationDetail::Person { person, .. } => {
            subject_location(&NotificationSubjectSnapshot::Person(person.clone()))
        }
        NotificationDetail::Place { place, .. } => Some(ReceiptLocation::Site(place.id)),
        NotificationDetail::Work { work, .. } => {
            subject_location(&NotificationSubjectSnapshot::Construction(work.clone()))
        }
        NotificationDetail::Movement {
            armies,
            destination,
            ..
        } => destination
            .as_ref()
            .map(|site| ReceiptLocation::Site(site.id))
            .or_else(|| {
                armies.iter().find_map(|army| {
                    army.site
                        .as_ref()
                        .map(|site| ReceiptLocation::Site(site.id))
                })
            }),
        NotificationDetail::Battle { battle } => Some(ReceiptLocation::Site(battle.site.id)),
        NotificationDetail::Remembrance { subject, .. } => subject_location(subject),
        NotificationDetail::Diplomacy { .. } | NotificationDetail::Facts { .. } => None,
    }
}
