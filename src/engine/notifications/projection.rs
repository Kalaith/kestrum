//! Local preferences turn saved typed facts into safe, data-driven view models.

use crate::{
    data::GameData,
    state::{
        notifications::{
            NotificationDelivery, NotificationGroup, NotificationPreferences,
            NotificationProjection, NotificationReceipt, NotificationSubjectSnapshot,
            RenderedNotification,
        },
        StrategicCampaign,
    },
};
use std::collections::BTreeMap;

pub fn project(
    campaign: &StrategicCampaign,
    data: &GameData,
    preferences: &NotificationPreferences,
) -> NotificationProjection {
    let rules = &data.notifications;
    let visible = campaign
        .notifications
        .receipts
        .iter()
        .filter(|receipt| {
            !receipt.is_dismissed
                && preferences.choice(receipt.kind, rules) != NotificationDelivery::Off
                && receipt.id.0
                    > campaign
                        .notifications
                        .hidden_after_id
                        .get(&receipt.kind)
                        .copied()
                        .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    let mut recent = visible
        .iter()
        .map(|receipt| render(receipt, data))
        .collect::<Vec<_>>();
    recent.sort_by_key(|item| std::cmp::Reverse(item.occurred_order));
    let unread_count = visible.iter().filter(|receipt| !receipt.is_read).count();
    let mut rail = Vec::<NotificationGroup>::new();
    let mut top_bar = visible
        .into_iter()
        .filter(|receipt| {
            preferences.choice(receipt.kind, rules) == NotificationDelivery::TopBarAndHistory
                && receipt.id.0
                    > campaign
                        .notifications
                        .top_bar_after_id
                        .get(&receipt.kind)
                        .copied()
                        .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    top_bar.sort_by_key(|receipt| std::cmp::Reverse(receipt.occurred_order));
    for receipt in top_bar {
        let rule = rules
            .kind(receipt.kind)
            .expect("validated notification kind");
        let group_index = if rule.group {
            rail.iter().position(|group| {
                group.kind == receipt.kind && group.completed_rounds == receipt.completed_rounds
            })
        } else {
            None
        };
        if let Some(index) = group_index {
            let group = &mut rail[index];
            group.receipt_ids.push(receipt.id);
            group.unread_count += usize::from(!receipt.is_read);
            group.label = format!("{} ×{}", rule.label, group.receipt_ids.len());
        } else {
            rail.push(NotificationGroup {
                kind: receipt.kind,
                category: receipt.kind.category(),
                priority: receipt.priority,
                completed_rounds: receipt.completed_rounds,
                receipt_ids: vec![receipt.id],
                unread_count: usize::from(!receipt.is_read),
                label: rule.label.clone(),
            });
        }
    }
    rail.sort_by_key(|group| {
        std::cmp::Reverse((group.completed_rounds, group.priority, group.receipt_ids[0]))
    });
    rail.truncate(rules.max_rail_items);
    NotificationProjection {
        rail,
        recent,
        unread_count,
        omitted_count: campaign.notifications.pruned_count,
        omitted_unread_count: campaign.notifications.pruned_unread_count,
    }
}

fn render(receipt: &NotificationReceipt, data: &GameData) -> RenderedNotification {
    let rule = data
        .notifications
        .kind(receipt.kind)
        .expect("validated notification kind");
    let values = detail_values(receipt);
    RenderedNotification {
        id: receipt.id,
        kind: receipt.kind,
        category: receipt.kind.category(),
        priority: receipt.priority,
        completed_rounds: receipt.completed_rounds,
        occurred_order: receipt.occurred_order,
        is_read: receipt.is_read,
        is_dismissed: receipt.is_dismissed,
        is_active: receipt.is_active,
        title: substitute(&rule.title, &values),
        body: substitute(&rule.body, &values),
        subject: receipt.subject.clone(),
        detail: receipt.detail.clone(),
    }
}

fn detail_values(receipt: &NotificationReceipt) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    if let Some(subject) = &receipt.subject {
        match subject {
            NotificationSubjectSnapshot::Person(person) => {
                values.insert("person".into(), person.name.clone());
                values.insert("class".into(), format!("{:?}", person.class));
                if let Some(place) = &person.site {
                    values.insert("place".into(), place.name.clone());
                }
                if let Some(army) = &person.army {
                    values.insert("army".into(), army.name.clone());
                }
            }
            NotificationSubjectSnapshot::Place(place) => {
                values.insert("place".into(), place.name.clone());
            }
            NotificationSubjectSnapshot::Army(army) => {
                values.insert("army".into(), army.name.clone());
                if let Some(place) = &army.site {
                    values.insert("place".into(), place.name.clone());
                }
            }
            NotificationSubjectSnapshot::Route(route) => {
                values.insert(
                    "place".into(),
                    format!("{}–{}", route.from.name, route.to.name),
                );
            }
            NotificationSubjectSnapshot::Construction(work) => {
                values.insert("work".into(), format!("{:?}", work.kind));
                if let Some(place) = &work.site {
                    values.insert("place".into(), place.name.clone());
                }
                if let Some(route) = &work.route {
                    values.insert(
                        "place".into(),
                        format!("{}–{}", route.from.name, route.to.name),
                    );
                }
            }
            NotificationSubjectSnapshot::Battle(battle) => {
                values.insert("place".into(), battle.site.name.clone());
            }
            NotificationSubjectSnapshot::Faction { name, .. } => {
                values.insert("faction".into(), name.clone());
            }
            NotificationSubjectSnapshot::History { label, .. } => {
                values.insert("subject".into(), label.clone());
            }
        }
    }
    match &receipt.detail {
        crate::state::notifications::NotificationDetail::Person {
            person,
            reason,
            opportunities,
            ..
        } => {
            values.insert("person".into(), person.name.clone());
            if let Some(reason) = reason {
                values.insert("reason".into(), reason.clone());
            }
            values.insert(
                "opportunities".into(),
                opportunities
                    .iter()
                    .map(|class| format!("{class:?}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        crate::state::notifications::NotificationDetail::Place {
            place,
            before,
            after,
            cause,
            forecast_round,
            conditions,
        } => {
            values.insert("place".into(), place.name.clone());
            if let Some(before) = before {
                values.insert("before".into(), before.clone());
            }
            if let Some(after) = after {
                values.insert("after".into(), after.clone());
            }
            if let Some(cause) = cause {
                values.insert("reason".into(), cause.clone());
                values.insert("cause".into(), cause.clone());
            }
            if let Some(round) = forecast_round {
                values.insert("when".into(), format!("season {round}"));
            }
            values.insert("conditions".into(), conditions.join(", "));
        }
        crate::state::notifications::NotificationDetail::Work {
            work,
            cause,
            forecast_round,
            ..
        } => {
            values.insert("work".into(), format!("{:?}", work.kind));
            if let Some(place) = &work.site {
                values.insert("place".into(), place.name.clone());
            }
            if let Some(route) = &work.route {
                values.insert(
                    "place".into(),
                    format!("{}–{}", route.from.name, route.to.name),
                );
            }
            if let Some(cause) = cause {
                values.insert("reason".into(), cause.clone());
            }
            if let Some(round) = forecast_round {
                values.insert("when".into(), format!("season {round}"));
            }
        }
        crate::state::notifications::NotificationDetail::Movement {
            armies,
            destination,
            cause,
        } => {
            values.insert(
                "army".into(),
                armies
                    .iter()
                    .map(|army| army.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if let Some(place) = destination {
                values.insert("place".into(), place.name.clone());
            }
            if let Some(cause) = cause {
                values.insert("reason".into(), cause.clone());
            }
        }
        crate::state::notifications::NotificationDetail::Battle { battle } => {
            values.insert("place".into(), battle.site.name.clone());
            values.insert("result".into(), format!("{:?}", battle.outcome));
            values.insert(
                "details".into(),
                format!(
                    "{} own forces withdrew and {} were destroyed",
                    battle.retreated.len(),
                    battle.destroyed_armies.len()
                ),
            );
        }
        crate::state::notifications::NotificationDetail::Diplomacy {
            faction,
            outcome_key,
            details,
        } => {
            values.insert("result".into(), outcome_key.replace('_', " "));
            if let Some((_, name)) = faction {
                values.insert("faction".into(), name.clone());
            }
            values.insert(
                "details".into(),
                details.values().cloned().collect::<Vec<_>>().join(", "),
            );
        }
        crate::state::notifications::NotificationDetail::Remembrance { subject, years, .. } => {
            values.insert("subject".into(), snapshot_label(subject));
            if let Some(years) = years {
                values.insert("years".into(), years.to_string());
            }
        }
        crate::state::notifications::NotificationDetail::Facts { values: facts } => {
            for (key, value) in facts {
                values.insert(key.clone(), value.clone());
            }
            if let Some(event) = facts.get("event") {
                values.insert("reason".into(), event.replace('_', " "));
            }
        }
    }
    values
        .entry("when".into())
        .or_insert_with(|| format!("season {}", receipt.completed_rounds));
    values
}

fn snapshot_label(snapshot: &NotificationSubjectSnapshot) -> String {
    match snapshot {
        NotificationSubjectSnapshot::Person(person) => person.name.clone(),
        NotificationSubjectSnapshot::Place(place) => place.name.clone(),
        NotificationSubjectSnapshot::Army(army) => army.name.clone(),
        NotificationSubjectSnapshot::Route(route) => {
            format!("{}–{}", route.from.name, route.to.name)
        }
        NotificationSubjectSnapshot::Construction(work) => format!("{:?}", work.kind),
        NotificationSubjectSnapshot::Battle(battle) => battle.site.name.clone(),
        NotificationSubjectSnapshot::Faction { name, .. } => name.clone(),
        NotificationSubjectSnapshot::History { label, .. } => label.clone(),
    }
}

fn substitute(template: &str, values: &BTreeMap<String, String>) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;
    while let Some(start) = remaining.find('{') {
        rendered.push_str(&remaining[..start]);
        let after_open = &remaining[start + 1..];
        let Some(end) = after_open.find('}') else {
            rendered.push_str(&remaining[start..]);
            return rendered;
        };
        let key = &after_open[..end];
        rendered.push_str(values.get(key).map_or("—", String::as_str));
        remaining = &after_open[end + 1..];
    }
    rendered.push_str(remaining);
    rendered
}
