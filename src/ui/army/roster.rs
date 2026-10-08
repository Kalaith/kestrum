//! Six visible formation slots with a separate selected-force summary.

use super::*;

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    let current = ctx.army.selected_army(campaign);

    if let Some(army) = current {
        if let Some(action) = army_header(ctx, campaign, army) {
            return Some(action);
        }

        if let Some(action) = formation_slots::draw(ctx, campaign, army) {
            return Some(action);
        }

        let action_y = army_summary(ctx, campaign, army).max(403.0);
        if let Some(formation) = ctx.army.selected {
            if button(
                ctx,
                Rect::new(704.0, action_y, 464.0, 44.0),
                &ctx.text("formation_develop"),
                true,
                false,
            ) {
                return Some(UiAction::OpenFormationProgression(formation));
            }
        }
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
            Rect::new(530.0, 626.0, 160.0, 48.0),
            &ctx.text("army_people"),
            true,
            false,
        )
    {
        return Some(UiAction::ArmyPeople);
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

fn army_summary(ctx: &Context<'_>, campaign: &VisibleCampaign, army: &Army) -> f32 {
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
        .map(|person| format!("{}: {}", ctx.text("commander"), person_name(ctx, person)))
        .unwrap_or_else(|| ctx.text("no_commander"));

    let mut y = block(ctx, &commander, vec2(704.0, 327.0), 464.0, CREAM) + 6.0;

    y = block(
        ctx,
        &ctx.text(if ctx.army.supplied {
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
        y = block(
            ctx,
            &ctx.text("upkeep_deficit"),
            vec2(704.0, y),
            464.0,
            BRASS,
        );
    }
    y + 10.0
}

fn army_header(ctx: &Context<'_>, campaign: &VisibleCampaign, army: &Army) -> Option<UiAction> {
    let name = truncate_text_to_width_ex(&army.name, 560.0, ctx.font(), 24.0);

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

    None
}
