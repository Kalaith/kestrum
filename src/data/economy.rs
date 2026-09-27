//! The complete existing economy table, with supported-policy validation.

use super::validation::require;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fmt, marker::PhantomData};

pub const SOURCE: &str = "assets/data/economy.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub gold: i64,
    pub wood: i64,
    pub stone: i64,
}

impl Resources {
    pub fn validate(&self, source: &str, field: &str) -> Result<(), String> {
        for (name, value) in [
            ("gold", self.gold),
            ("wood", self.wood),
            ("stone", self.stone),
        ] {
            require(
                source,
                &format!("{field}.{name}"),
                value >= 0,
                "must be nonnegative",
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TroopKind {
    Warriors,
    Spearmen,
    Archers,
    Riders,
    Medics,
    SiegeEngines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Habitation {
    Unsettled,
    Camp,
    Outpost,
    Hamlet,
    Village,
    Town,
    City,
    MajorCity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderKind {
    EstablishOutpost,
    ImproveRoad,
    BuildFort,
    ChangeFocus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationEconomy {
    pub capacity: u32,
    pub movement_allowance: u32,
    pub recruit_cost: Resources,
    pub upkeep_gold: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderEconomy {
    pub cost: Resources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recovery {
    pub capacity_percent_per_round: u32,
    pub full_replacement_recruit_gold_percent: u32,
    pub round_cost_up: bool,
    pub requires_supply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upkeep {
    pub named_character_gold: i64,
    pub charge_full_formation_rate: bool,
    pub carry_unpaid_debt: bool,
    pub block_recruitment_and_recovery_on_shortfall: bool,
    pub automatic_disbanding: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refunds {
    pub unstarted_construction_percent: u32,
    pub started_construction_percent: u32,
    pub disband_percent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecruitmentPopulation {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyStatus {
    ProvisionalDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyPeriod {
    FullRound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Economy {
    pub schema_version: u32,
    pub status: EconomyStatus,
    pub period: EconomyPeriod,
    pub starting_resources: Resources,
    pub headquarters_income_bonus: Resources,
    pub facility_failure_damage: u32,
    pub income_damage_divisor: u32,
    #[serde(deserialize_with = "unique_table")]
    pub settlement_income: BTreeMap<Habitation, Resources>,
    #[serde(deserialize_with = "unique_table")]
    pub formations: BTreeMap<TroopKind, FormationEconomy>,
    #[serde(deserialize_with = "unique_table")]
    pub orders: BTreeMap<OrderKind, OrderEconomy>,
    pub recovery: Recovery,
    pub upkeep: Upkeep,
    pub refunds: Refunds,
    pub recruitment_population: RecruitmentPopulation,
}

// These keyed content definitions carry IDs too. Serde's ordinary map would
// silently replace a duplicate key before semantic validation could inspect it.
pub(super) fn unique_table<'de, D, K, V>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: Deserialize<'de> + Ord + fmt::Debug,
    V: Deserialize<'de>,
{
    struct TableVisitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K, V> serde::de::Visitor<'de> for TableVisitor<K, V>
    where
        K: Deserialize<'de> + Ord + fmt::Debug,
        V: Deserialize<'de>,
    {
        type Value = BTreeMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a content table with unique definition IDs")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<K, V>()? {
                if entries.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate content ID {key:?}"
                    )));
                }
                entries.insert(key, value);
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(TableVisitor(PhantomData))
}

impl Economy {
    pub fn validate(&self) -> Result<(), String> {
        require(
            SOURCE,
            "schema_version",
            self.schema_version == 1,
            "unsupported version",
        )?;
        self.starting_resources
            .validate(SOURCE, "starting_resources")?;
        self.headquarters_income_bonus
            .validate(SOURCE, "headquarters_income_bonus")?;
        require(
            SOURCE,
            "facility_failure_damage",
            (1..=100).contains(&self.facility_failure_damage),
            "must be within 1..=100",
        )?;
        require(
            SOURCE,
            "income_damage_divisor",
            self.income_damage_divisor > 0,
            "must be positive",
        )?;
        require(
            SOURCE,
            "settlement_income",
            self.settlement_income.len() == 8,
            "all eight habitation tiers are required",
        )?;
        require(
            SOURCE,
            "formations",
            self.formations.len() == 6,
            "all six human troop types are required",
        )?;
        require(
            SOURCE,
            "orders",
            self.orders.len() == 4,
            "all four existing orders are required",
        )?;
        for (tier, income) in &self.settlement_income {
            income.validate(SOURCE, &format!("settlement_income.{tier:?}"))?;
        }
        for (kind, formation) in &self.formations {
            let field = format!("formations.{kind:?}");
            require(
                SOURCE,
                &format!("{field}.capacity"),
                formation.capacity > 0,
                "must be positive",
            )?;
            require(
                SOURCE,
                &format!("{field}.movement_allowance"),
                (1..=100).contains(&formation.movement_allowance),
                "must be within 1..=100",
            )?;
            formation
                .recruit_cost
                .validate(SOURCE, &format!("{field}.recruit_cost"))?;
            require(
                SOURCE,
                &format!("{field}.recruit_cost.gold"),
                formation.recruit_cost.gold > 0,
                "must be positive for recovery costing",
            )?;
            require(
                SOURCE,
                &format!("{field}.upkeep_gold"),
                formation.upkeep_gold >= 0,
                "must be nonnegative",
            )?;
        }
        for (kind, order) in &self.orders {
            order
                .cost
                .validate(SOURCE, &format!("orders.{kind:?}.cost"))?;
        }
        self.validate_policies()
    }

    fn validate_policies(&self) -> Result<(), String> {
        for (field, value) in [
            (
                "recovery.capacity_percent_per_round",
                self.recovery.capacity_percent_per_round,
            ),
            (
                "recovery.full_replacement_recruit_gold_percent",
                self.recovery.full_replacement_recruit_gold_percent,
            ),
        ] {
            require(
                SOURCE,
                field,
                (1..=100).contains(&value),
                "must be within 1..=100",
            )?;
        }
        for (field, supported) in [
            ("recovery.round_cost_up", self.recovery.round_cost_up),
            ("recovery.requires_supply", self.recovery.requires_supply),
            (
                "upkeep.named_character_gold",
                self.upkeep.named_character_gold == 0,
            ),
            (
                "upkeep.charge_full_formation_rate",
                self.upkeep.charge_full_formation_rate,
            ),
            ("upkeep.carry_unpaid_debt", !self.upkeep.carry_unpaid_debt),
            (
                "upkeep.block_recruitment_and_recovery_on_shortfall",
                self.upkeep.block_recruitment_and_recovery_on_shortfall,
            ),
            (
                "upkeep.automatic_disbanding",
                !self.upkeep.automatic_disbanding,
            ),
            (
                "refunds.unstarted_construction_percent",
                self.refunds.unstarted_construction_percent == 100,
            ),
            (
                "refunds.started_construction_percent",
                self.refunds.started_construction_percent == 0,
            ),
            ("refunds.disband_percent", self.refunds.disband_percent == 0),
            (
                "recruitment_population.enabled",
                !self.recruitment_population.enabled,
            ),
        ] {
            require(
                SOURCE,
                field,
                supported,
                "unsupported policy; see P06 and economy defaults",
            )?;
        }
        Ok(())
    }
}
