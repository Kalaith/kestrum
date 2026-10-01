//! Authored geography changes never silently relocate a saved campaign.
use kestrum::{
    data::{
        generation::ProductionSetup,
        rules::Emblem,
        world::{MarkerId, MarkerLocation, RouteId, SiteId},
        GameData,
    },
    engine::{apply, project_map, Actor, Command},
    navigation::{MapNavigation, MapScope, MapView},
    state::StrategicCampaign,
};

fn campaign(data: &GameData) -> StrategicCampaign {
    StrategicCampaign::new_production(
        data,
        &ProductionSetup {
            kingdom_name: "Coastward".into(),
            emblem: Emblem::Rose,
            factions: 8,
            seed: 88,
        },
    )
    .unwrap()
}

#[test]
fn corrected_coast_keeps_stable_ids_costs_entrances_and_connectivity() {
    let data = GameData::load().unwrap();
    let current = campaign(&data);
    assert_eq!(current.world.layout_revision, 3);
    assert_eq!(
        current.world.site(SiteId(40)).unwrap().position,
        [0.12, 0.55]
    );
    assert_eq!(
        current.world.site(SiteId(43)).unwrap().position,
        [0.25, 0.615]
    );
    let route = current.world.route(RouteId(99)).unwrap();
    assert_eq!(
        (route.from, route.to, route.terrain_cost),
        (SiteId(40), SiteId(43), 2)
    );
    assert!(!current.world.atlas_paths[&RouteId(99)].waypoints.is_empty());
    assert_eq!(data.production_layout.reachable_sites(SiteId(1)).len(), 152);
    for marker in &current.world.markers {
        if let MarkerLocation::Region { entrances, .. } = &marker.location {
            assert!(entrances.len() >= 2);
        }
    }
}

#[test]
fn new_layout_and_authored_crossings_survive_save_reload_and_orders() {
    let data = GameData::load().unwrap();
    let mut current = campaign(&data);
    let world = current.world.clone();
    assert!(!world.atlas_paths[&RouteId(186)].bridges.is_empty());
    let json = serde_json::to_vec(&current).unwrap();
    let mut loaded: StrategicCampaign = serde_json::from_slice(&json).unwrap();
    loaded.validate(&data).unwrap();
    inspect_at_all_scales(&loaded, &data);
    apply(&mut current, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(&mut loaded, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(loaded, current);
    assert_eq!(loaded.world.markers, world.markers);
    assert_eq!(loaded.world.atlas_paths, world.atlas_paths);
}

/// Camera exploration is presentation only, including on historical layouts.
fn inspect_at_all_scales(campaign: &StrategicCampaign, data: &GameData) {
    use macroquad::prelude::vec2;
    let bytes = serde_json::to_vec(campaign).unwrap();
    let visible = project_map(campaign, campaign.player).unwrap();
    let mut navigation = MapNavigation::configured(&data.presentation.map.camera);
    let mut view = MapView::configured(&data.presentation.map.camera, MapScope::World);
    let home = campaign.factions[&campaign.player].headquarters;
    navigation.focus_army_site(&visible.world, home, &mut view);
    let working = view;
    navigation.toggle_overview(&visible.world, &mut view);
    navigation.toggle_overview(&visible.world, &mut view);
    assert_eq!(view, working);
    for factor in [0.3, 2.0, 5.0] {
        view.zoom(vec2(1100.0, 480.0), factor);
        view.pan(vec2(-850.0, 250.0));
        let groups = navigation.place_groups(&visible.world, &view);
        for group in &groups {
            assert_eq!(
                navigation.pick_group_release(&visible.world, &view, group.center, group.center),
                Some(group.clone())
            );
        }
        for target in navigation.targets(&visible.world, &view) {
            let picked = navigation.pick(&visible.world, &view, target.center);
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
    assert_eq!(serde_json::to_vec(campaign).unwrap(), bytes);
}

fn old_layout(data: &GameData) -> StrategicCampaign {
    let mut old = campaign(data);
    old.world.layout_revision = 1;
    old.world.atlas_paths.clear();
    for site in &mut old.world.sites {
        if data
            .production_layout
            .headquarters_candidates
            .contains(&site.id)
        {
            site.tags
                .retain(|tag| *tag != kestrum::data::world::SiteTag::HorseAccess);
        }
    }
    for marker in &mut old.world.markers {
        if let Some(position) = data
            .production_layout
            .legacy_marker_positions
            .get(&marker.id)
        {
            marker.position = *position;
            if let MarkerLocation::Site { site } = marker.location {
                old.world
                    .sites
                    .iter_mut()
                    .find(|entry| entry.id == site)
                    .unwrap()
                    .position = *position;
            }
        }
    }
    old
}

#[test]
fn missing_revision_decodes_as_original_without_moving_any_saved_site() {
    let data = GameData::load().unwrap();
    let old = old_layout(&data);
    old.validate(&data).unwrap();
    let mut value = serde_json::to_value(&old).unwrap();
    let world = value.get_mut("world").unwrap().as_object_mut().unwrap();
    world.remove("layout_revision");
    world.remove("atlas_paths");
    let loaded: StrategicCampaign = serde_json::from_value(value).unwrap();
    loaded.validate(&data).unwrap();
    inspect_at_all_scales(&loaded, &data);
    assert_eq!(loaded, old);
    assert_eq!(
        loaded.world.site(SiteId(40)).unwrap().position,
        [0.035, 0.674]
    );
    assert_eq!(
        loaded.world.marker(MarkerId(77)).unwrap().position,
        [0.225, 0.54]
    );
    let mut expected = old;
    let mut actual = loaded;
    apply(&mut expected, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(&mut actual, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn unknown_or_mixed_revisions_and_changed_geometry_are_rejected() {
    let data = GameData::load().unwrap();
    let original = campaign(&data);
    for revision in [0, 1, 4] {
        let mut invalid = original.clone();
        invalid.world.layout_revision = revision;
        assert!(invalid.validate(&data).is_err());
    }
    let mut invalid = old_layout(&data);
    invalid
        .world
        .sites
        .iter_mut()
        .find(|s| s.id == SiteId(40))
        .unwrap()
        .position = [0.12, 0.55];
    assert!(invalid.validate(&data).is_err());
    let mut invalid = original;
    invalid
        .world
        .atlas_paths
        .get_mut(&RouteId(99))
        .unwrap()
        .waypoints
        .clear();
    assert!(invalid.validate(&data).is_err());
}

#[test]
fn atlas_content_rejects_unknown_routes_and_crossings_away_from_the_road() {
    let data = GameData::load().unwrap();
    let mut invalid = data.clone();
    let path = invalid.production_layout.atlas_paths[&RouteId(99)].clone();
    invalid
        .production_layout
        .atlas_paths
        .insert(RouteId(999), path);
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    invalid
        .production_layout
        .atlas_paths
        .get_mut(&RouteId(186))
        .unwrap()
        .bridges[0] = [0.0, 0.0];
    assert!(invalid.validate().is_err());
    StrategicCampaign::new(&data)
        .unwrap()
        .validate(&data)
        .unwrap();
}
