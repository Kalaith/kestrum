//! K04 case five: both map scopes share picking, camera and viewport coordinates.

use kestrum::{
    data::{
        world::{MarkerId, MarkerLocation, SiteId},
        GameData,
    },
    navigation::{MapNavigation, MapScope, MapSelection, MapView, HEIGHT, MAP_TAP_SIZE, WIDTH},
    state::world::CampaignWorld,
};
use macroquad::prelude::{vec2, Vec2};
use macroquad_toolkit::ui::VirtualUi;

#[test]
fn world_and_region_picking_stays_consistent_after_zoom_resize_and_return() {
    let data = GameData::load().unwrap();
    let world = CampaignWorld::from_scenario(&data.scenario);
    let original_world = world.clone();
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    let region = MarkerId(5);

    assert_eq!(navigation.scope(), MapScope::World);
    assert_eq!(navigation.targets(&world, &view).len(), world.markers.len());
    assert_scope_coordinates(&navigation, &world);
    navigation
        .select(&world, MapSelection::Marker(region))
        .unwrap();
    assert!(navigation
        .select(&world, MapSelection::Site(SiteId(5)))
        .is_err());
    assert!(navigation
        .enter_region(&world, MarkerId(1), &mut view)
        .is_err());
    assert!(navigation
        .enter_region(&world, MarkerId(u32::MAX), &mut view)
        .is_err());
    assert_eq!(navigation.selection(), Some(MapSelection::Marker(region)));
    assert_eq!(navigation.scope(), MapScope::World);

    view.zoom(vec2(600.0, 320.0), 2.0);
    view.pan(vec2(90.0, -25.0));
    let world_view = view;
    navigation.enter_region(&world, region, &mut view).unwrap();
    assert_eq!(view, MapView::default());
    assert_eq!(navigation.selection(), None);
    assert_region_scope(&navigation, &world, &view, region);
    assert_scope_coordinates(&navigation, &world);
    navigation
        .select(&world, MapSelection::Site(SiteId(5)))
        .unwrap();
    assert!(navigation
        .select(&world, MapSelection::Marker(region))
        .is_err());
    assert!(navigation
        .select(&world, MapSelection::Site(SiteId(1)))
        .is_err());
    assert!(navigation
        .select(&world, MapSelection::Site(SiteId(u32::MAX)))
        .is_err());
    assert_eq!(navigation.selection(), Some(MapSelection::Site(SiteId(5))));
    assert!(navigation.enter_region(&world, region, &mut view).is_err());

    view.zoom(vec2(730.0, 360.0), 1.75);
    view.pan(vec2(-100.0, 20.0));
    let region_view = view;
    navigation.show_world(&mut view);
    assert_eq!(view, world_view);
    assert_eq!(navigation.scope(), MapScope::World);
    assert_eq!(navigation.selection(), Some(MapSelection::Marker(region)));
    navigation.show_world(&mut view);
    assert_eq!(view, world_view);
    navigation.enter_region(&world, region, &mut view).unwrap();
    assert_eq!(view, region_view);
    navigation
        .select(&world, MapSelection::Site(SiteId(5)))
        .unwrap();
    navigation.clear_selection();
    assert_eq!(navigation.selection(), None);
    navigation.reset(&mut view);
    assert_eq!(navigation.scope(), MapScope::World);
    assert_eq!(view, MapView::default());
    navigation.enter_region(&world, region, &mut view).unwrap();
    assert_eq!(view, MapView::default());
    assert_eq!(world, original_world);

    assert_minimum_targets_and_stable_ties(world);
}

fn assert_region_scope(
    navigation: &MapNavigation,
    world: &CampaignWorld,
    view: &MapView,
    region: MarkerId,
) {
    assert_eq!(navigation.scope(), MapScope::Region(region));
    let marker = world
        .markers
        .iter()
        .find(|marker| marker.id == region)
        .unwrap();
    let MarkerLocation::Region { sites, .. } = &marker.location else {
        panic!("Rosemarch is a region");
    };
    let targets = navigation.targets(world, view);
    assert_eq!(targets.len(), sites.len());
    for target in targets {
        let MapSelection::Site(id) = target.selection else {
            panic!("Regional markers cannot become physical selection targets");
        };
        assert!(sites.contains(&id));
        assert_eq!(world.site(id).unwrap().marker, region);
    }
}

