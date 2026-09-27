//! P19 population, pressure, repair and administration rules consumed by settlements.

use super::{
    economy::{unique_table, Habitation, Resources},
    validation::require,
    world::Geography,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SOURCE: &str = "assets/data/development.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevelopmentRules {
    pub schema_version: u32,
    #[serde(deserialize_with = "unique_table")]
    pub geography_caps: BTreeMap<Geography, Habitation>,
    pub pressure: PressureRules,
    pub population: PopulationRules,
    pub conditions: ConditionRules,
    pub administration: AdministrationRules,
    pub income_focus_bonus_percent: u32,
    pub occupation_income_percent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressureRules {
    pub safe: i32,
    pub connected: i32,
    pub trade: i32,
    pub food: i32,
    pub capital: i32,
    pub growth_focus: i32,
    pub battle: i32,
    pub cut_off: i32,
    pub damaged: i32,
    pub occupied: i32,
    pub per_round_min: i32,
    pub per_round_max: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub upgrade_threshold: i32,
    pub downgrade_threshold: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationRules {
    pub capacity_percent: u32,
    pub growth_divisor: u32,
    pub minimum_growth: u32,
    pub displacement_divisor: u32,
    pub migration_max_edges: u32,
    pub resettle_limit: u32,
    pub resettle_cost: Resources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionRules {
    pub food_damage: u32,
    pub pressure_damage: u32,
    pub occupation_threshold: u32,
    pub displacement_damage: u32,
    pub ruin_damage: u32,
    pub ruin_population: u32,
    pub ruin_steps: u32,
    pub reclamation_damage: u32,
    pub occupation_garrison_decay: u32,
    pub occupation_safe_decay: u32,
    pub structural_repair: u32,
    pub fort_repair: u32,
    pub fort_focus_repair: u32,
    pub functional_fort_damage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdministrationRules {
    pub capital_cost: Resources,
    pub capital_minimum: Habitation,
    pub capital_max_occupation: u32,
    pub headquarters_cost: Resources,
    pub headquarters_minimum: Habitation,
    pub headquarters_cooldown: u32,
}

impl DevelopmentRules {
    pub fn validate(&self) -> Result<(), String> {
        require(
            SOURCE,
            "schema_version",
            self.schema_version == 1,
            "must be 1",
        )?;
        let expected = [
            Geography::Plains,
            Geography::Forest,
            Geography::Hill,
            Geography::River,
            Geography::Valley,
            Geography::Coast,
            Geography::Marsh,
            Geography::Pass,
            Geography::Island,
        ];
        require(
            SOURCE,
            "geography_caps",
            self.geography_caps.len() == expected.len()
                && expected.iter().all(|key| {
                    self.geography_caps
                        .get(key)
                        .is_some_and(|cap| *cap >= Habitation::Village)
                }),
            "must contain every geography with Village or higher capacity",
        )?;
        self.validate_pressure()?;
        self.validate_population()?;
        self.validate_conditions()?;
        self.administration
            .capital_cost
            .validate(SOURCE, "capital_cost")?;
        self.administration
            .headquarters_cost
            .validate(SOURCE, "headquarters_cost")?;
        require(
            SOURCE,
            "administration",
            self.administration.capital_minimum >= Habitation::Outpost
                && self.administration.headquarters_minimum >= Habitation::Outpost
                && (1..=100).contains(&self.administration.capital_max_occupation)
                && (1..=100).contains(&self.administration.headquarters_cooldown),
            "invalid role requirement",
        )?;
        require(
            SOURCE,
            "income modifiers",
            self.income_focus_bonus_percent <= 100 && self.occupation_income_percent <= 100,
            "must be percentages",
        )
    }

    fn validate_pressure(&self) -> Result<(), String> {
        let p = &self.pressure;
        require(
            SOURCE,
            "pressure",
            p.per_round_min < 0
                && p.per_round_max > 0
                && p.per_round_min >= p.minimum
                && p.per_round_max <= p.maximum
                && p.minimum < 0
                && p.maximum > 0
                && p.minimum >= -1000
                && p.maximum <= 1000
                && p.upgrade_threshold > 0
                && p.upgrade_threshold <= p.maximum
                && p.downgrade_threshold > 0
                && p.downgrade_threshold <= -p.minimum,
            "invalid pressure bounds or transition thresholds",
        )?;
        require(
            SOURCE,
            "pressure contributions",
            [
                p.safe,
                p.connected,
                p.trade,
                p.food,
                p.capital,
                p.growth_focus,
            ]
            .iter()
            .all(|value| (0..=100).contains(value))
                && [p.battle, p.cut_off, p.damaged, p.occupied]
                    .iter()
                    .all(|value| (-100..=0).contains(value)),
            "bonuses must be nonnegative and penalties nonpositive, bounded by 100",
        )
    }

    fn validate_population(&self) -> Result<(), String> {
        let p = &self.population;
        p.resettle_cost.validate(SOURCE, "resettle_cost")?;
        require(
            SOURCE,
            "population",
            (100..=1000).contains(&p.capacity_percent)
                && (1..=10_000).contains(&p.growth_divisor)
                && (1..=100).contains(&p.minimum_growth)
                && (1..=10_000).contains(&p.displacement_divisor)
                && (1..=20).contains(&p.migration_max_edges)
                && (1..=1000).contains(&p.resettle_limit),
            "invalid growth, capacity or migration limits",
        )
    }

    fn validate_conditions(&self) -> Result<(), String> {
        let c = &self.conditions;
        let percentages = [
            c.food_damage,
            c.pressure_damage,
            c.occupation_threshold,
            c.displacement_damage,
            c.ruin_damage,
            c.reclamation_damage,
            c.occupation_garrison_decay,
            c.occupation_safe_decay,
            c.structural_repair,
            c.fort_repair,
            c.fort_focus_repair,
            c.functional_fort_damage,
        ];
        require(
            SOURCE,
            "conditions",
            percentages.iter().all(|value| *value <= 100)
                && c.ruin_damage > 0
                && c.functional_fort_damage > 0
                && c.occupation_threshold > 0
                && (1..=1000).contains(&c.ruin_population)
                && (1..=100).contains(&c.ruin_steps),
            "invalid condition threshold or duration",
        )
    }
}
