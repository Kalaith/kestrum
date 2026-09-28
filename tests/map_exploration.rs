//! Discovery is earned by travel; the map and route picker use the same knowledge.

use kestrum::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, Actor, Command},
    navigation::{MapNavigation, MapSelection, MapView, MAP_RECT},
    state::{military::ArmyId, Campaign, StrategicCampaign},
};
use macroquad::prelude::vec2;

fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

#[test]
fn starting_map_and_picking_expose_only_the_home_neighborhood() {
    let (_, campaign) = fixture();
    let known = engine::explored_sites(&campaign, campaign.player);
    assert!(known.contains(&SiteId(1)) && known.contains(&SiteId(5)));
    assert!(!known.contains(&SiteId(7)));
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    assert!(visible
        .factions
        .iter()
        .all(|faction| engine::known_factions(&campaign, campaign.player).contains(&faction.id)));
    assert!(visible
        .world
        .sites
        .iter()
        .all(|site| known.contains(&site.id)));
    assert!(visible
        .world
        .routes
        .iter()
        .all(|route| known.contains(&route.from) && known.contains(&route.to)));
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    navigation
        .enter_region(
            &visible.world,
            campaign.world.site(SiteId(7)).unwrap().marker,
            &mut view,
        )
        .unwrap();
    assert!(navigation
        .select(&visible.world, MapSelection::Site(SiteId(7)))
        .is_err());
    assert!(navigation
        .targets(&visible.world, &view)
        .iter()
        .all(|target| target.selection != MapSelection::Site(SiteId(7))));
}

#[test]
fn preview_is_read_only_and_cannot_chart_unknown_routes() {
    let (data, campaign) = fixture();
    let before = campaign.clone();
    assert!(engine::map_movement_preview(
        &campaign,
        &data,
        campaign.player,
        &[ArmyId(1)],
        SiteId(7)
    )
    .is_err());
    let preview =
        engine::map_movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(5))
            .unwrap();
    assert_eq!(preview.order.path, [SiteId(1), SiteId(5)]);
    assert!(preview.total_cost > 0 && preview.reachable_steps == 1);
    assert_eq!(campaign, before);
}

#[test]
fn actual_travel_reveals_exits_and_discovery_survives_return_and_save() {
    let (data, mut campaign) = fixture();
    let other = engine::explored_sites(&campaign, FactionId(2));
    for destination in [SiteId(5), SiteId(6)] {
        let preview = engine::map_movement_preview(
            &campaign,
            &data,
            campaign.player,
            &[ArmyId(1)],
            destination,
        )
        .unwrap();
        engine::apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::Move(preview.order),
        )
        .unwrap();
    }
    let known = engine::explored_sites(&campaign, campaign.player);
    assert!(known.contains(&SiteId(7)));
    assert_eq!(engine::explored_sites(&campaign, FactionId(2)), other);
    let back =
        engine::map_movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(5))
            .unwrap();
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(back.order),
    )
    .unwrap();
    assert!(engine::explored_sites(&campaign, campaign.player).is_superset(&known));
    let saved = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let restored: Campaign = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        restored.strategic().unwrap().knowledge.explored,
        campaign.knowledge.explored
    );
    restored.strategic().unwrap().validate(&data).unwrap();
}

#[test]
fn rejected_moves_never_reveal_and_old_saves_keep_local_access() {
    let (data, mut campaign) = fixture();
    let before = campaign.clone();
    let order = engine::MoveOrder {
        armies: vec![ArmyId(1)],
        path: vec![SiteId(1), SiteId(7)],
    };
    assert!(engine::apply(&mut campaign, &data, Actor::Player, Command::Move(order)).is_err());
    assert_eq!(campaign, before);
    let mut raw = serde_json::to_value(&campaign).unwrap();
    raw["knowledge"].as_object_mut().unwrap().remove("explored");
    raw["knowledge"].as_object_mut().unwrap().remove("contacts");
    let loaded: Campaign = serde_json::from_value(raw).unwrap();
    let older = loaded.strategic().unwrap();
    assert_eq!(
        engine::explored_sites(older, older.player),
        engine::explored_sites(&campaign, campaign.player)
    );
    let mut malformed = campaign;
    malformed
        .knowledge
        .explored
        .entry(malformed.player)
        .or_default()
        .insert(SiteId(u32::MAX));
    assert!(malformed.validate(&data).is_err());
}

#[test]
fn home_camera_and_army_targets_stay_in_their_physical_scope() {
    let (_, campaign) = fixture();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    navigation.focus_site(&visible.world, SiteId(1), &mut view);
    assert!(view.camera.zoom() > 1.0);
    for factor in [1.0, 1.2, 0.7] {
        view.zoom(vec2(640.0, 360.0), factor);
        let targets = navigation.army_targets(&visible.world, &view, &visible.armies);
        let army = targets
            .iter()
            .find(|target| target.armies.contains(&ArmyId(1)))
            .unwrap();
        assert!(army.bounds.w >= 48.0 && army.bounds.h >= 48.0);
        assert!(MAP_RECT.contains(army.bounds.center()));
        assert!(army.bounds.contains(army.bounds.center()));
    }
    navigation.focus_site(&visible.world, SiteId(5), &mut view);
    assert_eq!(navigation.selection(), Some(MapSelection::Site(SiteId(5))));
    assert!(navigation
        .army_targets(&visible.world, &view, &visible.armies)
        .is_empty());
}
