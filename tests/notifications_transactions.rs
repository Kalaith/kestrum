use kestrum::{
    data::GameData,
    engine::{self, Actor, Command},
    state::{notifications::NotificationKind, StrategicCampaign},
};

#[test]
fn accepted_and_rejected_engine_commands_commit_only_real_event_receipts() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let original = campaign.clone();
    let player = campaign.player;
    let rival = *campaign.factions.keys().find(|id| **id != player).unwrap();

    let accepted = Command::DeclareWar { faction: rival };
    assert!(engine::preview(&campaign, &data, Actor::Player, accepted.clone()).is_ok());
    assert_eq!(
        campaign, original,
        "preview must not commit its candidate inbox"
    );
    assert!(campaign.notifications.receipts.is_empty());

    let rejected = Command::DeclareWar { faction: player };
    assert!(engine::preview(&campaign, &data, Actor::Player, rejected.clone()).is_err());
    assert!(engine::apply(&mut campaign, &data, Actor::Player, rejected).is_err());
    assert_eq!(
        campaign, original,
        "rejected commands must not commit receipts"
    );

    engine::apply(&mut campaign, &data, Actor::Player, accepted).unwrap();
    assert!(campaign
        .notifications
        .receipts
        .iter()
        .any(|receipt| receipt.kind == NotificationKind::DiplomacyOutcome));
    campaign.validate(&data).unwrap();
    let restored: StrategicCampaign =
        serde_json::from_slice(&serde_json::to_vec(&campaign).unwrap()).unwrap();
    assert_eq!(restored, campaign);
}
