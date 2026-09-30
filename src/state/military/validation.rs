//! Committed military membership, founder links, damage, and economy statements.

use crate::{
    data::GameData,
    state::{people::PersonAssignment, StrategicCampaign},
};
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub(crate) fn validate_military(&self, data: &GameData) -> Result<(), String> {
        let mut assigned = BTreeSet::new();
        for (id, army) in &self.armies {
            require(*id == army.id && id.0 > 0, "armies.id", "invalid identity")?;
            require(
                self.factions.contains_key(&army.faction),
                "armies.faction",
                "unknown faction",
            )?;
            require(
                self.world.site(army.site).is_some(),
                "armies.site",
                "unknown physical site",
            )?;
            require(valid_name(&army.name), "armies.name", "invalid army name")?;
            if let Some(doctrine) = army.battle_doctrine {
                require(
                    data.battle_tactics.doctrines.contains_key(&doctrine),
                    "armies.battle_doctrine",
                    "unknown battle doctrine",
                )?;
            }
            require(
                !army.is_empty(),
                "armies.slots",
                "empty armies must be removed",
            )?;
            for formation in army.formation_ids() {
                require(
                    assigned.insert(formation),
                    "armies.slots",
                    "formation belongs to multiple slots",
                )?;
                require(
                    self.formations
                        .get(&formation)
                        .is_some_and(|entry| entry.faction == army.faction),
                    "armies.slots",
                    "unknown or foreign formation",
                )?;
            }
            if let Some(commander) = army.commander {
                require(
                    self.people.get(&commander).is_some_and(|person| {
                        person.faction == army.faction
                            && !person.career.retired
                            && person.is_fit_for_field(
                                self.completed_rounds,
                                data.rules.leadership.field_min_age_years,
                            )
                            && matches!(person.assignment, PersonAssignment::Formation {formation}
                            if army.formation_ids().any(|id| id == formation))
                    }),
                    "armies.commander",
                    "commander must be an attached adult of this faction",
                )?;
            }
        }
        require(
            assigned.len() == self.formations.len(),
            "formations",
            "every formation needs exactly one army slot",
        )?;
        for (id, formation) in &self.formations {
            let definition = &data.economy.formations[&formation.kind];
            let movement_allowance = formation.movement_allowance(data);
            require(
                *id == formation.id && id.0 > 0,
                "formations.id",
                "invalid identity",
            )?;
            require(
                self.factions.contains_key(&formation.faction),
                "formations.faction",
                "unknown faction",
            )?;
            require(
                formation.capacity == definition.capacity
                    && formation.headcount > 0
                    && formation.headcount <= formation.capacity,
                "formations.headcount",
                "invalid capacity or surviving headcount",
            )?;
            require(
                formation.movement_spent <= movement_allowance,
                "formations.movement_spent",
                "exceeds seasonal allowance",
            )?;
            require(
                formation.created_round <= self.completed_rounds,
                "formations.created_round",
                "future creation date",
            )?;
            if let Some(tactics) = &formation.tactics {
                data.battle_tactics
                    .validate_configuration(formation.kind, tactics)
                    .map_err(|error| format!("formations.tactics: {error}"))?;
            }
            require(
                formation.tactics_override != Some(true) || formation.tactics.is_some(),
                "formations.tactics_override",
                "an explicit tactics override needs saved rules",
            )?;
            if let Some(leader) = formation.battle_leader {
                require(
                    self.people
                        .get(&leader)
                        .is_some_and(|person| person.faction == formation.faction),
                    "formations.battle_leader",
                    "unknown or foreign person",
                )?;
            }
        }
        self.validate_people(data)?;
        self.validate_economy_statements(data)?;
        self.validate_recovery_statements(data)?;
        require(
            self.world
                .site_damage
                .iter()
                .all(|(site, damage)| self.world.site(*site).is_some() && *damage <= 100),
            "world.site_damage",
            "unknown physical site or damage above 100",
        )
    }

    fn validate_economy_statements(&self, data: &GameData) -> Result<(), String> {
        for faction in self.factions.values() {
            let Some(statement) = &faction.last_economy else {
                continue;
            };
            self.validate_income_inputs(statement, data)?;
            statement
                .income
                .validate("campaign", "factions.last_economy.income")?;
            statement
                .closing
                .validate("campaign", "factions.last_economy.closing")?;
            require(
                statement.completed_rounds > 0
                    && statement.completed_rounds <= self.completed_rounds
                    && statement.upkeep_due >= 0
                    && statement.upkeep_paid >= 0
                    && statement.shortfall >= 0
                    && faction.deficit == (statement.shortfall > 0)
                    && statement.upkeep_paid.checked_add(statement.shortfall)
                        == Some(statement.upkeep_due),
                "factions.last_economy",
                "invalid date, upkeep arithmetic or deficit flag",
            )?;
        }
        Ok(())
    }

    fn validate_recovery_statements(&self, data: &GameData) -> Result<(), String> {
        for faction in self.factions.values() {
            let Some(statement) = &faction.last_recovery else {
                continue;
            };
            require(
                statement.completed_rounds > 0
                    && statement.completed_rounds <= self.completed_rounds
                    && statement.opening_gold >= 0
                    && statement.closing_gold >= 0
                    && statement.gold_spent >= 0
                    && statement.closing_gold.checked_add(statement.gold_spent)
                        == Some(statement.opening_gold)
                    && faction.last_economy.as_ref().is_some_and(|economy| {
                        economy.completed_rounds == statement.completed_rounds
                            && economy.closing.gold == statement.opening_gold
                            && (economy.shortfall == 0 || statement.entries.is_empty())
                    }),
                "factions.last_recovery",
                "invalid date, balance or post-upkeep eligibility",
            )?;
            let mut restored = 0_u64;
            let mut gold = 0_i64;
            let mut formations = BTreeSet::new();
            let mut last_army = 0;
            for entry in &statement.entries {
                self.validate_recovery_entry(entry, data)?;
                require(
                    formations.insert(entry.formation) && entry.army.0 >= last_army,
                    "factions.last_recovery.entries",
                    "duplicate formation or unordered army",
                )?;
                last_army = entry.army.0;
                restored = restored
                    .checked_add(u64::from(entry.restored))
                    .ok_or("campaign.factions.last_recovery: restored headcount overflow")?;
                gold = gold
                    .checked_add(entry.gold_cost)
                    .ok_or("campaign.factions.last_recovery: Gold total overflow")?;
            }
            require(
                restored == statement.restored && gold == statement.gold_spent,
                "factions.last_recovery",
                "entry totals do not match summary",
            )?;
        }
        Ok(())
    }

    fn validate_recovery_entry(
        &self,
        entry: &super::RecoveryEntry,
        data: &GameData,
    ) -> Result<(), String> {
        let definition = &data.economy.formations[&entry.kind];
        let cap = u64::from(entry.capacity)
            * u64::from(data.economy.recovery.capacity_percent_per_round)
            / 100;
        require(
            entry.army.0 > 0
                && entry.army < self.next_ids.army
                && entry.formation.0 > 0
                && entry.formation < self.next_ids.formation
                && self.world.site(entry.site).is_some()
                && entry.capacity == definition.capacity
                && entry.capacity > 0
                && entry.headcount_before > 0
                && entry.restored > 0
                && u64::from(entry.restored) <= cap
                && entry
                    .headcount_before
                    .checked_add(entry.restored)
                    .is_some_and(|after| after <= entry.capacity),
            "factions.last_recovery.entries",
            "invalid historical identity or headcount",
        )?;
        let numerator = i128::from(definition.recruit_cost.gold)
            * i128::from(data.economy.recovery.full_replacement_recruit_gold_percent)
            * i128::from(entry.restored);
        let denominator = 100 * i128::from(entry.capacity);
        require(
            i128::from(entry.gold_cost) == (numerator + denominator - 1) / denominator,
            "factions.last_recovery.entries",
            "incorrect recovery Gold cost",
        )
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.trim() == name
        && name.chars().count() <= 64
        && !name.chars().any(char::is_control)
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.{field}: {reason}"))
    }
}
