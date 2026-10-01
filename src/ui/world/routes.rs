//! The normal road, selected route and bridge marks share authored atlas geometry.
use super::*;
use kestrum::{data::world::Route, state::world::CampaignWorld};

pub(super) fn draw(ctx: &Context<'_>, exploration: Option<&MapExploration>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    let overview = ctx.navigation.scope() == MapScope::World;
    if let Some(area) = exploration {
        for points in &area.connection_hints {
            frontier(ctx, area, points);
        }
    }
    for route in &world.routes {
        let Some((from, to)) = endpoints(ctx, world, route) else {
            continue;
        };
        let geometry = overview.then(|| world.atlas_paths.get(&route.id)).flatten();
        let points = std::iter::once(from)
            .chain(
                geometry
                    .into_iter()
                    .flat_map(|path| path.waypoints.iter().copied()),
            )
            .chain(std::iter::once(to))
            .map(|point| ctx.view.project_normalized(point))
            .collect::<Vec<_>>();
        let production = overview && world.markers.len() > 24;
        stroke(&points, if production { 3.0 } else { 7.0 }, INK);
        stroke(&points, if production { 1.0 } else { 2.5 }, BRASS);
        selected_route(ctx, route, &points);
        if let Some(geometry) = geometry {
            for position in &geometry.bridges {
                bridge(ctx.view.project_normalized(*position), &points);
            }
        }
    }
}

fn frontier(ctx: &Context<'_>, exploration: &MapExploration, points: &[Vec2]) {
    // Sample authored bends in atlas space and fade the road into the same cover
    // as the terrain. Unknown endpoints and entirely hidden roads stay concealed.
    for segment in points.windows(2) {
        let steps = (segment[0].distance(segment[1]) / 6.0).ceil().max(1.0) as u32;
        for step in 0..steps {
            let start = segment[0].lerp(segment[1], step as f32 / steps as f32);
            let end = segment[0].lerp(segment[1], (step + 1) as f32 / steps as f32);
            let alpha = 1.0 - exploration.opacity((start + end) * 0.5);
            if alpha <= 0.0 {
                continue;
            }
            let projected = [ctx.view.project(start), ctx.view.project(end)];
            stroke(&projected, 3.0, Color::new(INK.r, INK.g, INK.b, alpha));
            stroke(
                &projected,
                1.0,
                Color::new(BRASS.r, BRASS.g, BRASS.b, alpha),
            );
        }
    }
}

fn endpoints(
    ctx: &Context<'_>,
    world: &CampaignWorld,
    route: &Route,
) -> Option<([f32; 2], [f32; 2])> {
    match ctx.navigation.scope() {
        MapScope::World => route.major_connection.and_then(|[from, to]| {
            world
                .marker(from)
                .zip(world.marker(to))
                .map(|(a, b)| (a.position, b.position))
        }),
        MapScope::Region(region) => world
            .site(route.from)
            .zip(world.site(route.to))
            .filter(|(a, b)| a.marker == region && b.marker == region)
            .map(|(a, b)| (a.position, b.position)),
    }
}

fn stroke(points: &[Vec2], width: f32, color: Color) {
    for segment in points.windows(2) {
        draw_line(
            segment[0].x,
            segment[0].y,
            segment[1].x,
            segment[1].y,
            width,
            color,
        );
    }
}

fn selected_route(ctx: &Context<'_>, route: &Route, points: &[Vec2]) {
    if ctx.movement.stage == super::super::MoveStage::Inactive {
        return;
    }
    let Some(preview) = ctx.movement.preview.as_ref() else {
        return;
    };
    let Some((index, step)) = preview
        .steps
        .iter()
        .enumerate()
        .find(|(_, step)| step.route == route.id)
    else {
        return;
    };
    let color = if index < preview.reachable_steps {
        CREAM
    } else if preview.blocked.is_none() {
        BRASS
    } else {
        Color::new(0.86, 0.51, 0.39, 1.0)
    };
    stroke(points, 5.0, color);
    let center = midpoint(points);
    draw_circle(center.x, center.y, 14.0, INK);
    body(
        ctx,
        &step.cost.to_string(),
        center + vec2(-5.0, 6.0),
        16.0,
        color,
    );
}

fn midpoint(points: &[Vec2]) -> Vec2 {
    let mut remaining = points.windows(2).map(|p| p[0].distance(p[1])).sum::<f32>() * 0.5;
    for segment in points.windows(2) {
        let length = segment[0].distance(segment[1]);
        if remaining <= length && length > 0.0 {
            return segment[0].lerp(segment[1], remaining / length);
        }
        remaining -= length;
    }
    points[0]
}

fn bridge(position: Vec2, points: &[Vec2]) {
    let Some((center, direction, _)) = points
        .windows(2)
        .filter_map(|segment| {
            let delta = segment[1] - segment[0];
            let length = delta.length_squared();
            if length == 0.0 {
                return None;
            }
            let t = ((position - segment[0]).dot(delta) / length).clamp(0.0, 1.0);
            let center = segment[0] + delta * t;
            Some((center, delta.normalize(), center.distance_squared(position)))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
    else {
        return;
    };
    let side = vec2(-direction.y, direction.x) * 4.0;
    let start = center - direction * 7.0;
    let end = center + direction * 7.0;
    draw_line(start.x, start.y, end.x, end.y, 12.0, INK);
    for offset in [-1.0, 1.0] {
        let a = start + side * offset;
        let b = end + side * offset;
        draw_line(a.x, a.y, b.x, b.y, 1.5, CREAM);
    }
    for offset in [-5.0, 0.0, 5.0] {
        let center = center + direction * offset;
        let a = center - side;
        let b = center + side;
        draw_line(a.x, a.y, b.x, b.y, 1.5, BRASS);
    }
}
