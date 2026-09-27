//! Consumed construction rules. Existing order prices stay in the economy table.

use super::economy::{unique_table, Habitation, Resources};
use super::validation::require;
use super::world::{Facility, SiteTag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SOURCE: &str = "assets/data/construction_rules.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionRules {
    pub schema_version: u32,
    pub outpost_steps: u32,
    pub road_steps: u32,
    pub fort_steps: u32,
    pub road_repair: ConstructionDefinition,
    #[serde(deserialize_with = "unique_table")]
    pub facilities: BTreeMap<Facility, FacilityDefinition>,
    pub population: PopulationRules,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionDefinition {
    pub cost: Resources,
    pub steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityDefinition {
    pub cost: Resources,
    pub steps: u32,
    pub minimum_habitation: Habitation,
    pub required_tag: Option<SiteTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationRules {
    #[serde(deserialize_with = "unique_table")]
    pub minimum: BTreeMap<Habitation, u32>,
    pub headquarters: u32,
    pub settler_limit: u32,
}

impl ConstructionRules {
    pub fn validate(&self) -> Result<(), String> {
        require(
            SOURCE,
            "schema_version",
            self.schema_version == 1,
            "must be 1",
        )?;
        for (field, steps) in [
            ("outpost_steps", self.outpost_steps),
            ("road_steps", self.road_steps),
            ("fort_steps", self.fort_steps),
            ("road_repair.steps", self.road_repair.steps),
        ] {
            validate_steps(field, steps)?;
        }
        validate_cost("road_repair.cost", &self.road_repair.cost)?;
        self.validate_facilities()?;
        self.population.validate()
    }

    fn validate_facilities(&self) -> Result<(), String> {
        let prerequisites = [
            (Facility::TrainingGround, Habitation::Outpost, None),
            (
                Facility::Stable,
                Habitation::Village,
                Some(SiteTag::HorseAccess),
            ),
            (Facility::Infirmary, Habitation::Village, None),
            (Facility::Workshop, Habitation::Village, None),
            (Facility::Temple, Habitation::Village, None),
        ];
        require(
            SOURCE,
            "facilities",
            self.facilities.len() == prerequisites.len(),
            "must define all five ordinary facilities exactly once",
        )?;
        for (facility, habitation, tag) in prerequisites {
            let field = format!("facilities.{facility:?}");
            let definition = self
                .facilities
                .get(&facility)
                .ok_or_else(|| format!("{SOURCE}: {field}: missing facility definition"))?;
            validate_steps(&format!("{field}.steps"), definition.steps)?;
            validate_cost(&format!("{field}.cost"), &definition.cost)?;
            require(
                SOURCE,
                &format!("{field}.minimum_habitation"),
                definition.minimum_habitation == habitation,
                "unsupported prerequisite; see P06",
            )?;
            require(
                SOURCE,
                &format!("{field}.required_tag"),
                definition.required_tag == tag,
                "unsupported prerequisite; only Stable requires HorseAccess",
            )?;
        }
        Ok(())
    }
}

impl PopulationRules {
    fn validate(&self) -> Result<(), String> {
        let tiers = [
            Habitation::Unsettled,
            Habitation::Camp,
            Habitation::Outpost,
            Habitation::Hamlet,
            Habitation::Village,
            Habitation::Town,
            Habitation::City,
            Habitation::MajorCity,
        ];
        require(
            SOURCE,
            "population.minimum",
            self.minimum.len() == tiers.len(),
            "must define all eight habitation tiers exactly once",
        )?;
        let mut previous = None;
        for tier in tiers {
            let field = format!("population.minimum.{tier:?}");
            let value = *self
                .minimum
                .get(&tier)
                .ok_or_else(|| format!("{SOURCE}: {field}: missing population minimum"))?;
            require(
                SOURCE,
                &field,
                previous.map_or(value == 0, |prior| value > prior) && value <= 1_000_000,
                "Unsettled must be zero; inhabited minima must increase and be at most 1000000",
            )?;
            previous = Some(value);
        }
        require(
            SOURCE,
            "population.headquarters",
            (self.minimum[&Habitation::Village]..=1_000_000).contains(&self.headquarters),
            "must support the starting Village and be at most 1000000",
        )?;
        require(
            SOURCE,
            "population.settler_limit",
            (1..=1_000_000).contains(&self.settler_limit),
            "must be within 1..=1000000",
        )
    }
}

fn validate_steps(field: &str, steps: u32) -> Result<(), String> {
    require(
        SOURCE,
        field,
        (1..=100).contains(&steps),
        "must be within 1..=100",
    )
}

fn validate_cost(field: &str, cost: &Resources) -> Result<(), String> {
    cost.validate(SOURCE, field)?;
    require(
        SOURCE,
        field,
        cost.gold > 0 || cost.wood > 0 || cost.stone > 0,
        "must charge at least one resource",
    )
}
