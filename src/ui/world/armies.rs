//! Compact force standards; exact strength and orders stay with the selection.

use super::*;
use kestrum::{
    engine::ArmyMapStatus,
    navigation::{ArmyGrouping, ArmyTarget},
};

pub(super) fn display_bounds(ctx: &Context<'_>, target: &ArmyTarget) -> Rect {
    if ctx.view.band() == MapScaleBand::Overview {
        return target.bounds;
    }
    let at = target.bounds.center();
    Rect::new(at.x - 57.0, at.y - 24.0, 114.0, 60.0)
}

pub(super) fn draw(ctx: &Context<'_>, panel: Option<Rect>) {
    let (Some(campaign), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return;
    };
    let attention = super::super::overview::attention_bounds(
        ctx.overview_ui,
        panel.is_some(),
        overview.attention.len(),
    );
    for target in ctx
        .navigation
        .army_targets(&campaign.world, ctx.view, &campaign.armies)
    {
        if !banner_visible(target.bounds, panel, attention) {
            continue;
        }
        let armies: Vec<_> = target
            .armies
            .iter()
            .filter_map(|id| overview.armies.get(id))
            .collect();
        let Some(first) = armies.first() else {
            continue;
        };
        let at = target.bounds.center();
        let selected = target
            .armies
            .iter()
            .any(|id| ctx.movement.armies.contains(id));
        if target.grouping != ArmyGrouping::Nearby {
            leader_line(ctx, first.site, target.bounds);
        }
        draw_circle(at.x, at.y, 15.0, INK);
        if selected {
            draw_circle_lines(at.x, at.y, 20.0, 2.0, CREAM);
        }
        let color = if armies
            .iter()
            .any(|army| army.status == ArmyMapStatus::Siege || !army.supplied)
        {
            Color::new(0.96, 0.64, 0.49, 1.0)
        } else {
            BRASS
        };
        draw_line(at.x - 8.0, at.y - 11.0, at.x - 8.0, at.y + 12.0, 2.0, color);
        draw_triangle(
            at + vec2(-7.0, -10.0),
            at + vec2(10.0, -6.0),
            at + vec2(-7.0, 2.0),
            color,
        );
        if target.grouping != ArmyGrouping::Site {
            draw_circle_lines(at.x, at.y, 17.0, 1.0, color);
        }
        let status = if armies
            .iter()
            .any(|army| army.status == ArmyMapStatus::Siege || !army.supplied)
        {
            "!"
        } else if armies
            .iter()
            .any(|army| army.status == ArmyMapStatus::Queued)
        {
            ">"
        } else {
            "·"
        };
        if armies.len() > 1 || target.grouping == ArmyGrouping::Region {
            let count = match target.grouping {
                ArmyGrouping::Region => format!("R{}", armies.len()),
                ArmyGrouping::Nearby => format!("+{}", armies.len()),
                ArmyGrouping::Site => armies.len().to_string(),
            };
            short_label(ctx, &count, at + vec2(19.0, -10.0), color);
        }
        short_label(ctx, status, at + vec2(17.0, 11.0), color);
        if ctx.view.band() != MapScaleBand::Overview
            && banner_visible(display_bounds(ctx, &target), panel, attention)
        {
            let identity = if target.grouping == ArmyGrouping::Region {
                ctx.text("map_regional_forces")
                    .replace("{count}", &armies.len().to_string())
            } else if armies.len() == 1 {
                first.name.clone()
            } else {
                format!("{} {}", armies.len(), ctx.data.map.text("armies"))
            };
            let label = truncate_text_to_width_ex(&identity, 106.0, ctx.body_font(), 16.0);
            short_label(ctx, &label, at + vec2(0.0, 28.0), CREAM);
        }
    }
}

fn short_label(ctx: &Context<'_>, text: &str, at: Vec2, color: Color) {
    let width = measure_text(text, ctx.body_font(), 16, 1.0).width;
    draw_rectangle(
        at.x - width * 0.5 - 3.0,
        at.y - 13.0,
        width + 6.0,
        20.0,
        INK,
    );
    body(ctx, text, at + vec2(-width * 0.5, 3.0), 16.0, color);
}

fn leader_line(ctx: &Context<'_>, site_id: SiteId, rect: Rect) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    if let Some(site) = campaign.world.site(site_id) {
        let projected_origin = match ctx.navigation.scope() {
            MapScope::World => campaign
                .world
                .marker(site.marker)
                .map(|marker| marker.position),
            MapScope::Region(_) => Some(site.position),
        }
        .map(|position| ctx.view.project_normalized(position));
        if let Some(origin) = projected_origin {
            let edge = rect.center();
            let distance = origin.distance(edge);
            if distance > 40.0 {
                let start = origin.lerp(edge, 22.0 / distance);
                let end = edge.lerp(origin, 17.0 / distance);
                draw_line(start.x, start.y, end.x, end.y, 1.0, BRASS);
            }
        }
    }
}
