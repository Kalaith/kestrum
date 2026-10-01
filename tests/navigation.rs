//! M01A cameras preserve world context and touch coordinates at the sole canvas.

use kestrum::{
    data::GameData,
    navigation::{MapNavigation, MapScaleBand, MapScope, MapView, HEIGHT, MAP_RECT, WIDTH},
    state::world::CampaignWorld,
};
use macroquad::prelude::vec2;
use macroquad_toolkit::input::gestures::TouchGestureFrame;

#[test]
fn working_camera_shows_part_of_the_world_and_pans_without_blank_edges() {
    let mut view = MapView::default();
    assert_eq!(view.extent(), vec2(WIDTH * 2.75, HEIGHT * 2.75));
    assert_eq!(view.band(), MapScaleBand::Campaign);
    assert_eq!(
        view.project_normalized([0.5, 0.5]),
        vec2(WIDTH * 0.5, HEIGHT * 0.5)
    );
    for delta in [vec2(-9999.0, 9999.0), vec2(9999.0, -9999.0)] {
        view.pan(delta);
        let top_left = view.project_normalized([0.0, 0.0]);
        let bottom_right = view.project_normalized([1.0, 1.0]);
        assert!(top_left.x <= 0.001 && top_left.y <= 0.001);
        assert!(bottom_right.x >= WIDTH - 0.001 && bottom_right.y >= HEIGHT - 0.001);
    }
    view.zoom(MAP_RECT.center(), 0.0001);
    view.pan(vec2(9000.0, -8000.0));
    assert!(view.project_normalized([0.0, 0.0]).length() < 0.001);
    assert!(
        view.project_normalized([1.0, 1.0])
            .distance(vec2(WIDTH, HEIGHT))
            < 0.001
    );
}

#[test]
fn anchor_and_touch_camera_operations_share_the_same_bounds() {
    let mut view = MapView::default();
    let anchor = vec2(1100.0, 600.0);
    let world = view.camera.screen_to_world(MAP_RECT, anchor).unwrap();
    view.zoom(anchor, 2.0);
    assert!(view.project(world).distance(anchor) < 0.001);
    let before = view;
    view.gesture(&TouchGestureFrame {
        pan: vec2(100.0, 50.0),
        scale: 2.0,
        ..Default::default()
    });
    assert_eq!(view, before);
    view.gesture(&TouchGestureFrame {
        claimed: true,
        scale: 50.0,
        center: anchor,
        ..Default::default()
    });
    assert_eq!(view.camera.zoom(), view.zoom_limits().1);
    view.reset();
    assert_eq!(view.camera.zoom(), 1.0);
    assert_eq!(view.band(), MapScaleBand::Campaign);
}

#[test]
fn scale_bands_use_separate_entry_and_exit_thresholds() {
    let mut view = MapView::default();
    for (zoom, band) in [
        (0.67, MapScaleBand::Overview),
        (0.73, MapScaleBand::Overview),
        (0.79, MapScaleBand::Campaign),
        (1.46, MapScaleBand::Detail),
        (1.36, MapScaleBand::Detail),
        (1.29, MapScaleBand::Campaign),
    ] {
        view.zoom(MAP_RECT.center(), zoom / view.camera.zoom());
        assert_eq!(view.band(), band);
    }
}

#[test]
fn overview_restores_exact_working_context_independently_in_each_scope() {
    let data = GameData::load().unwrap();
    let world = CampaignWorld::from_scenario(&data.scenario);
    let mut navigation = MapNavigation::configured(&data.presentation.map.camera);
    let mut view = MapView::configured(&data.presentation.map.camera, MapScope::World);
    view.focus([0.4, 0.55], 1.2);
    let working = view;
    navigation.toggle_overview(&world, &mut view);
    assert_eq!(view.band(), MapScaleBand::Overview);
    assert!(view.overview_active());
    let overview = view;
    let region = kestrum::data::world::MarkerId(5);
    navigation.enter_region(&world, region, &mut view).unwrap();
    assert_eq!(view.extent(), vec2(WIDTH * 1.75, HEIGHT * 1.75));
    view.focus([0.6, 0.5], 1.1);
    let regional = view;
    navigation.toggle_overview(&world, &mut view);
    navigation.show_world(&mut view);
    assert_eq!(view, overview);
    navigation.toggle_overview(&world, &mut view);
    assert_eq!(view, working);
    navigation.enter_region(&world, region, &mut view).unwrap();
    navigation.toggle_overview(&world, &mut view);
    assert_eq!(view, regional);
    navigation.reset(&mut view);
    assert_eq!(navigation.scope(), MapScope::World);
    assert_eq!(view.extent(), vec2(WIDTH * 2.75, HEIGHT * 2.75));
}

#[test]
fn data_rejects_invalid_camera_extents_and_unordered_scale_bands() {
    let data = GameData::load().unwrap();
    for invalid in 0..7 {
        let mut map = data.presentation.map.clone();
        match invalid {
            0 => map.camera.world_extent[0] = f32::NAN,
            1 => map.camera.region_extent = [1000.0, 800.0],
            2 => map.camera.overview_exit = map.camera.overview_enter,
            3 => map.camera.detail_enter = map.camera.maximum_zoom,
            4 => map.camera.army_group_distance = 10.0,
            5 => map.camera.world_extent = [2560.0, 1440.0],
            _ => map.camera.region_extent = [2560.0, 1440.0],
        }
        assert!(map.validate().is_err());
    }
}
