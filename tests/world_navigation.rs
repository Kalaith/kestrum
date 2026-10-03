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
    assert!(!navigation.targets(&world, &view).is_empty());
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
    let initial_region_view = view;
    assert_eq!(view.camera.zoom(), view.working_zoom());
    assert_eq!(view.extent(), vec2(3360.0, 1890.0));
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
    assert_eq!(view, initial_region_view);
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
    assert!(!targets.is_empty() && targets.len() <= sites.len());
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
        let mut view = MapView::configured(
            &GameData::load().unwrap().presentation.map.camera,
            navigation.scope(),
        );
        view.zoom(vec2(WIDTH * 0.5, HEIGHT * 0.5), zoom);
        view.pan(vec2(12.0, -8.0));
        let targets = navigation.targets(world, &view);
        let groups = navigation.place_groups(world, &view);
        assert!(!targets.is_empty());
        for (screen_width, screen_height) in [(1920.0, 1080.0), (1920.0, 1200.0), (2560.0, 1080.0)]
        {
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
                    let picked = navigation.pick(world, &view, logical_point);
                    if groups
                        .iter()
                        .any(|group| group.selections.contains(&target.selection))
                    {
                        assert_ne!(picked, Some(target.selection));
                    } else {
                        assert_eq!(picked, Some(target.selection));
                    }
                }
            }
            for group in &groups {
                let logical = viewport.screen_to_ui(viewport.ui_to_screen(group.center));
                assert_eq!(
                    navigation.pick_group_release(world, &view, logical, logical),
                    Some(group.clone())
                );
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
    let mut precise = MapView::default();
    precise.zoom(center, precise.zoom_limits().1);
    assert_eq!(
        regional.pick(&world, &precise, center),
        Some(MapSelection::Site(SiteId(5)))
    );
    world.markers.truncate(2);
    world.markers[0].position = [0.5, 0.5];
    world.markers[1].position = [0.5, 0.5];
    world.markers.reverse();
    // Vector ordering cannot decide which coincident location is selected.
    assert_eq!(
        navigation.pick(&world, &precise, center),
        Some(MapSelection::Marker(MarkerId(1)))
    );
    world.markers[0].position = [
        0.5 + 20.0 / (precise.extent().x * precise.camera.zoom()),
        0.5,
    ]; // 20 logical pixels east.
    assert_eq!(
        navigation.pick(&world, &precise, center + vec2(18.0, 0.0)),
        Some(MapSelection::Marker(MarkerId(2)))
    );
    assert_eq!(
        navigation.pick(&world, &precise, center + vec2(9.9, 0.0)),
        Some(MapSelection::Marker(MarkerId(1)))
    );
    assert_release_selection(&navigation, &world, &precise, center);
    world.markers.retain(|marker| marker.id == MarkerId(1));
    for zoom in [1.0, 1.5, 3.0] {
        let mut view = MapView::configured(
            &GameData::load().unwrap().presentation.map.camera,
            navigation.scope(),
        );
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

fn assert_release_selection(
    navigation: &MapNavigation,
    world: &CampaignWorld,
    view: &MapView,
    center: Vec2,
) {
    for distance in [0.0, 3.0, 6.0] {
        assert_eq!(
            navigation.pick_release(world, view, center, center + vec2(distance, 0.0)),
            Some(MapSelection::Marker(MarkerId(1)))
        );
    }
    // Both ends still pick marker1, but a final-frame drag must not select it.
    assert_eq!(
        navigation.pick_release(world, view, center, center + vec2(9.9, 0.0)),
        None
    );
    // A short move across overlapping targets is not a tap of either target.
    assert_eq!(
        navigation.pick_release(
            world,
            view,
            center + vec2(8.0, 0.0),
            center + vec2(12.0, 0.0)
        ),
        None
    );
    for (origin, release) in [(Vec2::NAN, center), (center, Vec2::NAN)] {
        assert_eq!(navigation.pick_release(world, view, origin, release), None);
    }
}

#[test]
fn entering_new_region_frames_its_known_entrance_then_retains_the_camera() {
    use kestrum::{engine, navigation::MAP_RECT, state::StrategicCampaign};
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    // A known authored entrance starts outside the working regional camera.
    let entrance = SiteId(5);
    campaign
        .knowledge
        .explored
        .entry(campaign.player)
        .or_default()
        .insert(entrance);
    let original = campaign.clone();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let region = visible.world.site(entrance).unwrap().marker;
    let default_region =
        MapView::configured(&data.presentation.map.camera, MapScope::Region(region));
    assert!(!MAP_RECT.contains(
        default_region.project_normalized(visible.world.site(entrance).unwrap().position)
    ));
    let mut view = MapView::default();
    let mut navigation = MapNavigation::default();
    navigation
        .enter_region(&visible.world, region, &mut view)
        .unwrap();
    let target = navigation
        .targets(&visible.world, &view)
        .into_iter()
        .find(|target| target.selection == MapSelection::Site(entrance))
        .unwrap();
    assert_eq!(
        navigation.pick(&visible.world, &view, target.center),
        Some(target.selection)
    );
    assert_eq!(view.camera.zoom(), view.working_zoom());
    view.zoom(MAP_RECT.center(), 1.6);
    view.pan(vec2(-120.0, 40.0));
    let working = view;
    navigation.show_world(&mut view);
    navigation
        .enter_region(&visible.world, region, &mut view)
        .unwrap();
    assert_eq!(view, working);
    assert_eq!(campaign, original);
}
