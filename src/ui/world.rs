//! Public map topology and clear world/region navigation.

use super::{components::*, selection, Context, UiAction};
use kestrum::{
    data::{
        rules::Emblem,
        world::{AnchorExpression, FactionId, MarkerLocation, MilitaryLayer, SiteId},
    },
    navigation::{MapExploration, MapScope, MapSelection, MapTarget},
    state::Overlay,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::truncate_text_to_width_ex;

mod fog;
mod routes;
pub use fog::{draw as draw_fog, exploration as map_exploration};

pub const WORLD_MAP: Rect = Rect::new(24.0, 20.0, 194.0, 48.0);

pub fn draw(ctx: &Context<'_>, exploration: Option<&MapExploration>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    let targets = ctx.navigation.targets(world, ctx.view);
    let panel = if ctx.movement.stage == super::MoveStage::Map {
        Some(super::movement_panel_bounds(
            ctx.movement,
            ctx.navigation,
            world,
            ctx.view,
        ))
    } else {
        selection::bounds(ctx.navigation, world, ctx.view)
    };
    routes::draw(ctx, exploration);
    for target in &targets {
        if panel.is_some_and(|panel| {
            panel.contains(target.center) && ctx.navigation.selection() != Some(target.selection)
        }) {
            continue;
        }
        draw_target(ctx, target);
        movement_cost(ctx, target);
        army_presence(ctx, target);
    }
    army_banners(ctx, panel);
}

fn army_banners(ctx: &Context<'_>, panel: Option<Rect>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    for target in ctx
        .navigation
        .army_targets(&campaign.world, ctx.view, &campaign.armies)
    {
        if panel.is_some_and(|panel| panel.overlaps(&target.bounds)) {
            continue;
        }
        let rect = target.bounds;
        let selected = target
            .armies
            .iter()
            .any(|id| ctx.movement.armies.contains(id));
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, INK);
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected { 3.0 } else { 1.5 },
            BRASS,
        );
        let label = if target.armies.len() == 1 {
            ctx.text("map_army")
        } else {
            format!("{} {}", ctx.text("map_army"), target.armies.len())
        };
        centered(
            ctx,
            &label,
            vec2(rect.x + rect.w / 2.0, rect.y + 30.0),
            17.0,
            CREAM,
        );
    }
}

fn movement_cost(ctx: &Context<'_>, target: &MapTarget) {
    if ctx.movement.stage != super::MoveStage::Map {
        return;
    }
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let cost = match target.selection {
        MapSelection::Site(id) => ctx.movement.nearby.get(&id),
        MapSelection::Marker(id) => ctx
            .movement
            .nearby
            .iter()
            .filter(|(site, _)| {
                campaign
                    .world
                    .site(**site)
                    .is_some_and(|site| site.marker == id)
            })
            .map(|(_, cost)| cost)
            .min(),
    };
    if let Some(cost) = cost {
        let at = target.center;
        draw_circle_lines(at.x, at.y, 29.0, 2.0, CREAM);
        let badge = at + vec2(0.0, -42.0);
        draw_circle(badge.x, badge.y, 14.0, INK);
        centered(ctx, &cost.to_string(), badge + vec2(0.0, 6.0), 17.0, CREAM);
    }
}

fn draw_target(ctx: &Context<'_>, target: &MapTarget) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    let production_world = ctx.navigation.scope() == MapScope::World && world.markers.len() > 24;
    match target.selection {
        MapSelection::Marker(id) => {
            if let Some(marker) = world.marker(id) {
                match &marker.location {
                    MarkerLocation::Site { site } => {
                        if let Some(site) = world.site(*site) {
                            let label = !production_world
                                || ctx.navigation.selection() == Some(target.selection);
                            draw_site(ctx, target, site, &marker.name, false, false, label);
                        }
                    }
                    MarkerLocation::Region { sites, .. } => {
                        let state = world.region_control(id);
                        marker_base(ctx, target, state.and_then(|state| state.political_owner));
                        draw_poly_lines(target.center.x, target.center.y, 4, 27.0, 0.0, 2.0, BRASS);
                        let mut index = 0;
                        for (owner, count) in world.controller_counts(id) {
                            for _ in 0..count {
                                let angle = std::f32::consts::TAU * index as f32
                                    / sites.len().max(1) as f32;
                                let from = target.center + vec2(angle.cos(), angle.sin()) * 28.0;
                                let to = target.center + vec2(angle.cos(), angle.sin()) * 34.0;
                                draw_line(
                                    from.x,
                                    from.y,
                                    to.x,
                                    to.y,
                                    4.0,
                                    faction_color(ctx, owner),
                                );
                                index += 1;
                            }
                        }
                        if state.is_some_and(|state| state.contested) {
                            contested(target.center);
                        }
                        let label_center = if production_world {
                            target.center - vec2(0.0, 108.0)
                        } else {
                            target.center
                        };
                        map_label(ctx, &marker.name, label_center, true);
                    }
                }
            }
        }
        MapSelection::Site(id) => {
            if let Some(site) = world.site(id) {
                let (gate, anchor) = world
                    .marker(site.marker)
                    .and_then(|marker| match &marker.location {
                        MarkerLocation::Region {
                            entrances, anchors, ..
                        } => Some((
                            entrances.iter().any(|entry| entry.site == site.id),
                            is_anchor(anchors, site.id),
                        )),
                        _ => None,
                    })
                    .unwrap_or_default();
                draw_site(ctx, target, site, &site.name, gate, anchor, true);
            }
        }
    }
}

