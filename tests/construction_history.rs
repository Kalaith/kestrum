//! Construction receipts remain factual through shared boundaries and forgetting.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

use kestrum::{
    data::{
        world::{Facility, FactionId, RouteId, SiteId},
        GameData,
    },
    engine::{apply, history_page, Actor, Command, HistoryFilter, MoveOrder},
    state::{
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget},
        history::{HistoryKind, HistoryKindFilter, HistorySubject},
        military::ArmyId,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/construction_history.rs"]
mod support;
use support::*;

#[test]
fn simultaneous_completions_enter_the_same_history_budget_once() {
    let mut data = GameData::load().unwrap();
    data.history.detail_max_entries = 1;
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    begin_stable_and_road(&mut campaign, &data);
    finish(&mut campaign, &data);
    assert_eq!(campaign.history.events.len(), 1);
    assert!(campaign
        .construction
        .values()
        .all(|order| order.progress == 1));
    campaign = reload(&campaign, &data);
    finish(&mut campaign, &data);
    assert_eq!(campaign.history.events.len(), 1);
    assert_eq!(campaign.construction.len(), 2);
    assert!(campaign.construction.values().all(|order| matches!(
        order.status,
        ConstructionStatus::Completed {
            completed_rounds: 1
        }
    )));
    let road = campaign.world.route(RouteId(13)).unwrap();
    assert!(road.road.improved);
    assert!(campaign
        .world
        .site(SiteId(1))
        .unwrap()
        .facilities
        .contains(&Facility::Stable));
    let completed: Vec<_> = campaign.history.site_notables[&SiteId(1)]
        .iter()
        .filter(|entry| matches!(entry.kind, HistoryKind::Construction { .. }))
        .collect();
    assert_eq!(completed.len(), 2);
    let ids: Vec<_> = completed.iter().map(|entry| entry.id).collect();
    campaign = reload(&campaign, &data);
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.history.site_notables[&SiteId(1)]
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        ids
    );
}

#[test]
fn one_road_receipt_has_two_place_views_and_keeps_private_builder_labels() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    begin_stable_and_road(&mut campaign, &data);
    let west = records(&campaign, FactionId(1), SiteId(5));
    let headquarters = records(&campaign, FactionId(1), SiteId(1));
    assert_eq!(west.len(), 1);
    assert!(headquarters.iter().any(|record| record.id == west[0].id));
    assert_eq!(
        west[0]
            .sites
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![SiteId(1), SiteId(5)]
    );
    assert_eq!(west[0].armies[0].name, "Rose Ward");
    assert!(records(&campaign, FactionId(2), SiteId(5)).is_empty());
    campaign.armies.get_mut(&ArmyId(1)).unwrap().name = "Later army label".into();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap()
        .name = "Later gate label".into();
    assert_eq!(records(&campaign, FactionId(1), SiteId(5)), west);
    assert_eq!(reload(&campaign, &data), campaign);
}

#[test]
fn forgotten_construction_cannot_reopen_paid_work_or_forge_its_owner() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    begin_stable_and_road(&mut campaign, &data);
    finish(&mut campaign, &data);
    finish(&mut campaign, &data);
    let record_id = campaign
        .history
        .events
        .iter()
        .find(|(_, event)| matches!(event.kind, HistoryKind::Construction { .. }))
        .unwrap()
        .0
        .to_owned();
    let mut corrupt = campaign.clone();
    corrupt
        .history
        .events
        .get_mut(&record_id)
        .unwrap()
        .visible_to = [FactionId(2)].into_iter().collect();
    assert!(corrupt.validate(&data).is_err());
    let paid = campaign.construction.clone();
    let previous_events: Vec<_> = campaign.history.events.keys().copied().collect();
    for _ in 0..41 {
        finish(&mut campaign, &data);
    }
    assert!(previous_events
        .iter()
        .all(|id| !campaign.history.events.contains_key(id)));
    assert!(campaign
        .history
        .events
        .values()
        .all(|event| !matches!(event.kind, HistoryKind::Construction { .. })));
    assert_eq!(campaign.construction, paid);
    campaign = reload(&campaign, &data);
    let before = campaign.clone();
    let order = *campaign.construction.keys().next().unwrap();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelConstruction { order }
    )
    .is_err());
    assert_eq!(campaign, before);
}
