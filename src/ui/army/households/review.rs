//! Touch-accessible explanations occupy the review before any irreversible family choice.
use super::*;

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    action: HouseholdAction,
) -> Option<UiAction> {
    body(
        ctx,
        &action_label(ctx, campaign, action),
        vec2(112.0, 183.0),
        24.0,
        CREAM,
    );
    let explanation = ctx
        .text(match action {
            HouseholdAction::Form => "family_review_form",
            HouseholdAction::Adopt => "family_review_adopt",
            HouseholdAction::Children => "family_review_children",
            HouseholdAction::End => "family_review_end",
            HouseholdAction::Train => "family_review_train",
            HouseholdAction::Service => "family_review_service",
            HouseholdAction::Invite => "family_review_invite",
            HouseholdAction::Designate => "family_review_designate",
        })
        .replace(
            "{age}",
            &ctx.household_rules
                .partnership_minimum_age_years
                .to_string(),
        )
        .replace(
            "{seasons}",
            &ctx.household_rules.partnership_shared_seasons.to_string(),
        )
        .replace(
            "{gold}",
            &ctx.household_rules.apprentice_cost_gold.to_string(),
        );
    block(ctx, &explanation, vec2(112.0, 219.0), 1040.0, MUTED);
    selected_people(ctx, campaign, action);
    let option = ctx.army.household_option.as_ref();
    let ready = option.is_some_and(|option| option.blocked.is_none() && option.command.is_some());
    let reason = option
        .and_then(|option| option.blocked.as_deref())
        .unwrap_or_else(|| {
            ctx.game_text.text(if ready {
                "family_review_ready"
            } else {
                "family_review_wait"
            })
        });
    body(
        ctx,
        &ctx.text(if ready {
            "family_ready"
        } else {
            "family_blocked"
        }),
        vec2(112.0, 500.0),
        21.0,
        BRASS,
    );
    block(ctx, reason, vec2(112.0, 536.0), 1040.0, CREAM);
    if button(
        ctx,
        Rect::new(868.0, 633.0, 300.0, 48.0),
        &ctx.text("family_confirm"),
        ready,
        false,
    ) {
        return Some(UiAction::ConfirmHousehold);
    }
    button(
        ctx,
        Rect::new(112.0, 633.0, 210.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::CancelHouseholdReview)
}

fn selected_people(ctx: &Context<'_>, campaign: &VisibleCampaign, action: HouseholdAction) {
    if action == HouseholdAction::Invite {
        return;
    }
    for (index, selected) in [ctx.army.household_first, ctx.army.household_second]
        .into_iter()
        .enumerate()
    {
        if index == 1 && !matches!(action, HouseholdAction::Form | HouseholdAction::Designate) {
            continue;
        }
        let Some(person) =
            selected.and_then(|id| campaign.people.iter().find(|person| person.id == id))
        else {
            continue;
        };
        let label = format!(
            "{}: {} / {} {}",
            ctx.text(if index == 0 {
                "choose_first"
            } else {
                "family_second"
            }),
            person.name,
            person.age_years(campaign.completed_rounds),
            ctx.text("years_old")
        );
        block(
            ctx,
            &label,
            vec2(112.0, 328.0 + index as f32 * 60.0),
            1040.0,
            CREAM,
        );
    }
    if action == HouseholdAction::Designate {
        let category = ctx.text(match ctx.army.legacy_category {
            LegacyCategory::Command => "legacy_command",
            LegacyCategory::Item => "legacy_item",
            LegacyCategory::Household => "legacy_household",
            LegacyCategory::Institution => "legacy_institution",
        });
        let link = ctx.text(match ctx.army.legacy_link {
            SuccessorLink::Blood => "link_blood",
            SuccessorLink::Adopted => "link_adopted",
            SuccessorLink::Martial => "link_martial",
            SuccessorLink::Religious => "link_religious",
            SuccessorLink::Political => "link_political",
        });
        body(
            ctx,
            &format!("{category} / {link}"),
            vec2(112.0, 452.0),
            18.0,
            MUTED,
        );
    } else if action == HouseholdAction::Form {
        if let Some((first, second)) = ctx.army.household_first.zip(ctx.army.household_second) {
            let seasons = campaign
                .people
                .iter()
                .find(|person| person.id == first)
                .and_then(|person| person.career.relationships.get(&second))
                .map_or(0, |relation| relation.shared_service_seasons);
            let text = ctx
                .text("family_shared_seasons")
                .replace("{count}", &seasons.to_string());
            body(ctx, &text, vec2(112.0, 452.0), 18.0, MUTED);
        }
    }
}

pub(super) fn action_label(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    action: HouseholdAction,
) -> String {
    let key = match action {
        HouseholdAction::Form => "household_form",
        HouseholdAction::Adopt => "household_adopt",
        HouseholdAction::Children => {
            let raising = ctx.army.household_first.is_some_and(|id| {
                campaign.households.iter().any(|household| {
                    household.partners.contains(&id)
                        && household.status == HouseholdStatus::Active
                        && household.raising_children
                })
            });
            if raising {
                "household_pause_births"
            } else {
                "household_raise"
            }
        }
        HouseholdAction::End => "household_end",
        HouseholdAction::Train => "household_train",
        HouseholdAction::Service => "household_service",
        HouseholdAction::Invite => "household_apprentice",
        HouseholdAction::Designate => "legacy_designate",
    };
    ctx.text(key)
}
