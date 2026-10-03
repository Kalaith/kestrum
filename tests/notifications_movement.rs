//! Accepted routes produce durable arrival and resumed-blocking receipts.

use kestrum::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, ActionOutcome, Actor, Command, MoveOrder},
    state::{
        military::ArmyId,
        notifications::{NotificationDetail, NotificationKind},
        Campaign, StrategicCampaign,
    },
};

fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn order(campaign: &mut StrategicCampaign, data: &GameData, path: &[u32]) -> ActionOutcome {
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: path.iter().copied().map(SiteId).collect(),
        }),
    )
    .unwrap()
}

fn season(campaign: &mut StrategicCampaign, data: &GameData) -> ActionOutcome {
    let round = campaign.completed_rounds;
    loop {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        let outcome = engine::apply(campaign, data, actor, Command::EndTurn).unwrap();
        if campaign.completed_rounds != round {
            return outcome;
        }
    }
}

#[test]
fn a_real_saved_route_notifies_when_the_requested_destination_is_reached() {
    let (data, mut campaign) = fixture();
    let accepted = order(&mut campaign, &data, &[1, 5, 6, 7, 14]);
    assert_eq!(
        accepted.movement.unwrap().requested_destination,
        Some(SiteId(14))
    );
    assert!(!campaign
        .notifications
        .receipts
        .iter()
        .any(|receipt| receipt.kind == NotificationKind::MovementArrived));

    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    let arrivals = campaign
        .notifications
        .receipts
        .iter()
        .filter(|receipt| receipt.kind == NotificationKind::MovementArrived)
        .collect::<Vec<_>>();
    assert_eq!(arrivals.len(), 1);
    let NotificationDetail::Movement { destination, .. } = &arrivals[0].detail else {
        panic!("arrival retains its event-time destination");
    };
    assert_eq!(destination.as_ref().unwrap().id, SiteId(14));
    campaign.validate(&data).unwrap();
    let restored: Campaign = serde_json::from_slice(
        &serde_json::to_vec(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    assert!(restored
        .strategic()
        .unwrap()
        .notifications
        .receipts
        .iter()
        .any(|receipt| receipt.kind == NotificationKind::MovementArrived));
}

#[test]
fn a_real_resumed_plan_blocked_by_new_peace_emits_one_durable_receipt() {
    let (data, mut campaign) = fixture();
    order(&mut campaign, &data, &[1, 5, 6, 7, 14]);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(7));
    assert_eq!(campaign.movement_plans[0].path, [SiteId(7), SiteId(14)]);
    campaign
        .set_site_control(&data, SiteId(14), Some(FactionId(2)), false)
        .unwrap();

    let resumed = season(&mut campaign, &data);
    assert_eq!(
        resumed.continued_movements[0].stop.as_ref().unwrap().site,
        SiteId(14)
    );
    assert!(campaign.movement_plans.is_empty());
    let blocked = campaign
        .notifications
        .receipts
        .iter()
        .filter(|receipt| receipt.kind == NotificationKind::MovementBlocked)
        .collect::<Vec<_>>();
    assert_eq!(blocked.len(), 1);
    let NotificationDetail::Movement {
        destination, cause, ..
    } = &blocked[0].detail
    else {
        panic!("blocked route retains its goal and reason");
    };
    assert_eq!(destination.as_ref().unwrap().id, SiteId(14));
    assert_eq!(cause.as_deref(), Some("movement_peaceboundary"));
    campaign.validate(&data).unwrap();
}
