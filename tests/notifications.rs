use kestrum::{
    data::{
        economy::Resources,
        threats::ThreatKind,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, ActionOutcome, MovementBlock, MovementOutcome, MovementStop},
    state::{
        campaign::{DomainFact, DomainFactKind, FactId},
        history::{AnniversarySubject, HistoryId, HistoryKind, HistoryRecord, LifeEvent},
        legacy::LegacyItemCustody,
        notifications::{
            NotificationDetail, NotificationId, NotificationKind, NotificationPreferences,
            NotificationPriority, NotificationReceipt, NotificationSourceId,
            NotificationSubjectSnapshot,
        },
        threat::{Threat, ThreatStatus},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn emergence_and_immediate_recognition_merge_into_one_durable_receipt() {
    let data = GameData::load().unwrap();
    let before = StrategicCampaign::new(&data).unwrap();
    let mut candidate = before.clone();
    candidate.accepted_sequence = 1;
    let player = candidate.player;
    let person = *candidate.people.keys().next().unwrap();
    let first = candidate.next_ids.history;
    let second = HistoryId(first.0 + 1);
    candidate.history.events.insert(
        first,
        record(
            first,
            player,
            HistoryKind::Life {
                owner: player,
                person,
                event: LifeEvent::Emerged {
                    troop: kestrum::data::economy::TroopKind::SiegeEngines,
                },
            },
        ),
    );
    candidate.history.events.insert(
        second,
        record(
            second,
            player,
            HistoryKind::Life {
                owner: player,
                person,
                event: LifeEvent::Recognized {
                    epithet: "The Bridge Keeper".into(),
                    cause: kestrum::data::progression::EpithetFact::DefendedAnchor,
                },
            },
        ),
    );
    candidate.next_ids.history = HistoryId(second.0 + 1);
    let outcome = outcome(&candidate, vec![first, second]);

    engine::notifications::collect(&before, &mut candidate, &data, &outcome).unwrap();
    let new_heroes: Vec<_> = candidate
        .notifications
        .receipts
        .iter()
        .filter(|receipt| receipt.kind == NotificationKind::NewHero)
        .collect();
    assert_eq!(new_heroes.len(), 1);
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::PersonRecognized)
            .count(),
        0
    );
    let NotificationDetail::Person { person, reason, .. } = &new_heroes[0].detail else {
        panic!("new hero notice must retain the event-time person snapshot");
    };
    assert_eq!(reason.as_deref(), Some("The Bridge Keeper"));
    assert!(person.age_years.is_some());
    assert!(person.appearance.is_some());

    let ids: Vec<_> = candidate
        .notifications
        .receipts
        .iter()
        .map(|receipt| receipt.id)
        .collect();
    engine::notifications::collect(&before, &mut candidate, &data, &outcome).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .map(|receipt| receipt.id)
            .collect::<Vec<_>>(),
        ids
    );
}

#[test]
fn repeated_anniversaries_and_custody_changes_keep_each_event_time_name() {
    let data = GameData::load().unwrap();
    let before = StrategicCampaign::new(&data).unwrap();
    let mut candidate = before.clone();
    candidate.accepted_sequence = 1;
    let player = candidate.player;
    let site = candidate.factions[&player].capital;
    let person = candidate
        .people
        .values()
        .find(|person| person.faction == player)
        .unwrap()
        .id;
    let item = candidate
        .legacy_items
        .values()
        .find(|item| item.faction == player)
        .unwrap()
        .clone();
    let first = candidate.next_ids.history;
    let second = HistoryId(first.0 + 1);
    let third = HistoryId(first.0 + 2);
    let fourth = HistoryId(first.0 + 3);
    for (id, years) in [(first, 4), (second, 8)] {
        candidate.history.events.insert(
            id,
            record(
                id,
                player,
                HistoryKind::Anniversary {
                    subject: AnniversarySubject::Site(site),
                    years,
                },
            ),
        );
    }
    for (id, from, to) in [
        (
            third,
            LegacyItemCustody::SiteEstate(site),
            LegacyItemCustody::Person(person),
        ),
        (
            fourth,
            LegacyItemCustody::Person(person),
            LegacyItemCustody::SiteEstate(site),
        ),
    ] {
        candidate.history.events.insert(
            id,
            record(
                id,
                player,
                HistoryKind::ItemCustodyChanged {
                    item: item.id,
                    from,
                    to,
                },
            ),
        );
    }
    candidate.next_ids.history = HistoryId(fourth.0 + 1);
    let mut outcome = outcome(&candidate, Vec::new());
    outcome.anniversary_reminders = vec![AnniversarySubject::Site(site)];
    outcome.legacy_items_changed = vec![item.id];

    engine::notifications::collect(&before, &mut candidate, &data, &outcome).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::Remembrance)
            .count(),
        2
    );
    let transfers: Vec<_> = candidate
        .notifications
        .receipts
        .iter()
        .filter(|receipt| receipt.kind == NotificationKind::LegacyTransfer)
        .collect();
    assert_eq!(transfers.len(), 2);
    let person_name = candidate.people[&person].name.as_str();
    let place_name = candidate.world.site(site).unwrap().name.as_str();
    let rendered = transfers
        .iter()
        .map(|receipt| {
            engine::notifications::project(&candidate, &data, &Default::default())
                .recent
                .into_iter()
                .find(|notice| notice.id == receipt.id)
                .unwrap()
                .body
        })
        .collect::<Vec<_>>();
    assert!(rendered.iter().any(|body| body.contains(person_name)));
    assert!(rendered.iter().any(|body| body.contains(place_name)));
    assert!(rendered.iter().all(|body| !body.contains("person_")));
    assert!(rendered.iter().all(|body| !body.contains("site_")));
}

