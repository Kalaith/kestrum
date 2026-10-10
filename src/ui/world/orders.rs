//! Persistent movement plans and actionable construction/supply map markers.

use super::{routes, status::*, *};
use kestrum::{
    data::world::RouteId,
    navigation::{MapScaleBand, MapScope, MapSelection},
    state::construction::ConstructionTarget,
};

struct WorkGroup {
    orders: Vec<(
        kestrum::state::construction::OrderId,
        bool,
        kestrum::data::world::SiteId,
    )>,
    at: Vec2,
}

pub(super) fn candidates(input: &MapStatusInput<'_>) -> Vec<StatusCandidate> {
    let campaign = input.campaign;
    if campaign.observer_mode {
        return Vec::new();
    }
    let mut markers = Vec::new();
    let overview = input.view.band() == MapScaleBand::Overview;
    let mut work_by_anchor: std::collections::BTreeMap<StatusAnchor, WorkGroup> =
        std::collections::BTreeMap::new();
    for order in campaign
        .construction
        .iter()
        .filter(|order| order.owner == campaign.player && order.is_open())
    {
        let (anchor, at, site) = match order.target {
            ConstructionTarget::Site(site) => {
                let Some((anchor, at)) = site_anchor(input, site) else {
                    continue;
                };
                (anchor, at, site)
            }
            ConstructionTarget::Route(route) => {
                let Some((anchor, at, site)) = route_work_anchor(input, route, order.builder)
                else {
                    continue;
                };
                (anchor, at, site)
            }
        };
        if overview {
            let entry = work_by_anchor.entry(anchor).or_insert_with(|| WorkGroup {
                orders: Vec::new(),
                at,
            });
            let paused = matches!(
                order.status,
                kestrum::state::construction::ConstructionStatus::Paused { .. }
            );
            entry.orders.push((order.id, paused, site));
        } else {
            markers.push(StatusCandidate {
                at,
                content: StatusContent::Work {
                    order: order.id,
                    count: 1,
                },
                intent: StatusIntent::Work(site),
            });
        }
    }
    for (_, group) in work_by_anchor {
        if let Some((order, _, site)) = group
            .orders
            .iter()
            .find(|(_, paused, _)| *paused)
            .or_else(|| group.orders.first())
        {
            markers.push(StatusCandidate {
                at: group.at,
                content: StatusContent::Work {
                    order: *order,
                    count: group.orders.len(),
                },
                intent: StatusIntent::Work(*site),
            });
        }
    }
    let mut supply_by_anchor: std::collections::BTreeMap<
        StatusAnchor,
        (usize, Vec2, kestrum::data::world::SiteId),
    > = std::collections::BTreeMap::new();
    for site in campaign.world.sites.iter().filter(|site| {
        site.controller == Some(campaign.player)
            && campaign.known_sites.contains(&site.id)
            && !campaign.supplied_sites.contains(&site.id)
    }) {
        let Some((anchor, at)) = site_anchor(input, site.id) else {
            continue;
        };
        let entry = supply_by_anchor.entry(anchor).or_insert((0, at, site.id));
        entry.0 += 1;
    }
    for (_, (count, at, site)) in supply_by_anchor {
        markers.push(StatusCandidate {
            at,
            content: StatusContent::Supply { count },
            intent: StatusIntent::Supply(site),
        });
    }
    markers
}

pub(super) fn draw_saved_routes(ctx: &Context<'_>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let selected = |plan: &kestrum::state::movement::MovementPlan| {
        plan.armies
            .iter()
            .any(|army| ctx.movement.armies.contains(army))
            || plan.path.first().is_some_and(|site| {
                let Some(campaign) = ctx.campaign_view else {
                    return false;
                };
                selection_matches_site(ctx.navigation, &campaign.world, *site)
            })
    };
    for plan in &campaign.movement_plans {
        let emphasized = selected(plan);
        if ctx.view.band() == MapScaleBand::Overview && !emphasized {
            continue;
        }
        for pair in plan.path.windows(2) {
            let Some(route) = campaign.world.connected_route(pair[0], pair[1]) else {
                continue;
            };
            let Some(points) = planned_route_points(ctx, route.id, pair[0], pair[1]) else {
                continue;
            };
            let color = if emphasized {
                CREAM
            } else {
                Color::new(CREAM.r, CREAM.g, CREAM.b, 0.85)
            };
            routes::stroke(&points, if emphasized { 6.0 } else { 4.5 }, INK);
            routes::stroke(&points, if emphasized { 3.0 } else { 2.0 }, color);
            direction_arrow(points.windows(2).next_back(), color);
        }
    }
}

