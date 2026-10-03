//! Small, validated notification receipts used by hidden capture scenarios.

use super::*;
use kestrum::state::notifications::{
    NotificationEntity, NotificationPriority, NotificationReceipt, NotificationSourceId,
    PersonNotificationSnapshot, WarningEpisode, WarningKey, WarningSubject,
};
use std::collections::BTreeMap;

pub(super) fn person_snapshot(
    campaign: &kestrum::state::StrategicCampaign,
) -> PersonNotificationSnapshot {
    let person = campaign
        .people
        .values()
        .find(|person| {
            person.faction == campaign.player
                && person.is_alive()
                && engine::person_site(campaign, person.id).is_some()
        })
        .expect("capture player has a living person at a site");
    person_snapshot_for(campaign, person.id)
}

pub(super) fn person_snapshot_for(
    campaign: &kestrum::state::StrategicCampaign,
    id: PersonId,
) -> PersonNotificationSnapshot {
    let person = &campaign.people[&id];
    let site = engine::person_site(campaign, id).and_then(|site| campaign.world.site(site));
    PersonNotificationSnapshot {
        id,
        faction: person.faction,
        name: person.name.clone(),
        age_years: Some(person.age_years(campaign.completed_rounds)),
        class: person.class,
        site: site.map(|site| PlaceNotificationSnapshot {
            id: site.id,
            name: site.name.clone(),
            controller: site.controller,
            habitation: site.habitation,
        }),
        army: None,
        appearance: Some(person.appearance.clone()),
    }
}

pub(super) fn place_snapshot(
    campaign: &kestrum::state::StrategicCampaign,
    id: SiteId,
) -> PlaceNotificationSnapshot {
    let site = campaign.world.site(id).expect("capture place exists");
    PlaceNotificationSnapshot {
        id,
        name: site.name.clone(),
        controller: site.controller,
        habitation: site.habitation,
    }
}

pub(super) fn facts_detail(person: &str, place: &str) -> NotificationDetail {
    let mut values = BTreeMap::from([
        ("person".into(), person.into()),
        ("class".into(), "Officer".into()),
        ("place".into(), place.into()),
        ("army".into(), "Rose Wardens".into()),
        ("reason".into(), "a winter patrol".into()),
        ("before".into(), "village".into()),
        ("after".into(), "town".into()),
        ("when".into(), "next season".into()),
        ("conditions".into(), "known winter pressure".into()),
        ("faction".into(), "Rose March".into()),
        ("result".into(), "peace agreed".into()),
        ("years".into(), "12".into()),
        ("subject".into(), person.into()),
    ]);
    values.insert("event".into(), "habitation_changed".into());
    NotificationDetail::Facts { values }
}

pub(super) fn append_receipt(
    campaign: &mut kestrum::state::StrategicCampaign,
    data: &GameData,
    kind: NotificationKind,
    ordinal: u32,
    subject: Option<NotificationSubjectSnapshot>,
    detail: NotificationDetail,
    active: bool,
) -> NotificationId {
    let id = NotificationId(campaign.notifications.next_id);
    let rule = data
        .notifications
        .kind(kind)
        .expect("capture notification rule");
    let entity = subject
        .as_ref()
        .map(|subject| match subject {
            NotificationSubjectSnapshot::Person(person) => NotificationEntity::Person(person.id),
            NotificationSubjectSnapshot::Place(place) => NotificationEntity::Site(place.id),
            _ => NotificationEntity::Faction(campaign.player),
        })
        .unwrap_or(NotificationEntity::Faction(campaign.player));
    campaign.notifications.receipts.push(NotificationReceipt {
        id,
        source: NotificationSourceId::Transition {
            accepted_sequence: campaign.accepted_sequence.max(1),
            kind,
            subject: entity,
            ordinal,
        },
        kind,
        priority: rule.default_priority,
        completed_rounds: campaign.completed_rounds,
        occurred_sequence: campaign.accepted_sequence.max(1),
        occurred_order: campaign.notifications.next_order,
        subject,
        detail,
        is_read: false,
        is_dismissed: false,
        is_active: active,
        closed_round: None,
    });
    campaign.notifications.next_id += 1;
    campaign.notifications.next_order += 1;
    id
}

pub(super) fn append_warning(
    campaign: &mut kestrum::state::StrategicCampaign,
    data: &GameData,
    place: &PlaceNotificationSnapshot,
) -> NotificationId {
    let kind = NotificationKind::RuinRisk;
    let id = NotificationId(campaign.notifications.next_id);
    let receipt = NotificationReceipt {
        id,
        source: NotificationSourceId::Warning {
            subject: NotificationEntity::Site(place.id),
            kind,
            episode: 1,
        },
        kind,
        priority: NotificationPriority::Warning,
        completed_rounds: campaign.completed_rounds,
        occurred_sequence: campaign.accepted_sequence.max(1),
        occurred_order: campaign.notifications.next_order,
        subject: Some(NotificationSubjectSnapshot::Place(place.clone())),
        detail: NotificationDetail::Place {
            place: place.clone(),
            before: Some("town".into()),
            after: Some("ruined".into()),
            cause: Some("ruin_condition_reaches_final_step".into()),
            forecast_round: Some(campaign.completed_rounds.saturating_add(1)),
            conditions: vec!["ruin_condition_reaches_final_step".into()],
        },
        is_read: false,
        is_dismissed: false,
        is_active: true,
        closed_round: None,
    };
    debug_assert!(data.notifications.kind(kind).is_some());
    campaign.notifications.receipts.push(receipt);
    campaign
        .notifications
        .warning_episodes
        .push(WarningEpisode {
            key: WarningKey {
                subject: WarningSubject::Site(place.id),
                kind,
            },
            next_episode: 2,
            active_receipt: Some(id),
            is_present: true,
        });
    campaign.notifications.next_id += 1;
    campaign.notifications.next_order += 1;
    id
}
