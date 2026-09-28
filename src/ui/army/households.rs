//! Locally witnessed family choices and evidence based succession.

mod review;
use super::*;
use kestrum::engine::HouseholdAction;
use kestrum::state::{
    people::{Person, PersonAssignment, PersonStatus},
    relationships::{FamilyOrigin, HouseholdStatus, LegacyCategory, SuccessorLink},
};

const ROWS_PER_PAGE: usize = 5;

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    legacy: bool,
) -> Option<UiAction> {
    if let Some(action) = ctx.army.household_review {
        return review::draw(ctx, campaign, action);
    }
    let people = super::people::local_people(ctx.army, campaign);
    let page_count = people.len().div_ceil(ROWS_PER_PAGE).max(1);
    let page = ctx.army.household_page.min(page_count - 1);
    block(
        ctx,
        &ctx.text(if legacy {
            "legacy_help"
        } else {
            "households_help"
        }),
        vec2(112.0, 174.0),
        1040.0,
        MUTED,
    );
    let visible = people
        .into_iter()
        .skip(page * ROWS_PER_PAGE)
        .take(ROWS_PER_PAGE)
        .collect::<Vec<_>>();
    if visible.is_empty() {
        body(
            ctx,
            &ctx.text("no_local_people"),
            vec2(112.0, 228.0),
            20.0,
            CREAM,
        );
    }
    for (index, person) in visible.iter().enumerate() {
        person_row(ctx, campaign, person, index, legacy);
        let y = 227.0 + index as f32 * 52.0;
        if button(
            ctx,
            Rect::new(924.0, y - 24.0, 112.0, 48.0),
            &ctx.text("choose_first"),
            true,
            ctx.army.household_first == Some(person.id),
        ) {
            return Some(UiAction::SelectHouseholdPerson(person.id, 0));
        }
        if button(
            ctx,
            Rect::new(1044.0, y - 24.0, 124.0, 48.0),
            &ctx.text(if legacy {
                "choose_heir"
            } else {
                "choose_partner"
            }),
            true,
            ctx.army.household_second == Some(person.id),
        ) {
            return Some(UiAction::SelectHouseholdPerson(person.id, 1));
        }
    }
    if let Some(action) = paging(ctx, page, page_count) {
        return Some(action);
    }
    if legacy {
        legacy_controls(ctx)
    } else {
        household_controls(ctx, campaign)
    }
}

fn paging(ctx: &Context<'_>, page: usize, page_count: usize) -> Option<UiAction> {
    if button(
        ctx,
        Rect::new(112.0, 466.0, 150.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::HouseholdPage(-1));
    }
    centered(
        ctx,
        &format!("{} / {page_count}", page + 1),
        vec2(640.0, 497.0),
        18.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1018.0, 466.0, 150.0, 48.0),
        &ctx.text("next"),
        page + 1 < page_count,
        false,
    ) {
        return Some(UiAction::HouseholdPage(1));
    }
    None
}

fn person_row(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    person: &Person,
    index: usize,
    legacy: bool,
) {
    let y = 227.0 + index as f32 * 52.0;
    let selected =
        ctx.army.household_first == Some(person.id) || ctx.army.household_second == Some(person.id);
    if selected {
        draw_rectangle(
            112.0,
            y - 24.0,
            798.0,
            48.0,
            Color::new(0.16, 0.23, 0.21, 1.0),
        );
        draw_rectangle_lines(112.0, y - 24.0, 798.0, 48.0, 1.0, BRASS);
    }
    let name = truncate_text_to_width_ex(&person.name, 300.0, ctx.body_font(), 18.0);
    body(ctx, &name, vec2(124.0, y - 1.0), 18.0, CREAM);
    let age = person.age_years(campaign.completed_rounds);
    let assignment = match person.assignment {
        PersonAssignment::Formation { .. } => ctx.text("in_service"),
        PersonAssignment::Site { .. } => ctx.text("assigned_site"),
        PersonAssignment::Dependent { .. } => ctx.text("dependent"),
        PersonAssignment::Trainee { .. } => ctx.text("trainee"),
        PersonAssignment::Dead => return,
    };
    let status = if matches!(person.status, PersonStatus::Wounded { .. }) {
        format!(" · {}", ctx.text("person_wounded"))
    } else {
        String::new()
    };
    let role = if legacy {
        ctx.text("person_legacy_role")
    } else {
        family_role(ctx, campaign, person)
    };
    let detail = format!(
        "{} · {assignment} · {age} {}{status}",
        role,
        ctx.text("years_old")
    );
    let detail = truncate_text_to_width_ex(&detail, 440.0, ctx.body_font(), 15.0);
    body(ctx, &detail, vec2(438.0, y - 1.0), 15.0, MUTED);
}

