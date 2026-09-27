//! Player choices for local, evidence-only mentorship.

use super::*;
use kestrum::{
    data::{
        economy::Habitation,
        progression::TrainingDiscipline,
        world::{Facility, PersonClass},
    },
    state::{
        mentorship::MentorshipStatus,
        people::{Person, PersonAssignment, PersonId, PersonStatus, PersonTrait},
    },
};

pub(super) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    learner_id: PersonId,
    page: usize,
) -> Option<UiAction> {
    let learner = campaign
        .people
        .iter()
        .find(|person| person.id == learner_id)?;
    body(
        ctx,
        &format!("{} · {}", ctx.text("mentorship_title"), learner.name),
        vec2(112.0, 181.0),
        22.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.text("mentorship_help"),
        vec2(112.0, 211.0),
        17.0,
        MUTED,
    );

    if let Some(existing) = campaign.mentorships.get(&learner_id) {
        let mentor = campaign
            .people
            .iter()
            .find(|person| person.id == existing.mentor);
        let name = mentor.map_or("?", |person| person.name.as_str());
        body(
            ctx,
            &format!(
                "{} · {} · {}",
                discipline_name(ctx, existing.discipline),
                name,
                kestrum::engine::mentorship_status_text(existing.status)
            ),
            vec2(112.0, 270.0),
            19.0,
            BRASS,
        );
        if matches!(existing.status, MentorshipStatus::Paused { .. }) {
            let can_resume = mentor.is_some_and(|mentor| {
                candidates(ctx, campaign, learner)
                    .iter()
                    .any(|(candidate, discipline)| {
                        candidate.id == mentor.id && *discipline == existing.discipline
                    })
            });
            if button(
                ctx,
                Rect::new(832.0, 247.0, 160.0, 48.0),
                &ctx.text("resume_mentorship_short"),
                can_resume,
                false,
            ) && can_resume
            {
                return Some(UiAction::StartMentorship(
                    existing.mentor,
                    learner_id,
                    existing.discipline,
                ));
            }
            if button(
                ctx,
                Rect::new(1008.0, 247.0, 160.0, 48.0),
                &ctx.text("end_mentorship_short"),
                true,
                false,
            ) {
                return Some(UiAction::EndMentorship(learner_id));
            }
        } else if button(
            ctx,
            Rect::new(832.0, 247.0, 336.0, 48.0),
            &ctx.text("end_mentorship"),
            true,
            false,
        ) {
            return Some(UiAction::EndMentorship(learner_id));
        }
    } else {
        let options = candidates(ctx, campaign, learner);
        let pages = options.len().div_ceil(7).max(1);
        let page = page.min(pages - 1);
        if options.is_empty() {
            body(
                ctx,
                &ctx.text("mentorship_no_teacher"),
                vec2(112.0, 270.0),
                18.0,
                MUTED,
            );
        }
        for (index, (mentor, discipline)) in options.iter().skip(page * 7).take(7).enumerate() {
            let y = 273.0 + index as f32 * 48.0;
            body(
                ctx,
                &format!(
                    "{} · {} · {} {}",
                    mentor.name,
                    discipline_name(ctx, *discipline),
                    ctx.text("discipline_service"),
                    mentor
                        .career
                        .discipline_service_seasons
                        .get(discipline)
                        .copied()
                        .unwrap_or(0)
                ),
                vec2(112.0, y + 29.0),
                17.0,
                CREAM,
            );
            if button(
                ctx,
                Rect::new(928.0, y, 240.0, 40.0),
                &ctx.text("begin_mentorship"),
                true,
                true,
            ) {
                return Some(UiAction::StartMentorship(
                    mentor.id,
                    learner_id,
                    *discipline,
                ));
            }
        }
        if pages > 1 {
            if button(
                ctx,
                Rect::new(832.0, 626.0, 112.0, 48.0),
                "‹",
                page > 0,
                false,
            ) {
                return Some(UiAction::MentorshipPage(-1));
            }
            body(
                ctx,
                &format!("{} / {}", page + 1, pages),
                vec2(970.0, 656.0),
                16.0,
                MUTED,
            );
            if button(
                ctx,
                Rect::new(1044.0, 626.0, 124.0, 48.0),
                "›",
                page + 1 < pages,
                false,
            ) {
                return Some(UiAction::MentorshipPage(1));
            }
        }
    }
    button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::OpenPersonProgression(learner_id))
}

