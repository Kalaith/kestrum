//! Sparse family and service-entry thresholds used by K15.

use super::{economy::Habitation, validation::require};
use serde::{Deserialize, Serialize};

pub const SOURCE: &str = "assets/data/household_rules.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HouseholdRules {
    pub schema_version: u32,
    pub partnership_minimum_age_years: u32,
    pub partnership_shared_seasons: u32,
    pub child_minimum_age_years: u32,
    pub child_maximum_age_years: u32,
    pub birth_chance_permille: u32,
    pub birth_gap_seasons: u32,
    pub maximum_dependent_children: u32,
    pub ward_age_years: u32,
    pub trainee_minimum_age_years: u32,
    pub service_minimum_age_years: u32,
    pub apprentice_cost_gold: i64,
    pub apprentice_minimum_habitation: Habitation,
}

impl HouseholdRules {
    pub fn validate(&self) -> Result<(), String> {
        require(
            SOURCE,
            "schema and thresholds",
            self.schema_version == 1
                && self.partnership_minimum_age_years == 18
                && self.partnership_shared_seasons == 4
                && self.child_minimum_age_years == 20
                && self.child_maximum_age_years == 40
                && self.birth_chance_permille == 200
                && self.birth_gap_seasons == 8
                && self.maximum_dependent_children == 2
                && self.ward_age_years == 8
                && self.trainee_minimum_age_years == 13
                && self.service_minimum_age_years == 17
                && self.apprentice_cost_gold == 20
                && self.apprentice_minimum_habitation == Habitation::Village,
            "must preserve the delegated P22 defaults",
        )
    }
}