fn planned_route_points(
    ctx: &Context<'_>,
    route_id: RouteId,
    from: kestrum::data::world::SiteId,
    to: kestrum::data::world::SiteId,
) -> Option<Vec<Vec2>> {
    let campaign = ctx.campaign_view?;
    let route = campaign.world.route(route_id)?;
    let (start, end) = routes::endpoints(ctx, &campaign.world, route)?;
    let mut waypoints: Vec<_> = if ctx.navigation.scope() == MapScope::World {
        campaign
            .world
            .atlas_paths
            .get(&route.id)
            .map(|path| path.waypoints.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if from == route.to && to == route.from {
        waypoints.reverse();
        Some(
            std::iter::once(end)
                .chain(waypoints)
                .chain(std::iter::once(start))
                .map(|point| ctx.view.project_normalized(point))
                .collect(),
        )
    } else {
        Some(
            std::iter::once(start)
                .chain(waypoints)
                .chain(std::iter::once(end))
                .map(|point| ctx.view.project_normalized(point))
                .collect(),
        )
    }
}

fn direction_arrow(segment: Option<&[Vec2]>, color: Color) {
    let Some(segment) = segment else {
        return;
    };
    if segment.len() != 2 {
        return;
    }
    let (from, to) = (&segment[0], &segment[1]);
    let direction = (*to - *from).normalize_or_zero();
    if direction.length_squared() == 0.0 {
        return;
    }
    let midpoint = from.lerp(*to, 0.68);
    let side = vec2(-direction.y, direction.x) * 7.0;
    let tail = midpoint - direction * 9.0;
    draw_triangle(midpoint + direction * 7.0, tail + side, tail - side, color);
}

pub(super) fn site_anchor(
    input: &MapStatusInput<'_>,
    site: kestrum::data::world::SiteId,
) -> Option<(StatusAnchor, Vec2)> {
    if !input.campaign.known_sites.contains(&site) {
        return None;
    }
    let world = &input.campaign.world;
    let place = world.site(site)?;
    let (selection, position) = match input.navigation.scope() {
        MapScope::World => {
            let marker = world.marker(place.marker)?;
            (MapSelection::Marker(marker.id), marker.position)
        }
        MapScope::Region(region) => (
            MapSelection::Site(site),
            world.region_site_position(region, site)?,
        ),
    };
    Some((
        StatusAnchor::Site(selection),
        input.view.project_normalized(position),
    ))
}

fn route_geometry(input: &MapStatusInput<'_>, route_id: RouteId) -> Option<(StatusAnchor, Vec2)> {
    let world = &input.campaign.world;
    let route = world.route(route_id)?;
    if !input.campaign.known_sites.contains(&route.from)
        || !input.campaign.known_sites.contains(&route.to)
    {
        return None;
    }
    let points = routes::projected_polyline(world, route, input.navigation.scope(), input.view)?;
    Some((StatusAnchor::Route(route_id), routes::midpoint(&points)))
}

pub(super) fn route_anchor(
    input: &MapStatusInput<'_>,
    route_id: RouteId,
) -> Option<(StatusAnchor, Vec2)> {
    route_geometry(input, route_id)
}

fn route_work_anchor(
    input: &MapStatusInput<'_>,
    route_id: RouteId,
    builder: Option<kestrum::state::military::ArmyId>,
) -> Option<(StatusAnchor, Vec2, kestrum::data::world::SiteId)> {
    let (anchor, at) = route_geometry(input, route_id)?;
    let route = input.campaign.world.route(route_id)?;
    let endpoint = [route.from, route.to].into_iter().find(|site_id| {
        input.campaign.known_sites.contains(site_id)
            && input
                .campaign
                .world
                .site(*site_id)
                .is_some_and(|site| site.controller == Some(input.campaign.player))
    });
    let builder_site = builder.and_then(|id| {
        input
            .campaign
            .armies
            .iter()
            .find(|army| army.id == id && army.faction == input.campaign.player)
            .map(|army| army.site)
            .filter(|site| input.campaign.known_sites.contains(site))
            .filter(|site| {
                input
                    .campaign
                    .world
                    .site(*site)
                    .is_some_and(|site| site.controller == Some(input.campaign.player))
            })
    });
    Some((anchor, at, endpoint.or(builder_site)?))
}

fn selection_matches_site(
    navigation: &kestrum::navigation::MapNavigation,
    world: &kestrum::state::world::CampaignWorld,
    site: kestrum::data::world::SiteId,
) -> bool {
    match (navigation.scope(), navigation.selection()) {
        (MapScope::World, Some(MapSelection::Marker(marker))) => {
            world.site(site).is_some_and(|place| place.marker == marker)
        }
        (MapScope::Region(_), Some(MapSelection::Site(selected))) => selected == site,
        _ => false,
    }
}
