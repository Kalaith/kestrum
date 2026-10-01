//! One compact order card leaves the atlas available throughout a move.

use super::*;
use kestrum::{
    navigation::{MapNavigation, MapScope, MapView},
    state::{world::CampaignWorld, Overlay},
};

pub fn panel_bounds(
    movement: &MoveView,
    navigation: &MapNavigation,
    world: &CampaignWorld,
    view: &MapView,
) -> Rect {
    let origin = movement.site.and_then(|id| world.site(id));
    let position = origin.and_then(|site| match navigation.scope() {
        MapScope::World => world.marker(site.marker).map(|marker| marker.position),
        MapScope::Region(region) if region == site.marker => Some(site.position),
        _ => None,
    });
    let left = position.is_none_or(|position| view.project_normalized(position).x >= 640.0);
    let choosing_region = matches!(navigation.selection(), Some(kestrum::navigation::MapSelection::Marker(id))
        if world.physical_site(id).is_none());
    let height = if movement.preview.is_some()
        || movement.planned_destination.is_some()
        || choosing_region
    {
        480.0
    } else {
        350.0
    };
    Rect::new(if left { 24.0 } else { 898.0 }, 100.0, 358.0, height)
}

pub fn draw_map_overlay(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let rect = panel_bounds(ctx.movement, ctx.navigation, &campaign.world, ctx.view);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, INK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    if let Some(action) = draw_army_heading(ctx, rect) {
        return Some(action);
    }
    let action = if let Some(preview) = &ctx.movement.preview {
        draw_route_preview(ctx, rect, preview)
    } else {
        draw_pick_destination(ctx, rect)
    };
    action.or_else(|| draw_order_controls(ctx, rect))
}

fn draw_army_heading(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    let active = ctx.state.overlay == Overlay::None;
    let army = campaign
        .armies
        .iter()
        .find(|army| ctx.movement.armies.contains(&army.id));
    let title = if ctx.movement.armies.len() == 1 {
        army.map(|army| army.name.clone()).unwrap_or_default()
    } else {
        format!(
            "{}: {}",
            ctx.text("selected_armies"),
            ctx.movement.armies.len()
        )
    };
    let ids =
        ctx.navigation
            .armies_at_selection(&campaign.world, ctx.movement.site?, &campaign.armies);
    let multiple = ids.len() > 1;
    lines(
        ctx,
        &title,
        vec2(x, 132.0),
        width - if multiple { 142.0 } else { 60.0 },
        2,
        CREAM,
    );
    if multiple
        && button(
            ctx,
            Rect::new(rect.right() - 146.0, 108.0, 74.0, 48.0),
            &ctx.text("next"),
            active,
            false,
        )
    {
        let current = ids
            .iter()
            .position(|id| ctx.movement.armies.first() == Some(id))
            .unwrap_or(0);
        return Some(UiAction::BeginMove(ids[(current + 1) % ids.len()]));
    }
    if button(
        ctx,
        Rect::new(rect.right() - 64.0, 108.0, 48.0, 48.0),
        "×",
        active,
        false,
    ) {
        return Some(UiAction::CancelMove);
    }
    let remaining = movement_remaining(ctx);
    body(
        ctx,
        &format!("{}: {remaining}", ctx.text("movement_left")),
        vec2(x, 188.0),
        20.0,
        BRASS,
    );
    if let Some(site) = ctx.movement.site.and_then(|id| campaign.world.site(id)) {
        lines(ctx, &site.name, vec2(x, 217.0), width, 1, MUTED);
    }
    None
}

fn movement_remaining(ctx: &Context<'_>) -> u32 {
    ctx.movement
        .armies
        .iter()
        .filter_map(|id| ctx.movement.remaining.get(id))
        .min()
        .copied()
        .unwrap_or(0)
}

fn draw_route_preview(
    ctx: &Context<'_>,
    rect: Rect,
    preview: &MovementPreview,
) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    let active = ctx.state.overlay == Overlay::None;
    let name = ctx
        .movement
        .destination
        .and_then(|id| campaign.world.site(id))
        .map(|site| site.name.as_str())
        .unwrap_or_default();
    lines(ctx, name, vec2(x, 260.0), width, 2, CREAM);
    body(
        ctx,
        &format!("{}: {}", ctx.text("route_cost"), preview.total_cost),
        vec2(x, 316.0),
        20.0,
        BRASS,
    );
    draw_route_consequence(ctx, rect, preview);
    let threat = campaign
        .threats
        .iter()
        .find(|threat| Some(threat.site) == ctx.movement.destination);
    let primary = if preview.reachable_steps == 0 {
        threat.map(|threat| ("clear_threat", UiAction::OpenThreat(threat.id)))
    } else {
        None
    };
    if button(
        ctx,
        Rect::new(x, 426.0, width, 48.0),
        &ctx.text(primary.map_or("confirm_move", |(key, _)| key)),
        active && campaign.player_turn && (preview.can_confirm() || primary.is_some()),
        true,
    ) {
        return Some(primary.map_or(UiAction::ConfirmMove, |(_, action)| action));
    }
    None
}

