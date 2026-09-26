//! All local named people remain selectable through a bounded page.

use super::*;
use kestrum::state::people::{Person, PersonAssignment, PersonStatus};

pub fn local_people<'a>(view: &ArmyView, campaign: &'a VisibleCampaign) -> Vec<&'a Person> {
    campaign
        .people
        .iter()
        .filter(|person| match person.assignment {
            PersonAssignment::Site { site } => Some(site) == view.site,
            PersonAssignment::Formation { formation } => campaign
                .armies
                .iter()
                .any(|army| Some(army.site) == view.site && army.slots.contains(&Some(formation))),
            PersonAssignment::Dead => false,
        })
        .collect()
}

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    block(
        ctx,
        &ctx.text("people_transfer_help"),
        vec2(112.0, 179.0),
        1040.0,
        MUTED,
    );
    let people = local_people(ctx.army, campaign);
    let page_count = people.len().div_ceil(PEOPLE_PAGE_SIZE).max(1);
    let page = ctx.army.people_page.min(page_count - 1);
    if people.is_empty() {
        body(
            ctx,
            &ctx.text("no_local_people"),
            vec2(112.0, 260.0),
            21.0,
            CREAM,
        );
    }
    for (index, person) in people
        .iter()
        .skip(page * PEOPLE_PAGE_SIZE)
        .take(PEOPLE_PAGE_SIZE)
        .enumerate()
    {
        let y = 219.0 + index as f32 * 81.0;
        let name = truncate_text_to_width_ex(&person.name, 822.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(112.0, y), 20.0, CREAM);
        let remaining = ctx
            .army
            .person_remaining
            .get(&person.id)
            .copied()
            .unwrap_or(0);
        let assignment = match person.assignment {
            PersonAssignment::Site { .. } => ctx.text("assigned_site"),
            PersonAssignment::Formation { formation: id } => formation(campaign, id)
                .map(|formation| ctx.text(troop_key(formation.kind)))
                .unwrap_or_default(),
            PersonAssignment::Dead => continue,
        };
        body(
            ctx,
            &format!(
                "{assignment} · {}: {remaining} · {}: {}",
                ctx.text("movement_left"),
                ctx.text("movement_spent"),
                person.movement_spent
            ),
            vec2(112.0, y + 25.0),
            18.0,
            MUTED,
        );
        if button(
            ctx,
            Rect::new(964.0, y - 23.0, 204.0, 48.0),
            &ctx.text("transfer"),
            true,
            false,
        ) {
            return Some(UiAction::BeginTransferPerson(person.id));
        }
        let status = match person.status {
            PersonStatus::Fit => ctx.text("person_fit"),
            PersonStatus::Wounded {
                remaining_steps, ..
            } => format!(
                "{} · {remaining_steps} {}",
                ctx.text("person_wounded"),
                ctx.text("wound_steps_remaining")
            ),
            PersonStatus::Dead { .. } => continue,
        };
        body(ctx, &status, vec2(112.0, y + 46.0), 16.0, MUTED);
        draw_line(
            112.0,
            y + 64.0,
            1168.0,
            y + 64.0,
            1.0,
            Color::new(0.21, 0.29, 0.25, 1.0),
        );
    }
    if button(
        ctx,
        Rect::new(112.0, 537.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::ArmyPeoplePage(-1));
    }
    centered(
        ctx,
        &format!("{} / {page_count}", page + 1),
        vec2(640.0, 568.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 537.0, 160.0, 48.0),
        &ctx.text("next"),
        page + 1 < page_count,
        false,
    ) {
        return Some(UiAction::ArmyPeoplePage(1));
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
