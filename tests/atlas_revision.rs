//! Authored geography changes never silently relocate a saved campaign.
use kestrum::{
    data::{
        generation::ProductionSetup,
        rules::Emblem,
        world::{MarkerLocation, RouteId, SiteId},
        GameData,
    },
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

fn old_layout(data: &GameData) -> StrategicCampaign {
    let mut old = campaign(data);
    old.world.layout_revision = 1;
    old.world.atlas_paths.clear();
    old.world.markers = data.production_layout.markers.clone();
    old.world.routes = data.production_layout.routes.clone();
    for site in &mut old.world.sites {
        let authored = data.production_layout.site(site.id).unwrap();
        site.marker = authored.marker;
        site.position = authored.position;
        site.tags.clone_from(&authored.tags);
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
    old.reconcile_region_control();
    old
}

#[test]
fn unknown_or_mixed_revisions_and_changed_geometry_are_rejected() {
    let data = GameData::load().unwrap();
    let original = campaign(&data);
    for revision in [0, 1, 2, 3, 5] {
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
        .position = [0.121, 0.55];
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
