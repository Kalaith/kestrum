//! Ordinary career prerequisites are derived from durable individual evidence.

use crate::{
    data::{economy::TroopKind, progression::CareerCourseRule, world::PersonClass, GameData},
    engine::RuleError,
    state::{
        evidence::EvidenceKind,
        people::{Person, PersonAssignment, PersonId, PersonStatus},
        StrategicCampaign,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CareerRequirement {
    pub label: &'static str,
    pub current: usize,
    pub required: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CareerOption {
    pub class: PersonClass,
    pub eligible: bool,
    pub requirements: Vec<CareerRequirement>,
    pub course: CareerCourseRuleView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CareerCourseRuleView {
    pub gold_cost: i64,
    pub facility: crate::data::world::Facility,
    pub requires_horse_access: bool,
}

pub fn career_options(
    campaign: &StrategicCampaign,
    data: &GameData,
    person: PersonId,
) -> Result<Vec<CareerOption>, RuleError> {
    let person = campaign
        .people
        .get(&person)
        .ok_or(RuleError::UnknownPerson { person })?;
    Ok([
        PersonClass::Infantry,
        PersonClass::Archer,
        PersonClass::Scout,
        PersonClass::Cavalry,
        PersonClass::Medic,
        PersonClass::Officer,
    ]
    .into_iter()
    .map(|class| option(campaign, data, person, class))
    .collect())
}

pub(super) fn requirements(
    person: &Person,
    data: &GameData,
    class: PersonClass,
) -> Vec<CareerRequirement> {
    let rules = &data.progression.careers;
    let battles = |kind| {
        person
            .evidence
            .service_by_troop
            .get(&kind)
            .copied()
            .unwrap_or(0) as usize
    };
    let count = |kind| person.evidence.counts.get(&kind).copied().unwrap_or(0) as usize;
    match class {
        PersonClass::Infantry => vec![alternative(
            "Warrior or Spearman battles",
            battles(TroopKind::Warriors) + battles(TroopKind::Spearmen),
            rules.infantry_battles as usize,
        )],
        PersonClass::Archer => vec![alternative(
            "Archer battles",
            battles(TroopKind::Archers),
            rules.archer_battles as usize,
        )],
        PersonClass::Scout => vec![alternative(
            "Distinct physical routes",
            person.evidence.traversed_routes.len(),
            rules.scout_routes as usize,
        )],
        PersonClass::Cavalry => vec![
            alternative(
                "Riding practice seasons",
                person.career.riding_practice_seasons as usize,
                rules.riding_seasons as usize,
            ),
            alternative(
                "Rider battles",
                battles(TroopKind::Riders),
                rules.rider_battles as usize,
            ),
        ],
        PersonClass::Medic => vec![alternative(
            "People treated",
            count(EvidenceKind::TreatedWounded),
            rules.treatment_occasions as usize,
        )],
        PersonClass::Officer => {
            let encounters = count(EvidenceKind::MeaningfulEncounter);
            let command =
                count(EvidenceKind::AssumedCommand) + count(EvidenceKind::CommandedVictory);
            vec![
                alternative(
                    "Meaningful encounters",
                    encounters,
                    rules.officer_encounters as usize,
                ),
                alternative(
                    "Command evidence",
                    command,
                    rules.officer_command_facts as usize,
                ),
            ]
        }
        PersonClass::Recruit => Vec::new(),
    }
}

pub(super) fn option(
    campaign: &StrategicCampaign,
    data: &GameData,
    person: &Person,
    class: PersonClass,
) -> CareerOption {
    let course = &data.progression.careers.courses[&class];
    let requirements = requirements(person, data, class);
    let base_valid = person.is_alive()
        && !person.career.retired
        && person.status == PersonStatus::Fit
        && person.age_years(campaign.completed_rounds) >= 17
        && !matches!(person.assignment, PersonAssignment::Dead);
    CareerOption {
        class,
        eligible: base_valid
            && requirements
                .iter()
                .all(|item| item.current >= item.required),
        requirements,
        course: view(course),
    }
}

fn view(course: &CareerCourseRule) -> CareerCourseRuleView {
    CareerCourseRuleView {
        gold_cost: course.gold_cost,
        facility: course.facility,
        requires_horse_access: course.requires_horse_access,
    }
}

fn alternative(label: &'static str, current: usize, required: usize) -> CareerRequirement {
    CareerRequirement {
        label,
        current,
        required,
    }
}
