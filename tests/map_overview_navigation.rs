//! M01 navigation geometry: importance, labels and picking across camera sizes.

use kestrum::{
    data::{world::MarkerId, GameData},
    engine,
    navigation::{
        place_map_labels, MapLabelCandidate, MapNavigation, MapSelection, MapTarget, MapView,
    },
    state::StrategicCampaign,
};
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::ui::VirtualUi;

#[test]
fn overview_labels_prioritize_capital_without_changing_zoomed_map_and_army_targets() {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    assert_edge_banners(&visible);
    let navigation = MapNavigation::default();
    for (width, height) in [(1920.0, 1080.0)] {
        let viewport = VirtualUi::from_screen_size(1920.0, 1080.0, width, height);
        for zoom in [1.0, 1.5, 3.0] {
            let mut view = MapView::default();
            view.zoom(vec2(960.0, 540.0), zoom);
            let groups = navigation.place_groups(&visible.world, &view);
            for target in navigation.targets(&visible.world, &view) {
                let screen = viewport.ui_to_screen(target.center);
                let logical = viewport.screen_to_ui(screen);
                let picked = navigation.pick(&visible.world, &view, logical);
                if groups
                    .iter()
                    .any(|group| group.selections.contains(&target.selection))
                {
                    assert_ne!(picked, Some(target.selection));
                } else {
                    assert_eq!(picked, Some(target.selection));
                }
                assert!(target.bounds().w >= 48.0 && target.bounds().h >= 48.0);
            }
            for group in &groups {
                let logical = viewport.screen_to_ui(viewport.ui_to_screen(group.center));
                assert_eq!(
                    navigation.pick_group_release(&visible.world, &view, logical, logical),
                    Some(group.clone())
                );
            }
            for target in navigation.army_targets(&visible.world, &view, &visible.armies) {
                assert!(target.bounds.w >= 48.0 && target.bounds.h >= 48.0);
                assert!(target.bounds.x >= 0.0 && target.bounds.right() <= 1920.0);
                assert!(target.bounds.y >= 92.0 && target.bounds.bottom() <= 944.0);
                let center = target.bounds.center();
                assert!(target
                    .bounds
                    .contains(viewport.screen_to_ui(viewport.ui_to_screen(center))));
            }
        }
    }
    let targets = [
        MapTarget {
            selection: MapSelection::Marker(MarkerId(1)),
            center: vec2(520.0, 300.0),
        },
        MapTarget {
            selection: MapSelection::Marker(MarkerId(2)),
            center: vec2(580.0, 300.0),
        },
        MapTarget {
            selection: MapSelection::Marker(MarkerId(3)),
            center: vec2(900.0, 450.0),
        },
    ];
    let candidates = || {
        targets
            .iter()
            .enumerate()
            .map(|(index, target)| MapLabelCandidate {
                selection: target.selection,
                center: target.center,
                width: 210.0,
                priority: if index == 0 { 95 } else { 30 },
                minimum_zoom: if index == 2 { 2.0 } else { 1.0 },
            })
            .collect()
    };
    let reserved = [Rect::new(450.0, 230.0, 250.0, 40.0)];
    let distant = place_map_labels(candidates(), &targets, &reserved, 1.0);
    assert_eq!(distant.len(), 1);
    assert_eq!(distant[0].selection, targets[0].selection);
    let close = place_map_labels(candidates(), &targets, &reserved, 2.0);
    assert_eq!(close.len(), 2);
    for label in &close {
        assert!(!reserved.iter().any(|rect| rect.overlaps(&label.bounds)));
        assert!(!targets
            .iter()
            .any(|target| target.bounds().overlaps(&label.bounds)));
    }
    // Coast/lake mask constrains ink independently of ownership and game topology.
    assert!(data.presentation.map.is_atlas_land([0.6, 0.5]));
    assert!(!data.presentation.map.is_atlas_land([0.5, 0.95]));
    assert!(!data.presentation.map.is_atlas_land([0.17, 0.31]));
}

