//! Focused army and selected-formation actions, with logistics beside the decision.

use super::*;

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    let army = ctx.army.selected_army(campaign)?;
    let name = truncate_text_to_width_ex(&army.name, 1040.0, ctx.font(), 24.0);
    text(ctx, &name, vec2(112.0, 188.0), 24.0, CREAM);
    body(
        ctx,
        &format!("{}: {}", ctx.text("army_movement_left"), ctx.army.remaining),
        vec2(112.0, 223.0),
        20.0,
        CREAM,
    );
    let selected = ctx.army.selected.and_then(|id| formation(campaign, id));
    if let Some(selected) = selected {
        let remaining = ctx
            .army
            .member_remaining
            .get(&selected.id)
            .copied()
            .unwrap_or(0);
        body(
            ctx,
            &format!(
                "{} · {} / {} · {}: {} · {}: {}",
                ctx.text(troop_key(selected.kind)),
                selected.headcount,
                selected.capacity,
                ctx.text("movement_left"),
                remaining,
                ctx.text("movement_spent"),
                selected.movement_spent
            ),
            vec2(112.0, 264.0),
            20.0,
            CREAM,
        );
        if let Some(recovery) = &ctx.army.recovery {
            let message = if let Some(reason) = &recovery.blocked {
                format!("{}: {reason}", ctx.text("recovery"))
            } else {
                format!(
                    "{}: +{} / {} {} · {} {}. {}",
                    ctx.text("next_recovery"),
                    recovery.restored,
                    recovery.maximum,
                    ctx.text("troops"),
                    recovery.gold_cost,
                    ctx.text("gold"),
                    ctx.text("recovery_forecast")
                )
            };
            block(ctx, &message, vec2(112.0, 300.0), 1040.0, MUTED);
        }
        if let Some(result) = campaign
            .factions
            .iter()
            .find(|faction| faction.id == campaign.observer)
            .and_then(|faction| faction.last_recovery.as_ref())
            .and_then(|statement| {
                statement
                    .entries
                    .iter()
                    .find(|entry| entry.formation == selected.id)
            })
        {
            body(
                ctx,
                &format!(
                    "{}: +{} · {} {}",
                    ctx.text("last_recovery"),
                    result.restored,
                    result.gold_cost,
                    ctx.text("gold")
                ),
                vec2(112.0, 369.0),
                18.0,
                CREAM,
            );
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 410.0, 496.0, 48.0),
        &ctx.text("move_army"),
        campaign.player_turn,
        true,
    ) {
        return Some(UiAction::BeginMove(army.id));
    }
    if button(
        ctx,
        Rect::new(672.0, 410.0, 496.0, 48.0),
        &ctx.text("army_people"),
        true,
        false,
    ) {
        return Some(UiAction::ArmyPeople);
    }
    if button(
        ctx,
        Rect::new(112.0, 478.0, 496.0, 48.0),
        &ctx.text("transfer_formation"),
        selected.is_some(),
        false,
    ) {
        return selected.map(|formation| UiAction::BeginTransferFormation(formation.id));
    }
    if button(
        ctx,
        Rect::new(672.0, 478.0, 496.0, 48.0),
        &ctx.text("disband"),
        selected.is_some() && campaign.player_turn,
        false,
    ) {
        return selected.map(|formation| UiAction::AskDisband(formation.id));
    }
    if ctx.army.status.is_empty() {
        block(
            ctx,
            &ctx.text(if campaign.player_turn {
                "transfer_preserves_movement"
            } else {
                "transfer_paused_help"
            }),
            vec2(112.0, 573.0),
            1040.0,
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
        return Some(UiAction::CancelArmyAction);
    }
    None
}
