//! Authored age, mortality, and mentorship thresholds for tracked people.

use crate::data::validation::require;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRules {
    pub schema_version: u32,
    pub elder_age_years: u32,
    pub reduced_movement_start_years: u32,
    pub reduced_movement_end_years: u32,
    pub light_cavalry_movement: u32,
    pub automatic_retirement_age_years: u32,
    pub governor_pressure: i32,
    pub death_chances: Vec<AgeDeathChance>,
    pub mentor_minimum_age_years: u32,
    pub mentor_service_seasons: u32,
    pub apprenticeship_seasons: u32,
    pub mentor_capacity: u32,
    pub learner_minimum_age_years: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeDeathChance {
    pub minimum_age_years: u32,
    pub chance_permille: u32,
}

impl LifecycleRules {
    pub fn validate(&self) -> Result<(), String> {
        let expected = [(60, 20), (70, 50), (80, 150), (90, 350)];
        require(
            "lifecycle.json",
            "age and role thresholds",
            self.schema_version == 1
                && self.elder_age_years == 56
                && self.reduced_movement_start_years == 41
                && self.reduced_movement_end_years == 55
                && self.light_cavalry_movement == 8
                && self.automatic_retirement_age_years == 70
                && self.governor_pressure == 1,
            "must preserve P20 age and governor effects",
        )?;
        require(
            "lifecycle.json",
            "death_chances",
            self.death_chances.len() == expected.len()
                && self
                    .death_chances
                    .iter()
                    .zip(expected)
                    .all(|(actual, (age, chance))| {
                        actual.minimum_age_years == age && actual.chance_permille == chance
                    }),
            "must match P20's birthday-only mortality table",
        )?;
        require(
            "lifecycle.json",
            "mentorship",
            self.mentor_minimum_age_years == 26
                && self.mentor_service_seasons == 4
                && self.apprenticeship_seasons == 4
                && self.mentor_capacity == 1
                && self.learner_minimum_age_years == 13,
            "must preserve P21 qualification and capacity",
        )
    }

    pub fn death_chance_permille(&self, age_years: u32) -> u32 {
        self.death_chances
            .iter()
            .rev()
            .find(|rule| age_years >= rule.minimum_age_years)
            .map_or(0, |rule| rule.chance_permille)
    }
}