fn assert_edge_banners(visible: &engine::VisibleCampaign) {
    let navigation = MapNavigation::default();
    let mut view = MapView::default();
    let army = visible.armies.first().unwrap();
    let marker_id = visible.world.site(army.site).unwrap().marker;
    let mut world = visible.world.clone();
    for position in [[0.98, 0.18], [0.02, 0.18], [0.98, 0.72], [0.02, 0.72]] {
        world
            .markers
            .iter_mut()
            .find(|marker| marker.id == marker_id)
            .unwrap()
            .position = position;
        view.focus(position, view.working_zoom());
        let center = view.project_normalized(position);
        let protected = Rect::new(center.x - 24.0, center.y - 24.0, 48.0, 48.0);
        let targets = navigation.army_targets(&world, &view, &visible.armies);
        let banner = targets
            .iter()
            .find(|target| target.armies.contains(&army.id))
            .unwrap();
        assert!(
            !protected.overlaps(&banner.bounds),
            "edge banner hid its own place or warnings"
        );
    }
}

#[test]
fn crowded_places_focus_before_selection_and_do_not_pick_hidden_members() {
    let data = GameData::load().unwrap();
    let mut world = kestrum::state::world::CampaignWorld::from_scenario(&data.scenario);
    world.markers.truncate(2);
    world.markers[0].position = [0.49, 0.5];
    world.markers[1].position = [0.51, 0.5];
    let original = world.clone();
    let navigation = MapNavigation::default();
    let mut view = MapView::default();
    view.focus([0.5, 0.5], 0.4);
    let groups = navigation.place_groups(&world, &view);
    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.selections.len(), 2);
    assert_eq!(group.bounds().w, 48.0);
    assert_eq!(navigation.pick(&world, &view, group.center), None);
    let tapped = navigation
        .pick_group_release(&world, &view, group.center, group.center)
        .unwrap();
    assert_eq!(&tapped, group);
    assert!(navigation
        .pick_group_release(&world, &view, group.center, group.center + vec2(10.0, 0.0))
        .is_none());
    view.focus(group.focus, group.zoom);
    assert!(navigation.place_groups(&world, &view).is_empty());
    for target in navigation.targets(&world, &view) {
        assert_eq!(
            navigation.pick(&world, &view, target.center),
            Some(target.selection)
        );
    }
    assert_eq!(world, original);
}

#[test]
fn compact_forces_distinguish_nearby_regions_and_physical_site_stacks() {
    use kestrum::{
        data::world::SiteId,
        navigation::{ArmyGrouping, MapScope},
        state::military::ArmyId,
    };
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    let mut world = campaign.world.clone();
    world.markers[0].position = [0.49, 0.5];
    world.markers[1].position = [0.51, 0.5];
    let mut first = campaign.armies.values().next().unwrap().clone();
    let mut second = first.clone();
    let first_marker = world.markers[0].id;
    let second_marker = world.markers[1].id;
    first.site = world
        .sites
        .iter()
        .find(|site| site.marker == first_marker)
        .unwrap()
        .id;
    second.id = ArmyId(999);
    second.site = world
        .sites
        .iter()
        .find(|site| site.marker == second_marker)
        .unwrap()
        .id;
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    view.focus([0.5, 0.5], 0.4);
    let grouped = navigation.army_targets(&world, &view, &[first.clone(), second.clone()]);
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0].grouping, ArmyGrouping::Nearby);
    assert_eq!(grouped[0].bounds.w, 48.0);
    assert_eq!(grouped[0].bounds.h, 48.0);
    view.focus_group(grouped[0].focus.unwrap());
    assert_eq!(
        navigation
            .army_targets(&world, &view, &[first.clone(), second.clone()])
            .len(),
        2
    );
    first.site = SiteId(5);
    second.site = SiteId(6);
    let region = world.site(first.site).unwrap().marker;
    view.focus(world.marker(region).unwrap().position, 1.0);
    let regional = navigation.army_targets(&world, &view, &[first.clone(), second.clone()]);
    assert_eq!(regional[0].grouping, ArmyGrouping::Region);
    assert!(regional[0].focus.is_none());
    navigation.enter_region(&world, region, &mut view).unwrap();
    first.site = SiteId(5);
    second.site = SiteId(5);
    view.focus(world.site(first.site).unwrap().position, 1.0);
    let shared = navigation.army_targets(&world, &view, &[first, second]);
    assert_eq!(navigation.scope(), MapScope::Region(region));
    assert_eq!(shared[0].grouping, ArmyGrouping::Site);
    assert_eq!(shared[0].armies.len(), 2);
}
