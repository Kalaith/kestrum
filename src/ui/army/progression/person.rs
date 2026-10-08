//! Character identity and local career choices.
use super::*;

pub(in crate::ui::army) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    id: PersonId,
) -> Option<UiAction> {
    let person = campaign.people.iter().find(|person| person.id == id)?;
    identity(ctx, campaign, person);
    if let Some(action) = active_course(ctx, person)
        .or_else(|| class_options(ctx, campaign, person))
        .or_else(|| riding(ctx, campaign, person))
        .or_else(|| field_controls(ctx, campaign, person))
        .or_else(|| site_controls(ctx, campaign, person))
    {
        return Some(action);
    }
    button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::ArmyPeople)
}

fn identity(ctx: &Context<'_>, campaign: &VisibleCampaign, person: &Person) {
    crate::ui::portraits::draw(
        ctx.portraits,
        Some(&person.appearance),
        Some(person.age_years(campaign.completed_rounds)),
        ctx.household_rules.service_minimum_age_years,
        Rect::new(112.0, 160.0, 128.0, 128.0),
    );
    let mut y = 170.0;
    for line in wrap_text_ex(&person_name(ctx, person), 908.0, ctx.body_font(), 22.0) {
        body(ctx, &line, vec2(260.0, y), 22.0, CREAM);
        y += 24.0;
    }

    let age = person.age_years(campaign.completed_rounds);
    let mut details = format!(
        "{} | {} {} | {} {}",
        class_name(ctx, person.class),
        ctx.text("person_age_label"),
        age,
        ctx.text("history_service_start"),
        person.service_start_round
    );
    if person.career.retired {
        details.push_str(&format!(" | {}", ctx.text("person_retired")));
    }
    if person.career.site_role == Some(kestrum::state::people::PersonSiteRole::Governor) {
        details.push_str(&format!(" | {}", ctx.text("person_governor")));
    }
    match person.status {
        PersonStatus::Fit => details.push_str(&format!(" | {}", ctx.text("person_fit"))),
        PersonStatus::Wounded {
            remaining_steps, ..
        } => details.push_str(&format!(
            " | {} {remaining_steps} {}",
            ctx.text("person_wounded"),
            ctx.text("wound_steps_remaining")
        )),
        PersonStatus::Dead { .. } => details.push_str(&format!(" | {}", ctx.text("person_dead"))),
        PersonStatus::Displaced { .. } => {}
    }
    if age >= 56 {
        details.push_str(&format!(" | {}", ctx.text("elder_choice_help")));
    }
    for line in wrap_text_ex(&details, 908.0, ctx.body_font(), 15.0) {
        body(ctx, &line, vec2(260.0, y), 15.0, MUTED);
        y += 17.0;
    }

    let mut provenance = Vec::new();
    if person.career.founding_lord {
        let kingdom = campaign
            .factions
            .iter()
            .find(|faction| faction.id == person.faction)
            .map(|faction| faction.name.as_str())
            .unwrap_or("?");
        provenance.push(
            ctx.text("person_founding_lord")
                .replace("{kingdom}", kingdom)
                .replace(
                    "{bonus}",
                    &format!(
                        "{:.1}",
                        ctx.rules.founder.commander_bonus_permille as f32 / 10.0
                    ),
                ),
        );
    }
    if let Some(emergence) = &person.career.emergence {
        let place = campaign
            .world
            .site(emergence.site)
            .map(|site| site.name.as_str())
            .unwrap_or("?");
        let deed = emergence
            .distinguishing_deed
            .map(|deed| format!(" | {}", ctx.text(epithet_key(deed))))
            .unwrap_or_default();
        provenance.push(format!(
            "{} {} | {place}{deed}",
            ctx.text("emerged_served_with"),
            ctx.text(troop_key(emergence.source_troop))
        ));
    }
    if !provenance.is_empty() {
        for line in wrap_text_ex(&provenance.join(" | "), 908.0, ctx.body_font(), 14.0) {
            body(ctx, &line, vec2(260.0, y), 14.0, BRASS);
            y += 16.0;
        }
    }
    if person.career.recognition.is_none()
        && (person.career.emergence.is_some() || person.career.hero_service_progress > 0)
    {
        let progress = ctx
            .text("hero_progress_detail")
            .replace(
                "{progress}",
                &person.career.hero_service_progress.to_string(),
            )
            .replace(
                "{total}",
                &ctx.progression.recognition.personal_engagements.to_string(),
            );
        draw_person_detail(ctx, &progress, vec2(260.0, y), 908.0, 14.0, BRASS, 1);
    }
    draw_traits(ctx, person, campaign);
}
fn active_course(ctx: &Context<'_>, person: &Person) -> Option<UiAction> {
    let id = person.id;
    if let Some(course) = &person.career.course {
        let (label, steps, total, refund) = match course {
            PersonCourse::Class {
                target,
                steps_completed,
                ..
            } => (
                format!(
                    "{}: {}",
                    ctx.text("course_in_progress"),
                    class_name(ctx, *target)
                ),
                *steps_completed,
                ctx.progression.careers.course_steps,
                if *steps_completed == 0 {
                    "course_refund_full"
                } else {
                    "course_refund_none"
                },
            ),
            PersonCourse::RidingPractice {
                steps_completed, ..
            } => (
                ctx.text("riding_in_progress"),
                *steps_completed,
                1,
                "course_free_cancel",
            ),
        };
        let status = format!(
            "{label} | {} {steps}/{total} | {}",
            ctx.text("course_steps"),
            ctx.text(refund)
        );
        for (line, copy) in wrap_text_ex(&status, 300.0, ctx.body_font(), 14.0)
            .into_iter()
            .enumerate()
        {
            body(
                ctx,
                &copy,
                vec2(868.0, 350.0 + line as f32 * 16.0),
                14.0,
                BRASS,
            );
        }
        if button(
            ctx,
            Rect::new(868.0, 298.0, 300.0, 48.0),
            &ctx.text("cancel_course"),
            true,
            false,
        ) {
            return Some(UiAction::CancelPersonCourse(id));
        }
    }

    None
}

