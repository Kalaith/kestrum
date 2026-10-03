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
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.observer_mode)
    {
        return Rect::new(at.x - 88.0, at.y - 24.0, 176.0, 68.0);
    }
    Rect::new(at.x - 57.0, at.y - 24.0, 114.0, 60.0)
}

pub(super) fn draw(ctx: &Context<'_>, panel: Option<Rect>) {
    let (Some(campaign), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return;
    };
    let attention = if campaign.observer_mode {
        Rect::new(0.0, 0.0, 0.0, 0.0)
    } else {
        super::super::overview::attention_bounds(
            ctx.overview_ui,
            panel.is_some(),
            overview.attention.len(),
        )
    };
    for target in ctx
        .navigation
        .army_targets(&campaign.world, ctx.view, &campaign.armies)
    {
        if !banner_visible(target.bounds, panel, attention)
            || (!campaign.observer_mode
                && ctx.state.overlay == Overlay::None
                && super::super::notification_reserved_rects(
                    ctx.notifications,
                    ctx.notification_projection,
                )
                .iter()
                .any(|rect| rect.overlaps(&target.bounds)))
        {
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
        let factions = if campaign.observer_mode {
            observer_factions(ctx, &target.armies)
        } else {
            Vec::new()
        };
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
        if campaign.observer_mode {
            faction_ticks(ctx, at, &factions);
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
            let identity = if campaign.observer_mode && factions.len() == 1 {
                campaign
                    .factions
                    .iter()
                    .find(|faction| faction.id == factions[0])
                    .map_or_else(|| first.name.clone(), |faction| faction.name.clone())
            } else if campaign.observer_mode && factions.len() > 1 {
                format!("{} {}", factions.len(), ctx.text("observer_factions_count"))
            } else if target.grouping == ArmyGrouping::Region {
                ctx.text("map_regional_forces")
                    .replace("{count}", &armies.len().to_string())
            } else if armies.len() == 1 {
                first.name.clone()
            } else {
                format!("{} {}", armies.len(), ctx.data.map.text("armies"))
            };
            let width = if campaign.observer_mode { 168.0 } else { 106.0 };
            let label = truncate_text_to_width_ex(&identity, width, ctx.body_font(), 16.0);
            short_label(ctx, &label, at + vec2(0.0, 28.0), CREAM);
        }
    }
}

fn observer_factions(
    ctx: &Context<'_>,
    armies: &[kestrum::state::military::ArmyId],
) -> Vec<kestrum::data::world::FactionId> {
    let Some(campaign) = ctx.campaign_view else {
        return Vec::new();
    };
    let mut factions: Vec<_> = campaign
        .armies
        .iter()
        .filter(|army| armies.contains(&army.id))
        .map(|army| army.faction)
        .collect();
    factions.sort_unstable();
    factions.dedup();
    factions
}

fn faction_ticks(ctx: &Context<'_>, center: Vec2, factions: &[kestrum::data::world::FactionId]) {
    for (index, faction) in factions.iter().enumerate() {
        let angle = std::f32::consts::TAU * index as f32 / factions.len().max(1) as f32;
        let direction = vec2(angle.cos(), angle.sin());
        let color = super::faction_color(ctx, Some(*faction));
        draw_line(
            center.x + direction.x * 18.0,
            center.y + direction.y * 18.0,
            center.x + direction.x * 24.0,
            center.y + direction.y * 24.0,
            4.0,
            color,
        );
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
            MapScope::Region(region) => campaign.world.region_site_position(region, site_id),
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
