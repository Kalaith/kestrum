//! World markers issue physical orders while army inspection keeps its map scope.

use kestrum::{
    data::{
        world::{FactionId, MarkerId, SiteId},
        GameData,
    },
    engine::{self, Actor, Command, MoveOrder, MovementBlock},
    navigation::{MapNavigation, MapScope, MapSelection, MapView},
    state::{military::ArmyId, people::PersonId, StrategicCampaign},
};

fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn enter_interior(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: [1, 5, 6].map(SiteId).to_vec(),
        }),
    )
    .unwrap();
}

fn renew_movement(campaign: &mut StrategicCampaign) {
    for formation in campaign.formations.values_mut() {
        formation.movement_spent = 0;
    }
    for person in campaign.people.values_mut() {
        person.movement_spent = 0;
    }
}

#[test]
fn world_region_destination_uses_a_known_physical_entrance() {
    let (data, campaign) = fixture();
    let before = campaign.clone();
    let preview = engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(5),
    )
    .unwrap();
    assert_eq!(preview.order.path, [SiteId(1), SiteId(5)]);
    assert_eq!((preview.total_cost, preview.reachable_steps), (2, 1));
    assert_eq!(campaign, before);
}

#[test]
fn leaving_a_region_charges_internal_travel_before_the_world_edge() {
    let (data, mut campaign) = fixture();
    enter_interior(&mut campaign, &data);
    renew_movement(&mut campaign);
    let preview = engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(1),
    )
    .unwrap();
    assert_eq!(preview.order.path, [6, 5, 1].map(SiteId));
    assert_eq!(
        preview
            .steps
            .iter()
            .map(|step| step.cost)
            .collect::<Vec<_>>(),
        [2, 2]
    );
    assert_eq!(preview.total_cost, 4);
    let result = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(preview.order),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(result.spent, 4);
    assert_eq!(result.path, [6, 5, 1].map(SiteId));
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(1));
    assert_eq!(campaign.people[&PersonId(1)].movement_spent, 4);
    assert!(campaign.armies[&ArmyId(1)]
        .formation_ids()
        .all(|id| campaign.formations[&id].movement_spent == 4));
    campaign.validate(&data).unwrap();
}

#[test]
fn insufficient_movement_stops_at_the_gate_without_crossing_the_world_edge() {
    let (data, mut campaign) = fixture();
    enter_interior(&mut campaign, &data);
    let preview = engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(1),
    )
    .unwrap();
    assert_eq!(
        (
            preview.total_cost,
            preview.remaining,
            preview.reachable_steps
        ),
        (4, 2, 1)
    );
    assert_eq!(preview.reachable_site, SiteId(5));
    assert!(matches!(
        preview.stop.as_ref().unwrap().reason,
        MovementBlock::InsufficientMovement {
            required: 2,
            remaining: 0
        }
    ));
    let result = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(preview.order),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(result.path, [SiteId(6), SiteId(5)]);
    assert_eq!(result.spent, 2);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(5));
}

#[test]
fn world_orders_preserve_peace_boundaries_and_hidden_geography() {
    let (data, mut campaign) = fixture();
    enter_interior(&mut campaign, &data);
    renew_movement(&mut campaign);
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(2)), false)
        .unwrap();
    let before = campaign.clone();
    let preview = engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(1),
    )
    .unwrap();
    assert_eq!(preview.reachable_steps, 0);
    assert_eq!(preview.stop.unwrap().reason, MovementBlock::PeaceBoundary);
    assert!(engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(preview.order)
    )
    .is_err());
    assert_eq!(campaign, before);
    assert!(engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(3)
    )
    .is_err());
    assert_eq!(campaign, before);
}

#[test]
fn recenter_and_banner_selection_keep_world_and_explicit_region_scopes() {
    let (data, mut campaign) = fixture();
    enter_interior(&mut campaign, &data);
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    navigation.focus_army_site(&visible.world, SiteId(6), &mut view);
    assert_eq!(navigation.scope(), MapScope::World);
    assert_eq!(
        navigation.selection(),
        Some(MapSelection::Marker(MarkerId(5)))
    );
    assert!(navigation
        .army_targets(&visible.world, &view, &visible.armies)
        .iter()
        .any(|target| target.armies.contains(&ArmyId(1))));
    navigation
        .enter_region(&visible.world, MarkerId(5), &mut view)
        .unwrap();
    navigation.focus_army_site(&visible.world, SiteId(6), &mut view);
    assert_eq!(navigation.scope(), MapScope::Region(MarkerId(5)));
    assert_eq!(navigation.selection(), Some(MapSelection::Site(SiteId(6))));
    navigation.focus_army_site(&visible.world, SiteId(1), &mut view);
    assert_eq!(navigation.scope(), MapScope::World);
}

#[test]
fn world_banner_cycles_armies_at_different_sites_but_region_groups_stay_local() {
    let (data, mut campaign) = fixture();
    enter_interior(&mut campaign, &data);
    renew_movement(&mut campaign);
    let formation = campaign.armies[&ArmyId(1)].formation_ids().nth(1).unwrap();
    let other = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy { formation },
    )
    .unwrap()
    .split_army
    .unwrap();
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![other],
            path: vec![SiteId(6), SiteId(5)],
        }),
    )
    .unwrap();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let mut navigation = MapNavigation::default();
    assert_eq!(
        navigation.armies_at_selection(&visible.world, SiteId(6), &visible.armies),
        [ArmyId(1), other]
    );
    assert!(engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1), other],
        MarkerId(1)
    )
    .is_err());
    navigation
        .enter_region(&visible.world, MarkerId(5), &mut MapView::default())
        .unwrap();
    assert_eq!(
        navigation.armies_at_selection(&visible.world, SiteId(6), &visible.armies),
        [ArmyId(1)]
    );
    assert!(engine::world_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        MarkerId(5)
    )
    .is_err());
}
