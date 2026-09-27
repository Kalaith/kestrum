//! Locally witnessed family choices and evidence based succession.

use super::*;
use kestrum::state::{
    people::{Person, PersonAssignment, PersonStatus},
    relationships::{FamilyLink, FamilyOrigin, HouseholdStatus, LegacyCategory, SuccessorLink},
};

const ROWS_PER_PAGE: usize = 5;

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    legacy: bool,
) -> Option<UiAction> {
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
    if legacy {
        legacy_controls(ctx, campaign)
    } else {
        household_controls(ctx, campaign)
    }
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
    let site = ctx.army.site?;
    let first = ctx.army.household_first;
    let second = ctx.army.household_second;
    let first_person = first.and_then(|id| campaign.people.iter().find(|person| person.id == id));
    let second_person = second.and_then(|id| campaign.people.iter().find(|person| person.id == id));
    let household = first.and_then(|id| {
        campaign.households.iter().find(|household| {
            household.partners.contains(&id) && matches!(household.status, HouseholdStatus::Active)
        })
    });
    let adult_at_site = |person: Option<&Person>| {
        person.is_some_and(|person| {
            person.is_alive()
                && !person.career.retired
                && person.age_years(campaign.completed_rounds)
                    >= ctx.household_rules.partnership_minimum_age_years
                && person_site(campaign, person) == Some(site)
        })
    };
    let has_active_partner = |person: Option<&Person>| {
        person.is_some_and(|person| {
            campaign.households.iter().any(|household| {
                household.partners.contains(&person.id)
                    && matches!(household.status, HouseholdStatus::Active)
            })
        })
    };
    let known_kin = first_person
        .zip(second_person)
        .is_some_and(|(first, second)| {
            let direct = campaign.families.get(&first.id).is_some_and(|family| {
                family
                    .links
                    .get(&second.id)
                    .is_some_and(|link| *link != FamilyLink::ApprenticeOf)
            }) || campaign.families.get(&second.id).is_some_and(|family| {
                family
                    .links
                    .get(&first.id)
                    .is_some_and(|link| *link != FamilyLink::ApprenticeOf)
            });
            let shared_guardian = campaign.families.get(&first.id).is_some_and(|family| {
                campaign.families.get(&second.id).is_some_and(|other| {
                    family.links.iter().any(|(guardian, link)| {
                        *link != FamilyLink::ApprenticeOf
                            && other
                                .links
                                .get(guardian)
                                .is_some_and(|other_link| *other_link != FamilyLink::ApprenticeOf)
                    })
                })
            });
            direct || shared_guardian
        });
    let familiar = first_person
        .zip(second_person)
        .is_some_and(|(first, second)| {
            first
                .career
                .relationships
                .get(&second.id)
                .is_some_and(|relation| {
                    relation.shared_service_seasons
                        >= ctx.household_rules.partnership_shared_seasons
                })
        });
    let form_eligible = safe_site(campaign, site)
        && adult_at_site(first_person)
        && adult_at_site(second_person)
        && first != second
        && !has_active_partner(first_person)
        && !has_active_partner(second_person)
        && !known_kin
        && familiar;
    let dependent_count = |guardian: PersonId| {
        campaign
            .families
            .iter()
            .filter(|(_id, family)| {
                household.is_some_and(|household| family.household == Some(household.id))
                    || family.links.contains_key(&guardian)
            })
            .filter(|(id, _)| {
                campaign
                    .people
                    .iter()
                    .find(|person| person.id == **id)
                    .is_some_and(|person| {
                        person.is_alive()
                            && person.age_years(campaign.completed_rounds)
                                < ctx.household_rules.service_minimum_age_years
                            && matches!(
                                person.assignment,
                                PersonAssignment::Dependent { .. }
                                    | PersonAssignment::Trainee { .. }
                            )
                    })
            })
            .count()
    };
    let adopt_eligible = safe_site(campaign, site)
        && adult_at_site(first_person)
        && campaign.world.population.get(&site).copied().unwrap_or(0) > 0
        && first.is_some_and(|guardian| {
            dependent_count(guardian) < ctx.household_rules.maximum_dependent_children as usize
        });
    let toggle_label = household.map_or_else(
        || ctx.text("household_raise"),
        |household| {
            ctx.text(if household.raising_children {
                "household_pause_births"
            } else {
                "household_raise"
            })
        },
    );
    let buttons = [
        ("household_form", form_eligible),
        ("household_adopt", adopt_eligible),
        ("household_toggle", household.is_some()),
        ("household_end", household.is_some()),
    ];
    for (index, (key, enabled)) in buttons.into_iter().enumerate() {
        let label = if key == "household_toggle" {
            toggle_label.clone()
        } else {
            ctx.text(key)
        };
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 264.0, 521.0, 252.0, 48.0),
            &label,
            enabled,
            false,
        ) {
            return match key {
                "household_form" => Some(UiAction::FormHousehold(first?, second?, site)),
                "household_adopt" => Some(UiAction::AdoptWard(first?, site)),
                "household_toggle" => {
                    let household = household?;
                    Some(UiAction::SetHouseholdChildraising(
                        household.id,
                        !household.raising_children,
                    ))
                }
                "household_end" => Some(UiAction::EndHousehold(household?.id)),
                _ => None,
            };
        }
    }
    let selected = first_person;
    let trainee_or_service = selected.and_then(|person| match person.assignment {
        PersonAssignment::Dependent { site }
            if (ctx.household_rules.trainee_minimum_age_years
                ..ctx.household_rules.service_minimum_age_years)
                .contains(&person.age_years(campaign.completed_rounds)) =>
        {
            Some(("household_train", UiAction::AssignTrainee(person.id, site)))
        }
        PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. }
            if person.age_years(campaign.completed_rounds)
                >= ctx.household_rules.service_minimum_age_years =>
        {
            Some(("household_service", UiAction::EnterService(person.id, None)))
        }
        _ => None,
    });
    let no_dependents = !campaign.people.iter().any(|person| {
        person.faction == campaign.observer
            && person.is_alive()
            && person.age_years(campaign.completed_rounds)
                < ctx.household_rules.service_minimum_age_years
    });
    let own_faction = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer);
    let apprentice_eligible = safe_site(campaign, site)
        && no_dependents
        && own_faction.is_some_and(|faction| {
            !faction.deficit.unwrap_or(true)
                && faction.resources.is_some_and(|resources| {
                    resources.gold >= ctx.household_rules.apprentice_cost_gold
                })
        })
        && campaign.world.site(site).is_some_and(|entry| {
            entry.controller == Some(campaign.observer)
                && entry.habitation >= ctx.household_rules.apprentice_minimum_habitation
                && campaign.supplied_sites.contains(&site)
                && campaign.world.population.get(&site).copied().unwrap_or(0) > 0
                && campaign.world.structural_damage(site) < ctx.economy.facility_failure_damage
        })
        && campaign.apprentice_last_invited_year != Some(campaign.completed_rounds / 4);
    let row = [
        ("household_train", trainee_or_service.is_some()),
        ("household_apprentice", apprentice_eligible),
        ("legacy_open", true),
        ("back", true),
    ];
    for (index, (key, enabled)) in row.into_iter().enumerate() {
        let label = if key == "household_train" {
            trainee_or_service.map_or_else(|| ctx.text(key), |(label, _)| ctx.text(label))
        } else {
            ctx.text(key)
        };
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 264.0, 577.0, 252.0, 48.0),
            &label,
            enabled,
            false,
        ) {
            return match key {
                "household_train" => trainee_or_service.map(|(_, action)| action),
                "household_apprentice" => Some(UiAction::InviteApprentice(site)),
                "legacy_open" => Some(UiAction::ArmyLegacy),
                "back" => Some(UiAction::CancelArmyAction),
                _ => None,
            };
        }
    }
    None
}

