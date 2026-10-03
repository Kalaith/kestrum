use kestrum::{
    data::{
        economy::Habitation,
        world::{MarkerId, MarkerLocation, RouteId, SiteId},
        GameData,
    },
    state::world::CampaignWorld,
};

fn city_region(data: &GameData) -> (CampaignWorld, MarkerId, SiteId) {
    let marker = data
        .scenario
        .markers
        .iter()
        .find(|marker| matches!(&marker.location, MarkerLocation::Site { .. }))
        .expect("scenario has a simple site marker");
    let MarkerLocation::Site { site } = &marker.location else {
        unreachable!();
    };
    let site = *site;
    let mut world = CampaignWorld::from_scenario(&data.scenario);
    world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .expect("marker site exists")
        .habitation = Habitation::City;
    (world, marker.id, site)
}

#[test]
fn city_regions_require_a_nonruined_city_center() {
    let data = GameData::load().unwrap();
    let (mut world, marker, center) = city_region(&data);

    assert!(world.is_region_available(marker));
    assert!(world.region_sites(marker).contains(&center));

    world
        .sites
        .iter_mut()
        .find(|site| site.id == center)
        .unwrap()
        .habitation = Habitation::Town;
    assert!(!world.is_region_available(marker));
    assert!(world.region_sites(marker).is_empty());

    world
        .sites
        .iter_mut()
        .find(|site| site.id == center)
        .unwrap()
        .habitation = Habitation::City;
    world.development.get_mut(&center).unwrap().ruined = true;
    assert!(!world.is_region_available(marker));
    assert!(world.region_sites(marker).is_empty());
    assert_eq!(world.region_site_position(marker, center), None);
}

#[test]
fn city_region_members_and_positions_follow_topology_and_survive_visibility_filtering() {
    let data = GameData::load().unwrap();
    let (mut world, marker, center) = city_region(&data);
    world
        .sites
        .iter_mut()
        .find(|site| site.id == center)
        .unwrap()
        .position = [0.5, 0.5];
    let original_neighbors = world.adjacent_sites(center);
    let near = *original_neighbors.iter().next().unwrap();
    let extra_neighbors: Vec<_> = world
        .sites
        .iter()
        .map(|site| site.id)
        .filter(|site| *site != center && !original_neighbors.contains(site))
        .take(2)
        .collect();
    assert_eq!(extra_neighbors.len(), 2);
    let far = extra_neighbors[0];
    let fallback = extra_neighbors[1];
    for (route_id, site) in [RouteId(u32::MAX), RouteId(u32::MAX - 1)]
        .into_iter()
        .zip(extra_neighbors.iter().copied())
    {
        // Add fixture-only spokes so two collinear and one coincident neighbor
        // exercise spacing and the ID fallback without changing game topology.
        let mut route = world.routes[0].clone();
        route.id = route_id;
        route.from = center;
        route.to = site;
        route.major_connection = None;
        world.routes.push(route);
    }
    for (site, position) in [
        (near, [0.6, 0.5]),
        (far, [0.8, 0.5]),
        (fallback, [0.5, 0.5]),
    ] {
        world
            .sites
            .iter_mut()
            .find(|entry| entry.id == site)
            .unwrap()
            .position = position;
    }

    let neighbors = world.adjacent_sites(center);
    assert!(!neighbors.is_empty(), "city center has authored neighbors");
    let mut expected = vec![center];
    expected.extend(neighbors.iter().copied());
    assert_eq!(world.region_sites(marker), expected);
    assert_eq!(world.region_site_position(marker, center), Some([0.5, 0.5]));

    let near_position = world.region_site_position(marker, near).unwrap();
    let far_position = world.region_site_position(marker, far).unwrap();
    let fallback_position = world.region_site_position(marker, fallback).unwrap();
    assert!(near_position[0] > 0.5 && far_position[0] > near_position[0]);
    assert_eq!(near_position[1], 0.5);
    assert_eq!(far_position[1], 0.5);
    assert_ne!(fallback_position, [0.5, 0.5]);

    let mut partial = world.clone();
    let hidden_marker = partial.site(far).unwrap().marker;
    partial.sites.retain(|site| site.id != far);
    partial
        .routes
        .retain(|route| route.from != far && route.to != far);
    partial.development.remove(&far);
    if hidden_marker != marker
        && !partial
            .sites
            .iter()
            .any(|site| site.marker == hidden_marker)
    {
        partial.markers.retain(|entry| entry.id != hidden_marker);
    }
    assert_eq!(
        partial.region_sites(marker),
        expected
            .into_iter()
            .filter(|site| *site != far)
            .collect::<Vec<_>>()
    );
    assert_eq!(partial.region_site_position(marker, far), None);
    assert_eq!(
        partial.region_site_position(marker, near),
        Some(near_position)
    );
    assert_eq!(
        partial.region_site_position(marker, fallback),
        Some(fallback_position)
    );
    assert_eq!(world.region_site_position(marker, SiteId(u32::MAX)), None);
}

#[test]
fn authored_regions_keep_declared_members_and_original_positions() {
    let data = GameData::load().unwrap();
    let world = CampaignWorld::from_scenario(&data.scenario);
    let marker = world
        .markers
        .iter()
        .find(|marker| matches!(&marker.location, MarkerLocation::Region { .. }))
        .unwrap();
    let MarkerLocation::Region { sites, .. } = &marker.location else {
        unreachable!();
    };
    let existing: Vec<_> = sites
        .iter()
        .copied()
        .filter(|site| {
            world
                .site(*site)
                .is_some_and(|entry| entry.marker == marker.id)
        })
        .collect();

    assert!(world.is_region_available(marker.id));
    assert_eq!(world.region_sites(marker.id), existing);
    for site in existing {
        assert_eq!(
            world.region_site_position(marker.id, site),
            world.site(site).map(|entry| entry.position)
        );
    }
}
