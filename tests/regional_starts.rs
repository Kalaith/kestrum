//! Fog must leave the opening movement and regional-navigation lessons playable.

use kestrum::{
    data::{
        generation::ProductionSetup,
        rules::Emblem,
        world::{MarkerLocation, SiteTag},
        GameData,
    },
    engine::{self, Actor, Command},
    navigation::{MapNavigation, MapScope, MapSelection, MapView, MAP_RECT},
    state::{Campaign, GameState, StrategicCampaign},
};
use std::collections::BTreeSet;

fn campaign(data: &GameData, factions: usize, seed: u64) -> StrategicCampaign {
    StrategicCampaign::new_production(
        data,
        &ProductionSetup {
            kingdom_name: "Rose".into(),
            emblem: Emblem::Rose,
            factions,
            seed,
        },
    )
    .unwrap()
}

#[test]
fn all_factions_found_in_distinct_regions_with_only_local_discovery() {
    let data = GameData::load().unwrap();
    for count in 4..=8 {
        let campaign = campaign(&data, count, data.production_layout.default_seed);
        let mut regions = BTreeSet::new();
        for faction in campaign.factions.values() {
            let home = campaign.world.site(faction.headquarters).unwrap();
            assert!(regions.insert(home.marker));
            let MarkerLocation::Region { sites, .. } =
                &campaign.world.marker(home.marker).unwrap().location
            else {
                panic!("every new home has a regional world marker");
            };
            let known = engine::explored_sites(&campaign, faction.id);
            assert!(known.contains(&home.id));
            assert!(known.len() < sites.len());
            let visible = engine::project_map(&campaign, faction.id).unwrap();
            assert_eq!(visible.world.markers.len(), 1);
            assert_eq!(visible.world.markers[0].id, home.marker);
        }
    }
}

#[test]
fn first_journey_is_visible_affordable_and_clear_across_seeds() {
    let data = GameData::load().unwrap();
    for count in [4, 8] {
        for seed in [0, 1, 88, 42017, data.production_layout.default_seed] {
            let mut campaign = campaign(&data, count, seed);
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player)
                .unwrap()
                .id;
            let home = campaign.armies[&army].site;
            let preview = campaign
                .world
                .adjacent_sites(home)
                .into_iter()
                .find_map(|site| {
                    engine::map_movement_preview(&campaign, &data, campaign.player, &[army], site)
                        .ok()
                        .filter(|preview| preview.stop.is_none() && preview.reachable_steps > 0)
                })
                .expect("a new army must have a safe first journey");
            let destination = *preview.order.path.last().unwrap();
            assert_eq!(
                campaign.world.site(home).unwrap().marker,
                campaign.world.site(destination).unwrap().marker
            );
            engine::apply(
                &mut campaign,
                &data,
                Actor::Player,
                Command::Move(preview.order),
            )
            .unwrap();
            assert_eq!(campaign.armies[&army].site, destination);
        }
    }
}

#[test]
fn home_camera_and_region_round_trip_use_the_discovered_projection() {
    let data = GameData::load().unwrap();
    let campaign = campaign(&data, 4, data.production_layout.default_seed);
    let home = campaign.factions[&campaign.player].headquarters;
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let region = visible.world.site(home).unwrap().marker;
    let known = engine::explored_sites(&campaign, campaign.player);
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    navigation.focus_site(&visible.world, home, &mut view);
    assert_eq!(navigation.scope(), MapScope::Region(region));
    assert!(navigation
        .army_targets(&visible.world, &view, &visible.armies)
        .iter()
        .all(|target| MAP_RECT.contains(target.bounds.center())));
    navigation.show_world(&mut view);
    assert_eq!(view.camera.zoom(), view.working_zoom());
    assert_eq!(navigation.selection(), Some(MapSelection::Marker(region)));
    let target = navigation
        .targets(&visible.world, &view)
        .into_iter()
        .find(|target| target.selection == MapSelection::Marker(region))
        .unwrap();
    assert_eq!(
        navigation.pick(&visible.world, &view, target.center),
        Some(target.selection)
    );
    navigation
        .enter_region(&visible.world, region, &mut view)
        .unwrap();
    assert_eq!(navigation.scope(), MapScope::Region(region));
    assert_eq!(engine::explored_sites(&campaign, campaign.player), known);
}

#[test]
fn earlier_atlas_saves_keep_their_terrain_headquarters_and_discoveries() {
    let data = GameData::load().unwrap();
    let mut old = campaign(&data, 4, 88);
    old.world.layout_revision = 2;
    for site in &mut old.world.sites {
        if data
            .production_layout
            .headquarters_candidates
            .contains(&site.id)
        {
            site.tags.retain(|tag| *tag != SiteTag::HorseAccess);
        }
    }
    old.validate(&data).unwrap();
    let expected = old.clone();
    let saved = Campaign::Strategic(Box::new(old));
    let mut state = GameState::default();
    state.load_campaign(saved, &data).unwrap();
    assert_eq!(state.campaign.unwrap().strategic().unwrap(), &expected);
}

#[test]
fn regional_start_requirements_and_current_revision_terrain_are_validated() {
    let data = GameData::load().unwrap();
    let mut invalid = data.clone();
    invalid.production_layout.headquarters_candidates[0] = kestrum::data::world::SiteId(5);
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    let home = invalid.production_layout.headquarters_candidates[0];
    invalid
        .production_layout
        .sites
        .iter_mut()
        .find(|site| site.id == home)
        .unwrap()
        .tags
        .clear();
    assert!(invalid.validate().is_err());
    let mut invalid = campaign(&data, 4, 88);
    invalid
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == home)
        .unwrap()
        .tags
        .clear();
    assert!(invalid.validate(&data).is_err());
}
