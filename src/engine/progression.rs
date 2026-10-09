//! Evidence-led people and formation progression.

mod career;
mod commands;
mod courses;
mod emergence;
mod pricing;
mod relationships;
pub(crate) use emergence::has_emerged;
pub use pricing::course_gold_cost;
pub(crate) use pricing::refund_departures;
mod specialization;

use crate::{
    data::GameData,
    engine::{Command, RuleError},
    state::{
        campaign::DomainFact, military::FormationId, people::PersonAssignment, StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub use career::{career_options, CareerOption, CareerRequirement};
pub use specialization::{specialization_options, SpecializationOption};

pub(super) fn validate_command(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    commands::validate(campaign, data, owner, command)
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    commands::execute(campaign, data, owner, command)
}

pub(super) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    facts: &[DomainFact],
    round_completed: bool,
) -> Result<(), RuleError> {
    emergence::update_tracked(campaign, data);
    relationships::record(campaign, data, facts, round_completed)?;
    courses::advance(campaign, data, facts, round_completed)?;
    if round_completed {
        emergence::advance(campaign, data)?;
    }
    Ok(())
}

pub(super) fn reset_formation_vacancy_progress(
    campaign: &mut StrategicCampaign,
    formation: FormationId,
) {
    let sequence = campaign.accepted_sequence;
    if let Some(formation) = campaign.formations.get_mut(&formation) {
        formation.service.vacancy_service_progress = 0;
        formation.service.vacancy_service_after_sequence = sequence;
    }
}

pub(super) fn reset_vacancy_progress_for_assignments(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
) {
    let mut reset = BTreeSet::new();
    for person in campaign.people.values() {
        let prior = before
            .people
            .get(&person.id)
            .map(|previous| previous.assignment);
        if prior != Some(person.assignment) {
            if let Some(PersonAssignment::Formation { formation }) = prior {
                reset.insert(formation);
            }
            if let PersonAssignment::Formation { formation } = person.assignment {
                reset.insert(formation);
            }
        }
    }
    for person in before.people.values() {
        if !campaign.people.contains_key(&person.id) {
            if let PersonAssignment::Formation { formation } = person.assignment {
                reset.insert(formation);
            }
        }
    }
    for formation in reset {
        reset_formation_vacancy_progress(campaign, formation);
    }
}
