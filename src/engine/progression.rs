//! Evidence-led people and formation progression.

mod career;
mod commands;
mod courses;
mod emergence;
mod relationships;
mod specialization;

use crate::{
    data::GameData,
    engine::{Command, RuleError},
    state::{campaign::DomainFact, StrategicCampaign},
};

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
    relationships::record(campaign, data, facts)?;
    courses::advance(campaign, data, facts, round_completed)?;
    if round_completed {
        emergence::advance(campaign, data)?;
    }
    Ok(())
}
