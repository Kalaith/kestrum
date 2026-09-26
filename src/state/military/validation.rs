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
                            && person.age_years(self.completed_rounds)
                                >= data.rules.leadership.field_min_age_years
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
                formation.movement_spent <= definition.movement_allowance,
                "formations.movement_spent",
                "exceeds seasonal allowance",
            )?;
            require(
                formation.created_round <= self.completed_rounds,
                "formations.created_round",
                "future creation date",
            )?;
        }
        self.validate_people(data)?;
        self.validate_economy_statements()?;
        require(
            self.world
                .site_damage
                .iter()
                .all(|(site, damage)| self.world.site(*site).is_some() && *damage <= 100),
            "world.site_damage",
            "unknown physical site or damage above 100",
        )
    }

    fn validate_people(&self, data: &GameData) -> Result<(), String> {
        for (id, person) in &self.people {
            require(
                *id == person.id && id.0 > 0,
                "people.id",
                "invalid identity",
            )?;
            require(
                self.factions.contains_key(&person.faction),
                "people.faction",
                "unknown faction",
            )?;
            require(
                valid_name(&person.name),
                "people.name",
                "invalid person name",
            )?;
            let age = i64::from(self.completed_rounds).checked_sub(person.birth_round);
            require(
                age.is_some_and(|age| (0..=i64::from(u32::MAX) * 4).contains(&age))
                    && person.service_start_round <= self.completed_rounds
                    && person.birth_round <= i64::from(person.service_start_round),
                "people.birth_round/service_start_round",
                "invalid birth or service date",
            )?;
            require(
                person.movement_spent <= data.rules.leadership.officer_movement_allowance,
                "people.movement_spent",
                "exceeds seasonal allowance",
            )?;
            let valid_assignment = match person.assignment {
                PersonAssignment::Formation { formation } => self
                    .formations
                    .get(&formation)
                    .is_some_and(|entry| entry.faction == person.faction),
                PersonAssignment::Site { site } => self.world.site(site).is_some(),
            };
            require(
                valid_assignment,
                "people.assignment",
                "unknown or foreign assignment",
            )?;
        }
        Ok(())
    }

    fn validate_economy_statements(&self) -> Result<(), String> {
        for faction in self.factions.values() {
            let Some(statement) = &faction.last_economy else {
                continue;
            };
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