fn class_options(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    person: &Person,
) -> Option<UiAction> {
    if person.career.course.is_none() {
        body(
            ctx,
            &ctx.text("course_pause_help"),
            vec2(112.0, 371.0),
            18.0,
            MUTED,
        );
    }
    let requirements =
        super::super::career_requirements::career_requirements(person, ctx.progression);
    for (index, (class, facts)) in requirements.into_iter().enumerate() {
        if let Some(action) = class_row(ctx, campaign, person, index, class, facts) {
            return Some(action);
        }
    }

    None
}

fn class_row(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    person: &Person,
    index: usize,
    class: PersonClass,
    facts: Vec<Vec<(&str, usize, usize)>>,
) -> Option<UiAction> {
    let id = person.id;
    let age = person.age_years(campaign.completed_rounds);
    let site = person_site(campaign, person);
    let x = if index.is_multiple_of(2) {
        112.0
    } else {
        648.0
    };
    let y = 390.0 + (index / 2) as f32 * 49.0;
    let rule = &ctx.progression.careers.courses[&class];
    let price = kestrum::engine::course_gold_cost(
        rule.gold_cost,
        &ctx.progression.careers,
        site.and_then(|id| campaign.world.focus.get(&id).copied()),
    );
    let met = facts
        .iter()
        .any(|path| path.iter().all(|(_, current, needed)| current >= needed));
    let target =
        site.filter(|site| course_site(campaign, *site, rule.facility, rule.requires_horse_access));
    let enough_gold = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer)
        .and_then(|faction| faction.resources)
        .is_some_and(|resources| resources.gold >= price);
    let can_start = person.class != class
        && person.status == PersonStatus::Fit
        && age >= 17
        && !person.career.retired
        && person.career.course.is_none()
        && !matches!(person.assignment, PersonAssignment::Dead)
        && met
        && target.is_some()
        && enough_gold;
    let fact_line = requirement_line(ctx, &facts);
    let description = format!(
        "{} / {price}{} / {}",
        class_name(ctx, class),
        ctx.text("gold_short"),
        ctx.text(course_facility_key(rule.facility))
    );
    let line = truncate_text_to_width_ex(&description, 340.0, ctx.body_font(), 14.0);
    body(
        ctx,
        &line,
        vec2(x, y + 17.0),
        14.0,
        if met { CREAM } else { MUTED },
    );
    body(
        ctx,
        &truncate_text_to_width_ex(&fact_line, 340.0, ctx.body_font(), 14.0),
        vec2(x, y + 36.0),
        14.0,
        if met { CREAM } else { MUTED },
    );
    if let (Some(site), true) = (target, can_start) {
        if button(
            ctx,
            Rect::new(x + 354.0, y, 166.0, 48.0),
            &ctx.text("begin_course"),
            true,
            false,
        ) {
            return Some(UiAction::TrainPerson(id, class, site));
        }
    } else {
        let button_text = if !met {
            ctx.text("requirements_missing")
        } else if !enough_gold {
            ctx.text("course_gold_missing")
        } else if target.is_none() {
            ctx.text("course_no_facility")
        } else {
            ctx.text("course_unavailable")
        };
        button(
            ctx,
            Rect::new(x + 354.0, y, 166.0, 48.0),
            &button_text,
            false,
            false,
        );
    }
    None
}

