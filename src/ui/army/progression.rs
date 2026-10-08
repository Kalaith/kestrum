//! Evidence, prerequisites and direct orders for ordinary careers.

use super::*;
use kestrum::{
    data::{
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

mod formation;
mod person;
pub(super) use formation::draw as formation;
pub(super) use person::draw as person;

fn owned_settlement(campaign: &VisibleCampaign, site: SiteId) -> bool {
    campaign.world.site(site).is_some_and(|site| {
        site.controller == Some(campaign.observer)
            && site.habitation != kestrum::data::economy::Habitation::Unsettled
            && !campaign.sieges.iter().any(|siege| siege.site == site.id)
    })
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
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => Some(site),
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
    let width = if person.career.course.is_some() {
        744.0
    } else {
        1056.0
    };
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
    let traits = if labels.is_empty() {
        ctx.text("traits_none")
    } else {
        format!("{}: {}", ctx.text("earned_traits"), labels.join(" | "))
    };
    let mut y = draw_person_detail(ctx, &traits, vec2(112.0, 298.0), width, 16.0, CREAM, 1) + 2.0;
    if let Some(recognition) = &person.career.recognition {
        let site = campaign
            .world
            .site(recognition.site)
            .map(|site| site.name.as_str())
            .unwrap_or("?");
        y = draw_person_detail(
            ctx,
            &format!(
                "{} {} | {site}",
                ctx.text("recognized_as"),
                recognition.epithet
            ),
            vec2(112.0, y),
            width,
            15.0,
            BRASS,
            1,
        ) + 2.0;
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
        draw_person_detail(
            ctx,
            &associations.join(" | "),
            vec2(112.0, y),
            width,
            14.0,
            MUTED,
            2,
        );
    }
}

fn draw_person_detail(
    ctx: &Context<'_>,
    label: &str,
    origin: Vec2,
    width: f32,
    size: f32,
    color: Color,
    max_lines: usize,
) -> f32 {
    let lines = wrap_text_ex(label, width, ctx.body_font(), size);
    let visible_lines = lines.len().min(max_lines);
    for (line, text) in lines.iter().take(visible_lines).enumerate() {
        let copy = if line + 1 == visible_lines && lines.len() > visible_lines {
            truncate_text_to_width_ex(&format!("{text}…"), width, ctx.body_font(), size)
        } else {
            text.clone()
        };
        body(
            ctx,
            &copy,
            vec2(origin.x, origin.y + line as f32 * (size + 2.0)),
            size,
            color,
        );
    }
    origin.y + visible_lines as f32 * (size + 2.0)
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
        EpithetFact::BattleService => "deed_battle_service",
    }
}
