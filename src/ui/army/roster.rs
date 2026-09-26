//! Six visible formation slots with a separate selected-force summary.

use super::*;

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    let current = ctx.army.selected_army(campaign);
    if let Some(army) = current {
        let name = truncate_text_to_width_ex(&army.name, 624.0, ctx.font(), 24.0);
        text(ctx, &name, vec2(112.0, 185.0), 24.0, CREAM);
        let pages = ctx.army.page_count(campaign);
        if button(
            ctx,
            Rect::new(788.0, 155.0, 108.0, 48.0),
            &ctx.text("previous"),
            ctx.army.page > 0,
            false,
        ) {
            return Some(UiAction::ArmyPage(-1));
        }
        centered(
            ctx,
            &format!("{} / {pages}", ctx.army.page + 1),
            vec2(971.0, 185.0),
            20.0,
            CREAM,
        );
        if button(
            ctx,
            Rect::new(1060.0, 155.0, 108.0, 48.0),
            &ctx.text("next"),
            ctx.army.page + 1 < pages,
            false,
        ) {
            return Some(UiAction::ArmyPage(1));
        }
        for (index, slot) in army.slots.iter().enumerate() {
            let rect = Rect::new(112.0, 217.0 + index as f32 * 59.0, 552.0, 52.0);
            let selected = slot.is_some() && *slot == ctx.army.selected;
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if selected {
                    Color::new(0.21, 0.29, 0.25, 1.0)
                } else {
                    Color::new(0.10, 0.17, 0.16, 1.0)
                },
            );
            if selected {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
            }
            body(
                ctx,
                &format!("{}", index + 1),
                vec2(rect.x + 12.0, rect.y + 32.0),
                18.0,
                MUTED,
            );
            if let Some(formation) = slot.and_then(|id| formation(campaign, id)) {
                body(
                    ctx,
                    &ctx.text(troop_key(formation.kind)),
                    vec2(rect.x + 46.0, rect.y + 23.0),
                    21.0,
                    CREAM,
                );
                body(
                    ctx,
                    &format!("{} / {}", formation.headcount, formation.capacity),
                    vec2(rect.x + 390.0, rect.y + 32.0),
                    18.0,
                    CREAM,
                );
                let remaining = ctx
                    .army
                    .member_remaining
                    .get(&formation.id)
                    .copied()
                    .unwrap_or(0);
                body(
                    ctx,
                    &format!(
                        "{}: {remaining} · {}: {}",
                        ctx.text("movement_left"),
                        ctx.text("movement_spent"),
                        formation.movement_spent
                    ),
                    vec2(rect.x + 46.0, rect.y + 44.0),
                    16.0,
                    MUTED,
                );
                if tapped(ctx, rect) {
                    return Some(UiAction::SelectFormation(formation.id));
                }
            } else {
                body(
                    ctx,
                    &ctx.text("empty_slot"),
                    vec2(rect.x + 46.0, rect.y + 32.0),
                    18.0,
                    MUTED,
                );
                if tapped(ctx, rect) {
                    return Some(UiAction::BeginRecruit(Some(army.id)));
                }
            }
        }
        army_summary(ctx, campaign, army);
    } else {
        text(ctx, &ctx.text("no_armies"), vec2(112.0, 226.0), 24.0, CREAM);
        block(
            ctx,
            &ctx.text("new_army_help"),
            vec2(112.0, 274.0),
            890.0,
            CREAM,
        );
    }
    if !campaign.player_turn && ctx.army.status.is_empty() {
        body(
            ctx,
            &ctx.text("transfer_paused_help"),
            vec2(112.0, 599.0),
            18.0,
            MUTED,
        );
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    if current.is_some()
        && button(
            ctx,
            Rect::new(300.0, 626.0, 214.0, 48.0),
            &ctx.text("army_orders"),
            true,
            false,
        )
    {
        return Some(UiAction::ArmyOrders);
    }
    if current.is_some()
        && button(
            ctx,
            Rect::new(700.0, 626.0, 202.0, 48.0),
            &ctx.text("new_army"),
            true,
            false,
        )
    {
        return Some(UiAction::BeginRecruit(None));
    }
    if button(
        ctx,
        Rect::new(920.0, 626.0, 248.0, 48.0),
        &ctx.text(if current.is_some() {
            "recruit"
        } else {
            "new_army"
        }),
        true,
        true,
    ) {
        return Some(UiAction::BeginRecruit(current.map(|army| army.id)));
    }
    None
}

fn army_summary(ctx: &Context<'_>, campaign: &VisibleCampaign, army: &Army) {
    let upkeep: i64 = army
        .slots
        .iter()
        .flatten()
        .filter_map(|id| formation(campaign, *id))
        .filter_map(|formation| ctx.economy.formations.get(&formation.kind))
        .map(|rule| rule.upkeep_gold)
        .sum();
    body(
        ctx,
        &format!(
            "{}: {upkeep} {} / {}",
            ctx.text("army_upkeep"),
            ctx.text("gold"),
            ctx.text("round")
        ),
        vec2(704.0, 237.0),
        18.0,
        CREAM,
    );
    body(
        ctx,
        &format!(
            "{}: {:.1}%",
            ctx.text("leadership"),
            ctx.army.leadership_permille as f32 / 10.0
        ),
        vec2(704.0, 265.0),
        18.0,
        CREAM,
    );
    body(
        ctx,
        &format!("{}: {}", ctx.text("army_movement_left"), ctx.army.remaining),
        vec2(704.0, 293.0),
        18.0,
        CREAM,
    );
    let commander = army
        .commander
        .and_then(|id| campaign.people.iter().find(|person| person.id == id));
    let commander = commander
        .map(|person| format!("{}: {}", ctx.text("commander"), person.name))
        .unwrap_or_else(|| ctx.text("no_commander"));
    let mut y = block(ctx, &commander, vec2(704.0, 327.0), 464.0, CREAM) + 6.0;
    y = block(
        ctx,
        &ctx.text(if campaign.supplied_sites.contains(&army.site) {
            "site_supplied"
        } else {
            "site_unsupplied"
        }),
        vec2(704.0, y),
        464.0,
        MUTED,
    ) + 12.0;
    let viewer = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer);
    if viewer.is_some_and(|faction| faction.deficit == Some(true)) {
        block(
            ctx,
            &ctx.text("upkeep_deficit"),
            vec2(704.0, y),
            464.0,
            BRASS,
        );
    }
}
