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
    for (width, height) in [(1280.0, 720.0), (1920.0, 1080.0)] {
        let viewport = VirtualUi::from_screen_size(1280.0, 720.0, width, height);
        for zoom in [1.0, 1.5, 3.0] {
            let mut view = MapView::default();
            view.zoom(vec2(640.0, 360.0), zoom);
            for target in navigation.targets(&visible.world, &view) {
                let screen = viewport.ui_to_screen(target.center);
                let logical = viewport.screen_to_ui(screen);
                assert_eq!(
                    navigation.pick(&visible.world, &view, logical),
                    Some(target.selection)
                );
                assert!(target.bounds().w >= 48.0 && target.bounds().h >= 48.0);
            }
            for target in navigation.army_targets(&visible.world, &view, &visible.armies) {
                assert!(target.bounds.w >= 48.0 && target.bounds.h >= 48.0);
                assert!(target.bounds.x >= 0.0 && target.bounds.right() <= 1280.0);
                assert!(target.bounds.y >= 92.0 && target.bounds.bottom() <= 572.0);
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
    let view = MapView::default();
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
        let center = view.project_normalized(position);
        let protected = Rect::new(center.x - 46.0, center.y - 50.0, 92.0, 88.0);
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
