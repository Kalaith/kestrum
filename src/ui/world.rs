//! Public map topology and clear world/region navigation.

use super::{components::*, selection, Context, UiAction};
use kestrum::{
    data::{
        rules::Emblem,
        world::{AnchorExpression, FactionId, SiteId},
    },
    navigation::{MapExploration, MapScope, MapSelection, MapTarget},
    state::Overlay,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::truncate_text_to_width_ex;

mod armies;
mod fog;
mod labels;
mod routes;
mod symbols;
mod territory;
pub use fog::{draw as draw_fog, exploration as map_exploration};

pub const WORLD_MAP: Rect = Rect::new(24.0, 20.0, 194.0, 48.0);

/// Drawing and release picking reject an occluded banner as one whole control.
pub fn banner_visible(bounds: Rect, panel: Option<Rect>, attention: Rect) -> bool {
    !panel.is_some_and(|panel| panel.overlaps(&bounds)) && !attention.overlaps(&bounds)
}

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
    territory::draw(ctx, exploration);
    routes::draw(ctx, exploration);
    for target in &targets {
        if panel.is_some_and(|panel| {
            panel.contains(target.center) && ctx.navigation.selection() != Some(target.selection)
        }) {
            continue;
        }
        symbols::draw_target(ctx, target);
        movement_cost(ctx, target);
        symbols::danger(ctx, target);
    }
    armies::draw(ctx, panel);
    labels::draw(ctx, &targets, panel, exploration);
    if panel.is_none() {
        let legend = ctx.data.map.text("legend");
        let width = measure_text(legend, ctx.body_font(), 16, 1.0).width;
        draw_rectangle(
            20.0,
            65.0,
            width + 12.0,
            23.0,
            Color::new(INK.r, INK.g, INK.b, 0.88),
        );
        body(ctx, legend, vec2(26.0, 82.0), 16.0, CREAM);
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
