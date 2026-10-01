//! Silhouettes express places; separate marks express control and known danger.

use super::*;
use kestrum::data::{
    economy::Habitation,
    world::{MarkerLocation, MilitaryLayer, Site, SiteTag},
};

pub(super) fn draw_target(ctx: &Context<'_>, target: &MapTarget) {
    let (Some(view), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return;
    };
    match target.selection {
        MapSelection::Marker(id) => {
            let Some(marker) = view.world.marker(id) else {
                return;
            };
            match &marker.location {
                MarkerLocation::Site { site } => {
                    if let Some(site) = view.world.site(*site) {
                        draw_site(ctx, target, site);
                    }
                }
                MarkerLocation::Region { sites, .. } => {
                    let Some(summary) = overview.markers.get(&id) else {
                        return;
                    };
                    base(ctx, target, summary.political_owner);
                    draw_poly_lines(target.center.x, target.center.y, 4, 28.0, 0.0, 2.0, BRASS);
                    silhouette(target.center, summary.habitation);
                    if !summary.political_known {
                        badge(ctx, target.center + vec2(-22.0, 19.0), "?", MUTED);
                    }
                    let mut index = 0;
                    for (owner, count) in view.world.controller_counts(id) {
                        for _ in 0..count {
                            let angle =
                                std::f32::consts::TAU * index as f32 / sites.len().max(1) as f32;
                            let direction = vec2(angle.cos(), angle.sin());
                            let from = target.center + direction * 28.0;
                            let to = target.center + direction * 34.0;
                            draw_line(from.x, from.y, to.x, to.y, 3.0, faction_color(ctx, owner));
                            index += 1;
                        }
                    }
                    control_marks(
                        ctx,
                        target.center,
                        summary.capital,
                        summary.occupied,
                        summary.contested,
                    );
                }
            }
        }
        MapSelection::Site(id) => {
            if let Some(site) = view.world.site(id) {
                draw_site(ctx, target, site);
            }
        }
    }
}

fn base(ctx: &Context<'_>, target: &MapTarget, owner: Option<FactionId>) {
    let at = target.center;
    if ctx.navigation.selection() == Some(target.selection) {
        draw_circle(at.x, at.y, 28.0, CREAM);
    }
    draw_circle(at.x, at.y, 24.0, INK);
    draw_circle_lines(at.x, at.y, 22.0, 3.0, faction_color(ctx, owner));
}

fn draw_site(ctx: &Context<'_>, target: &MapTarget, site: &Site) {
    let Some(view) = ctx.campaign_view else {
        return;
    };
    let Some(summary) = ctx
        .overview
        .and_then(|overview| overview.sites.get(&site.id))
    else {
        return;
    };
    base(ctx, target, site.controller);
    silhouette(target.center, site.habitation);
    if site.military == MilitaryLayer::Fort {
        for offset in [-16.0, -2.0, 12.0] {
            draw_rectangle(
                target.center.x + offset,
                target.center.y - 27.0,
                7.0,
                7.0,
                CREAM,
            );
        }
    }
    if ctx.navigation.scope() != MapScope::World {
        if let Some(MarkerLocation::Region {
            entrances, anchors, ..
        }) = view
            .world
            .marker(site.marker)
            .map(|marker| &marker.location)
        {
            if entrances.iter().any(|entry| entry.site == site.id) {
                draw_rectangle_lines(
                    target.center.x - 26.0,
                    target.center.y - 27.0,
                    52.0,
                    52.0,
                    2.0,
                    BRASS,
                );
            }
            if is_anchor(anchors, site.id) {
                draw_circle_lines(target.center.x, target.center.y, 29.0, 1.5, CREAM);
            }
        }
    }
    control_marks(
        ctx,
        target.center,
        summary.capital,
        summary.occupied,
        summary.contested,
    );
    if ctx.navigation.scope() != MapScope::World || ctx.view.camera.zoom() >= 1.8 {
        let key = if view.construction.iter().any(|order| {
            order.is_open() && matches!(order.target, kestrum::state::construction::ConstructionTarget::Site(id) if id == site.id)
        }) { Some("work") }
        else if view.world.structural_damage(site.id) > 0 { Some("damage") }
        else if site.tags.contains(&SiteTag::WoodSource) { Some("wood") }
        else if site.tags.contains(&SiteTag::StoneSource) { Some("stone") }
        else { None };
        if let Some(key) = key {
            let label = ctx.data.map.text(key);
            let width = measure_text(label, ctx.body_font(), 14, 1.0).width;
            let at = target.center + vec2(-width * 0.5, 11.0);
            draw_rectangle(at.x - 3.0, at.y + 11.0, width + 6.0, 18.0, INK);
            body(ctx, label, at + vec2(0.0, 25.0), 14.0, BRASS);
        }
    }
}