fn army_presence(ctx: &Context<'_>, target: &MapTarget) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let physical_site = match target.selection {
        MapSelection::Site(site) => Some(site),
        MapSelection::Marker(marker) => campaign.world.physical_site(marker),
    };
    if physical_site.is_some_and(|site| {
        campaign.hostile_presence.contains(&site)
            || campaign.threats.iter().any(|threat| threat.site == site)
    }) {
        let at = target.center + vec2(-30.0, -30.0);
        draw_circle(at.x, at.y, 13.0, INK);
        draw_circle_lines(at.x, at.y, 12.0, 2.0, Color::new(0.86, 0.51, 0.39, 1.0));
        body(ctx, "!", at + vec2(-3.0, 6.0), 18.0, CREAM);
    }
}

fn marker_base(ctx: &Context<'_>, target: &MapTarget, owner: Option<FactionId>) {
    let center = target.center;
    let selected = ctx.navigation.selection() == Some(target.selection);
    if selected {
        draw_circle(center.x, center.y, 31.0, CREAM);
    }
    draw_circle(center.x, center.y, 24.0, INK);
    draw_circle_lines(center.x, center.y, 22.0, 3.0, faction_color(ctx, owner));
    if let Some(faction) = owner.and_then(|owner| {
        ctx.campaign_view?
            .factions
            .iter()
            .find(|faction| faction.id == owner)
    }) {
        let initial = match faction.emblem {
            Emblem::Rose => "R",
            Emblem::Oak => "O",
            Emblem::Fern => "F",
            Emblem::Iris => "Ir",
            Emblem::Thistle => "T",
            Emblem::Ivy => "Iv",
            Emblem::Hawthorn => "H",
            Emblem::Laurel => "L",
        };
        centered(ctx, initial, center + vec2(0.0, 7.0), 20.0, CREAM);
    } else {
        draw_circle(center.x, center.y, 3.0, MUTED);
    }
}

fn draw_site(
    ctx: &Context<'_>,
    target: &MapTarget,
    site: &kestrum::data::world::Site,
    name: &str,
    gate: bool,
    anchor: bool,
    show_label: bool,
) {
    marker_base(ctx, target, site.controller);
    let center = target.center;
    if gate {
        draw_rectangle_lines(center.x - 26.0, center.y - 27.0, 52.0, 52.0, 3.0, BRASS);
        draw_line(
            center.x - 18.0,
            center.y - 28.0,
            center.x + 18.0,
            center.y - 28.0,
            5.0,
            CREAM,
        );
    } else if site.military == MilitaryLayer::Fort {
        for offset in [-16.0, -2.0, 12.0] {
            draw_rectangle(center.x + offset, center.y - 29.0, 7.0, 8.0, CREAM);
        }
    }
    if anchor {
        draw_circle_lines(center.x, center.y, 28.0, 1.5, BRASS);
    }
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.world.contested_sites.contains(&site.id))
    {
        contested(center);
    }
    if show_label {
        map_label(ctx, name, center, gate);
    }
}

fn contested(center: Vec2) {
    draw_rectangle(center.x + 15.0, center.y - 31.0, 20.0, 20.0, INK);
    draw_line(
        center.x + 20.0,
        center.y - 27.0,
        center.x + 30.0,
        center.y - 16.0,
        2.0,
        CREAM,
    );
    draw_line(
        center.x + 30.0,
        center.y - 27.0,
        center.x + 20.0,
        center.y - 16.0,
        2.0,
        CREAM,
    );
}