#[test]
fn observer_collection_never_reveals_other_factions_event_records() {
    let data = GameData::load().unwrap();
    let before = StrategicCampaign::new_observer(&data).unwrap();
    let mut candidate = before.clone();
    candidate.accepted_sequence = 1;
    let player = candidate.player;
    let (other, person) = candidate
        .people
        .values()
        .find(|person| person.faction != player)
        .map(|person| (person.faction, person.id))
        .unwrap();
    let id = candidate.next_ids.history;
    candidate.history.events.insert(
        id,
        record(
            id,
            player,
            HistoryKind::Life {
                owner: other,
                person,
                event: LifeEvent::Emerged {
                    troop: kestrum::data::economy::TroopKind::Warriors,
                },
            },
        ),
    );
    let outcome = outcome(&candidate, vec![id]);
    engine::notifications::collect(&before, &mut candidate, &data, &outcome).unwrap();
    assert!(candidate.notifications.receipts.is_empty());
}

#[test]
fn preferences_hide_suppressed_backlog_and_projection_keeps_priority_overflow_and_copy() {
    let mut data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign.notifications.receipts = vec![
        receipt(
            1,
            NotificationKind::MovementArrived,
            NotificationPriority::Information,
            "old",
        ),
        receipt(
            2,
            NotificationKind::UnpaidUpkeepBegan,
            NotificationPriority::Urgent,
            "urgent",
        ),
        receipt(
            3,
            NotificationKind::PersonArrived,
            NotificationPriority::Information,
            "group-a",
        ),
        receipt(
            4,
            NotificationKind::PersonArrived,
            NotificationPriority::Information,
            "group-b",
        ),
        receipt(
            5,
            NotificationKind::LegacyTransfer,
            NotificationPriority::History,
            "city",
        ),
    ];
    campaign.notifications.next_id = 6;
    campaign.notifications.next_order = 6;
    let mut preferences = NotificationPreferences::default();
    preferences.all_off();
    campaign
        .notifications
        .sync_preferences(&preferences, &data.notifications);
    preferences.restore_defaults();
    campaign
        .notifications
        .sync_preferences(&preferences, &data.notifications);
    campaign.notifications.receipts.extend([
        receipt(
            6,
            NotificationKind::MovementArrived,
            NotificationPriority::Information,
            "new route",
        ),
        receipt(
            7,
            NotificationKind::UnpaidUpkeepBegan,
            NotificationPriority::Urgent,
            "new upkeep",
        ),
        receipt(
            8,
            NotificationKind::PersonArrived,
            NotificationPriority::Information,
            "group-a",
        ),
        receipt(
            9,
            NotificationKind::PersonArrived,
            NotificationPriority::Information,
            "group-b",
        ),
        receipt(
            10,
            NotificationKind::LegacyTransfer,
            NotificationPriority::History,
            "city",
        ),
    ]);
    campaign.notifications.next_id = 11;
    campaign.notifications.next_order = 11;
    let mut dismissed = receipt(
        11,
        NotificationKind::MovementBlocked,
        NotificationPriority::Warning,
        "dismissed",
    );
    dismissed.is_dismissed = true;
    campaign.notifications.receipts.push(dismissed);
    campaign.notifications.next_id = 12;
    campaign.notifications.next_order = 12;

    let complete = engine::notifications::project(&campaign, &data, &preferences);
    assert_eq!(complete.unread_count, 5);
    assert_eq!(complete.top_bar_unread_count, 4);
    assert!(complete.top_bar_unread_count > 0);
    assert!(complete.top_bar_unread_count < complete.unread_count);
    assert!(complete
        .recent
        .iter()
        .all(|notice| notice.id != NotificationId(1)));
    assert!(complete
        .recent
        .iter()
        .any(|notice| notice.id == NotificationId(11)));
    assert!(complete
        .rail
        .iter()
        .all(|group| !group.receipt_ids.contains(&NotificationId(11))));
    let grouped = complete
        .rail
        .iter()
        .find(|group| group.kind == NotificationKind::PersonArrived)
        .unwrap();
    assert_eq!(grouped.receipt_ids.len(), 2);
    let legacy = complete
        .recent
        .iter()
        .find(|notice| notice.id == NotificationId(10))
        .unwrap();
    assert!(legacy.body.contains("city"));

    data.notifications.max_rail_items = 1;
    let compact = engine::notifications::project(&campaign, &data, &preferences);
    assert_eq!(compact.rail[0].kind, NotificationKind::UnpaidUpkeepBegan);
    assert!(compact.rail_overflow_count > 0);
    assert_eq!(compact.top_bar_unread_count, complete.top_bar_unread_count);
    assert_eq!(compact.recent[0].kind, NotificationKind::UnpaidUpkeepBegan);

    let mut observer = campaign.clone();
    observer.observer_mode = true;
    assert_eq!(
        engine::notifications::project(&observer, &data, &preferences),
        Default::default()
    );
}