fn draw_route_consequence(ctx: &Context<'_>, rect: Rect, preview: &MovementPreview) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    let consequence = if ctx.movement.status.is_empty() {
        route_consequence(ctx, preview)
    } else {
        ctx.movement.status.clone()
    };
    let exit_cost: u32 = if ctx.navigation.scope() == MapScope::World {
        preview
            .steps
            .iter()
            .position(|step| {
                campaign
                    .world
                    .route(step.route)
                    .is_some_and(|route| route.major_connection.is_some())
            })
            .map(|crossing| preview.steps[..crossing].iter().map(|step| step.cost).sum())
            .unwrap_or(0)
    } else {
        0
    };
    if exit_cost > 0 {
        body(
            ctx,
            &ctx.text("map_region_exit_cost")
                .replace("{cost}", &exit_cost.to_string()),
            vec2(x, 343.0),
            16.0,
            MUTED,
        );
    }
    lines(
        ctx,
        &consequence,
        vec2(x, if exit_cost > 0 { 373.0 } else { 345.0 }),
        width,
        if exit_cost > 0 { 2 } else { 3 },
        CREAM,
    );
}

fn draw_pick_destination(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    let active = ctx.state.overlay == Overlay::None;
    let remaining = movement_remaining(ctx);
    let label = if let Some(destination) = ctx.movement.planned_destination {
        let name = campaign
            .world
            .site(destination)
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        ctx.text("move_planned_destination").replace("{site}", name)
    } else if ctx.movement.status.is_empty() {
        ctx.text(if remaining == 0 {
            "map_move_exhausted"
        } else {
            "map_pick_destination"
        })
    } else {
        ctx.movement.status.clone()
    };
    lines(ctx, &label, vec2(x, 265.0), width, 5, CREAM);
    if ctx.movement.planned_destination.is_some() {
        if button(
            ctx,
            Rect::new(x, 426.0, width, 48.0),
            &ctx.text("cancel_movement_plan"),
            active && campaign.player_turn,
            false,
        ) {
            return campaign
                .movement_plans
                .iter()
                .find(|plan| {
                    plan.armies
                        .iter()
                        .any(|army| ctx.movement.armies.contains(army))
                })
                .and_then(|plan| plan.armies.first())
                .copied()
                .map(UiAction::CancelMovementPlan);
        }
        return None;
    }
    if let Some(kestrum::navigation::MapSelection::Marker(id)) = ctx.navigation.selection() {
        if campaign.world.physical_site(id).is_none()
            && button(
                ctx,
                Rect::new(x, 426.0, width, 48.0),
                &ctx.text("enter_region"),
                active,
                true,
            )
        {
            return Some(UiAction::EnterRegion(id));
        }
    }
    None
}

fn draw_order_controls(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    let x = rect.x + 16.0;
    let active = ctx.state.overlay == Overlay::None;
    let secondary = if ctx.movement.preview.is_some() {
        ("route_review", UiAction::ReviewMove)
    } else {
        ("map_army_details", UiAction::OpenArmies(ctx.movement.site?))
    };
    let secondary_y = rect.bottom() - 90.0;
    if button(
        ctx,
        Rect::new(x, secondary_y, 158.0, 48.0),
        &ctx.text(secondary.0),
        active,
        false,
    ) {
        return Some(secondary.1);
    }
    if ctx.movement.remaining.len() > 1 {
        if button(
            ctx,
            Rect::new(x + 168.0, secondary_y, 158.0, 48.0),
            &ctx.text("map_move_group"),
            active,
            false,
        ) {
            return Some(UiAction::EditMoveGroup);
        }
    } else if let Some(preview) = &ctx.movement.preview {
        lines(
            ctx,
            &ctx.text(if preview.supplied_after {
                "map_supply_connected"
            } else {
                "map_supply_cutoff"
            }),
            vec2(x + 168.0, 508.0),
            158.0,
            2,
            MUTED,
        );
    }
    None
}
