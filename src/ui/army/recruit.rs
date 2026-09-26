//! Six ordinary troop choices, each with its actual cost and rejection reason.

use super::*;

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    army: Option<ArmyId>,
    kind: Option<TroopKind>,
) -> Option<UiAction> {
    let destination = army
        .and_then(|id| campaign.armies.iter().find(|army| army.id == id))
        .map(|army| army.name.clone())
        .unwrap_or_else(|| ctx.text("new_army"));
    body(
        ctx,
        &truncate_text_to_width_ex(&destination, 1040.0, ctx.body_font(), 20.0),
        vec2(112.0, 169.0),
        20.0,
        CREAM,
    );
    let mut action = None;
    for (index, option) in ctx.army.options.iter().enumerate() {
        let rect = Rect::new(
            112.0 + (index % 2) as f32 * 540.0,
            183.0 + (index / 2) as f32 * 131.0,
            516.0,
            122.0,
        );
        let selected = kind == Some(option.kind);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected {
                Color::new(0.20, 0.28, 0.24, 1.0)
            } else {
                Color::new(0.10, 0.17, 0.16, 1.0)
            },
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected { 2.0 } else { 1.0 },
            if selected { BRASS } else { MUTED },
        );
        body(
            ctx,
            &ctx.text(troop_key(option.kind)),
            vec2(rect.x + 14.0, rect.y + 24.0),
            21.0,
            CREAM,
        );
        body(
            ctx,
            &resources_text(ctx, option.cost),
            vec2(rect.x + 14.0, rect.y + 46.0),
            18.0,
            CREAM,
        );
        let elements = ctx.text(if option.kind == TroopKind::SiegeEngines {
            "engine_teams"
        } else {
            "troops"
        });
        body(
            ctx,
            &format!(
                "{} {elements}  ·  {} {} {} / {}",
                option.capacity,
                ctx.text("upkeep"),
                option.upkeep_gold,
                ctx.text("gold"),
                ctx.text("round")
            ),
            vec2(rect.x + 14.0, rect.y + 68.0),
            18.0,
            MUTED,
        );
        let reason = option
            .blocked
            .clone()
            .unwrap_or_else(|| ctx.text("recruit_ready"));
        for (line, text) in wrap_text_ex(&reason, rect.w - 28.0, ctx.body_font(), 18.0)
            .into_iter()
            .take(2)
            .enumerate()
        {
            body(
                ctx,
                &text,
                vec2(rect.x + 14.0, rect.y + 90.0 + line as f32 * 22.0),
                18.0,
                if option.blocked.is_some() {
                    BRASS
                } else {
                    CREAM
                },
            );
        }
        if tapped(ctx, rect) {
            action = Some(UiAction::SelectRecruit(option.kind));
        }
    }
    if ctx.army.status.is_empty() {
        body(
            ctx,
            &ctx.text("recruit_exhausted"),
            vec2(112.0, 599.0),
            18.0,
            MUTED,
        );
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("cancel"),
        true,
        false,
    ) {
        action = Some(UiAction::CancelArmyAction);
    }
    let selected = kind.and_then(|kind| ctx.army.options.iter().find(|option| option.kind == kind));
    if button(
        ctx,
        Rect::new(836.0, 626.0, 332.0, 48.0),
        &ctx.text("confirm_recruit"),
        selected.is_some_and(|option| option.blocked.is_none()),
        true,
    ) {
        action = Some(UiAction::ConfirmRecruit);
    }
    action
}