#[test]
fn known_local_threat_start_and_clear_are_dated_once() {
    let data = GameData::load().unwrap();
    let mut before = StrategicCampaign::new(&data).unwrap();
    engine::notifications::baseline_current_conditions(&mut before, &data).unwrap();
    let player = before.player;
    let army = before
        .armies
        .values()
        .find(|army| army.faction == player)
        .unwrap()
        .id;
    let site = before.armies[&army].site;
    let id = before.next_ids.threat;
    let mut candidate = before.clone();
    candidate.accepted_sequence += 1;
    candidate.threats.insert(
        id,
        Threat {
            id,
            site,
            kind: ThreatKind::Bandits,
            name: "Ashwood bandits".into(),
            headcount: 18,
            status: ThreatStatus::Active,
            ruination: None,
        },
    );
    let collected = outcome(&candidate, Vec::new());
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::LocalThreatStarted)
            .count(),
        1
    );

    before = candidate.clone();
    candidate = before.clone();
    candidate.accepted_sequence += 1;
    candidate.threats.get_mut(&id).unwrap().status = ThreatStatus::Cleared {
        round: candidate.completed_rounds,
        by: player,
        payout: Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
    };
    let collected = outcome(&candidate, Vec::new());
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::LocalThreatCleared)
            .count(),
        1
    );
    let projection = engine::notifications::project(&candidate, &data, &Default::default());
    assert!(projection
        .recent
        .iter()
        .any(|notice| notice.body.contains("Ashwood bandits")));

    let mut hidden = before.clone();
    hidden.accepted_sequence += 1;
    let distant = hidden
        .world
        .sites
        .iter()
        .find(|entry| entry.id != site && hidden.world.connected_route(site, entry.id).is_none())
        .unwrap()
        .id;
    hidden.armies.get_mut(&army).unwrap().site = distant;
    hidden.threats.get_mut(&id).unwrap().status = ThreatStatus::Cleared {
        round: hidden.completed_rounds,
        by: player,
        payout: Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
    };
    let collected = outcome(&hidden, Vec::new());
    engine::notifications::collect(&before, &mut hidden, &data, &collected).unwrap();
    assert!(!hidden
        .notifications
        .receipts
        .iter()
        .any(|receipt| receipt.kind == NotificationKind::LocalThreatCleared));
}

#[test]
fn reclamation_notifies_only_when_a_ruined_owned_place_returns() {
    let data = GameData::load().unwrap();
    let mut before = StrategicCampaign::new(&data).unwrap();
    let site = before.factions[&before.player].capital;
    let round = before.completed_rounds;
    let development = before.world.development.get_mut(&site).unwrap();
    development.ruined = true;
    development.ruination = 1;
    development.ruined_round = Some(round);
    let mut candidate = before.clone();
    candidate.accepted_sequence += 1;
    let development = candidate.world.development.get_mut(&site).unwrap();
    development.ruined = false;
    development.ruined_round = None;
    development.ruin_streak = 0;
    development.lawless_rounds = 0;
    candidate
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .unwrap()
        .habitation = kestrum::data::economy::Habitation::Outpost;
    let collected = outcome(&candidate, Vec::new());
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::SiteReclaimed)
            .count(),
        1
    );

    before = candidate.clone();
    candidate = before.clone();
    candidate.accepted_sequence += 1;
    let other = candidate
        .world
        .sites
        .iter()
        .find(|entry| entry.controller != Some(candidate.player))
        .unwrap()
        .id;
    candidate.world.development.get_mut(&other).unwrap().ruined = true;
    candidate
        .world
        .development
        .get_mut(&other)
        .unwrap()
        .ruination = 1;
    candidate
        .world
        .development
        .get_mut(&other)
        .unwrap()
        .ruined_round = Some(0);
    let collected = outcome(&candidate, Vec::new());
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::SiteReclaimed)
            .count(),
        1,
        "foreign ruin changes must not produce a player reclamation receipt"
    );
}

