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
                    draw_poly_lines(target.center.x, target.center.y, 4, 17.0, 0.0, 1.5, BRASS);
                    silhouette(target.center, summary.habitation);
                    if !summary.political_known {
                        badge(ctx, target.center + vec2(-17.0, 15.0), "?", MUTED);
                    }
                    let mut index = 0;
                    for (owner, count) in view
                        .world
                        .controller_counts(id)
                        .into_iter()
                        .filter(|_| ctx.view.band() != MapScaleBand::Overview)
                    {
                        for _ in 0..count {
                            let angle =
                                std::f32::consts::TAU * index as f32 / sites.len().max(1) as f32;
                            let direction = vec2(angle.cos(), angle.sin());
                            let from = target.center + direction * 17.0;
                            let to = target.center + direction * 21.0;
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
        draw_circle_lines(at.x, at.y, 21.0, 2.0, CREAM);
    }
    draw_circle(at.x, at.y, 14.0, INK);
    draw_circle_lines(at.x, at.y, 12.5, 2.0, faction_color(ctx, owner));
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
    let quiet = ctx.view.band() == MapScaleBand::Overview
        && !summary.capital
        && !summary.danger.any()
        && ctx.navigation.selection() != Some(target.selection)
        && site.habitation < Habitation::Town;
    if quiet {
        draw_circle(target.center.x, target.center.y, 5.5, INK);
        draw_circle(
            target.center.x,
            target.center.y,
            3.5,
            faction_color(ctx, site.controller),
        );
        control_marks(
            ctx,
            target.center,
            summary.capital,
            summary.occupied,
            summary.contested,
        );
        return;
    }
    base(ctx, target, site.controller);
    silhouette(target.center, site.habitation);
    if site.military == MilitaryLayer::Fort {
        for offset in [-8.0, -1.0, 6.0] {
            draw_rectangle(
                target.center.x + offset,
                target.center.y - 16.0,
                4.0,
                4.0,
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
                    target.center.x - 18.0,
                    target.center.y - 16.0,
                    36.0,
                    36.0,
                    2.0,
                    BRASS,
                );
            }
            if is_anchor(anchors, site.id) {
                draw_circle_lines(target.center.x, target.center.y, 20.0, 1.5, CREAM);
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
    if ctx.view.band() == MapScaleBand::Detail {
        let key = if view.construction.iter().any(|order| {
            order.is_open() && matches!(order.target, kestrum::state::construction::ConstructionTarget::Site(id) if id == site.id)
        }) { Some("work") }
        else if view.world.structural_damage(site.id) > 0 { Some("damage") }
        else if site.tags.contains(&SiteTag::WoodSource) { Some("wood") }
        else if site.tags.contains(&SiteTag::StoneSource) { Some("stone") }
        else { None };
        if let Some(key) = key {
            let label = ctx.data.map.text(key);
            let width = measure_text(label, ctx.body_font(), 16, 1.0).width;
            let at = target.center + vec2(-width * 0.5, 11.0);
            draw_rectangle(at.x - 3.0, at.y + 8.0, width + 6.0, 23.0, INK);
            body(ctx, label, at + vec2(0.0, 25.0), 16.0, BRASS);
        }
    }
}

fn silhouette(at: Vec2, habitation: Habitation) {
    match habitation {
        Habitation::Unsettled => {
            draw_circle(at.x, at.y, 1.65, MUTED);
        }
        Habitation::Camp => {
            draw_triangle(
                at + vec2(-6.6, 5.5),
                at + vec2(0.0, -6.05),
                at + vec2(6.6, 5.5),
                CREAM,
            );
            draw_triangle(
                at + vec2(-2.2, 5.5),
                at + vec2(0.0, -1.0),
                at + vec2(2.2, 5.5),
                INK,
            );
        }
        Habitation::Outpost => {
            draw_rectangle(at.x - 3.3, at.y - 4.95, 6.6, 11.55, CREAM);
            draw_rectangle(at.x - 5.5, at.y - 7.15, 11.0, 3.3, CREAM);
            draw_rectangle(at.x - 1.1, at.y + 1.1, 2.2, 5.5, INK);
        }
        Habitation::Hamlet => house(at, 0.55),
        Habitation::Village => {
            house(at + vec2(-3.85, 2.2), 0.36);
            house(at + vec2(3.85, -2.2), 0.36);
        }
        Habitation::Town => {
            house(at + vec2(-4.95, 2.2), 0.33);
            house(at + vec2(4.95, 2.2), 0.33);
            house(at + vec2(0.0, -2.75), 0.41);
        }
        Habitation::City | Habitation::MajorCity => {
            draw_rectangle(at.x - 7.7, at.y - 1.65, 15.4, 8.25, CREAM);
            for offset in [-7.15, 0.0, 7.15] {
                let height = if offset == 0.0 { 13.75 } else { 10.45 };
                draw_rectangle(
                    at.x + offset - 1.65,
                    at.y + 6.6 - height,
                    3.3,
                    height,
                    CREAM,
                );
            }
            draw_rectangle(at.x - 1.65, at.y + 1.65, 3.3, 4.95, INK);
            if habitation == Habitation::MajorCity {
                draw_line(at.x - 9.35, at.y + 8.8, at.x + 9.35, at.y + 8.8, 1.1, CREAM);
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
        let at = center + vec2(0.0, -25.0);
        draw_rectangle(at.x - 11.0, at.y - 8.0, 22.0, 17.0, INK);
        for offset in [-6.0, 0.0, 6.0] {
            draw_triangle(
                at + vec2(offset - 3.0, 4.0),
                at + vec2(offset, -5.0),
                at + vec2(offset + 3.0, 4.0),
                CREAM,
            );
        }
        draw_rectangle(at.x - 9.0, at.y + 3.0, 18.0, 3.0, CREAM);
    }
    if occupied {
        badge(ctx, center + vec2(17.0, 16.0), "/", BRASS);
    }
    if contested {
        badge(ctx, center + vec2(17.0, -16.0), "X", CREAM);
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
        let at = target.center + vec2(-20.0, -18.0);
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
