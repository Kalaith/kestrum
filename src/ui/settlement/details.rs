//! Construction previews show prepaid costs, persistent work and exact refunds.

use super::*;
use kestrum::data::world::MilitaryLayer;

pub(super) fn overview(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.campaign_view?;
    let site = view.world.site(ctx.settlement.site?)?;
    body(
        ctx,
        &labels::habitation(ctx, site.habitation),
        vec2(112.0, 236.0),
        20.0,
        CREAM,
    );
    let population = view.world.population.get(&site.id).copied().unwrap_or(0);
    lines(
        ctx,
        &format!("{}: {population}", ctx.text("settlement_population")),
        vec2(112.0, 264.0),
        470.0,
        1,
        MUTED,
    );
    lines(
        ctx,
        &ctx.text(if view.supplied_sites.contains(&site.id) {
            "site_supplied"
        } else {
            "site_unsupplied"
        }),
        vec2(112.0, 292.0),
        470.0,
        2,
        MUTED,
    );
    let focus = view
        .world
        .focus
        .get(&site.id)
        .map(|focus| ctx.text(focus_key(*focus)))
        .unwrap_or_else(|| ctx.text("none"));
    lines(
        ctx,
        &format!("{}: {focus}", ctx.text("settlement_focus")),
        vec2(112.0, 338.0),
        470.0,
        1,
        CREAM,
    );
    let damage = view.world.site_damage.get(&site.id).copied().unwrap_or(0);
    lines(
        ctx,
        &format!(
            "{}: {damage}% · {}: {} · {}: {}%",
            ctx.text("settlement_damage"),
            ctx.text("settlement_fortification"),
            ctx.text(if site.military == MilitaryLayer::Fort {
                "construction_fort"
            } else {
                "none"
            }),
            ctx.text("siege_fort_damage"),
            view.world.fort_damage.get(&site.id).copied().unwrap_or(0)
        ),
        vec2(112.0, 367.0),
        470.0,
        2,
        MUTED,
    );
    body(
        ctx,
        &ctx.text("settlement_facilities"),
        vec2(112.0, 427.0),
        20.0,
        CREAM,
    );
    let facilities = if site.facilities.is_empty() {
        ctx.text("none")
    } else {
        site.facilities
            .iter()
            .map(|f| ctx.text(labels::facility_key(*f)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    lines(ctx, &facilities, vec2(112.0, 456.0), 470.0, 3, MUTED);
    if damage >= ctx.economy.facility_failure_damage {
        lines(
            ctx,
            &ctx.text("facility_damaged"),
            vec2(112.0, 545.0),
            470.0,
            2,
            BRASS,
        );
    }
    local_orders(ctx)
}

fn local_orders(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.campaign_view?;
    let site = ctx.settlement.site?;
    let mut orders: Vec<_> = view
        .construction
        .iter()
        .filter(|order| match order.target {
            ConstructionTarget::Site(id) => id == site,
            ConstructionTarget::Route(id) => view
                .world
                .route(id)
                .is_some_and(|r| r.other_endpoint(site).is_some()),
        })
        .collect();
    orders.sort_by_key(|order| std::cmp::Reverse(order.id));
    body(
        ctx,
        &ctx.text("construction_orders"),
        vec2(652.0, 236.0),
        20.0,
        CREAM,
    );
    if orders.is_empty() {
        lines(
            ctx,
            &ctx.text("construction_no_orders"),
            vec2(652.0, 273.0),
            516.0,
            2,
            MUTED,
        );
    }
    for (index, order) in orders
        .iter()
        .skip(ctx.settlement.page * 4)
        .take(4)
        .enumerate()
    {
        let y = 272.0 + index as f32 * 76.0;
        let label = format!(
            "{} · {} / {}",
            kind_name(ctx, order.kind),
            order.progress,
            order.required_steps
        );
        lines(ctx, &label, vec2(652.0, y), 365.0, 1, CREAM);
        lines(
            ctx,
            &order_status(ctx, order),
            vec2(652.0, y + 23.0),
            365.0,
            2,
            MUTED,
        );
        if control(
            ctx,
            Rect::new(1034.0, y - 16.0, 134.0, 48.0),
            "construction_details",
            true,
            false,
        ) {
            return Some(UiAction::OpenConstructionOrder(order.id));
        }
    }
    page_controls(ctx, orders.len())
}

pub(super) fn review(ctx: &Context<'_>) -> Option<UiAction> {
    let choice = current_choice(ctx)?;
    body(
        ctx,
        &kind_name(ctx, choice.option.kind),
        vec2(112.0, 238.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &labels::target_name(ctx, choice.target),
        vec2(112.0, 268.0),
        1056.0,
        2,
        MUTED,
    );
    lines(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("construction_prepaid"),
            resources(ctx, choice.option.cost)
        ),
        vec2(112.0, 321.0),
        1056.0,
        1,
        BRASS,
    );
    lines(
        ctx,
        &format!(
            "{} {}. {}",
            choice.option.steps,
            ctx.text("construction_steps"),
            ctx.text("construction_timing")
        ),
        vec2(112.0, 352.0),
        1056.0,
        2,
        MUTED,
    );
    lines(
        ctx,
        &ctx.text(labels::effect_key(choice.option.kind)),
        vec2(112.0, 403.0),
        1056.0,
        2,
        CREAM,
    );
    builder_details(ctx, choice.option.kind, 458.0);
    let blocked = ctx
        .settlement
        .blocked
        .as_ref()
        .or(choice.option.blocked.as_ref());
    if let Some(reason) = blocked {
        lines(ctx, reason, vec2(112.0, 547.0), 1056.0, 2, BRASS);
    }
    if control(
        ctx,
        Rect::new(632.0, 626.0, 252.0, 48.0),
        "construction_choose_builder",
        true,
        false,
    ) {
        return Some(UiAction::ChooseBuilder);
    }
    if control(
        ctx,
        Rect::new(916.0, 626.0, 252.0, 48.0),
        "construction_confirm",
        ctx.settlement.builder.is_some() && blocked.is_none(),
        true,
    ) {
        return Some(UiAction::ConfirmConstruction);
    }
    None
}

fn builder_details(ctx: &Context<'_>, kind: ConstructionKind, y: f32) {
    let builder = ctx
        .settlement
        .builder
        .and_then(|id| ctx.campaign_view?.armies.iter().find(|army| army.id == id))
        .map(|army| army.name.clone())
        .unwrap_or_else(|| ctx.text("construction_no_builder"));
    lines(
        ctx,
        &format!("{}: {builder}", ctx.text("construction_builder")),
        vec2(112.0, y),
        1056.0,
        1,
        CREAM,
    );
    let key = if current_order(ctx).is_some_and(|order| !order.is_open()) {
        "construction_builder_released"
    } else if matches!(kind, ConstructionKind::Facility(_)) {
        "construction_builder_placement"
    } else {
        "construction_builder_remain"
    };
    lines(ctx, &ctx.text(key), vec2(112.0, y + 27.0), 1056.0, 2, MUTED);
}

pub(super) fn order(ctx: &Context<'_>) -> Option<UiAction> {
    let order = current_order(ctx)?;
    body(
        ctx,
        &kind_name(ctx, order.kind),
        vec2(112.0, 238.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &labels::target_name(ctx, order.target),
        vec2(112.0, 268.0),
        1056.0,
        2,
        MUTED,
    );
    let progress = format!(
        "{}: {} / {} · {}",
        ctx.text("construction_progress"),
        order.progress,
        order.required_steps,
        order_status(ctx, order)
    );
    lines(ctx, &progress, vec2(112.0, 321.0), 1056.0, 2, CREAM);
    lines(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("construction_prepaid"),
            resources(ctx, order.paid)
        ),
        vec2(112.0, 381.0),
        1056.0,
        1,
        MUTED,
    );
    if ctx.settlement.mode == SettlementMode::Cancel {
        return cancel(ctx);
    }
    builder_details(ctx, order.kind, 425.0);
    lines(
        ctx,
        &ctx.text(labels::effect_key(order.kind)),
        vec2(112.0, 523.0),
        1056.0,
        2,
        MUTED,
    );
    let active = order.is_open();
    if active
        && !matches!(order.kind, ConstructionKind::Facility(_))
        && control(
            ctx,
            Rect::new(620.0, 626.0, 270.0, 48.0),
            "construction_replace_builder",
            true,
            false,
        )
    {
        return Some(UiAction::ChooseBuilder);
    }
    if active
        && control(
            ctx,
            Rect::new(916.0, 626.0, 252.0, 48.0),
            "construction_cancel",
            true,
            false,
        )
    {
        return Some(UiAction::AskCancelConstruction);
    }
    None
}

fn cancel(ctx: &Context<'_>) -> Option<UiAction> {
    lines(
        ctx,
        &ctx.text("construction_cancel_help"),
        vec2(112.0, 437.0),
        1056.0,
        2,
        CREAM,
    );
    if let Some(refund) = ctx.settlement.refund {
        lines(
            ctx,
            &format!(
                "{}: {}",
                ctx.text("construction_refund"),
                resources(ctx, refund)
            ),
            vec2(112.0, 500.0),
            1056.0,
            1,
            BRASS,
        );
    }
    if let Some(reason) = &ctx.settlement.blocked {
        lines(ctx, reason, vec2(112.0, 546.0), 1056.0, 2, BRASS);
    }
    if control(
        ctx,
        Rect::new(916.0, 626.0, 252.0, 48.0),
        "construction_confirm_cancel",
        ctx.settlement.refund.is_some() && ctx.settlement.blocked.is_none(),
        true,
    ) {
        return Some(UiAction::ConfirmCancelConstruction);
    }
    None
}
