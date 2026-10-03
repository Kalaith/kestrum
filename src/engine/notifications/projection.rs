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
    if campaign.observer_mode {
        return NotificationProjection::default();
    }
    let rules = &data.notifications;
    let visible = campaign
        .notifications
        .receipts
        .iter()
        .filter(|receipt| {
            preferences.choice(receipt.kind, rules) != NotificationDelivery::Off
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
    recent
        .sort_by_key(|item| std::cmp::Reverse((priority_rank(item.priority), item.occurred_order)));
    let unread_count = visible
        .iter()
        .filter(|receipt| !receipt.is_dismissed && !receipt.is_read)
        .count();
    let mut rail = Vec::<NotificationGroup>::new();
    let mut top_bar = visible
        .into_iter()
        .filter(|receipt| {
            !receipt.is_dismissed
                && preferences.choice(receipt.kind, rules) == NotificationDelivery::TopBarAndHistory
                && receipt.id.0
                    > campaign
                        .notifications
                        .top_bar_after_id
                        .get(&receipt.kind)
                        .copied()
                        .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    top_bar.sort_by_key(|receipt| {
        std::cmp::Reverse((priority_rank(receipt.priority), receipt.occurred_order))
    });
    let top_bar_unread_count = top_bar.iter().filter(|receipt| !receipt.is_read).count();
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
        std::cmp::Reverse((
            priority_rank(group.priority),
            group.completed_rounds,
            group.receipt_ids[0],
        ))
    });
    let rail_overflow_count = rail.len().saturating_sub(rules.max_rail_items);
    rail.truncate(rules.max_rail_items);
    NotificationProjection {
        rail,
        rail_overflow_count,
        recent,
        unread_count,
        top_bar_unread_count,
        omitted_count: campaign.notifications.pruned_count,
        omitted_unread_count: campaign.notifications.pruned_unread_count,
    }
}

fn priority_rank(priority: crate::state::notifications::NotificationPriority) -> u8 {
    use crate::state::notifications::NotificationPriority::*;
    match priority {
        Urgent => 4,
        Warning => 3,
        Information => 2,
        History => 1,
    }
}

fn render(receipt: &NotificationReceipt, data: &GameData) -> RenderedNotification {
    let rule = data
        .notifications
        .kind(receipt.kind)
        .expect("validated notification kind");
    let values = detail_values(receipt, data);
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

fn detail_values(receipt: &NotificationReceipt, data: &GameData) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    if let Some(subject) = &receipt.subject {
        match subject {
            NotificationSubjectSnapshot::Person(person) => {
                values.insert("person".into(), person.name.clone());
                values.insert("class".into(), class_label(data, person.class));
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
                values.insert("work".into(), construction_label(data, work.kind));
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
            reason_key,
            reason,
            opportunities,
        } => {
            values.insert("person".into(), person.name.clone());
            if let Some(reason) = reason {
                let is_authored_term = matches!(
                    reason_key.as_deref(),
                    Some("emerged" | "arrived" | "class_completed")
                );
                values.insert(
                    "reason".into(),
                    if is_authored_term {
                        authored_term(data, reason)
                    } else {
                        reason.clone()
                    },
                );
            }
            values.insert("class".into(), class_label(data, person.class));
            values.insert(
                "opportunities".into(),
                opportunities
                    .iter()
                    .map(|class| class_label(data, *class))
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
                values.insert(
                    "before".into(),
                    place_fact_label(data, receipt.kind, before),
                );
            }
            if let Some(after) = after {
                values.insert("after".into(), place_fact_label(data, receipt.kind, after));
            }
            if let Some(cause) = cause {
                let label = authored_term(data, cause);
                values.insert("reason".into(), label.clone());
                values.insert("cause".into(), label);
            }
            if let Some(round) = forecast_round {
                values.insert("when".into(), format!("season {round}"));
            }
            values.insert("conditions".into(), authored_list(data, conditions));
        }
        crate::state::notifications::NotificationDetail::Work {
            work,
            cause,
            forecast_round,
            ..
        } => {
            values.insert("work".into(), construction_label(data, work.kind));
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
                values.insert("reason".into(), authored_term(data, cause));
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
                values.insert("reason".into(), authored_term(data, cause));
            }
        }
        crate::state::notifications::NotificationDetail::Battle { battle } => {
            values.insert("place".into(), battle.site.name.clone());
            values.insert(
                "result".into(),
                authored_term(data, battle_outcome_key(battle.outcome)),
            );
            values.insert("details".into(), battle_details(data, battle));
        }
        crate::state::notifications::NotificationDetail::Diplomacy {
            faction,
            outcome_key,
            details,
        } => {
            values.insert("result".into(), authored_term(data, outcome_key));
            if let Some((_, name)) = faction {
                values.insert("faction".into(), name.clone());
            }
            values.insert(
                "details".into(),
                details
                    .values()
                    .map(|value| authored_term(data, value))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        crate::state::notifications::NotificationDetail::Remembrance { subject, years, .. } => {
            values.insert("subject".into(), snapshot_label(subject, data));
            if let Some(years) = years {
                values.insert("years".into(), years.to_string());
            }
        }
        crate::state::notifications::NotificationDetail::Facts { values: facts } => {
            for (key, value) in facts {
                let value = if matches!(
                    key.as_str(),
                    "event" | "outcome" | "facility" | "before" | "after"
                ) {
                    authored_term(data, value)
                } else {
                    value.clone()
                };
                values.insert(key.clone(), value);
            }
            if let Some(event) = facts.get("event") {
                values.insert("reason".into(), authored_term(data, event));
            }
            if let Some(shortfall) = facts.get("shortfall") {
                values.insert("amount".into(), shortfall.clone());
            }
        }
    }
    values
        .entry("when".into())
        .or_insert_with(|| format!("season {}", receipt.completed_rounds));
    values
}

fn snapshot_label(snapshot: &NotificationSubjectSnapshot, data: &GameData) -> String {
    match snapshot {
        NotificationSubjectSnapshot::Person(person) => person.name.clone(),
        NotificationSubjectSnapshot::Place(place) => place.name.clone(),
        NotificationSubjectSnapshot::Army(army) => army.name.clone(),
        NotificationSubjectSnapshot::Route(route) => {
            format!("{}–{}", route.from.name, route.to.name)
        }
        NotificationSubjectSnapshot::Construction(work) => construction_label(data, work.kind),
        NotificationSubjectSnapshot::Battle(battle) => battle.site.name.clone(),
        NotificationSubjectSnapshot::Faction { name, .. } => name.clone(),
        NotificationSubjectSnapshot::History { label, .. } => label.clone(),
    }
}

fn place_fact_label(
    data: &GameData,
    kind: crate::state::notifications::NotificationKind,
    value: &str,
) -> String {
    use crate::state::notifications::NotificationKind;
    match kind {
        NotificationKind::GrowthApproaching
        | NotificationKind::DeclineRisk
        | NotificationKind::HabitationChanged
        | NotificationKind::SiteReclaimed
        | NotificationKind::ControlGained
        | NotificationKind::ControlLost
        | NotificationKind::ContestEntered
        | NotificationKind::ContestCleared => authored_term(data, value),
        NotificationKind::OccupationStarted | NotificationKind::OccupationCleared => value
            .strip_prefix("occupation_")
            .and_then(|count| count.parse::<u32>().ok())
            .map(|count| {
                substitute(
                    &authored_term(data, "ui_occupation_level"),
                    &BTreeMap::from([("count".into(), count.to_string())]),
                )
            })
            .unwrap_or_else(|| value.into()),
        _ => value.into(),
    }
}

fn authored_list(data: &GameData, values: &[String]) -> String {
    let separator = authored_term(data, "ui_list_separator");
    values
        .iter()
        .map(|value| authored_term(data, value))
        .collect::<Vec<_>>()
        .join(&separator)
}

fn battle_outcome_key(outcome: crate::state::battle::BattleOutcome) -> &'static str {
    use crate::state::battle::BattleOutcome;
    match outcome {
        BattleOutcome::AttackerVictory => "attacker_victory",
        BattleOutcome::DefenderVictory => "defender_victory",
        BattleOutcome::Stalemate => "stalemate",
        BattleOutcome::MutualDestruction => "mutual_destruction",
    }
}

fn battle_details(
    data: &GameData,
    battle: &crate::state::notifications::BattleNotificationSnapshot,
) -> String {
    let mut details = Vec::new();
    if !battle.retreated.is_empty() {
        let count = battle.retreated.len();
        let key = if count == 1 {
            "battle_one_force_withdrew"
        } else {
            "battle_many_forces_withdrew"
        };
        details.push(substitute(
            &authored_term(data, key),
            &BTreeMap::from([("count".into(), count.to_string())]),
        ));
    }
    if !battle.destroyed_armies.is_empty() {
        let count = battle.destroyed_armies.len();
        let key = if count == 1 {
            "battle_one_force_destroyed"
        } else {
            "battle_many_forces_destroyed"
        };
        details.push(substitute(
            &authored_term(data, key),
            &BTreeMap::from([("count".into(), count.to_string())]),
        ));
    }
    if details.is_empty() {
        authored_term(data, "battle_no_own_losses")
    } else {
        let separator = authored_term(data, "ui_detail_separator");
        details.join(&separator)
    }
}

fn authored_term(data: &GameData, key: &str) -> String {
    data.notifications
        .terms
        .get(key)
        .cloned()
        .unwrap_or_else(|| "Event details recorded".into())
}

fn class_label(data: &GameData, class: crate::data::world::PersonClass) -> String {
    let key = match class {
        crate::data::world::PersonClass::Recruit => "class_recruit",
        crate::data::world::PersonClass::Infantry => "class_infantry",
        crate::data::world::PersonClass::Archer => "class_archer",
        crate::data::world::PersonClass::Scout => "class_scout",
        crate::data::world::PersonClass::Cavalry => "class_cavalry",
        crate::data::world::PersonClass::Medic => "class_medic",
        crate::data::world::PersonClass::Officer => "class_officer",
    };
    authored_term(data, key)
}

fn construction_label(
    data: &GameData,
    kind: crate::state::construction::ConstructionKind,
) -> String {
    use crate::{data::world::Facility, state::construction::ConstructionKind};
    let key = match kind {
        ConstructionKind::Outpost => "construction_outpost",
        ConstructionKind::Road => "construction_road",
        ConstructionKind::RoadRepair => "construction_road_repair",
        ConstructionKind::Fort => "construction_fort",
        ConstructionKind::Facility(Facility::TrainingGround) => {
            "construction_facility_trainingground"
        }
        ConstructionKind::Facility(Facility::Stable) => "construction_facility_stable",
        ConstructionKind::Facility(Facility::Infirmary) => "construction_facility_infirmary",
        ConstructionKind::Facility(Facility::Workshop) => "construction_facility_workshop",
        ConstructionKind::Facility(Facility::Temple) => "construction_facility_temple",
    };
    authored_term(data, key)
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