fn requirement_line(ctx: &Context<'_>, facts: &[Vec<(&str, usize, usize)>]) -> String {
    facts
        .iter()
        .map(|path| {
            path.iter()
                .map(|(key, current, needed)| format!("{} {current}/{needed}", ctx.text(key)))
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .collect::<Vec<_>>()
        .join(&format!(" {} ", ctx.text("requirement_or")))
}

fn riding(ctx: &Context<'_>, campaign: &VisibleCampaign, person: &Person) -> Option<UiAction> {
    let id = person.id;
    let age = person.age_years(campaign.completed_rounds);
    let site = person_site(campaign, person);
    if let Some(site) = site.filter(|site| course_site(campaign, *site, Facility::Stable, true)) {
        let needed = ctx.progression.careers.riding_seasons;
        body(
            ctx,
            &truncate_text_to_width_ex(
                &format!(
                    "{} {} / {needed} · {}",
                    ctx.text("riding_practice"),
                    person.career.riding_practice_seasons,
                    ctx.text("course_season_free")
                ),
                394.0,
                ctx.body_font(),
                14.0,
            ),
            vec2(770.0, 605.0),
            14.0,
            MUTED,
        );
        if button(
            ctx,
            Rect::new(880.0, 552.0, 288.0, 48.0),
            &ctx.text("practice_riding"),
            person.career.course.is_none() && person.status == PersonStatus::Fit && age >= 17,
            false,
        ) {
            return Some(UiAction::PracticeRiding(id, site));
        }
    }

    None
}

fn field_controls(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    person: &Person,
) -> Option<UiAction> {
    let id = person.id;
    let age = person.age_years(campaign.completed_rounds);
    if let Some((army, current)) = attached_army(campaign, person) {
        let eligible =
            person.is_fit_for_field(campaign.completed_rounds, 17) && !person.career.retired;
        let (label, appointed) = if current {
            ("commander_relieve", None)
        } else {
            ("commander_appoint", Some(id))
        };
        if button(
            ctx,
            Rect::new(112.0, 552.0, 340.0, 48.0),
            &ctx.text(label),
            eligible,
            false,
        ) {
            return Some(UiAction::SetCommander(army, appointed));
        }
        body(
            ctx,
            &ctx.text("commander_any_adult"),
            vec2(112.0, 605.0),
            15.0,
            MUTED,
        );
    }
    if button(
        ctx,
        Rect::new(470.0, 552.0, 250.0, 48.0),
        &ctx.text("open_mentorship"),
        age >= 13 && !matches!(person.assignment, PersonAssignment::Dead),
        false,
    ) {
        return Some(UiAction::OpenMentorship(id));
    }
    None
}

fn site_controls(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    person: &Person,
) -> Option<UiAction> {
    let id = person.id;
    let age = person.age_years(campaign.completed_rounds);
    let site = person_site(campaign, person);
    if let Some(site) = site.filter(|site| owned_settlement(campaign, *site)) {
        let can_retire = age >= 17 && !person.career.retired && person.is_alive();
        if button(
            ctx,
            Rect::new(310.0, 626.0, 240.0, 48.0),
            &ctx.text("retire_person"),
            can_retire,
            false,
        ) {
            return Some(UiAction::RetirePerson(id, site));
        }
        let governor_exists = campaign.people.iter().any(|other| {
            other.id != id
                && other.career.site_role == Some(kestrum::state::people::PersonSiteRole::Governor)
                && other.assignment == (PersonAssignment::Site { site })
        });
        let can_govern = person.status == PersonStatus::Fit
            && age >= 17
            && person.career.site_role.is_none()
            && !governor_exists;
        if button(
            ctx,
            Rect::new(570.0, 626.0, 240.0, 48.0),
            &ctx.text("appoint_governor"),
            can_govern,
            false,
        ) {
            return Some(UiAction::AppointGovernor(id, site));
        }
        let can_recover = matches!(person.status, PersonStatus::Wounded { .. })
            && campaign.supplied_sites.contains(&site)
            && !campaign.sieges.iter().any(|siege| siege.site == site);
        if can_recover
            && button(
                ctx,
                Rect::new(830.0, 626.0, 240.0, 48.0),
                &ctx.text("recover_at_site"),
                true,
                false,
            )
        {
            return Some(UiAction::RecoverPersonAtSite(id, site));
        }
    }
    None
}
