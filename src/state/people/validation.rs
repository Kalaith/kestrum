//! Living assignments and dated battle injuries/deaths remain valid after removal.

use super::{Person, PersonAssignment, PersonStatus, StrategicCampaign};
use crate::data::GameData;

impl StrategicCampaign {
    pub(crate) fn validate_people(&self, data: &GameData) -> Result<(), String> {
        for (id, person) in &self.people {
            require(*id == person.id && id.0 > 0, "id", "invalid identity")?;
            require(
                self.factions.contains_key(&person.faction),
                "faction",
                "unknown faction",
            )?;
            require(
                !person.name.is_empty()
                    && person.name.trim() == person.name
                    && person.name.chars().count() <= 64
                    && !person.name.chars().any(char::is_control),
                "name",
                "invalid person name",
            )?;
            let age = i64::from(self.completed_rounds).checked_sub(person.birth_round);
            require(
                age.is_some_and(|age| (0..=i64::from(u32::MAX) * 4).contains(&age))
                    && person.service_start_round <= self.completed_rounds
                    && person.birth_round <= i64::from(person.service_start_round),
                "birth_round/service_start_round",
                "invalid birth or service date",
            )?;
            require(
                person.movement_spent <= data.rules.leadership.officer_movement_allowance,
                "movement_spent",
                "exceeds seasonal allowance",
            )?;
            self.validate_person_status(person, data)?;
            let valid_assignment = match person.assignment {
                PersonAssignment::Formation { formation } => {
                    person.is_alive()
                        && self
                            .formations
                            .get(&formation)
                            .is_some_and(|entry| entry.faction == person.faction)
                }
                PersonAssignment::Site { site } => {
                    person.is_alive() && self.world.site(site).is_some()
                }
                PersonAssignment::Dead => !person.is_alive(),
            };
            require(
                valid_assignment,
                "assignment",
                "unknown, foreign or incompatible assignment",
            )?;
        }
        Ok(())
    }

    fn validate_person_status(&self, person: &Person, data: &GameData) -> Result<(), String> {
        let valid = match person.status {
            PersonStatus::Fit => true,
            PersonStatus::Wounded {
                since_round,
                remaining_steps,
            } => {
                since_round >= person.service_start_round
                    && since_round <= self.completed_rounds
                    && remaining_steps > 0
                    && remaining_steps <= data.combat.wound_recovery_steps
            }
            PersonStatus::Dead {
                completed_rounds,
                site,
            } => {
                completed_rounds >= person.service_start_round
                    && completed_rounds <= self.completed_rounds
                    && self.world.site(site).is_some()
                    && person.movement_spent == 0
            }
        };
        require(valid, "status", "invalid injury progress or death record")
    }
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.people.{field}: {reason}"))
    }
}
