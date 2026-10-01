//! Fog describes unknown territory while road hints remain presentation only.

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, world::MarkerId, GameData},
    engine,
    navigation::{
        MapExploration, MapNavigation, MapScope, MapSelection, MapView, HEIGHT, MAP_RECT, WIDTH,
    },
    state::{world::CampaignWorld, StrategicCampaign},
};
use macroquad::prelude::{vec2, Vec2};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production(
        &data,
        &ProductionSetup {
            kingdom_name: "Northward".into(),
            emblem: Emblem::Rose,
            factions: 8,
            seed: data.production_layout.default_seed,
        },
    )
    .unwrap();
    (data, campaign)
}

fn point(position: [f32; 2]) -> Vec2 {
    vec2(position[0] * WIDTH, position[1] * HEIGHT)
}

fn only_marker(world: &CampaignWorld, marker: MarkerId) -> CampaignWorld {
    let mut visible = world.clone();
    visible.markers.retain(|entry| entry.id == marker);
    visible.sites.retain(|site| site.marker == marker);
    let sites: std::collections::BTreeSet<_> = visible.sites.iter().map(|site| site.id).collect();
    visible
        .routes
        .retain(|route| sites.contains(&route.from) && sites.contains(&route.to));
    visible
}

#[test]
fn owning_the_north_clears_empty_land_above_it() {
    let (data, mut campaign) = fixture();
    let northern: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| campaign.world.marker(site.marker).unwrap().position[1] <= 0.4)
        .map(|site| site.id)
        .collect();
    for site in northern {
        campaign
            .set_site_control(&data, site, Some(campaign.player), false)
            .unwrap();
    }
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let fog = MapExploration::new(&campaign.world, &visible.world, MapScope::World);
    for column in 0..=40 {
        for y in [0.0, 0.05, 0.1] {
            let at = point([column as f32 / 40.0, y]);
            assert_eq!(
                fog.opacity(at),
                0.0,
                "fog above northern holdings at {at:?}"
            );
        }
    }
    assert!(campaign
        .world
        .markers
        .iter()
        .any(|marker| visible.world.marker(marker.id).is_none()
            && fog.opacity(point(marker.position)) == 1.0));
}

#[test]
fn fog_softens_the_frontier_and_disappears_after_complete_discovery() {
    let (_, campaign) = fixture();
    let mut world = campaign.world.clone();
    world.markers.truncate(2);
    let visible = only_marker(&world, world.markers[0].id);
    let fog = MapExploration::new(&world, &visible, MapScope::World);
    let a = point(world.markers[0].position);
    let b = point(world.markers[1].position);
    assert_eq!(fog.opacity(a), 0.0);
    assert_eq!(fog.opacity(b), 1.0);
    assert!((fog.opacity((a + b) * 0.5) - 0.5).abs() < 0.001);
    let samples: Vec<_> = (0..=20)
        .map(|step| fog.opacity(a.lerp(b, step as f32 / 20.0)))
        .collect();
    assert!(samples.windows(2).all(|pair| pair[0] <= pair[1]));
    let revealed = MapExploration::new(&campaign.world, &campaign.world, MapScope::World);
    let mut empty = campaign.world.clone();
    empty.markers.clear();
    empty.sites.clear();
    empty.routes.clear();
    let hidden = MapExploration::new(&campaign.world, &empty, MapScope::World);
    for at in [
        Vec2::ZERO,
        vec2(WIDTH, HEIGHT),
        vec2(WIDTH * 0.5, HEIGHT * 0.5),
    ] {
        assert_eq!(revealed.opacity(at), 0.0);
        assert_eq!(hidden.opacity(at), 1.0);
    }
    assert!(revealed.connection_hints.is_empty() && hidden.connection_hints.is_empty());
}

#[test]
fn frontier_hints_preserve_authored_bends_without_revealing_targets() {
    let (_, campaign) = fixture();
    let before = campaign.clone();
    let route = campaign
        .world
        .routes
        .iter()
        .find(|route| {
            route.major_connection.is_some()
                && campaign
                    .world
                    .atlas_paths
                    .get(&route.id)
                    .is_some_and(|path| !path.waypoints.is_empty())
        })
        .unwrap();
    let [from, to] = route.major_connection.unwrap();
    let visible = only_marker(&campaign.world, from);
    let fog = MapExploration::new(&campaign.world, &visible, MapScope::World);
    let expected: Vec<_> = std::iter::once(campaign.world.marker(from).unwrap().position)
        .chain(
            campaign.world.atlas_paths[&route.id]
                .waypoints
                .iter()
                .copied(),
        )
        .chain(std::iter::once(campaign.world.marker(to).unwrap().position))
        .map(point)
        .collect();
    assert!(fog.connection_hints.contains(&expected));
    let eligible = campaign
        .world
        .routes
        .iter()
        .filter(|route| {
            route
                .major_connection
                .is_some_and(|ends| ends.contains(&from))
        })
        .count();
    assert_eq!(fog.connection_hints.len(), eligible);
    let mut navigation = MapNavigation::default();
    assert!(navigation
        .select(&visible, MapSelection::Marker(to))
        .is_err());
    assert!(navigation
        .targets(&visible, &MapView::default())
        .iter()
        .all(|target| target.selection != MapSelection::Marker(to)));
    assert_eq!(campaign, before);
}

#[test]
fn regional_fog_and_connections_use_only_the_regions_sites() {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let region = MarkerId(5);
    let fog = MapExploration::new(&campaign.world, &visible.world, MapScope::Region(region));
    let eligible: Vec<_> = campaign
        .world
        .routes
        .iter()
        .filter(|route| {
            let a = campaign.world.site(route.from).unwrap();
            let b = campaign.world.site(route.to).unwrap();
            a.marker == region
                && b.marker == region
                && (visible.world.site(a.id).is_some() != visible.world.site(b.id).is_some())
        })
        .collect();
    assert!(!eligible.is_empty());
    assert_eq!(fog.connection_hints.len(), eligible.len());
    for route in eligible {
        let a = campaign.world.site(route.from).unwrap();
        let b = campaign.world.site(route.to).unwrap();
        assert!(fog
            .connection_hints
            .contains(&vec![point(a.position), point(b.position)]));
    }
    let mut outside_changed = campaign.world.clone();
    for site in outside_changed
        .sites
        .iter_mut()
        .filter(|site| site.marker != region)
    {
        site.position = [0.5, 0.5];
    }
    let same = MapExploration::new(&outside_changed, &visible.world, MapScope::Region(region));
    for position in [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]] {
        assert_eq!(fog.opacity(point(position)), same.opacity(point(position)));
    }
}

#[test]
fn pan_and_zoom_preserve_the_discovery_boundary() {
    let (_, campaign) = fixture();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let fog = MapExploration::new(&campaign.world, &visible.world, MapScope::World);
    let mut view = MapView::default();
    for zoom in [1.0, 1.7, 3.0] {
        view.focus([0.4, 0.2], zoom);
        view.pan(vec2(80.0, -45.0));
        for position in [[0.0, 0.0], [0.4, 0.25], [0.5, 0.5], [1.0, 1.0]] {
            let at = point(position);
            let projected = view.project(at);
            let restored = view.camera.screen_to_world(MAP_RECT, projected).unwrap();
            assert!((fog.opacity(at) - fog.opacity(restored)).abs() < 0.0001);
        }
    }
}
