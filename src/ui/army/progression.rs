//! Evidence, prerequisites and direct orders for ordinary careers.

use super::*;
use kestrum::{
    data::{
        economy::TroopKind,
        progression::{EpithetFact, FormationSpecialization, SpecializationRequirement},
        world::{Facility, PersonClass, PersonClass::*, SiteId, SiteTag},
    },
    engine::VisibleCampaign,
    state::{
        evidence::EvidenceKind,
        military::FormationId,
        people::{Person, PersonAssignment, PersonCourse, PersonId, PersonStatus, PersonTrait},
    },
};

type CareerFact = (&'static str, usize, usize);
type CareerPath = (PersonClass, Vec<CareerFact>);

pub(super) fn person(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    id: PersonId,
) -> Option<UiAction> {
    let person = campaign.people.iter().find(|person| person.id == id)?;
    let site = person_site(campaign, person);
    body(
        ctx,
        &format!("{} · {}", class_name(ctx, person.class), person.name),
        vec2(112.0, 180.0),
        22.0,
        CREAM,
    );
    if let Some(emergence) = &person.career.emergence {
        let place = campaign
            .world
            .site(emergence.site)
            .map(|site| site.name.as_str())
            .unwrap_or("?");
        let deed = emergence
            .distinguishing_deed
            .map(|deed| format!(" · {}", ctx.text(epithet_key(deed))))
            .unwrap_or_default();
        block(
            ctx,
            &format!(
                "{} {} · {place}{deed}",
                ctx.text("emerged_served_with"),
                ctx.text(troop_key(emergence.source_troop))
            ),
            vec2(112.0, 211.0),
            1030.0,
            MUTED,
        );
    }
    let age = person.age_years(campaign.completed_rounds);
    body(
        ctx,
        &format!(
            "{} {age} · {} {}",
            ctx.text("person_age_label"),
            ctx.text("history_service_start"),
            person.service_start_round
        ),
        vec2(112.0, 252.0),
        18.0,
        MUTED,
    );
    draw_traits(ctx, person, campaign);

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
        body(
            ctx,
            &truncate_text_to_width_ex(
                &format!(
                    "{label} · {} {steps}/{total} · {}",
                    ctx.text("course_steps"),
                    ctx.text(refund)
                ),
                744.0,
                ctx.body_font(),
                18.0,
            ),
            vec2(112.0, 350.0),
            18.0,
            BRASS,
        );
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

    body(
        ctx,
        &ctx.text("course_pause_help"),
        vec2(112.0, 371.0),
        18.0,
        MUTED,
    );
    let requirements = career_requirements(person, ctx.progression);
    for (index, (class, facts)) in requirements.into_iter().enumerate() {
        let x = if index % 2 == 0 { 112.0 } else { 648.0 };
        let y = 390.0 + (index / 2) as f32 * 49.0;
        let rule = &ctx.progression.careers.courses[&class];
        let met = facts.iter().all(|(_, current, needed)| current >= needed);
        let target = site
            .filter(|site| course_site(campaign, *site, rule.facility, rule.requires_horse_access));
        let enough_gold = campaign
            .factions
            .iter()
            .find(|faction| faction.id == campaign.observer)
            .and_then(|faction| faction.resources)
            .is_some_and(|resources| resources.gold >= rule.gold_cost);
        let can_start = person.class != class
            && person.status == PersonStatus::Fit
            && age >= 17
            && !person.career.retired
            && person.career.course.is_none()
            && !matches!(person.assignment, PersonAssignment::Dead)
            && met
            && target.is_some()
            && enough_gold;
        let fact_line = facts
            .iter()
            .map(|(key, current, needed)| format!("{} {current}/{needed}", ctx.text(key)))
            .collect::<Vec<_>>()
            .join(" · ");
        let description = format!(
            "{} · {} · {}{} · {}",
            class_name(ctx, class),
            fact_line,
            rule.gold_cost,
            ctx.text("gold_short"),
            ctx.text(course_facility_key(rule.facility))
        );
        let line = truncate_text_to_width_ex(&description, 340.0, ctx.body_font(), 14.0);
        body(
            ctx,
            &line,
            vec2(x, y + 32.0),
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
    }

    if let Some(site) = site.filter(|site| course_site(campaign, *site, Facility::Stable, true)) {
        let needed = ctx.progression.careers.riding_seasons;
        body(
            ctx,
            &format!(
                "{} {} / {needed} · {}",
                ctx.text("riding_practice"),
                person.career.riding_practice_seasons,
                ctx.text("course_season_free")
            ),
            vec2(500.0, 578.0),
            15.0,
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
            Rect::new(112.0, 552.0, 370.0, 48.0),
            &ctx.text(label),
            eligible,
            false,
        ) {
            return Some(UiAction::SetCommander(army, appointed));
        }
        body(
            ctx,
            &ctx.text("commander_any_adult"),
            vec2(500.0, 605.0),
            16.0,
            MUTED,
        );
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

pub(super) fn formation(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    id: FormationId,
) -> Option<UiAction> {
    let formation = campaign
        .formations
        .iter()
        .find(|formation| formation.id == id)?;
    body(
        ctx,
        &format!(
            "{} · {} / {}",
            ctx.text("formation_specialization"),
            ctx.text(troop_key(formation.kind)),
            formation.headcount
        ),
        vec2(112.0, 180.0),
        22.0,
        CREAM,
    );
    let site = campaign
        .armies
        .iter()
        .find(|army| army.formation_ids().any(|member| member == id))
        .map(|army| army.site);
    let existing = formation
        .service
        .specialization
        .map(|specialization| ctx.text(specialization_key(specialization)))
        .unwrap_or_else(|| ctx.text("specialization_none"));
    body(
        ctx,
        &format!("{}: {existing}", ctx.text("current_specialization")),
        vec2(112.0, 212.0),
        18.0,
        MUTED,
    );
    if let Some(course) = &formation.service.course {
        let total = ctx.progression.specializations[&course.target].course_steps;
        let status = format!(
            "{} · {} {}/{} · {}",
            ctx.text("specialization_in_progress"),
            ctx.text(specialization_key(course.target)),
            course.steps_completed,
            total,
            ctx.text("course_refund_none")
        );
        body(ctx, &status, vec2(112.0, 252.0), 18.0, BRASS);
        if button(
            ctx,
            Rect::new(868.0, 226.0, 300.0, 48.0),
            &ctx.text("cancel_course"),
            true,
            false,
        ) {
            return Some(UiAction::CancelFormationCourse(id));
        }
    }
    body(
        ctx,
        &ctx.text("formation_course_pause_help"),
        vec2(112.0, 279.0),
        18.0,
        MUTED,
    );
    let mut y = 302.0;
    for (kind, rule) in &ctx.progression.specializations {
        let (current, required, secondary) = match rule.requirement {
            SpecializationRequirement::DefendedAnchors { occasions } => (
                formation
                    .service
                    .ledger
                    .counts
                    .get(&EvidenceKind::DefendedAnchor)
                    .copied()
                    .unwrap_or(0) as usize,
                occasions as usize,
                None,
            ),
            SpecializationRequirement::MeaningfulAgainst { troop, occasions } => (
                formation
                    .service
                    .ledger
                    .meaningful_against
                    .get(&troop)
                    .copied()
                    .unwrap_or(0) as usize,
                occasions as usize,
                None,
            ),
            SpecializationRequirement::RoutesAndRetreatingVictories { routes, victories } => (
                formation.service.ledger.traversed_routes.len(),
                routes,
                Some((
                    formation
                        .service
                        .ledger
                        .counts
                        .get(&EvidenceKind::RetreatingEnemyVictory)
                        .copied()
                        .unwrap_or(0) as usize,
                    victories as usize,
                )),
            ),
        };
        let mut description = format!(
            "{} · {} {current}/{required} · 30 {} · 2 {} · {}",
            ctx.text(specialization_key(*kind)),
            ctx.text(specialization_requirement_key(*kind)),
            ctx.text("gold"),
            ctx.text("course_steps"),
            ctx.text(specialization_effect_key(*kind))
        );
        if let Some((actual, needed)) = secondary {
            description.push_str(&format!(
                " · {} {actual}/{needed}",
                ctx.text("retreating_victories")
            ));
        }
        body(
            ctx,
            &truncate_text_to_width_ex(&description, 760.0, ctx.body_font(), 15.0),
            vec2(112.0, y + 26.0),
            15.0,
            if current >= required && secondary.is_none_or(|(actual, needed)| actual >= needed) {
                CREAM
            } else {
                MUTED
            },
        );
        let source = rule.sources.contains(&formation.kind);
        let available_site =
            site.filter(|site| course_site(campaign, *site, Facility::TrainingGround, false));
        let resources = campaign
            .factions
            .iter()
            .find(|faction| faction.id == campaign.observer)
            .and_then(|faction| faction.resources);
        let enough_gold = resources.is_some_and(|resources| resources.gold >= rule.gold_cost);
        let met = current >= required && secondary.is_none_or(|(actual, needed)| actual >= needed);
        let enabled = source
            && met
            && enough_gold
            && available_site.is_some()
            && formation.service.specialization.is_none()
            && formation.service.course.is_none();
        if let (Some(site), true) = (available_site, enabled) {
            if button(
                ctx,
                Rect::new(880.0, y, 288.0, 48.0),
                &ctx.text("convert_formation"),
                true,
                false,
            ) {
                return Some(UiAction::SpecializeFormation(id, *kind, site));
            }
        } else {
            let key = if !source {
                "specialization_wrong_source"
            } else if !met {
                "requirements_missing"
            } else if !enough_gold {
                "course_gold_missing"
            } else {
                "course_unavailable"
            };
            button(
                ctx,
                Rect::new(880.0, y, 288.0, 48.0),
                &ctx.text(key),
                false,
                false,
            );
        }
        y += 76.0;
    }
    button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::CancelArmyAction)
}

fn career_requirements(
    person: &Person,
    rules: &kestrum::data::progression::ProgressionRules,
) -> Vec<CareerPath> {
    let count = |kind| person.evidence.counts.get(&kind).copied().unwrap_or(0) as usize;
    let service = |kind| {
        person
            .evidence
            .service_by_troop
            .get(&kind)
            .copied()
            .unwrap_or(0) as usize
    };
    vec![
        (
            Infantry,
            vec![(
                "requirement_infantry",
                service(TroopKind::Warriors) + service(TroopKind::Spearmen),
                rules.careers.infantry_battles as usize,
            )],
        ),
        (
            Archer,
            vec![(
                "requirement_archer",
                service(TroopKind::Archers),
                rules.careers.archer_battles as usize,
            )],
        ),
        (
            Scout,
            vec![(
                "requirement_routes",
                person.evidence.traversed_routes.len(),
                rules.careers.scout_routes as usize,
            )],
        ),
        (
            Cavalry,
            vec![
                (
                    "requirement_riding",
                    person.career.riding_practice_seasons as usize,
                    rules.careers.riding_seasons as usize,
                ),
                (
                    "requirement_rider_battles",
                    service(TroopKind::Riders),
                    rules.careers.rider_battles as usize,
                ),
            ],
        ),
        (
            Medic,
            vec![(
                "requirement_treatment",
                count(EvidenceKind::TreatedWounded),
                rules.careers.treatment_occasions as usize,
            )],
        ),
        (
            Officer,
            vec![
                (
                    "requirement_encounters",
                    count(EvidenceKind::MeaningfulEncounter),
                    rules.careers.officer_encounters as usize,
                ),
                (
                    "requirement_command",
                    count(EvidenceKind::AssumedCommand) + count(EvidenceKind::CommandedVictory),
                    rules.careers.officer_command_facts as usize,
                ),
            ],
        ),
    ]
}

fn course_site(campaign: &VisibleCampaign, site: SiteId, facility: Facility, horse: bool) -> bool {
    campaign.world.site(site).is_some_and(|site| {
        site.controller == Some(campaign.observer)
            && site.facilities.contains(&facility)
            && campaign.world.structural_damage(site.id) == 0
            && campaign.supplied_sites.contains(&site.id)
            && !campaign.sieges.iter().any(|siege| siege.site == site.id)
            && (!horse || site.tags.contains(&SiteTag::HorseAccess))
    })
}

fn person_site(campaign: &VisibleCampaign, person: &Person) -> Option<SiteId> {
    match person.assignment {
        PersonAssignment::Formation { formation } => campaign
            .armies
            .iter()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| army.site),
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Dead => None,
    }
}

fn attached_army(
    campaign: &VisibleCampaign,
    person: &Person,
) -> Option<(kestrum::state::military::ArmyId, bool)> {
    let PersonAssignment::Formation { formation } = person.assignment else {
        return None;
    };
    campaign
        .armies
        .iter()
        .find(|army| army.formation_ids().any(|id| id == formation))
        .map(|army| (army.id, army.commander == Some(person.id)))
}

fn draw_traits(ctx: &Context<'_>, person: &Person, campaign: &VisibleCampaign) {
    let labels = person
        .career
        .traits
        .iter()
        .map(|trait_kind| {
            ctx.text(match trait_kind {
                PersonTrait::Bold => "trait_bold",
                PersonTrait::Protective => "trait_protective",
                PersonTrait::NaturalCommander => "trait_natural_commander",
            })
        })
        .collect::<Vec<_>>();
    let text = if labels.is_empty() {
        ctx.text("traits_none")
    } else {
        format!("{}: {}", ctx.text("earned_traits"), labels.join(" · "))
    };
    body(ctx, &text, vec2(112.0, 288.0), 18.0, CREAM);
    if let Some(recognition) = &person.career.recognition {
        let site = campaign
            .world
            .site(recognition.site)
            .map(|site| site.name.as_str())
            .unwrap_or("?");
        body(
            ctx,
            &format!(
                "{} {} · {site}",
                ctx.text("recognized_as"),
                recognition.epithet
            ),
            vec2(112.0, 310.0),
            17.0,
            BRASS,
        );
    }
    let associations = person
        .career
        .relationships
        .iter()
        .filter_map(|(id, relationship)| {
            let other = campaign.people.iter().find(|entry| entry.id == *id)?;
            let label = if relationship.mutual_combat_rounds >= 2 {
                "relationship_rival"
            } else if relationship.shared_service_seasons > 0 {
                "relationship_served_with"
            } else {
                return None;
            };
            Some(format!("{} {}", ctx.text(label), other.name))
        })
        .take(2)
        .collect::<Vec<_>>();
    if !associations.is_empty() {
        body(
            ctx,
            &associations.join(" · "),
            vec2(112.0, 332.0),
            17.0,
            MUTED,
        );
    }
}

fn class_name(ctx: &Context<'_>, class: PersonClass) -> String {
    ctx.text(match class {
        Recruit => "class_recruit",
        Infantry => "class_infantry",
        Archer => "class_archer",
        Scout => "class_scout",
        Cavalry => "class_cavalry",
        Medic => "class_medic",
        Officer => "class_officer",
    })
}

fn course_facility_key(facility: Facility) -> &'static str {
    match facility {
        Facility::TrainingGround => "course_facility_training",
        Facility::Stable => "course_facility_stable",
        Facility::Infirmary => "course_facility_infirmary",
        Facility::Workshop => "facility_workshop",
        Facility::Temple => "facility_temple",
    }
}
fn troop_key(kind: TroopKind) -> &'static str {
    match kind {
        TroopKind::Warriors => "troop_warriors",
        TroopKind::Spearmen => "troop_spearmen",
        TroopKind::Archers => "troop_archers",
        TroopKind::Riders => "troop_riders",
        TroopKind::Medics => "troop_medics",
        TroopKind::SiegeEngines => "troop_siege_engines",
    }
}
fn specialization_key(kind: FormationSpecialization) -> &'static str {
    match kind {
        FormationSpecialization::ShieldGuard => "specialization_shield_guard",
        FormationSpecialization::Pikemen => "specialization_pikemen",
        FormationSpecialization::LightCavalry => "specialization_light_cavalry",
    }
}
fn specialization_requirement_key(kind: FormationSpecialization) -> &'static str {
    match kind {
        FormationSpecialization::ShieldGuard => "requirement_anchor_defense",
        FormationSpecialization::Pikemen => "requirement_rider_encounters",
        FormationSpecialization::LightCavalry => "requirement_routes",
    }
}

fn specialization_effect_key(kind: FormationSpecialization) -> &'static str {
    match kind {
        FormationSpecialization::ShieldGuard => "specialization_effect_shield_guard",
        FormationSpecialization::Pikemen => "specialization_effect_pikemen",
        FormationSpecialization::LightCavalry => "specialization_effect_light_cavalry",
    }
}
fn epithet_key(deed: EpithetFact) -> &'static str {
    match deed {
        EpithetFact::SurvivedOutnumbered => "deed_outnumbered",
        EpithetFact::DefendedAnchor => "deed_defended_anchor",
        EpithetFact::CapturedAnchor => "deed_captured_anchor",
        EpithetFact::TreatedWounded => "deed_treated_wounded",
        EpithetFact::AssumedCommand => "deed_assumed_command",
        EpithetFact::CommandedVictory => "deed_commanded_victory",
    }
}