fn silhouette(at: Vec2, habitation: Habitation) {
    match habitation {
        Habitation::Unsettled => {
            draw_circle(at.x, at.y, 3.0, MUTED);
        }
        Habitation::Camp => {
            draw_triangle(
                at + vec2(-12.0, 10.0),
                at + vec2(0.0, -11.0),
                at + vec2(12.0, 10.0),
                CREAM,
            );
            draw_triangle(
                at + vec2(-4.0, 10.0),
                at + vec2(0.0, -1.0),
                at + vec2(4.0, 10.0),
                INK,
            );
        }
        Habitation::Outpost => {
            draw_rectangle(at.x - 6.0, at.y - 9.0, 12.0, 21.0, CREAM);
            draw_rectangle(at.x - 10.0, at.y - 13.0, 20.0, 6.0, CREAM);
            draw_rectangle(at.x - 2.0, at.y + 2.0, 4.0, 10.0, INK);
        }
        Habitation::Hamlet => house(at, 1.0),
        Habitation::Village => {
            house(at + vec2(-7.0, 4.0), 0.65);
            house(at + vec2(7.0, -4.0), 0.65);
        }
        Habitation::Town => {
            house(at + vec2(-9.0, 4.0), 0.6);
            house(at + vec2(9.0, 4.0), 0.6);
            house(at + vec2(0.0, -5.0), 0.75);
        }
        Habitation::City | Habitation::MajorCity => {
            draw_rectangle(at.x - 14.0, at.y - 3.0, 28.0, 15.0, CREAM);
            for offset in [-13.0, 0.0, 13.0] {
                let height = if offset == 0.0 { 25.0 } else { 19.0 };
                draw_rectangle(
                    at.x + offset - 3.0,
                    at.y + 12.0 - height,
                    6.0,
                    height,
                    CREAM,
                );
            }
            draw_rectangle(at.x - 3.0, at.y + 3.0, 6.0, 9.0, INK);
            if habitation == Habitation::MajorCity {
                draw_line(
                    at.x - 17.0,
                    at.y + 16.0,
                    at.x + 17.0,
                    at.y + 16.0,
                    2.0,
                    CREAM,
                );
            }
        }
    }
}

fn house(at: Vec2, scale: f32) {
    draw_rectangle(
        at.x - 9.0 * scale,
        at.y - 2.0 * scale,
        18.0 * scale,
        14.0 * scale,
        CREAM,
    );
    draw_triangle(
        at + vec2(-12.0, -2.0) * scale,
        at + vec2(0.0, -14.0) * scale,
        at + vec2(12.0, -2.0) * scale,
        CREAM,
    );
    draw_rectangle(
        at.x - 2.0 * scale,
        at.y + 4.0 * scale,
        4.0 * scale,
        8.0 * scale,
        INK,
    );
}

fn control_marks(ctx: &Context<'_>, center: Vec2, capital: bool, occupied: bool, contested: bool) {
    if capital {
        let at = center + vec2(0.0, -36.0);
        draw_rectangle(at.x - 15.0, at.y - 9.0, 30.0, 21.0, INK);
        for offset in [-9.0, 0.0, 9.0] {
            draw_triangle(
                at + vec2(offset - 4.0, 5.0),
                at + vec2(offset, -6.0),
                at + vec2(offset + 4.0, 5.0),
                CREAM,
            );
        }
        draw_rectangle(at.x - 12.0, at.y + 4.0, 24.0, 4.0, CREAM);
    }
    if occupied {
        badge(ctx, center + vec2(22.0, 20.0), "/", BRASS);
    }
    if contested {
        badge(ctx, center + vec2(22.0, -21.0), "X", CREAM);
    }
}

pub(super) fn danger(ctx: &Context<'_>, target: &MapTarget) {
    let Some(overview) = ctx.overview else {
        return;
    };
    let danger = match target.selection {
        MapSelection::Marker(id) => overview.markers.get(&id).map(|summary| summary.danger),
        MapSelection::Site(id) => overview.sites.get(&id).map(|summary| summary.danger),
    };
    if let Some(danger) = danger.filter(|danger| danger.any()) {
        let at = target.center + vec2(-25.0, -25.0);
        // The number counts known conditions, never armies or enemy strength.
        let label = if danger.count() > 1 {
            format!("!{}", danger.count())
        } else {
            "!".into()
        };
        badge(ctx, at, &label, Color::new(0.96, 0.64, 0.49, 1.0));
    }
}

fn badge(ctx: &Context<'_>, at: Vec2, label: &str, color: Color) {
    let width = measure_text(label, ctx.body_font(), 16, 1.0)
        .width
        .max(14.0)
        + 8.0;
    draw_rectangle(at.x - width * 0.5, at.y - 12.0, width, 24.0, INK);
    draw_rectangle_lines(at.x - width * 0.5, at.y - 12.0, width, 24.0, 1.0, color);
    let text_width = measure_text(label, ctx.body_font(), 16, 1.0).width;
    body(ctx, label, at + vec2(-text_width * 0.5, 6.0), 16.0, color);
}
