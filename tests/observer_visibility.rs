//! Observer map projections reveal campaign state without changing faction knowledge.

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, GameData},
    engine,
    navigation::{MapNavigation, MapScope, MapSelection, MapView},
    state::StrategicCampaign,
};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production(&data, &setup(&data)).unwrap();
    (data, campaign)
}

fn observer_fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production_observer(&data, &setup(&data)).unwrap();
    (data, campaign)
}

fn setup(data: &GameData) -> ProductionSetup {
    ProductionSetup {
        kingdom_name: "Rose".into(),
        emblem: Emblem::Rose,
        factions: 4,
        seed: data.production_layout.default_seed,
    }
}

#[test]
fn observer_map_shows_the_full_atlas_without_recording_exploration() {
    let (data, campaign) = fixture();
    let knowledge = campaign.knowledge.clone();
    let ordinary = engine::project_map(&campaign, campaign.player).unwrap();
    let undiscovered = campaign
        .world
        .sites
        .iter()
        .find(|site| !ordinary.known_sites.contains(&site.id))
        .expect("scenario begins with undiscovered geography")
        .id;

    let campaign = StrategicCampaign::new_production_observer(&data, &setup(&data)).unwrap();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let all_sites: std::collections::BTreeSet<_> =
        campaign.world.sites.iter().map(|site| site.id).collect();

    assert!(visible.full_map_visibility && visible.observer_mode);
    assert!(!visible.player_turn);
    assert!(visible.known_sites.contains(&undiscovered));
    assert_eq!(visible.known_sites, all_sites);
    assert_eq!(visible.world.sites, campaign.world.sites);
    assert_eq!(visible.world.routes, campaign.world.routes);
    assert_eq!(campaign.knowledge, knowledge);
}

#[test]
fn faction_ai_projection_keeps_its_knowledge_boundary_in_observer_mode() {
    let (_, campaign) = observer_fixture();
    let view = engine::project(&campaign, campaign.player).unwrap();
    let foreign = campaign
        .factions
        .keys()
        .copied()
        .find(|id| *id != campaign.player)
        .unwrap();
    let foreign_armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == foreign)
        .collect();

    assert!(view.observer_mode);
    assert!(!view.full_map_visibility);
    assert!(view
        .armies
        .iter()
        .all(|army| army.faction == campaign.player));
    assert!(foreign_armies
        .iter()
        .all(|army| !view.armies.iter().any(|visible| visible.id == army.id)));
    let foreign_view = view
        .factions
        .iter()
        .find(|faction| faction.id == foreign)
        .unwrap();
    assert!(foreign_view.resources.is_none());
    assert!(foreign_view.headquarters.is_none());
    assert!(view.faction_supplied_sites.is_empty());
}

#[test]
fn observer_map_exposes_all_faction_rosters_and_private_inspection_data() {
    let (_, campaign) = observer_fixture();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();

    assert_eq!(visible.factions.len(), campaign.factions.len());
    assert_eq!(visible.armies.len(), campaign.armies.len());
    assert_eq!(visible.formations.len(), campaign.formations.len());
    assert_eq!(visible.people.len(), campaign.people.len());
    assert_eq!(visible.world.population, campaign.world.population);
    assert_eq!(visible.world.focus, campaign.world.focus);
    assert_eq!(visible.construction.len(), campaign.construction.len());
    assert_eq!(visible.battles.len(), campaign.battles.len());

    for faction in campaign.factions.values() {
        let inspected = visible
            .factions
            .iter()
            .find(|entry| entry.id == faction.id)
            .unwrap();
        assert_eq!(inspected.resources, Some(faction.resources));
        assert_eq!(inspected.headquarters, Some(faction.headquarters));
        assert_eq!(inspected.capital, Some(faction.capital));
    }
    assert_eq!(
        visible.threats.len(),
        campaign
            .threats
            .values()
            .filter(|threat| threat.status == kestrum::state::threat::ThreatStatus::Active)
            .count()
    );
    assert!(visible
        .threats
        .iter()
        .all(|threat| threat.headcount.is_some()));
}

#[test]
fn map_overview_counts_and_labels_every_faction_army() {
    let (_, campaign) = observer_fixture();
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let overview = engine::map_overview(&visible);

    assert_eq!(overview.armies.len(), campaign.armies.len());
    assert_eq!(
        visible.faction_supplied_sites.len(),
        campaign.factions.len()
    );
    for army in campaign.armies.values() {
        let summary = &overview.armies[&army.id];
        let expected_troops: u32 = army
            .formation_ids()
            .filter_map(|id| campaign.formations.get(&id))
            .map(|formation| formation.headcount)
            .sum();
        assert_eq!(summary.faction, army.faction);
        assert_eq!(summary.site, army.site);
        assert_eq!(summary.troops, expected_troops);
        assert_eq!(summary.supplied, campaign.army_is_supplied(army.id));
    }
    for faction in campaign.factions.keys() {
        assert_eq!(
            visible.faction_supplied_sites[faction],
            campaign.supplied_sites(*faction)
        );
    }
}

#[test]
fn foreign_armies_can_be_targeted_on_world_and_regional_maps() {
    let (_, campaign) = observer_fixture();
    let foreign_army = campaign
        .armies
        .values()
        .find(|army| army.faction != campaign.player)
        .unwrap();
    let site = foreign_army.site;
    let foreign_army_id = foreign_army.id;
    let foreign_faction = foreign_army.faction;
    let visible = engine::project_map(&campaign, campaign.player).unwrap();
    let marker = visible.world.site(site).unwrap().marker;
    let mut navigation = MapNavigation::default();
    let mut map_view = MapView::default();
    navigation.toggle_overview(&visible.world, &mut map_view);

    assert!(navigation
        .army_targets(&visible.world, &map_view, &visible.armies)
        .iter()
        .any(|target| target.armies.contains(&foreign_army_id)));
    assert!(navigation
        .armies_at_selection(&visible.world, site, &visible.armies)
        .contains(&foreign_army_id));
    navigation
        .enter_region(&visible.world, marker, &mut map_view)
        .unwrap();
    map_view.focus(
        visible.world.site(site).unwrap().position,
        map_view.working_zoom(),
    );
    assert_eq!(navigation.scope(), MapScope::Region(marker));
    assert!(navigation
        .army_targets(&visible.world, &map_view, &visible.armies)
        .iter()
        .any(|target| target.armies.contains(&foreign_army_id)));
    navigation
        .select(&visible.world, MapSelection::Site(site))
        .unwrap();
    assert!(visible
        .armies
        .iter()
        .any(|army| army.id == foreign_army_id && army.faction == foreign_faction));
}

#[test]
fn ordinary_map_projection_still_filters_unknown_land_and_foreign_forces() {
    let (_, campaign) = fixture();
    let ordinary = engine::project_map(&campaign, campaign.player).unwrap();

    assert!(!ordinary.full_map_visibility && !ordinary.observer_mode);
    assert!(ordinary.world.sites.len() < campaign.world.sites.len());
    assert!(ordinary
        .world
        .sites
        .iter()
        .all(|site| ordinary.known_sites.contains(&site.id)));
    assert!(ordinary
        .armies
        .iter()
        .all(|army| army.faction == ordinary.observer));
}