#[test]
fn movement_facts_match_one_exact_result_instead_of_reusing_every_army_overlap() {
    let data = GameData::load().unwrap();
    let before = StrategicCampaign::new(&data).unwrap();
    let mut candidate = before.clone();
    candidate.accepted_sequence = 1;
    let army = *candidate.armies.keys().next().unwrap();
    let origin = candidate.armies[&army].site;
    let destinations = candidate
        .world
        .sites
        .iter()
        .map(|site| site.id)
        .filter(|site| *site != origin)
        .take(2)
        .collect::<Vec<_>>();
    let make_result = |destination: SiteId, spent: u32| MovementOutcome {
        requested_destination: Some(destination),
        planned_destination: None,
        battle: None,
        armies: vec![army],
        path: vec![origin, destination],
        spent,
        stop: Some(MovementStop {
            site: destination,
            reason: MovementBlock::RouteUnavailable,
        }),
    };
    let first_path = vec![origin, destinations[0]];
    let second_path = vec![origin, destinations[1]];
    let first = make_result(destinations[0], 1);
    let second = make_result(destinations[1], 2);
    let round = candidate.completed_rounds;
    let faction = candidate.player;
    let fact = |id: u64, path: Vec<SiteId>, spent| DomainFact {
        id: FactId(id),
        sequence: 1,
        completed_rounds: round,
        kind: DomainFactKind::ArmiesMoved {
            faction,
            armies: vec![army],
            path,
            spent,
            movement: None,
        },
    };
    let facts = vec![fact(10, first_path, 1), fact(11, second_path, 2)];
    let outcome = ActionOutcome {
        continued_movements: vec![second],
        facts,
        movement: Some(first),
        ..outcome(&candidate, Vec::new())
    };
    engine::notifications::collect(&before, &mut candidate, &data, &outcome).unwrap();
    assert_eq!(
        candidate
            .notifications
            .receipts
            .iter()
            .filter(|receipt| receipt.kind == NotificationKind::MovementBlocked)
            .count(),
        2
    );
}

fn receipt(
    id: u64,
    kind: NotificationKind,
    priority: NotificationPriority,
    item: &str,
) -> NotificationReceipt {
    NotificationReceipt {
        id: NotificationId(id),
        source: NotificationSourceId::Fact { id: FactId(id) },
        kind,
        priority,
        completed_rounds: 0,
        occurred_sequence: 1,
        occurred_order: id,
        subject: Some(NotificationSubjectSnapshot::History {
            id: HistoryId(id),
            label: item.into(),
        }),
        detail: NotificationDetail::Facts {
            values: BTreeMap::from([
                ("item".into(), item.into()),
                ("from".into(), "west".into()),
                ("to".into(), "east".into()),
            ]),
        },
        is_read: false,
        is_dismissed: false,
        is_active: false,
        closed_round: None,
    }
}

fn outcome(campaign: &StrategicCampaign, life_events: Vec<HistoryId>) -> ActionOutcome {
    ActionOutcome {
        continued_movements: Vec::new(),
        life_events,
        automatic_retirements: Vec::new(),
        battle: None,
        battle_pending: false,
        accepted_sequence: campaign.accepted_sequence,
        active_faction: campaign.player,
        round_completed: false,
        facts: Vec::new(),
        consumed_facts: Vec::new(),
        recruited: None,
        disbanded: None,
        movement: None,
        split_army: None,
        succession: Vec::new(),
        new_people: Vec::new(),
        legacy_items_changed: Vec::new(),
        anniversary_reminders: Vec::new(),
    }
}

fn record(id: HistoryId, visible_to: FactionId, kind: HistoryKind) -> HistoryRecord {
    HistoryRecord {
        id,
        completed_rounds: 0,
        source_fact: None,
        kind,
        sites: Vec::new(),
        armies: Vec::new(),
        people: Vec::new(),
        formations: Vec::new(),
        items: Vec::new(),
        related_events: Vec::new(),
        visible_to: BTreeSet::from([visible_to]),
    }
}
