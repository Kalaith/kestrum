//! Orders advance only at a completed season and remain paused by real conditions.

use super::specialization;
use crate::{
    data::{world::Facility, GameData},
    engine::actions::RuleError,
    state::{
        campaign::{DomainFact, DomainFactKind},
        evidence::FormationCourse,
        people::{PersonAssignment, PersonCourse, PersonId, PersonStatus},
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub(super) fn advance(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    facts: &[DomainFact],
    round_completed: bool,
) -> Result<(), RuleError> {
    let mut moved_people = BTreeSet::new();
    let mut moved_formations = BTreeSet::new();
    for fact in facts {
        match &fact.kind {
            DomainFactKind::ArmiesMoved {
                movement: Some(receipt),
                ..
            }
            | DomainFactKind::BattleResolved {
                movement: Some(receipt),
                ..
            } => {
                moved_people.extend(receipt.people.iter().copied());
                moved_formations.extend(receipt.formations.iter().copied());
            }
            _ => {}
        }
    }
    let person_ids = campaign.people.keys().copied().collect::<Vec<_>>();
    for id in person_ids {
        advance_person(campaign, data, id, &moved_people, round_completed)?;
    }
    let formation_ids = campaign.formations.keys().copied().collect::<Vec<_>>();
    for id in formation_ids {
        advance_formation(campaign, data, id, &moved_formations, round_completed)?;
    }
    Ok(())
}

fn refund_person_if_unstarted(
    campaign: &mut StrategicCampaign,
    id: PersonId,
    course: &PersonCourse,
) -> Result<(), RuleError> {
    let amount = super::pricing::person_refund(course);
    if amount > 0 {
        refund(campaign, campaign.people[&id].faction, amount)?;
    }
    Ok(())
}

fn refund(
    campaign: &mut StrategicCampaign,
    owner: crate::data::world::FactionId,
    amount: i64,
) -> Result<(), RuleError> {
    let resources = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("course owner")
        .resources;
    resources.gold = resources
        .gold
        .checked_add(amount)
        .ok_or(RuleError::Overflow {
            field: "course refund",
        })?;
    Ok(())
}

fn person_site(
    campaign: &StrategicCampaign,
    person: PersonId,
) -> Option<crate::data::world::SiteId> {
    match campaign.people.get(&person)?.assignment {
        PersonAssignment::Formation { formation } => campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| army.site),
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Dead => None,
    }
}

fn advance_person(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: PersonId,
    moved_people: &BTreeSet<PersonId>,
    round_completed: bool,
) -> Result<(), RuleError> {
    let Some(course) = campaign.people[&id].career.course.clone() else {
        return Ok(());
    };
    let (site, facility, riding) = match course {
        PersonCourse::Class { target, site, .. } => (
            site,
            data.progression.careers.courses[&target].facility,
            false,
        ),
        PersonCourse::RidingPractice { site, .. } => (site, Facility::Stable, true),
    };
    let person = &campaign.people[&id];
    let owner = person.faction;
    if !person.is_alive()
        || person.career.retired
        || !campaign.is_independent(owner)
        || campaign
            .world
            .site(site)
            .is_none_or(|entry| entry.controller != Some(owner))
    {
        refund_person_if_unstarted(campaign, id, &course)?;
        campaign.people.get_mut(&id).expect("person").career.course = None;
        return Ok(());
    }
    let valid_site = specialization::course_site(campaign, owner, site, facility)
        && person_site(campaign, id) == Some(site);
    if !valid_site || person.status != PersonStatus::Fit || moved_people.contains(&id) {
        return Ok(());
    }
    if !round_completed {
        return Ok(());
    }
    let next_steps = match course {
        PersonCourse::Class {
            steps_completed, ..
        }
        | PersonCourse::RidingPractice {
            steps_completed, ..
        } => steps_completed.saturating_add(1),
    };
    if riding {
        let person = campaign.people.get_mut(&id).expect("person");
        person.career.riding_practice_seasons =
            person.career.riding_practice_seasons.saturating_add(1);
        person.career.course = None;
    } else if let PersonCourse::Class {
        target,
        site,
        paid_gold,
        ..
    } = course
    {
        let complete = next_steps >= data.progression.careers.course_steps;
        let person = campaign.people.get_mut(&id).expect("person");
        if complete {
            person.class = target;
            person.career.course = None;
        } else {
            person.career.course = Some(PersonCourse::Class {
                target,
                site,
                paid_gold,
                steps_completed: next_steps,
            });
        }
    }

    Ok(())
}

fn advance_formation(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: crate::state::military::FormationId,
    moved_formations: &BTreeSet<crate::state::military::FormationId>,
    round_completed: bool,
) -> Result<(), RuleError> {
    let Some(course) = campaign.formations[&id].service.course.clone() else {
        return Ok(());
    };
    let owner = campaign.formations[&id].faction;
    if !campaign.is_independent(owner)
        || campaign
            .world
            .site(course.site)
            .is_none_or(|entry| entry.controller != Some(owner))
    {
        if course.steps_completed == 0 {
            refund(campaign, owner, course.paid_gold)?;
        }
        campaign
            .formations
            .get_mut(&id)
            .expect("formation")
            .service
            .course = None;
        return Ok(());
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|member| member == id));
    let Some(army) = army else {
        return Ok(());
    };
    if army.site != course.site
        || !specialization::course_site(campaign, owner, course.site, Facility::TrainingGround)
        || moved_formations.contains(&id)
    {
        return Ok(());
    }
    if !round_completed {
        return Ok(());
    }
    let steps = course.steps_completed.saturating_add(1);
    if steps >= data.progression.specializations[&course.target].course_steps {
        let formation = campaign.formations.get_mut(&id).expect("formation");
        formation.service.specialization = Some(course.target);
        formation.service.course = None;
    } else {
        campaign
            .formations
            .get_mut(&id)
            .expect("formation")
            .service
            .course = Some(FormationCourse {
            steps_completed: steps,
            ..course
        });
    }

    Ok(())
}