fn candidates<'a>(
    ctx: &Context<'_>,
    campaign: &'a VisibleCampaign,
    learner: &Person,
) -> Vec<(&'a Person, TrainingDiscipline)> {
    let Some(site_id) = person_site(campaign, learner) else {
        return Vec::new();
    };
    let Some(site) = campaign.world.site(site_id) else {
        return Vec::new();
    };
    let disciplines = disciplines_for(learner.class);
    let mut candidates = campaign
        .people
        .iter()
        .filter(|mentor| {
            mentor.id != learner.id
                && mentor.faction == learner.faction
                && mentor.status == PersonStatus::Fit
                && mentor.age_years(campaign.completed_rounds) >= 26
                && person_site(campaign, mentor) == Some(site_id)
                && !campaign.mentorships.iter().any(|(assigned, relation)| {
                    *assigned != learner.id && relation.mentor == mentor.id
                })
        })
        .flat_map(|mentor| {
            disciplines.iter().copied().filter_map(move |discipline| {
                let count = mentor
                    .career
                    .discipline_service_seasons
                    .get(&discipline)
                    .copied()
                    .unwrap_or(0);
                (count >= 4 && mentor_qualified(mentor, discipline)).then_some((mentor, discipline))
            })
        })
        .filter(|(_, discipline)| {
            site.controller == Some(campaign.observer)
                && site.habitation != Habitation::Unsettled
                && !campaign.sieges.iter().any(|siege| siege.site == site_id)
                && campaign.supplied_sites.contains(&site_id)
                && campaign.world.structural_damage(site_id) < ctx.economy.facility_failure_damage
                && site.facilities.contains(&facility_for(*discipline))
                && (*discipline != TrainingDiscipline::Riding
                    || site
                        .tags
                        .contains(&kestrum::data::world::SiteTag::HorseAccess))
                && learner.status == PersonStatus::Fit
                && !learner.career.retired
                && learner.career.course.is_none()
                && learner.age_years(campaign.completed_rounds) >= 13
                && (!matches!(learner.assignment, PersonAssignment::Formation { .. })
                    || learner.age_years(campaign.completed_rounds) >= 17)
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(mentor, discipline)| (discipline_priority(*discipline), mentor.id));
    candidates
}

fn disciplines_for(_class: PersonClass) -> Vec<TrainingDiscipline> {
    use TrainingDiscipline::*;
    vec![Command, Medicine, Infantry, Archery, Riding, Scouting]
}

fn mentor_qualified(person: &Person, discipline: TrainingDiscipline) -> bool {
    match discipline {
        TrainingDiscipline::Infantry => {
            person.class == PersonClass::Infantry
                || person.career.traits.contains(&PersonTrait::Bold)
                || person.career.traits.contains(&PersonTrait::Protective)
        }
        TrainingDiscipline::Archery => {
            person.class == PersonClass::Archer || person.career.traits.contains(&PersonTrait::Bold)
        }
        TrainingDiscipline::Scouting => person.class == PersonClass::Scout,
        TrainingDiscipline::Riding => {
            person.class == PersonClass::Cavalry
                || person.career.traits.contains(&PersonTrait::Bold)
        }
        TrainingDiscipline::Medicine => {
            person.class == PersonClass::Medic
                || person.career.traits.contains(&PersonTrait::Protective)
        }
        TrainingDiscipline::Command => {
            person.class == PersonClass::Officer
                || person
                    .career
                    .traits
                    .contains(&PersonTrait::NaturalCommander)
        }
    }
}

fn facility_for(discipline: TrainingDiscipline) -> Facility {
    match discipline {
        TrainingDiscipline::Riding => Facility::Stable,
        TrainingDiscipline::Medicine => Facility::Infirmary,
        _ => Facility::TrainingGround,
    }
}

fn discipline_priority(discipline: TrainingDiscipline) -> u8 {
    match discipline {
        TrainingDiscipline::Command => 0,
        TrainingDiscipline::Medicine => 1,
        TrainingDiscipline::Infantry => 2,
        TrainingDiscipline::Archery => 3,
        TrainingDiscipline::Riding => 4,
        TrainingDiscipline::Scouting => 5,
    }
}

fn discipline_name(ctx: &Context<'_>, discipline: TrainingDiscipline) -> String {
    ctx.text(match discipline {
        TrainingDiscipline::Infantry => "discipline_infantry",
        TrainingDiscipline::Archery => "discipline_archery",
        TrainingDiscipline::Scouting => "discipline_scouting",
        TrainingDiscipline::Riding => "discipline_riding",
        TrainingDiscipline::Medicine => "discipline_medicine",
        TrainingDiscipline::Command => "discipline_command",
    })
}

fn person_site(
    campaign: &VisibleCampaign,
    person: &Person,
) -> Option<kestrum::data::world::SiteId> {
    match person.assignment {
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Formation { formation } => campaign
            .armies
            .iter()
            .find(|army| army.formation_ids().any(|member| member == formation))
            .map(|army| army.site),
        PersonAssignment::Dead => None,
    }
}