fn assert_scope_coordinates(navigation: &MapNavigation, world: &CampaignWorld) {
    for zoom in [1.0, 1.5, 3.0] {
        let mut view = MapView::default();
        view.zoom(vec2(WIDTH * 0.5, HEIGHT * 0.5), zoom);
        view.pan(vec2(12.0, -8.0));
        let targets = navigation.targets(world, &view);
        assert!(!targets.is_empty());
        for (screen_width, screen_height) in [
            (1280.0, 720.0),
            (1920.0, 1080.0),
            (1920.0, 1200.0),
            (2560.0, 1080.0),
        ] {
            let viewport = VirtualUi::from_screen_size(WIDTH, HEIGHT, screen_width, screen_height);
            for target in &targets {
                let position = match target.selection {
                    MapSelection::Marker(id) => {
                        world
                            .markers
                            .iter()
                            .find(|marker| marker.id == id)
                            .unwrap()
                            .position
                    }
                    MapSelection::Site(id) => world.site(id).unwrap().position,
                };
                assert_eq!(target.center, view.project_normalized(position));
                assert_eq!(target.bounds().w, MAP_TAP_SIZE);
                assert_eq!(target.bounds().h, MAP_TAP_SIZE);
                assert!(target.bounds().w * viewport.scale >= 48.0);
                for dpi in [1.0, 1.5, 2.0] {
                    let framebuffer_point = viewport.ui_to_screen(target.center) * dpi;
                    let logical_point = viewport
                        .screen_to_ui_checked(framebuffer_point / dpi)
                        .unwrap();
                    assert_eq!(
                        navigation.pick(world, &view, logical_point),
                        Some(target.selection)
                    );
                }
            }
            if viewport.offset.x > 0.0 || viewport.offset.y > 0.0 {
                let letterbox = vec2(1.0, 1.0);
                assert!(viewport.screen_to_ui_checked(letterbox).is_none());
                assert_eq!(
                    navigation.pick(world, &view, viewport.screen_to_ui(letterbox)),
                    None
                );
            }
        }
    }
    for invalid in [vec2(-1.0, 360.0), vec2(WIDTH + 1.0, 360.0), Vec2::NAN] {
        assert_eq!(navigation.pick(world, &MapView::default(), invalid), None);
    }
}

fn assert_minimum_targets_and_stable_ties(mut world: CampaignWorld) {
    let navigation = MapNavigation::default();
    let mut regional = MapNavigation::default();
    for site in &mut world.sites {
        if matches!(site.id, SiteId(5) | SiteId(6)) {
            site.position = [0.5, 0.5];
        }
    }
    world.sites.reverse();
    regional
        .enter_region(&world, MarkerId(5), &mut MapView::default())
        .unwrap();
    let center = vec2(WIDTH * 0.5, HEIGHT * 0.5);
    assert_eq!(
        regional.pick(&world, &MapView::default(), center),
        Some(MapSelection::Site(SiteId(5)))
    );
    world.markers.truncate(2);
    world.markers[0].position = [0.5, 0.5];
    world.markers[1].position = [0.5, 0.5];
    world.markers.reverse();
    // Vector ordering cannot decide which coincident location is selected.
    assert_eq!(
        navigation.pick(&world, &MapView::default(), center),
        Some(MapSelection::Marker(MarkerId(1)))
    );
    world.markers[0].position = [0.515625, 0.5]; // Exactly 20 logical pixels east.
    assert_eq!(
        navigation.pick(&world, &MapView::default(), center + vec2(18.0, 0.0)),
        Some(MapSelection::Marker(MarkerId(2)))
    );
    assert_eq!(
        navigation.pick(&world, &MapView::default(), center + vec2(10.0, 0.0)),
        Some(MapSelection::Marker(MarkerId(1)))
    );
    assert_release_selection(&navigation, &world, center);
    world.markers.retain(|marker| marker.id == MarkerId(1));
    for zoom in [1.0, 1.5, 3.0] {
        let mut view = MapView::default();
        view.zoom(center, zoom);
        for offset in [
            vec2(-23.9, -23.9),
            vec2(23.9, -23.9),
            vec2(-23.9, 23.9),
            vec2(23.9, 23.9),
        ] {
            assert_eq!(
                navigation.pick(&world, &view, center + offset),
                Some(MapSelection::Marker(MarkerId(1)))
            );
        }
        assert_eq!(
            navigation.pick(&world, &view, center + vec2(24.1, 0.0)),
            None
        );
    }
    world.markers[0].position = [1.0, 0.5];
    let mut zoomed = MapView::default();
    zoomed.zoom(center, 3.0);
    assert!(navigation.targets(&world, &zoomed).is_empty());
}

fn assert_release_selection(navigation: &MapNavigation, world: &CampaignWorld, center: Vec2) {
    let view = MapView::default();
    for distance in [0.0, 3.0, 6.0] {
        assert_eq!(
            navigation.pick_release(world, &view, center, center + vec2(distance, 0.0)),
            Some(MapSelection::Marker(MarkerId(1)))
        );
    }
    // Both ends still pick marker1, but a final-frame drag must not select it.
    assert_eq!(
        navigation.pick_release(world, &view, center, center + vec2(10.0, 0.0)),
        None
    );
    // A short move across overlapping targets is not a tap of either target.
    assert_eq!(
        navigation.pick_release(
            world,
            &view,
            center + vec2(8.0, 0.0),
            center + vec2(12.0, 0.0)
        ),
        None
    );
    for (origin, release) in [(Vec2::NAN, center), (center, Vec2::NAN)] {
        assert_eq!(navigation.pick_release(world, &view, origin, release), None);
    }
}
