//! Validate, charge and persist progression decisions through ordinary commands.

use super::{career, specialization};
use crate::{
    data::{
        economy::Resources,
        world::{Facility, PersonClass, SiteId},
        GameData,
    },
    engine::{movement, Command, RuleError},
    state::{
        evidence::FormationCourse,
        people::{PersonAssignment, PersonCourse, PersonId, PersonStatus},
        StrategicCampaign,
    },
};

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::SetCommander { army, person } => {
            let entry = campaign
                .armies
                .get(army)
                .ok_or(RuleError::UnknownArmy { army: *army })?;
            if entry.faction != owner {
                return Err(RuleError::ArmyNotOwned { army: *army });
            }
            if let Some(person) = person {
                let candidate = campaign
                    .people
                    .get(person)
                    .ok_or(RuleError::UnknownPerson { person: *person })?;
                if candidate.faction != owner
                    || candidate.career.retired
                    || !candidate.is_fit_for_field(
                        campaign.completed_rounds,
                        data.rules.leadership.field_min_age_years,
                    )
                    || !matches!(candidate.assignment, PersonAssignment::Formation { formation } if entry.formation_ids().any(|id| id == formation))
                {
                    return Err(RuleError::Progression(
                        "Choose a fit adult attached to this army.".into(),
                    ));
                }
            }
        }
        Command::TrainPerson {
            person,
            class,
            site,
        } => {
            if *class == PersonClass::Recruit {
                return Err(RuleError::Progression(
                    "Recruit is the starting class, not a course.".into(),
                ));
            }
            let entry = campaign
                .people
                .get(person)
                .ok_or(RuleError::UnknownPerson { person: *person })?;
            if entry.faction != owner {
                return Err(RuleError::PersonNotOwned { person: *person });
            }
            let option = career::option(campaign, data, entry, *class);
            if !option.eligible {
                return Err(RuleError::Progression(format!(
                    "{} career prerequisites are not met.",
                    class_name(*class)
                )));
            }
            if entry.class == *class || entry.career.course.is_some() {
                return Err(RuleError::Progression(
                    "This person already has that career or an active course.".into(),
                ));
            }
            validate_person_site(
                campaign,
                data,
                owner,
                *person,
                *site,
                option.course.facility,
                option.course.requires_horse_access,
            )?;
            check_gold(campaign, owner, option.course.gold_cost)?;
        }
        Command::PracticeRiding { person, site } => {
            let entry = campaign
                .people
                .get(person)
                .ok_or(RuleError::UnknownPerson { person: *person })?;
            if entry.faction != owner {
                return Err(RuleError::PersonNotOwned { person: *person });
            }
            if !entry.is_alive()
                || entry.career.retired
                || entry.status != PersonStatus::Fit
                || entry.age_years(campaign.completed_rounds) < 17
                || entry.career.course.is_some()
                || matches!(entry.assignment, PersonAssignment::Dead)
            {
                return Err(RuleError::Progression(
                    "Riding practice needs a fit, unretired adult with no active course.".into(),
                ));
            }
            validate_person_site(
                campaign,
                data,
                owner,
                *person,
                *site,
                Facility::Stable,
                true,
            )?;
        }
        Command::CancelPersonCourse { person } => {
            let entry = campaign
                .people
                .get(person)
                .ok_or(RuleError::UnknownPerson { person: *person })?;
            if entry.faction != owner {
                return Err(RuleError::PersonNotOwned { person: *person });
            }
            if entry.career.course.is_none() {
                return Err(RuleError::Progression(
                    "This person has no active course.".into(),
                ));
            }
        }
        Command::SpecializeFormation {
            formation,
            specialization: kind,
            site,
        } => {
            let entry = campaign
                .formations
                .get(formation)
                .ok_or(RuleError::UnknownFormation {
                    formation: *formation,
                })?;
            if entry.faction != owner {
                return Err(RuleError::FormationNotOwned {
                    formation: *formation,
                });
            }
            let option = specialization::specialization_options(campaign, data, *formation)?
                .into_iter()
                .find(|option| option.specialization == *kind)
                .ok_or_else(|| {
                    RuleError::Progression("That specialization is unavailable.".into())
                })?;
            if !option.eligible {
                return Err(RuleError::Progression(
                    "This formation does not meet the specialization requirements.".into(),
                ));
            }
            let army = campaign
                .armies
                .values()
                .find(|army| army.formation_ids().any(|id| id == *formation))
                .ok_or(RuleError::UnknownFormation {
                    formation: *formation,
                })?;
            if army.site != *site
                || !specialization::course_site(
                    campaign,
                    data,
                    owner,
                    *site,
                    Facility::TrainingGround,
                )
            {
                return Err(RuleError::Progression("Specialization needs this formation at its supplied, unbesieged Training Ground.".into()));
            }
            if movement::formation_remaining(campaign, data, *formation)?
                < entry.movement_allowance(data)
            {
                return Err(RuleError::Progression(
                    "A formation that has moved this season cannot begin training.".into(),
                ));
            }
            check_gold(campaign, owner, option.gold_cost)?;
        }
        Command::CancelFormationCourse { formation } => {
            let entry = campaign
                .formations
                .get(formation)
                .ok_or(RuleError::UnknownFormation {
                    formation: *formation,
                })?;
            if entry.faction != owner {
                return Err(RuleError::FormationNotOwned {
                    formation: *formation,
                });
            }
            if entry.service.course.is_none() {
                return Err(RuleError::Progression(
                    "This formation has no active specialization course.".into(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::SetCommander { army, person } => {
            campaign
                .armies
                .get_mut(army)
                .ok_or(RuleError::UnknownArmy { army: *army })?
                .commander = *person
        }
        Command::TrainPerson {
            person,
            class,
            site,
        } => {
            let course = &data.progression.careers.courses[class];
            charge(campaign, owner, course.gold_cost)?;
            campaign
                .people
                .get_mut(person)
                .expect("validated person")
                .career
                .course = Some(PersonCourse::Class {
                target: *class,
                site: *site,
                steps_completed: 0,
            });
        }
        Command::PracticeRiding { person, site } => {
            campaign
                .people
                .get_mut(person)
                .expect("validated person")
                .career
                .course = Some(PersonCourse::RidingPractice {
                site: *site,
                steps_completed: 0,
            })
        }
        Command::CancelPersonCourse { person } => {
            let course = campaign
                .people
                .get_mut(person)
                .expect("validated person")
                .career
                .course
                .take()
                .expect("validated course");
            let refund = match course {
                PersonCourse::Class {
                    target,
                    steps_completed: 0,
                    ..
                } => data.progression.careers.courses[&target].gold_cost,
                _ => 0,
            };
            refund_gold(campaign, owner, refund)?;
        }
        Command::SpecializeFormation {
            formation,
            specialization: kind,
            site,
        } => {
            let rule = &data.progression.specializations[kind];
            charge(campaign, owner, rule.gold_cost)?;
            campaign
                .formations
                .get_mut(formation)
                .expect("validated formation")
                .service
                .course = Some(FormationCourse {
                target: *kind,
                site: *site,
                steps_completed: 0,
            });
        }
        Command::CancelFormationCourse { formation } => {
            let course = campaign
                .formations
                .get_mut(formation)
                .expect("validated formation")
                .service
                .course
                .take()
                .expect("validated course");
            let refund = if course.steps_completed == 0 {
                data.progression.specializations[&course.target].gold_cost
            } else {
                0
            };
            refund_gold(campaign, owner, refund)?;
        }
        _ => {
            return Err(RuleError::InvalidState(
                "Unexpected progression dispatch.".into(),
            ))
        }
    }
    Ok(())
}

fn validate_person_site(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    person: PersonId,
    site: SiteId,
    facility: Facility,
    horses: bool,
) -> Result<(), RuleError> {
    if !specialization::course_site(campaign, data, owner, site, facility) {
        return Err(RuleError::Progression(format!(
            "Course requires a supplied, unbesieged site with a functional {}.",
            facility_name(facility)
        )));
    }
    if horses
        && !campaign.world.site(site).is_some_and(|entry| {
            entry
                .tags
                .contains(&crate::data::world::SiteTag::HorseAccess)
        })
    {
        return Err(RuleError::MissingHorses { site });
    }
    let person = &campaign.people[&person];
    let location = match person.assignment {
        PersonAssignment::Formation { formation } => campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| army.site),
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Dead => None,
    };
    if location != Some(site) {
        return Err(RuleError::Progression(
            "The person's formation must be present at the course site.".into(),
        ));
    }
    Ok(())
}

fn check_gold(
    campaign: &StrategicCampaign,
    owner: crate::data::world::FactionId,
    cost: i64,
) -> Result<(), RuleError> {
    let available = campaign.factions[&owner].resources;
    if available.gold < cost {
        return Err(RuleError::InsufficientResources {
            required: Resources {
                gold: cost,
                wood: 0,
                stone: 0,
            },
            available,
        });
    }
    Ok(())
}

fn charge(
    campaign: &mut StrategicCampaign,
    owner: crate::data::world::FactionId,
    cost: i64,
) -> Result<(), RuleError> {
    check_gold(campaign, owner, cost)?;
    campaign
        .factions
        .get_mut(&owner)
        .expect("validated owner")
        .resources
        .gold -= cost;
    Ok(())
}

fn refund_gold(
    campaign: &mut StrategicCampaign,
    owner: crate::data::world::FactionId,
    refund: i64,
) -> Result<(), RuleError> {
    let resources = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("validated owner")
        .resources;
    resources.gold = resources
        .gold
        .checked_add(refund)
        .ok_or(RuleError::Overflow {
            field: "course refund",
        })?;
    Ok(())
}

fn class_name(class: PersonClass) -> &'static str {
    match class {
        PersonClass::Infantry => "Infantry",
        PersonClass::Archer => "Archer",
        PersonClass::Scout => "Scout",
        PersonClass::Cavalry => "Cavalry",
        PersonClass::Medic => "Medic",
        PersonClass::Officer => "Officer",
        PersonClass::Recruit => "Recruit",
    }
}
fn facility_name(facility: Facility) -> &'static str {
    match facility {
        Facility::TrainingGround => "Training Ground",
        Facility::Stable => "Stable",
        Facility::Infirmary => "Infirmary",
        Facility::Workshop => "Workshop",
        Facility::Temple => "Temple",
    }
}
