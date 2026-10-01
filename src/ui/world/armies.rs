//! Owned strength and durable order facts on the map's existing army controls.

use super::*;
use kestrum::engine::ArmyMapStatus;

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
        let rect = target.bounds;
        leader_line(ctx, first.site, rect);
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
        let identity = if armies.len() == 1 {
            first.name.clone()
        } else {
            format!("{} {}", armies.len(), ctx.data.map.text("armies"))
        };
        let fitted_identity =
            truncate_text_to_width_ex(&identity, rect.w - 16.0, ctx.body_font(), 17.0);
        body(
            ctx,
            &fitted_identity,
            vec2(rect.x + 8.0, rect.y + 21.0),
            17.0,
            CREAM,
        );
        let strength: u32 = armies.iter().map(|army| army.troops).sum();
        let status = if armies
            .iter()
            .any(|army| army.status == ArmyMapStatus::Siege)
        {
            "siege"
        } else if armies.iter().any(|army| !army.supplied) {
            "unsupplied"
        } else if armies
            .iter()
            .any(|army| army.status == ArmyMapStatus::Queued)
        {
            "queued"
        } else {
            "idle"
        };
        let state = format!(
            "{strength} {} · {}",
            ctx.data.map.text("troops"),
            ctx.data.map.text(status)
        );
        let fitted_state = truncate_text_to_width_ex(&state, rect.w - 16.0, ctx.body_font(), 16.0);
        body(
            ctx,
            &fitted_state,
            vec2(rect.x + 8.0, rect.y + 44.0),
            16.0,
            BRASS,
        );
    }
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
            let edge = vec2(
                origin.x.clamp(rect.x, rect.right()),
                origin.y.clamp(rect.y, rect.bottom()),
            );
            let distance = origin.distance(edge);
            if distance > 25.0 {
                let start = origin.lerp(edge, 25.0 / distance);
                draw_line(start.x, start.y, edge.x, edge.y, 1.5, BRASS);
            }
        }
    }
}