fn household_controls(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
    let actions = [
        HouseholdAction::Form,
        HouseholdAction::Adopt,
        HouseholdAction::Children,
        HouseholdAction::End,
        HouseholdAction::Train,
        HouseholdAction::Service,
        HouseholdAction::Invite,
    ];
    for (index, action) in actions.into_iter().enumerate() {
        let label = review::action_label(ctx, campaign, action);
        if button(
            ctx,
            Rect::new(
                112.0 + (index % 4) as f32 * 264.0,
                521.0 + (index / 4) as f32 * 56.0,
                252.0,
                48.0,
            ),
            &label,
            true,
            false,
        ) {
            return Some(UiAction::ReviewHousehold(action));
        }
    }
    if button(
        ctx,
        Rect::new(904.0, 577.0, 264.0, 48.0),
        &ctx.text("legacy_open"),
        true,
        false,
    ) {
        return Some(UiAction::ArmyLegacy);
    }
    button(
        ctx,
        Rect::new(112.0, 633.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::CancelArmyAction)
}

fn family_role(ctx: &Context<'_>, campaign: &VisibleCampaign, person: &Person) -> String {
    let Some(family) = campaign.families.get(&person.id) else {
        return if campaign.households.iter().any(|household| {
            household.partners.contains(&person.id)
                && matches!(household.status, HouseholdStatus::Active)
        }) {
            ctx.text("family_partner")
        } else {
            ctx.text("person_family_role")
        };
    };
    let names = family
        .links
        .keys()
        .filter_map(|id| {
            campaign
                .people
                .iter()
                .find(|relative| relative.id == *id)
                .map(|relative| relative.name.as_str())
        })
        .collect::<Vec<_>>()
        .join(", ");
    match family.origin {
        FamilyOrigin::Birth => ctx.text("family_birth_role").replace("{names}", &names),
        FamilyOrigin::AdoptedWard => ctx.text("family_adopted_role").replace("{names}", &names),
        FamilyOrigin::LocalApprentice => ctx.text("family_apprentice_role"),
    }
}

fn legacy_controls(ctx: &Context<'_>) -> Option<UiAction> {
    let categories = [
        (LegacyCategory::Command, "legacy_command"),
        (LegacyCategory::Item, "legacy_item"),
        (LegacyCategory::Household, "legacy_household"),
        (LegacyCategory::Institution, "legacy_institution"),
    ];
    for (index, (category, key)) in categories.into_iter().enumerate() {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 264.0, 521.0, 252.0, 48.0),
            &ctx.text(key),
            true,
            ctx.army.legacy_category == category,
        ) {
            return Some(UiAction::SelectLegacyCategory(category));
        }
    }
    let links = [
        (SuccessorLink::Blood, "link_blood"),
        (SuccessorLink::Martial, "link_martial"),
        (SuccessorLink::Religious, "link_religious"),
        (SuccessorLink::Political, "link_political"),
        (SuccessorLink::Adopted, "link_adopted"),
    ];
    for (index, (link, key)) in links.into_iter().enumerate() {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 211.0, 577.0, 202.0, 48.0),
            &ctx.text(key),
            true,
            ctx.army.legacy_link == link,
        ) {
            return Some(UiAction::SelectLegacyLink(link));
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 633.0, 360.0, 48.0),
        &ctx.text("legacy_designate"),
        true,
        false,
    ) {
        return Some(UiAction::ReviewHousehold(HouseholdAction::Designate));
    }
    if button(
        ctx,
        Rect::new(492.0, 633.0, 250.0, 48.0),
        &ctx.text("households_open"),
        true,
        false,
    ) {
        return Some(UiAction::ArmyHouseholds);
    }
    if button(
        ctx,
        Rect::new(988.0, 633.0, 180.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::CancelArmyAction);
    }
    None
}
