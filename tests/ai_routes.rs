//! The batched planner graph must agree with the original destination search.
use kestrum::{
    data::{
        generation::ProductionSetup,
        rules::Emblem,
        world::{FactionId, Road, Route, RouteId, SiteId},
        GameData,
    },
    engine::{self, ai::ObservedRoutes, VisibleCampaign},
    state::StrategicCampaign,
};
use std::collections::{BTreeMap, BTreeSet};

fn world() -> (GameData, VisibleCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production(
        &data,
        &ProductionSetup {
            kingdom_name: "Road survey".into(),
            emblem: Emblem::Rose,
            factions: 8,
            seed: 180_018,
        },
    )
    .unwrap();
    let view = engine::project(&campaign, campaign.player).unwrap();
    (data, view)
}

fn original_search(
    view: &VisibleCampaign,
    data: &GameData,
    enemies: &BTreeSet<FactionId>,
    origin: SiteId,
    destination: SiteId,
    attack: bool,
) -> Option<(u32, Vec<SiteId>)> {
    let mut best = BTreeMap::from([(origin, (0_u32, vec![origin]))]);
    let mut settled = BTreeSet::new();
    loop {
        let (&site, (cost, path)) = best
            .iter()
            .filter(|(id, _)| !settled.contains(*id))
            .min_by(|a, b| a.1.cmp(b.1))?;
        let (cost, path) = (*cost, path.clone());
        if site == destination {
            return Some((cost, path));
        }
        settled.insert(site);
        for next in view.world.adjacent_sites(site) {
            let terminal =
                view.hostile_presence.contains(&next) || view.world.contested_sites.contains(&next);
            let allowed = !view.threats.iter().any(|threat| threat.site == next)
                && (!terminal || (attack && next == destination))
                && view.world.site(next).is_some_and(|site| {
                    site.controller
                        .is_none_or(|owner| owner == view.observer || enemies.contains(&owner))
                });
            if settled.contains(&next) || !allowed {
                continue;
            }
            let route = view.world.connected_route(site, next)?;
            let next_cost = cost.checked_add(engine::route_cost(route, data))?;
            let mut next_path = path.clone();
            next_path.push(next);
            let candidate = (next_cost, next_path);
            if best.get(&next).is_none_or(|old| candidate < *old) {
                best.insert(next, candidate);
            }
        }
    }
}

fn compare(view: &VisibleCampaign, data: &GameData, enemies: &BTreeSet<FactionId>, origin: SiteId) {
    let routes = ObservedRoutes::new(view, data, enemies);
    for attack in [false, true] {
        for site in &view.world.sites {
            assert_eq!(
                routes.path(origin, site.id, attack),
                original_search(view, data, enemies, origin, site.id, attack),
                "origin={origin:?} destination={:?} attack={attack}",
                site.id
            );
        }
    }
}

#[test]
fn production_routes_preserve_peaceful_paths_and_unreachable_destinations() {
    let (data, view) = world();
    compare(&view, &data, &BTreeSet::new(), view.armies[0].site);
    compare(&view, &data, &BTreeSet::new(), SiteId(1));
}

#[test]
fn hostile_and_contested_destinations_never_become_transit_sites() {
    let (data, mut view) = world();
    let origin = view.armies[0].site;
    let adjacent: Vec<_> = view.world.adjacent_sites(origin).into_iter().collect();
    view.hostile_presence.insert(adjacent[0]);
    view.world.contested_sites.insert(adjacent[1]);
    let enemies = (1..=8).map(FactionId).collect();
    compare(&view, &data, &enemies, origin);
}

#[test]
fn observed_threats_block_attacks_and_travel_but_allow_departure() {
    let (data, view) = world();
    let threat = view.threats.first().expect("production local threat");
    let origin = *view.world.adjacent_sites(threat.site).first().unwrap();
    let enemies = (1..=8).map(FactionId).collect();
    let routes = ObservedRoutes::new(&view, &data, &enemies);
    assert!(routes.path(origin, threat.site, true).is_none());
    compare(&view, &data, &enemies, threat.site);
}

#[test]
fn equal_cost_paths_keep_the_lexicographic_site_tie_break() {
    let (data, mut view) = world();
    view.world.sites.retain(|site| site.id.0 <= 4);
    for site in &mut view.world.sites {
        site.controller = None;
    }
    view.threats.clear();
    view.hostile_presence.clear();
    view.world.contested_sites.clear();
    view.world.routes = [(1, 3), (3, 4), (1, 2), (2, 4)]
        .into_iter()
        .enumerate()
        .map(|(i, (from, to))| Route {
            id: RouteId(i as u32 + 1),
            from: SiteId(from),
            to: SiteId(to),
            terrain_cost: 1,
            road: Road {
                improved: false,
                damage: 0,
            },
            major_connection: None,
        })
        .collect();
    let routes = ObservedRoutes::new(&view, &data, &BTreeSet::new());
    assert_eq!(
        routes.path(SiteId(1), SiteId(4), true).unwrap().1,
        vec![SiteId(1), SiteId(2), SiteId(4)]
    );
    compare(&view, &data, &BTreeSet::new(), SiteId(1));
}

#[test]
fn rebuilding_after_an_order_replaces_cached_territory_permissions() {
    let (data, mut view) = world();
    let origin = view.armies[0].site;
    let destination = *view.world.adjacent_sites(origin).first().unwrap();
    view.threats.retain(|threat| threat.site != destination);
    view.world
        .sites
        .iter_mut()
        .find(|site| site.id == destination)
        .unwrap()
        .controller = None;
    let before = ObservedRoutes::new(&view, &data, &BTreeSet::new());
    assert!(before.path(origin, destination, false).is_some());
    view.world
        .sites
        .iter_mut()
        .find(|site| site.id == destination)
        .unwrap()
        .controller = Some(FactionId(2));
    let peaceful = ObservedRoutes::new(&view, &data, &BTreeSet::new());
    assert!(peaceful.path(origin, destination, true).is_none());
    let war = ObservedRoutes::new(&view, &data, &BTreeSet::from([FactionId(2)]));
    assert!(war.path(origin, destination, true).is_some());
    assert!(before.path(origin, destination, false).is_some());
}
