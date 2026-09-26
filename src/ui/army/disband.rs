//! A separate irreversible composition decision with the people policy stated.

use super::*;

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    id: FormationId,
) -> Option<UiAction> {
    if let Some(formation) = formation(campaign, id) {
        text(
            ctx,
            &ctx.text(troop_key(formation.kind)),
            vec2(160.0, 231.0),
            28.0,
            CREAM,
        );
        body(
            ctx,
            &format!("{} / {}", formation.headcount, formation.capacity),
            vec2(160.0, 268.0),
            21.0,
            CREAM,
        );
        let mut y = block(
            ctx,
            &ctx.text("disband_warning"),
            vec2(160.0, 321.0),
            960.0,
            CREAM,
        ) + 24.0;
        y = block(
            ctx,
            &ctx.text("disband_people"),
            vec2(160.0, y),
            960.0,
            CREAM,
        ) + 24.0;
        if campaign
            .armies
            .iter()
            .find(|army| army.slots.contains(&Some(id)))
            .is_some_and(|army| army.slots.iter().flatten().count() == 1)
        {
            block(ctx, &ctx.text("disband_last"), vec2(160.0, y), 960.0, BRASS);
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("cancel"),
        true,
        false,
    ) {
        return Some(UiAction::CancelArmyAction);
    }
    if button(
        ctx,
        Rect::new(836.0, 626.0, 332.0, 48.0),
        &ctx.text("confirm_disband"),
        campaign.player_turn && formation(campaign, id).is_some(),
        true,
    ) {
        return Some(UiAction::ConfirmDisband(id));
    }
    None
}
