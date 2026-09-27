//! Historical identity validation does not consult mutable enemy details.

use super::*;
use crate::state::StrategicCampaign;

impl StrategicCampaign {
    pub(crate) fn validate_knowledge(&self) -> Result<(), String> {
        for (observer, known) in &self.knowledge.observers {
            ensure(self.factions.contains_key(observer), "unknown observer")?;
            for (id, person) in &known.people {
                ensure(
                    *id == person.id && id.0 > 0 && id.0 < self.next_ids.person.0,
                    "invalid person identity",
                )?;
                ensure(
                    person.army.0 > 0 && person.army.0 < self.next_ids.army.0,
                    "invalid encountered army",
                )?;
                ensure(
                    person.battle.0 > 0 && person.battle.0 < self.next_ids.battle.0,
                    "invalid source battle",
                )?;
                ensure(
                    person.completed_rounds <= self.completed_rounds
                        && self.world.site(person.site).is_some(),
                    "invalid date or location",
                )?;
                ensure(
                    [&person.name, &person.site_name, &person.army_name]
                        .into_iter()
                        .all(|name| !name.trim().is_empty()),
                    "invalid witnessed label",
                )?;
                if let Some(report) = self.battles.get(&person.battle) {
                    let enemy = if report.attacker.faction == *observer {
                        &report.defender
                    } else if report.defender.faction == *observer {
                        &report.attacker
                    } else {
                        return Err(
                            "knowledge: observer did not participate in the source battle".into(),
                        );
                    };
                    ensure(
                        encounter_people(report, enemy)
                            .iter()
                            .any(|entry| entry == person),
                        "snapshot contradicts its retained encounter",
                    )?;
                }
            }
        }
        Ok(())
    }
}

fn ensure(valid: bool, message: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("knowledge: {message}"))
    }
}
