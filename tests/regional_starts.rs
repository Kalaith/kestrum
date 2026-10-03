//! Fog must leave the opening movement and regional-navigation lessons playable.

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, world::MarkerLocation, GameData},
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
fn production_starts_are_distinct_single_site_markers_with_local_discovery() {
    let data = GameData::load().unwrap();
    for count in 4..=8 {
        let campaign = campaign(&data, count, data.production_layout.default_seed);
        let mut starts = BTreeSet::new();
        for faction in campaign.factions.values() {
            let home = campaign.world.site(faction.headquarters).unwrap();
            assert!(starts.insert(home.marker));
            assert!(matches!(
                campaign.world.marker(home.marker).unwrap().location,
                MarkerLocation::Site { site } if site == home.id
            ));
            let known = engine::explored_sites(&campaign, faction.id);
            assert!(known.contains(&home.id));
            assert!(known.len() < campaign.world.sites.len());
            let visible = engine::project_map(&campaign, faction.id).unwrap();
            assert!(visible.world.marker(home.marker).is_some());
            for marker in &visible.world.markers {
                let site = visible.world.physical_site(marker.id).unwrap();
                assert!(known.contains(&site));
            }
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
            assert_ne!(
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
fn new_production_sites_do_not_offer_a_region_view_before_city_development() {
    let data = GameData::load().unwrap();
    let campaign = campaign(&data, 4, data.production_layout.default_seed);
    let home = campaign.factions[&campaign.player].headquarters;
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let known = engine::explored_sites(&campaign, campaign.player);
    let mut navigation = MapNavigation::default();
    let mut view = MapView::default();
    navigation.focus_site(&visible.world, home, &mut view);
    assert_eq!(navigation.scope(), MapScope::World);
    assert!(navigation
        .army_targets(&visible.world, &view, &visible.armies)
        .iter()
        .all(|target| MAP_RECT.contains(target.bounds.center())));
    assert_eq!(
        navigation.selection(),
        Some(MapSelection::Marker(
            visible.world.site(home).unwrap().marker
        ))
    );
    let region = visible.world.site(home).unwrap().marker;
    assert!(navigation
        .enter_region(&visible.world, region, &mut view)
        .is_err());
    assert_eq!(engine::explored_sites(&campaign, campaign.player), known);
}

#[test]
fn mismatched_revision_load_returns_a_recoverable_error_and_preserves_the_active_campaign() {
    let data = GameData::load().unwrap();
    let active = campaign(&data, 4, 87);
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(active.clone())), &data)
        .unwrap();
    let mut invalid = campaign(&data, 4, 88);
    invalid.world.layout_revision = 3;
    invalid.world.atlas_paths = data.production_layout.atlas_paths.clone();
    let error = state
        .load_campaign(Campaign::Strategic(Box::new(invalid)), &data)
        .unwrap_err();
    assert!(error.contains("fixed topology counts changed"));
    assert_eq!(state.campaign.unwrap().strategic().unwrap(), &active);
}

#[test]
fn production_start_requirements_and_current_revision_terrain_are_validated() {
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
