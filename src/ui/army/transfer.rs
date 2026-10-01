//! Choose a co-located recipient, then a real slot or surviving formation.

use super::*;

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    let view = &ctx.army.transfer;
    let subject = view.subject?;
    let label = match subject {
        TransferSubject::Formation(id) => formation(campaign, id)
            .map(|formation| formation_label(ctx, campaign, formation, 822.0))
            .unwrap_or_default(),
        TransferSubject::Person(id) => campaign
            .people
            .iter()
            .find(|person| person.id == id)
            .map(|person| person.name.clone())
            .unwrap_or_default(),
    };
    body(
        ctx,
        &truncate_text_to_width_ex(&label, 822.0, ctx.body_font(), 21.0),
        vec2(112.0, 177.0),
        21.0,
        CREAM,
    );
    let action = if let Some(army) = view
        .army
        .and_then(|id| campaign.armies.iter().find(|army| army.id == id))
    {
        if button(
            ctx,
            Rect::new(966.0, 153.0, 202.0, 48.0),
            &ctx.text("choose_army"),
            true,
            false,
        ) {
            return Some(UiAction::ClearTransferArmy);
        }
        slots(ctx, campaign, army, subject)
    } else {
        recipients(ctx, campaign)
    };
    if action.is_some() {
        return action;
    }
    if ctx.army.status.is_empty() {
        let reason = view
            .split_blocked
            .as_ref()
            .filter(|_| view.army.is_none() && matches!(subject, TransferSubject::Formation(_)))
            .or(view.blocked.as_ref())
            .map(String::as_str)
            .unwrap_or_else(|| ctx.data.text("transfer_preserves_movement"));
        for (index, line) in wrap_text_ex(reason, 1040.0, ctx.body_font(), 18.0)
            .iter()
            .take(2)
            .enumerate()
        {
            body(
                ctx,
                line,
                vec2(112.0, 594.0 + index as f32 * 22.0),
                18.0,
                if view.blocked.is_some() { BRASS } else { MUTED },
            );
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
    if let TransferSubject::Formation(formation) = subject {
        if view.army.is_none()
            && button(
                ctx,
                Rect::new(356.0, 626.0, 380.0, 48.0),
                &ctx.text("split_new_army"),
                view.split_blocked.is_none(),
                false,
            )
        {
            return Some(UiAction::SplitArmy(formation));
        }
    }
    let selected = match subject {
        TransferSubject::Formation(_) => view.army.is_some() && view.slot.is_some(),
        TransferSubject::Person(_) => view.formation.is_some(),
    };
    if button(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        &ctx.text("confirm_transfer"),
        selected && view.blocked.is_none(),
        true,
    ) {
        return Some(UiAction::ConfirmTransfer);
    }
    None
}

fn recipients(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("transfer_choose_army"),
        vec2(112.0, 211.0),
        18.0,
        MUTED,
    );
    let armies = ctx.army.armies_at_site(campaign);
    let page_count = armies.len().div_ceil(TRANSFER_PAGE_SIZE).max(1);
    let page = ctx.army.transfer.page.min(page_count - 1);
    for (index, army) in armies
        .iter()
        .skip(page * TRANSFER_PAGE_SIZE)
        .take(TRANSFER_PAGE_SIZE)
        .enumerate()
    {
        let rect = Rect::new(112.0, 232.0 + index as f32 * 58.0, 1056.0, 50.0);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.10, 0.17, 0.16, 1.0),
        );
        let name = truncate_text_to_width_ex(&army.name, 795.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(rect.x + 14.0, rect.y + 32.0), 20.0, CREAM);
        let filled = army.formation_ids().count();
        body(
            ctx,
            &format!("{filled} / 6 {}", ctx.text("formation_slots")),
            vec2(948.0, rect.y + 31.0),
            18.0,
            MUTED,
        );
        if tapped(ctx, rect) {
            return Some(UiAction::SelectTransferArmy(army.id));
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 531.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::TransferPage(-1));
    }
    centered(
        ctx,
        &format!("{} / {page_count}", page + 1),
        vec2(640.0, 562.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 531.0, 160.0, 48.0),
        &ctx.text("next"),
        page + 1 < page_count,
        false,
    ) {
        return Some(UiAction::TransferPage(1));
    }
    None
}

fn slots(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    army: &Army,
    subject: TransferSubject,
) -> Option<UiAction> {
    body(
        ctx,
        &truncate_text_to_width_ex(&army.name, 1040.0, ctx.body_font(), 20.0),
        vec2(112.0, 212.0),
        20.0,
        MUTED,
    );
    for (index, slot) in army.slots.iter().enumerate() {
        let rect = Rect::new(112.0, 227.0 + index as f32 * 57.0, 1056.0, 50.0);
        let selected = match subject {
            TransferSubject::Formation(_) => ctx.army.transfer.slot == Some(index as u8),
            TransferSubject::Person(_) => slot.is_some() && ctx.army.transfer.formation == *slot,
        };
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
        let label = slot
            .and_then(|id| formation(campaign, id))
            .map(|formation| formation_label(ctx, campaign, formation, 980.0))
            .unwrap_or_else(|| {
                ctx.text(if matches!(subject, TransferSubject::Person(_)) {
                    "person_needs_formation"
                } else {
                    "empty_formation_slot"
                })
            });
        body(
            ctx,
            &format!("{}   {label}", index + 1),
            vec2(rect.x + 14.0, rect.y + 32.0),
            20.0,
            CREAM,
        );
        if tapped(ctx, rect) {
            match subject {
                TransferSubject::Formation(_) => {
                    return Some(UiAction::SelectTransferSlot(index as u8))
                }
                TransferSubject::Person(_) => {
                    if let Some(formation) = slot {
                        return Some(UiAction::SelectTransferFormation(*formation));
                    }
                }
            }
        }
    }
    None
}

fn formation_label(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    formation: &Formation,
    width: f32,
) -> String {
    let troops = format!(
        "{} · {} / {}",
        ctx.text(troop_key(formation.kind)),
        formation.headcount,
        formation.capacity
    );
    let member = campaign.people.iter().find(|person| {
        person.assignment
            == (kestrum::state::people::PersonAssignment::Formation {
                formation: formation.id,
            })
    });
    let Some(person) = member else {
        return troops;
    };
    let suffix = format!(" + {troops}");
    let name_width = (width - measure_text(&suffix, ctx.body_font(), 20, 1.0).width).max(0.0);
    let name =
        truncate_text_to_width_ex(&person_name(ctx, person), name_width, ctx.body_font(), 20.0);
    format!("{name}{suffix}")
}
