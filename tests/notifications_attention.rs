use kestrum::{
    data::GameData,
    engine,
    state::{
        campaign::FactId,
        history::HistoryId,
        notifications::{
            NotificationDetail, NotificationId, NotificationInbox, NotificationKind,
            NotificationPriority, NotificationReceipt, NotificationSourceId,
            NotificationSubjectSnapshot, WarningEpisode, WarningKey, WarningSubject,
        },
        StrategicCampaign,
    },
};
use std::collections::BTreeMap;

#[test]
fn active_site_risk_remains_attention_when_its_receipt_was_omitted() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let site = campaign.factions[&campaign.player].capital;
    campaign
        .notifications
        .warning_episodes
        .push(WarningEpisode {
            key: WarningKey {
                subject: WarningSubject::Site(site),
                kind: NotificationKind::RuinRisk,
            },
            next_episode: 2,
            active_receipt: None,
            is_present: true,
        });
    let visible = engine::project(&campaign, campaign.player).unwrap();
    let mut overview = engine::map_overview(&visible);
    engine::notifications::extend_attention(&campaign, &mut overview);
    assert!(overview.attention.iter().any(|attention| {
        attention.target == kestrum::engine::AttentionTarget::Site(site)
            && attention.kind == kestrum::engine::AttentionKind::RuinRisk
    }));
}

#[test]
fn single_dismissal_can_hide_a_warning_without_clearing_its_active_condition() {
    let mut inbox = NotificationInbox::default();
    let mut active = receipt(
        1,
        NotificationKind::GrowthApproaching,
        NotificationPriority::Warning,
        "warning",
    );
    active.source = NotificationSourceId::Warning {
        subject: kestrum::state::notifications::NotificationEntity::Site(
            kestrum::data::world::SiteId(1),
        ),
        kind: NotificationKind::GrowthApproaching,
        episode: 1,
    };
    active.is_active = true;
    let mut read = receipt(
        2,
        NotificationKind::MovementArrived,
        NotificationPriority::Information,
        "read",
    );
    read.is_read = true;
    let unread = receipt(
        3,
        NotificationKind::MovementArrived,
        NotificationPriority::Information,
        "unread",
    );
    inbox.receipts = vec![active, read, unread];

    assert!(inbox.dismiss(NotificationId(1)));
    assert!(inbox.receipt(NotificationId(1)).unwrap().is_active);
    assert!(inbox.dismiss_read());
    assert!(!inbox.receipt(NotificationId(2)).unwrap().is_active);
    assert!(inbox.receipt(NotificationId(2)).unwrap().is_dismissed);
    assert!(!inbox.receipt(NotificationId(3)).unwrap().is_dismissed);
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
