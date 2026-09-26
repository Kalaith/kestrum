//! Army group choice and physical-route review; the atlas remains the destination picker.

mod group;
mod review;

use super::{components::*, selection, Context, UiAction};
use kestrum::{
    data::world::{MarkerLocation, SiteId},
    engine::MovementPreview,
    navigation::{MapNavigation, MapSelection, MapView},
    state::{military::ArmyId, world::CampaignWorld},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};
use std::collections::BTreeMap;

pub const MOVE_GROUP_PAGE_SIZE: usize = 6;
pub const ROUTE_PAGE_SIZE: usize = 6;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MoveStage {
    #[default]
    Inactive,
    Group,
    Map,
    Review,
}

#[derive(Debug, Default)]
pub struct MoveView {
    pub stage: MoveStage,
    pub site: Option<SiteId>,
    pub armies: Vec<ArmyId>,
    pub page: usize,
    pub route_page: usize,
    pub destination: Option<SiteId>,
    pub preview: Option<MovementPreview>,
    pub remaining: BTreeMap<ArmyId, u32>,
    pub status: String,
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.02, 0.05, 0.05, 0.78));
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(
        ctx,
        &ctx.text(if ctx.movement.stage == MoveStage::Group {
            "move_group"
        } else {
            "route_review"
        }),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    match ctx.movement.stage {
        MoveStage::Group => group::draw(ctx),
        MoveStage::Review => review::draw(ctx),
        _ => None,
    }
}

pub fn map_controls_contain(
    point: Vec2,
    navigation: &MapNavigation,
    world: Option<&CampaignWorld>,
    view: &MapView,
) -> bool {
    panel_bounds(navigation, world, view).contains(point)
}

fn panel_bounds(navigation: &MapNavigation, world: Option<&CampaignWorld>, view: &MapView) -> Rect {
    world
        .and_then(|world| selection::bounds(navigation, world, view))
        .unwrap_or(Rect::new(320.0, 640.0, 720.0, 60.0))
}

pub fn draw_map_overlay(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let rect = panel_bounds(ctx.navigation, Some(&campaign.world), ctx.view);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, INK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    if ctx.navigation.selection().is_none() {
        lines(
            ctx,
            &ctx.text("move_pick_destination"),
            vec2(rect.x + 16.0, rect.y + 23.0),
            500.0,
            2,
            CREAM,
        );
        return button(
            ctx,
            Rect::new(rect.x + rect.w - 182.0, rect.y + 6.0, 166.0, 48.0),
            &ctx.text("cancel_move"),
            true,
            false,
        )
        .then_some(UiAction::CancelMove);
    }
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    text(ctx, &ctx.text("move_armies"), vec2(x, 129.0), 24.0, CREAM);
    if button(
        ctx,
        Rect::new(rect.x + rect.w - 116.0, 100.0, 100.0, 48.0),
        &ctx.text("close"),
        true,
        false,
    ) {
        return Some(UiAction::CloseSelection);
    }
    let count = format!(
        "{}: {}",
        ctx.text("selected_armies"),
        ctx.movement.armies.len()
    );
    body(ctx, &count, vec2(x, 162.0), 18.0, MUTED);
    let selected_region = match ctx.navigation.selection() {
        Some(MapSelection::Marker(id)) => campaign
            .world
            .marker(id)
            .filter(|marker| matches!(marker.location, MarkerLocation::Region { .. })),
        _ => None,
    };
    if let Some(region) = selected_region {
        lines(ctx, &region.name, vec2(x, 200.0), width, 2, CREAM);
        lines(
            ctx,
            &ctx.text("move_enter_region"),
            vec2(x, 261.0),
            width,
            3,
            MUTED,
        );
        if button(
            ctx,
            Rect::new(x, 450.0, width, 48.0),
            &ctx.text("enter_region"),
            true,
            true,
        ) {
            return Some(UiAction::EnterRegion(region.id));
        }
    } else if let Some(preview) = &ctx.movement.preview {
        let destination = ctx
            .movement
            .destination
            .and_then(|id| campaign.world.site(id))
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        lines(ctx, destination, vec2(x, 198.0), width, 2, CREAM);
        body(
            ctx,
            &cost_summary(ctx, preview),
            vec2(x, 252.0),
            18.0,
            CREAM,
        );
        let consequence = if ctx.movement.status.is_empty() {
            route_consequence(ctx, preview)
        } else {
            ctx.movement.status.clone()
        };
        lines(ctx, &consequence, vec2(x, 281.0), width, 4, BRASS);
        lines(
            ctx,
            &ctx.text(if preview.supplied_after {
                "move_supplied_after"
            } else {
                "move_unsupplied_after"
            }),
            vec2(x, 383.0),
            width,
            2,
            MUTED,
        );
        if button(
            ctx,
            Rect::new(x, 450.0, width, 48.0),
            &ctx.text("route_review"),
            true,
            false,
        ) {
            return Some(UiAction::ReviewMove);
        }
        if button(
            ctx,
            Rect::new(x, 506.0, width, 48.0),
            &ctx.text("confirm_move"),
            preview.reachable_steps > 0,
            true,
        ) {
            return Some(UiAction::ConfirmMove);
        }
    } else {
        lines(
            ctx,
            &ctx.text("move_pick_destination"),
            vec2(x, 206.0),
            width,
            4,
            CREAM,
        );
        if !ctx.movement.status.is_empty() {
            lines(ctx, &ctx.movement.status, vec2(x, 326.0), width, 5, BRASS);
        }
    }
    if button(
        ctx,
        Rect::new(x, 568.0, width, 48.0),
        &ctx.text("cancel_move"),
        true,
        false,
    ) {
        return Some(UiAction::CancelMove);
    }
    None
}

fn cost_summary(ctx: &Context<'_>, preview: &MovementPreview) -> String {
    format!(
        "{}: {}  ·  {}: {}",
        ctx.text("route_cost"),
        preview.total_cost,
        ctx.text("movement_left"),
        preview.remaining
    )
}

fn route_consequence(ctx: &Context<'_>, preview: &MovementPreview) -> String {
    if let Some(stop) = &preview.stop {
        let end = ctx
            .campaign_view
            .and_then(|campaign| campaign.world.site(preview.reachable_site))
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        format!("{}: {end}. {}", ctx.text("move_stops_at"), stop.reason)
    } else if preview.uncertain_contact {
        ctx.text("move_uncertain_contact")
    } else {
        ctx.text("move_route_clear")
    }
}

fn lines(ctx: &Context<'_>, label: &str, at: Vec2, width: f32, limit: usize, color: Color) {
    let wrapped = wrap_text_ex(label, width, ctx.body_font(), 18.0);
    for (index, line) in wrapped.iter().take(limit).enumerate() {
        let label = if index + 1 == limit && wrapped.len() > limit {
            truncate_text_to_width_ex(&format!("{line}…"), width, ctx.body_font(), 18.0)
        } else {
            line.clone()
        };
        body(
            ctx,
            &label,
            at + vec2(0.0, index as f32 * 23.0),
            18.0,
            color,
        );
    }
}

fn tapped(ctx: &Context<'_>, rect: Rect) -> bool {
    ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin))
}