fn map_label(ctx: &Context<'_>, name: &str, center: Vec2, prominent: bool) {
    let mut left = 12.0_f32;
    let mut right = 1268.0_f32;
    let mut y = center.y + if prominent { 54.0 } else { 48.0 };
    if let Some(panel) = ctx
        .campaign_view
        .and_then(|campaign| {
            if ctx.movement.stage == super::MoveStage::Map {
                Some(super::movement_panel_bounds(
                    ctx.movement,
                    ctx.navigation,
                    &campaign.world,
                    ctx.view,
                ))
            } else {
                selection::bounds(ctx.navigation, &campaign.world, ctx.view)
            }
        })
        .filter(|panel| !panel.overlaps(&Rect::new(center.x - 24.0, center.y - 24.0, 48.0, 48.0)))
    {
        if y - 20.0 < panel.y + panel.h && y + 7.0 > panel.y {
            if center.x < panel.x {
                right = panel.x - 14.0;
            } else if center.x >= panel.x + panel.w {
                left = panel.x + panel.w + 14.0;
            } else {
                // A target above the panel keeps its label above it as well.
                y = center.y - 34.0;
            }
        }
    }
    if right - left < 48.0 {
        return;
    }
    let label = truncate_text_to_width_ex(name, 190.0_f32.min(right - left), ctx.body_font(), 18.0);
    let width = measure_text(&label, ctx.body_font(), 18, 1.0).width;
    let x = (center.x - width * 0.5).clamp(left, right - width);
    draw_rectangle(
        x - 7.0,
        y - 20.0,
        width + 14.0,
        27.0,
        Color::new(0.07, 0.12, 0.12, 0.94),
    );
    body(ctx, &label, vec2(x, y), 18.0, CREAM);
}

pub fn faction_color(ctx: &Context<'_>, owner: Option<FactionId>) -> Color {
    match owner.and_then(|owner| {
        ctx.campaign_view?
            .factions
            .iter()
            .find(|faction| faction.id == owner)
    }) {
        Some(faction) => match faction.emblem {
            Emblem::Rose => Color::new(0.88, 0.51, 0.55, 1.0),
            Emblem::Oak => Color::new(0.88, 0.71, 0.38, 1.0),
            Emblem::Hawthorn => Color::new(0.54, 0.71, 0.92, 1.0),
            Emblem::Fern => Color::new(0.51, 0.81, 0.62, 1.0),
            Emblem::Iris => Color::new(0.72, 0.62, 0.91, 1.0),
            Emblem::Thistle => Color::new(0.86, 0.62, 0.87, 1.0),
            Emblem::Ivy => Color::new(0.61, 0.83, 0.76, 1.0),
            Emblem::Laurel => Color::new(0.86, 0.83, 0.55, 1.0),
        },
        None => MUTED,
    }
}

pub fn is_anchor(expression: &AnchorExpression, target: SiteId) -> bool {
    match expression {
        AnchorExpression::All { conditions } | AnchorExpression::Any { conditions } => conditions
            .iter()
            .any(|condition| is_anchor(condition, target)),
        AnchorExpression::ControlledSite { site } | AnchorExpression::SuppliedEntrance { site } => {
            *site == target
        }
    }
}

pub fn navigation(ctx: &Context<'_>) -> Option<UiAction> {
    if let MapScope::Region(id) = ctx.navigation.scope() {
        if let Some(marker) = ctx
            .campaign_view?
            .world
            .marker(id)
            .filter(|_| super::tutorial_bounds(ctx.state).is_none())
        {
            let label = truncate_text_to_width_ex(
                &format!("/ {}", marker.name),
                265.0,
                ctx.body_font(),
                20.0,
            );
            body(ctx, &label, vec2(232.0, 49.0), 20.0, CREAM);
        }
        if button(
            ctx,
            WORLD_MAP,
            &ctx.text("world_map"),
            ctx.state.overlay == Overlay::None,
            false,
        ) {
            return Some(UiAction::WorldMap);
        }
    } else if ctx.navigation.selection().is_none()
        && ctx.state.overlay == Overlay::None
        && ctx.movement.stage == super::MoveStage::Inactive
    {
        let label = ctx.text("select_place");
        let width = measure_text(&label, ctx.body_font(), 18, 1.0).width;
        if ctx
            .campaign_view
            .is_some_and(|campaign| campaign.player_turn)
        {
            body(ctx, &label, vec2(640.0 - width * 0.5, 678.0), 18.0, CREAM);
        }
    }
    None
}