fn safe_site(campaign: &VisibleCampaign, site: SiteId) -> bool {
    campaign.world.site(site).is_some_and(|entry| {
        entry.controller == Some(campaign.observer)
            && entry.habitation != kestrum::data::economy::Habitation::Unsettled
    }) && !campaign.sieges.iter().any(|siege| siege.site == site)
        && !campaign.threats.iter().any(|threat| threat.site == site)
        && !campaign.hostile_presence.contains(&site)
        && !campaign.battles.iter().any(|battle| {
            battle.site == site && battle.completed_rounds == campaign.completed_rounds
        })
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

fn person_site(campaign: &VisibleCampaign, person: &Person) -> Option<SiteId> {
    match person.assignment {
        PersonAssignment::Formation { formation } => campaign
            .formations
            .iter()
            .find(|entry| entry.id == formation)
            .and_then(|formation| {
                campaign
                    .armies
                    .iter()
                    .find(|army| army.formation_ids().any(|id| id == formation.id))
                    .map(|army| army.site)
            }),
        PersonAssignment::Site { site }
        | PersonAssignment::Dependent { site }
        | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Dead => None,
    }
}

fn legacy_controls(ctx: &Context<'_>, campaign: &VisibleCampaign) -> Option<UiAction> {
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
    let eligible = ctx
        .army
        .household_first
        .zip(ctx.army.household_second)
        .is_some_and(|(predecessor, successor)| {
            designation_eligible(ctx, campaign, predecessor, successor)
        });
    if button(
        ctx,
        Rect::new(112.0, 633.0, 360.0, 48.0),
        &ctx.text("legacy_designate"),
        eligible,
        false,
    ) {
        return Some(UiAction::DesignateSuccessor(
            ctx.army.household_first?,
            ctx.army.household_second?,
            ctx.army.legacy_category,
            ctx.army.legacy_link,
        ));
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

fn designation_eligible(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    predecessor_id: PersonId,
    successor_id: PersonId,
) -> bool {
    if predecessor_id == successor_id
        || campaign
            .successors
            .get(&predecessor_id)
            .is_some_and(|items| items.contains_key(&ctx.army.legacy_category))
    {
        return false;
    }
    let Some(predecessor) = campaign
        .people
        .iter()
        .find(|person| person.id == predecessor_id)
    else {
        return false;
    };
    let Some(successor) = campaign
        .people
        .iter()
        .find(|person| person.id == successor_id)
    else {
        return false;
    };
    if predecessor.faction != campaign.observer
        || successor.faction != campaign.observer
        || !successor.is_alive()
    {
        return false;
    }
    let Some(site) = person_site(campaign, predecessor) else {
        return false;
    };
    let successor_site = person_site(campaign, successor);
    let shared_seasons = predecessor
        .career
        .relationships
        .get(&successor_id)
        .map_or(0, |relationship| relationship.shared_service_seasons);
    let link_exists = match ctx.army.legacy_link {
        SuccessorLink::Blood => blood_relation(campaign, predecessor_id, successor_id),
        SuccessorLink::Adopted => adopted_relation(campaign, predecessor_id, successor_id),
        SuccessorLink::Martial => is_pupil(campaign, predecessor, successor_id),
        SuccessorLink::Religious => {
            is_pupil(campaign, predecessor, successor_id)
                && successor_site == Some(site)
                && has_facility(campaign, site, kestrum::data::world::Facility::Temple)
        }
        SuccessorLink::Political => {
            successor_site == Some(site)
                && shared_seasons >= 4
                && (predecessor.career.site_role.is_some() || successor.career.site_role.is_some())
        }
    };
    if !link_exists {
        return false;
    }
    match ctx.army.legacy_category {
        LegacyCategory::Command => campaign.armies.iter().any(|army| {
            army.commander == Some(predecessor_id)
                && successor.is_alive()
                && !successor.career.retired
                && successor.is_fit_for_field(
                    campaign.completed_rounds,
                    ctx.rules.leadership.field_min_age_years,
                )
                && successor.faction == army.faction
                && matches!(successor.assignment, PersonAssignment::Formation { formation }
                    if army.formation_ids().any(|id| id == formation))
        }),
        LegacyCategory::Item => successor_site == Some(site),
        LegacyCategory::Household => {
            successor_site == Some(site)
                && matches!(
                    ctx.army.legacy_link,
                    SuccessorLink::Blood | SuccessorLink::Adopted
                )
                && shares_household(campaign, predecessor_id, successor_id)
        }
        LegacyCategory::Institution => {
            successor_site == Some(site)
                && safe_site(campaign, site)
                && match ctx.army.legacy_link {
                    SuccessorLink::Religious => {
                        has_facility(campaign, site, kestrum::data::world::Facility::Temple)
                            && is_pupil(campaign, predecessor, successor_id)
                    }
                    SuccessorLink::Political => {
                        predecessor.career.site_role.is_some()
                            || successor.career.site_role.is_some()
                    }
                    SuccessorLink::Martial => {
                        has_facility(
                            campaign,
                            site,
                            kestrum::data::world::Facility::TrainingGround,
                        ) && is_pupil(campaign, predecessor, successor_id)
                    }
                    SuccessorLink::Blood | SuccessorLink::Adopted => false,
                }
        }
    }
}

fn has_facility(
    campaign: &VisibleCampaign,
    site: SiteId,
    facility: kestrum::data::world::Facility,
) -> bool {
    campaign
        .world
        .site(site)
        .is_some_and(|entry| entry.facilities.contains(&facility))
}

fn is_pupil(campaign: &VisibleCampaign, predecessor: &Person, successor: PersonId) -> bool {
    campaign
        .mentorships
        .get(&successor)
        .is_some_and(|mentorship| mentorship.mentor == predecessor.id)
        || campaign
            .people
            .iter()
            .find(|person| person.id == successor)
            .is_some_and(|person| {
                person
                    .career
                    .completed_mentors
                    .iter()
                    .any(|record| record.mentor == predecessor.id)
            })
}

fn adopted_relation(campaign: &VisibleCampaign, first: PersonId, second: PersonId) -> bool {
    use kestrum::state::relationships::FamilyLink;
    campaign
        .families
        .get(&first)
        .is_some_and(|family| family.links.get(&second) == Some(&FamilyLink::AdoptiveGuardian))
        || campaign
            .families
            .get(&second)
            .is_some_and(|family| family.links.get(&first) == Some(&FamilyLink::AdoptiveGuardian))
}

fn blood_relation(campaign: &VisibleCampaign, first: PersonId, second: PersonId) -> bool {
    let first_ancestors = blood_ancestors(campaign, first);
    let second_ancestors = blood_ancestors(campaign, second);
    first_ancestors.contains(&second)
        || second_ancestors.contains(&first)
        || first_ancestors
            .iter()
            .any(|ancestor| second_ancestors.contains(ancestor))
}

fn blood_ancestors(
    campaign: &VisibleCampaign,
    person: PersonId,
) -> std::collections::BTreeSet<PersonId> {
    use kestrum::state::relationships::FamilyLink;
    let mut result = std::collections::BTreeSet::new();
    let mut pending = vec![person];
    while let Some(child) = pending.pop() {
        if let Some(family) = campaign.families.get(&child) {
            for (parent, link) in &family.links {
                if *link == FamilyLink::BiologicalParent && result.insert(*parent) {
                    pending.push(*parent);
                }
            }
        }
    }
    result
}

fn shares_household(campaign: &VisibleCampaign, first: PersonId, second: PersonId) -> bool {
    let first_ancestors = family_ancestors(campaign, first);
    let second_ancestors = family_ancestors(campaign, second);
    if first_ancestors.contains(&second)
        || second_ancestors.contains(&first)
        || first_ancestors
            .iter()
            .any(|ancestor| second_ancestors.contains(ancestor))
    {
        return true;
    }
    let first_family = campaign.families.get(&first);
    let second_family = campaign.families.get(&second);
    let first_ids = first_family
        .and_then(|family| family.household)
        .into_iter()
        .chain(
            campaign
                .households
                .iter()
                .filter(|household| household.partners.contains(&first))
                .map(|household| household.id),
        )
        .collect::<std::collections::BTreeSet<_>>();
    let second_ids = second_family
        .and_then(|family| family.household)
        .into_iter()
        .chain(
            campaign
                .households
                .iter()
                .filter(|household| household.partners.contains(&second))
                .map(|household| household.id),
        )
        .collect::<std::collections::BTreeSet<_>>();
    if first_ids.iter().any(|id| second_ids.contains(id)) {
        return true;
    }
    false
}

fn family_ancestors(
    campaign: &VisibleCampaign,
    person: PersonId,
) -> std::collections::BTreeSet<PersonId> {
    use kestrum::state::relationships::FamilyLink;
    let mut result = std::collections::BTreeSet::new();
    let mut pending = campaign
        .families
        .get(&person)
        .map(|family| {
            family
                .links
                .iter()
                .filter(|(_, link)| **link != FamilyLink::ApprenticeOf)
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    while let Some(parent) = pending.pop() {
        if !result.insert(parent) {
            continue;
        }
        if let Some(family) = campaign.families.get(&parent) {
            pending.extend(
                family
                    .links
                    .iter()
                    .filter(|(_, link)| **link != FamilyLink::ApprenticeOf)
                    .map(|(id, _)| *id),
            );
        }
    }
    result
}
