//! All local named people remain selectable through a bounded page.

use super::super::portraits;
use super::*;
use kestrum::state::people::{Person, PersonAssignment, PersonStatus};

pub fn local_people<'a>(view: &ArmyView, campaign: &'a VisibleCampaign) -> Vec<&'a Person> {
    campaign
        .people
        .iter()
        .filter(|person| {
            !matches!(
                person.status,
                PersonStatus::Dead { .. } | PersonStatus::Displaced { .. }
            )
        })
        .filter(|person| match person.assignment {
            PersonAssignment::Site { site } => Some(site) == view.site,
            PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => {
                Some(site) == view.site
            }
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
    let visible: Vec<_> = people
        .into_iter()
        .skip(page * PEOPLE_PAGE_SIZE)
        .take(PEOPLE_PAGE_SIZE)
        .collect();
    if let Some(action) = person_rows(ctx, campaign, &visible) {
        return Some(action);
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
        Rect::new(840.0, 626.0, 200.0, 48.0),
        &ctx.text("households_open"),
        true,
        false,
    ) {
        return Some(UiAction::ArmyHouseholds);
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

fn person_rows(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    people: &[&Person],
) -> Option<UiAction> {
    for (index, person) in people.iter().enumerate() {
        let y = 219.0 + index as f32 * 81.0;
        let age = person.age_years(campaign.completed_rounds);
        portraits::draw(
            ctx.portraits,
            Some(&person.appearance),
            Some(age),
            ctx.household_rules.service_minimum_age_years,
            Rect::new(112.0, y - 23.0, 64.0, 64.0),
        );
        let name =
            truncate_text_to_width_ex(&person_name(ctx, person), 324.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(188.0, y), 20.0, CREAM);
        let remaining = ctx
            .army
            .person_remaining
            .get(&person.id)
            .copied()
            .unwrap_or(0);
        let assignment = match person.assignment {
            PersonAssignment::Site { .. } => ctx.text("assigned_site"),
            PersonAssignment::Dependent { .. } => ctx.text("dependent"),
            PersonAssignment::Trainee { .. } => ctx.text("trainee"),
            PersonAssignment::Formation { formation: id } => formation(campaign, id)
                .map(|formation| ctx.text(troop_key(formation.kind)))
                .unwrap_or_default(),
            PersonAssignment::Dead => continue,
        };
        body(
            ctx,
            &format!(
                "{} | {assignment} | {} | {}: {}",
                class_name(ctx, person.class),
                ctx.text("person_age").replace("{age}", &age.to_string()),
                ctx.text("movement_left"),
                remaining
            ),
            vec2(188.0, y + 42.0),
            16.0,
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
        if button(
            ctx,
            Rect::new(752.0, y - 23.0, 194.0, 48.0),
            &ctx.text("history_service"),
            true,
            false,
        ) {
            return Some(UiAction::OpenHistory(
                kestrum::state::history::HistorySubject::Person(person.id),
            ));
        }
        if button(
            ctx,
            Rect::new(526.0, y - 23.0, 210.0, 48.0),
            &ctx.text("career_open"),
            true,
            false,
        ) {
            return Some(UiAction::OpenPersonProgression(person.id));
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
            PersonStatus::Dead { .. } | PersonStatus::Displaced { .. } => continue,
        };
        let traits = person
            .career
            .traits
            .iter()
            .map(|trait_kind| {
                ctx.text(match trait_kind {
                    kestrum::state::people::PersonTrait::Bold => "trait_bold",
                    kestrum::state::people::PersonTrait::Protective => "trait_protective",
                    kestrum::state::people::PersonTrait::NaturalCommander => {
                        "trait_natural_commander"
                    }
                })
            })
            .collect::<Vec<_>>()
            .join(" · ");
        let epithet = person
            .career
            .recognition
            .as_ref()
            .map(|record| format!(" · {}", record.epithet))
            .unwrap_or_default();
        let note = format!(
            "{status}{}{}",
            if traits.is_empty() {
                String::new()
            } else {
                format!(" · {traits}")
            },
            epithet
        );
        let note = truncate_text_to_width_ex(&note, 980.0, ctx.body_font(), 14.0);
        body(ctx, &note, vec2(188.0, y + 61.0), 14.0, MUTED);
        draw_line(
            112.0,
            y + 68.0,
            1168.0,
            y + 68.0,
            1.0,
            Color::new(0.21, 0.29, 0.25, 1.0),
        );
    }
    None
}

fn class_name(ctx: &Context<'_>, class: kestrum::data::world::PersonClass) -> String {
    ctx.text(match class {
        kestrum::data::world::PersonClass::Recruit => "class_recruit",
        kestrum::data::world::PersonClass::Infantry => "class_infantry",
        kestrum::data::world::PersonClass::Archer => "class_archer",
        kestrum::data::world::PersonClass::Scout => "class_scout",
        kestrum::data::world::PersonClass::Cavalry => "class_cavalry",
        kestrum::data::world::PersonClass::Medic => "class_medic",
        kestrum::data::world::PersonClass::Officer => "class_officer",
    })
}
