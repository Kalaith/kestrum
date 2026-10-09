use kestrum::{
    data::{economy::Resources, threats::ThreatKind, GameData},
    engine::{self, ActionOutcome},
    state::{
        construction::{
            ConstructionKind, ConstructionOrder, ConstructionStatus, ConstructionTarget, OrderId,
        },
        military::ArmyId,
        notifications::{NotificationKind, NotificationSourceId, WarningSubject},
        threat::{Threat, ThreatStatus},
        StrategicCampaign,
    },
};

#[test]
fn active_warning_budget_and_episode_ids_survive_clear_and_save_reload() {
    let mut data = GameData::load().unwrap();
    data.notifications.max_active_warnings = 1;
    let mut before = StrategicCampaign::new(&data).unwrap();
    engine::notifications::baseline_current_conditions(&mut before, &data).unwrap();
    let mut candidate = before.clone();
    candidate.accepted_sequence = 1;
    let owner = candidate.player;
    let site = candidate.factions[&owner].headquarters;
    for (id, kind) in [
        (
            OrderId(100),
            ConstructionKind::Facility(kestrum::data::world::Facility::Stable),
        ),
        (
            OrderId(101),
            ConstructionKind::Facility(kestrum::data::world::Facility::Workshop),
        ),
    ] {
        candidate.construction.insert(
            id,
            ConstructionOrder {
                id,
                owner,
                kind,
                target: ConstructionTarget::Site(site),
                builder: Some(ArmyId(1)),
                created_round: candidate.completed_rounds,
                progress: 0,
                required_steps: 1,
                paid: Resources {
                    gold: 0,
                    wood: 0,
                    stone: 0,
                },
                status: ConstructionStatus::Active,
                last_progress_round: None,
            },
        );
    }
    candidate.next_ids.order = OrderId(102);
    let collected = outcome(&candidate);
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    let active: Vec<_> = candidate
        .notifications
        .receipts
        .iter()
        .filter(|receipt| receipt.is_active)
        .collect();
    assert_eq!(active.len(), 1);
    assert!(construction_warning(&active[0].source, OrderId(100), 1));
    assert_eq!(candidate.notifications.pruned_unread_count, 1);
    let omitted = candidate
        .notifications
        .warning_episodes
        .iter()
        .find(|episode| {
            episode.key.subject == WarningSubject::Construction(OrderId(101))
                && episode.key.kind == NotificationKind::ConstructionExpected
        })
        .unwrap();
    assert!(omitted.is_present);
    assert_eq!(omitted.next_episode, 2);
    assert!(omitted.active_receipt.is_none());
    candidate
        .notifications
        .validate(&data.notifications)
        .unwrap();

    before = candidate.clone();
    candidate = before.clone();
    candidate.accepted_sequence = 2;
    candidate.completed_rounds += 1;
    let threat_id = candidate.next_ids.threat;
    candidate.threats.insert(
        threat_id,
        Threat {
            id: threat_id,
            site,
            kind: ThreatKind::Bandits,
            name: "Capital road raiders".into(),
            headcount: 12,
            status: ThreatStatus::Active,
            ruination: None,
            raid: None,
        },
    );
    let collected = outcome(&candidate);
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    let ended_episode = candidate
        .notifications
        .warning_episodes
        .iter()
        .find(|episode| {
            episode.key.subject == WarningSubject::Construction(OrderId(100))
                && episode.key.kind == NotificationKind::ConstructionExpected
        })
        .unwrap();
    assert!(!ended_episode.is_present);
    assert_eq!(ended_episode.next_episode, 2);
    let ended_omitted = candidate
        .notifications
        .warning_episodes
        .iter()
        .find(|episode| {
            episode.key.subject == WarningSubject::Construction(OrderId(101))
                && episode.key.kind == NotificationKind::ConstructionExpected
        })
        .unwrap();
    assert!(!ended_omitted.is_present);
    assert_eq!(ended_omitted.next_episode, 2);
    let first_receipt = candidate
        .notifications
        .receipts
        .iter()
        .find(|receipt| construction_warning(&receipt.source, OrderId(100), 1))
        .unwrap();
    assert!(!first_receipt.is_active);

    before = candidate.clone();
    candidate = before.clone();
    candidate.accepted_sequence = 3;
    candidate.threats.remove(&threat_id);
    let collected = outcome(&candidate);
    engine::notifications::collect(&before, &mut candidate, &data, &collected).unwrap();
    let renewed = candidate
        .notifications
        .receipts
        .iter()
        .find(|receipt| construction_warning(&receipt.source, OrderId(100), 2))
        .unwrap();
    assert!(renewed.is_active);
    let omitted_again = candidate
        .notifications
        .warning_episodes
        .iter()
        .find(|episode| {
            episode.key.subject == WarningSubject::Construction(OrderId(101))
                && episode.key.kind == NotificationKind::ConstructionExpected
        })
        .unwrap();
    assert!(omitted_again.is_present);
    assert_eq!(omitted_again.next_episode, 3);
    assert!(omitted_again.active_receipt.is_none());
    candidate
        .notifications
        .validate(&data.notifications)
        .unwrap();
    let restored: StrategicCampaign =
        serde_json::from_slice(&serde_json::to_vec(&candidate).unwrap()).unwrap();
    assert_eq!(restored.notifications, candidate.notifications);
}

fn construction_warning(source: &NotificationSourceId, order: OrderId, episode: u32) -> bool {
    matches!(
        source,
        NotificationSourceId::Warning {
            subject: kestrum::state::notifications::NotificationEntity::Construction(id),
            kind: NotificationKind::ConstructionExpected,
            episode: number,
        } if *id == order && *number == episode
    )
}

fn outcome(campaign: &StrategicCampaign) -> ActionOutcome {
    ActionOutcome {
        continued_movements: Vec::new(),
        life_events: Vec::new(),
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
