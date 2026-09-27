//! Current civilian values are strict; dated receipts retain weak public identities.

use super::*;

impl StrategicCampaign {
    pub(crate) fn validate_development(&self, data: &GameData) -> Result<(), String> {
        let rules = &data.development;
        ensure(
            self.world.development.len() == self.world.sites.len(),
            "missing development site",
        )?;
        for (id, state) in &self.world.development {
            ensure(self.world.site(*id).is_some(), "unknown development site")?;
            ensure(
                (rules.pressure.minimum..=rules.pressure.maximum).contains(&state.pressure),
                "pressure outside bounds",
            )?;
            ensure(
                state.displaced <= self.world.population[id],
                "displaced population exceeds inhabitants",
            )?;
            ensure(
                state.ruin_streak <= rules.conditions.ruin_steps
                    && state.lawless_rounds <= data.threats.lawless_rounds,
                "invalid condition duration",
            )?;
            ensure(
                state.ruined == state.ruined_round.is_some()
                    && state
                        .ruined_round
                        .is_none_or(|round| round <= self.completed_rounds)
                    && (!state.ruined || state.ruination > 0),
                "invalid current ruin transition",
            )?;
            ensure(
                state.ruined || state.lawless_rounds == 0,
                "inhabited state cannot accumulate lawless ruin time",
            )?;
            ensure(
                !state.threat_created || state.ruination > 0,
                "threat flag has no ruination identity",
            )?;
            ensure(
                state
                    .last_resolved_round
                    .is_none_or(|round| round < self.completed_rounds),
                "development advanced beyond completed boundary",
            )?;
        }
        for faction in self.factions.values() {
            ensure(
                faction
                    .last_hq_relocation
                    .is_none_or(|round| round <= self.completed_rounds),
                "HQ relocation is in the future",
            )?;
        }
        Ok(())
    }

    pub(crate) fn validate_development_receipt(
        &self,
        receipt: &DevelopmentReceipt,
    ) -> Result<(), String> {
        let site = |id: &SiteId| self.world.site(*id).is_some();
        let owner = |id: &FactionId| self.factions.contains_key(id);
        let valid = match receipt {
            DevelopmentReceipt::HabitationChanged {
                site: id,
                owner: faction,
                from,
                to,
            } => site(id) && faction.as_ref().is_none_or(owner) && from != to,
            DevelopmentReceipt::Ruined {
                site: id,
                owner: faction,
                ruination,
            } => {
                site(id)
                    && faction.as_ref().is_none_or(owner)
                    && *ruination > 0
                    && self
                        .world
                        .development
                        .get(id)
                        .is_some_and(|state| state.ruination >= *ruination)
            }
            DevelopmentReceipt::PopulationMoved {
                owner: faction,
                from,
                to,
                amount,
                ..
            } => owner(faction) && site(from) && site(to) && from != to && *amount > 0,
            DevelopmentReceipt::SiteRenamed {
                owner: faction,
                site: id,
                old_name,
                new_name,
            } => {
                owner(faction)
                    && site(id)
                    && valid_name(old_name)
                    && valid_name(new_name)
                    && old_name != new_name
            }
            DevelopmentReceipt::CapitalMoved {
                owner: faction,
                from,
                to,
            }
            | DevelopmentReceipt::HeadquartersMoved {
                owner: faction,
                from,
                to,
            } => owner(faction) && site(from) && site(to) && from != to,
        };
        ensure(valid, "invalid development receipt")
    }
}

pub(crate) fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.trim() == name
        && name.chars().count() <= 40
        && !name.chars().any(char::is_control)
}

fn ensure(valid: bool, message: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.development: {message}"))
    }
}
