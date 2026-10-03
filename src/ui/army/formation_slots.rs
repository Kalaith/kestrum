//! Each formation slot holds one named member plus troops, or troops alone.

use super::super::portraits;
use super::*;
use kestrum::state::{
    evidence::Veterancy,
    people::{PersonAssignment, PersonStatus},
};

pub(super) fn draw(ctx: &Context<'_>, campaign: &VisibleCampaign, army: &Army) -> Option<UiAction> {
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
            &(index + 1).to_string(),
            vec2(rect.x + 12.0, rect.y + 32.0),
            18.0,
            MUTED,
        );
        if let Some(member) = slot.and_then(|id| formation(campaign, id)) {
            occupied(ctx, campaign, member, rect);
            if tapped(ctx, rect) {
                return Some(UiAction::SelectFormation(member.id));
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
    None
}

fn occupied(ctx: &Context<'_>, campaign: &VisibleCampaign, member: &Formation, rect: Rect) {
    let person = first_member(campaign, member);
    if let Some(person) = person {
        portraits::draw(
            ctx.portraits,
            Some(&person.appearance),
            Some(person.age_years(campaign.completed_rounds)),
            ctx.household_rules.service_minimum_age_years,
            Rect::new(rect.x + 34.0, rect.y + 6.0, 40.0, 40.0),
        );
    }
    let count = format!("{} / {}", member.headcount, member.capacity);
    let count_width = measure_text(&count, ctx.body_font(), 18, 1.0).width;
    let count_x = rect.x + rect.w - 16.0 - count_width;
    let label_x = rect.x + if person.is_some() { 90.0 } else { 46.0 };
    body(
        ctx,
        &label(ctx, campaign, member, count_x - label_x - 16.0),
        vec2(label_x, rect.y + 23.0),
        20.0,
        CREAM,
    );
    body(ctx, &count, vec2(count_x, rect.y + 23.0), 18.0, CREAM);
    let remaining = ctx
        .army
        .member_remaining
        .get(&member.id)
        .copied()
        .unwrap_or(0);
    let detail = format!(
        "{} · {}: {remaining} · {}: {}",
        ctx.text(match member.service.tier {
            Veterancy::Ordinary => "tier_ordinary",
            Veterancy::Seasoned => "tier_seasoned",
            Veterancy::Veteran => "tier_veteran",
        }),
        ctx.text("movement_left"),
        ctx.text("movement_spent"),
        member.movement_spent,
    );
    body(ctx, &detail, vec2(label_x, rect.y + 44.0), 16.0, MUTED);
}

fn label(ctx: &Context<'_>, campaign: &VisibleCampaign, member: &Formation, width: f32) -> String {
    let troop = ctx.text(troop_key(member.kind));
    let first = first_member(campaign, member);
    let Some(first) = first else {
        return troop;
    };
    let suffix = format!(" + {troop}");
    // Preserve troop type when the person's name is long.
    let name_width = (width - measure_text(&suffix, ctx.body_font(), 20, 1.0).width).max(0.0);
    let name =
        truncate_text_to_width_ex(&person_name(ctx, first), name_width, ctx.body_font(), 20.0);
    format!("{name}{suffix}")
}

fn first_member<'a>(
    campaign: &'a VisibleCampaign,
    member: &Formation,
) -> Option<&'a kestrum::state::people::Person> {
    campaign.people.iter().find(|person| {
        person.assignment
            == (PersonAssignment::Formation {
                formation: member.id,
            })
            && !matches!(
                person.status,
                PersonStatus::Dead { .. } | PersonStatus::Displaced { .. }
            )
    })
}
