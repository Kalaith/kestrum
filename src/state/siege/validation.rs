//! Live partitions are strong references; dated siege facts keep only valid old IDs.

use super::{Siege, SiegeChange, StrategicCampaign};
use crate::data::world::{DiplomaticState, MilitaryLayer};
use crate::state::military::ArmyId;
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub(crate) fn validate_sieges(&self) -> Result<(), String> {
        ensure(self.next_ids.siege.0 > 0, "zero siege counter")?;
        for (site, damage) in &self.world.fort_damage {
            ensure(
                *damage <= 100
                    && self
                        .world
                        .site(*site)
                        .is_some_and(|site| site.military == MilitaryLayer::Fort),
                "invalid persistent fort damage",
            )?;
        }
        let mut identities = BTreeSet::new();
        let mut participants = BTreeSet::new();
        for (site, siege) in &self.sieges {
            self.validate_siege_identity(siege, self.completed_rounds)?;
            ensure(
                self.is_independent(siege.defender) && self.is_independent(siege.besieger),
                "inactive siege participants require military cleanup",
            )?;
            ensure(
                *site == siege.site && identities.insert(siege.id),
                "duplicate or mismatched siege identity",
            )?;
            ensure(
                self.world.site(*site).is_some_and(|entry| {
                    entry.military == MilitaryLayer::Fort
                        && entry.controller == Some(siege.defender)
                }) && self.world.contested_sites.contains(site),
                "siege must contest its defender's controlled Fort",
            )?;
            ensure(
                self.relations.iter().any(|relation| {
                    relation.state == DiplomaticState::War
                        && relation.factions.contains(&siege.defender)
                        && relation.factions.contains(&siege.besieger)
                }),
                "siege participants must be hostile",
            )?;
            ensure(
                !siege.defending.is_empty()
                    && !siege.besieging.is_empty()
                    && siege
                        .last_progress_round
                        .is_none_or(|round| round < self.completed_rounds),
                "empty side or progress beyond the completed boundary",
            )?;
            for (faction, armies) in [
                (siege.defender, &siege.defending),
                (siege.besieger, &siege.besieging),
            ] {
                for id in armies {
                    ensure(
                        participants.insert(*id)
                            && self.armies.get(id).is_some_and(|army| {
                                army.faction == faction && army.site == *site && !army.is_empty()
                            }),
                        "missing, duplicated or misplaced siege army",
                    )?;
                }
            }
            let actual: BTreeSet<_> = self
                .armies
                .values()
                .filter(|army| army.site == *site)
                .map(|army| army.id)
                .collect();
            let partition: BTreeSet<_> = siege
                .defending
                .iter()
                .chain(&siege.besieging)
                .copied()
                .collect();
            ensure(
                actual == partition,
                "siege partition omits a physical occupant",
            )?;
        }
        Ok(())
    }

    pub(crate) fn validate_siege_receipt(
        &self,
        siege: &Siege,
        change: SiegeChange,
        recorded_round: u32,
    ) -> Result<(), String> {
        self.validate_siege_identity(siege, recorded_round)?;
        ensure(
            change == SiegeChange::Lifted
                || (!siege.defending.is_empty() && !siege.besieging.is_empty()),
            "nonterminal receipt has an empty side",
        )?;
        match change {
            SiegeChange::Established => ensure(
                siege.elapsed_steps == 0 && siege.started_round == recorded_round,
                "invalid establishment date/progress",
            ),
            SiegeChange::Progressed => ensure(
                siege.elapsed_steps > 0 && siege.last_progress_round == Some(recorded_round),
                "invalid seasonal progress receipt",
            ),
            SiegeChange::Reinforced | SiegeChange::Lifted => Ok(()),
        }
    }

    fn validate_siege_identity(&self, siege: &Siege, recorded_round: u32) -> Result<(), String> {
        ensure(
            siege.id.0 > 0
                && siege.id < self.next_ids.siege
                && self.world.site(siege.site).is_some()
                && siege.defender != siege.besieger
                && self.factions.contains_key(&siege.defender)
                && self.factions.contains_key(&siege.besieger)
                && siege.started_round <= recorded_round,
            "invalid historical siege identity, factions or date",
        )?;
        ensure(
            (siege.elapsed_steps == 0) == siege.last_progress_round.is_none()
                && siege.last_progress_round.is_none_or(|round| {
                    round >= siege.started_round
                        && round <= recorded_round
                        && u64::from(siege.elapsed_steps)
                            <= u64::from(round) - u64::from(siege.started_round) + 1
                }),
            "invalid elapsed siege progress",
        )?;
        let valid_ids = |ids: &[ArmyId]| {
            ids.windows(2).all(|pair| pair[0] < pair[1])
                && ids.iter().all(|id| id.0 > 0 && *id < self.next_ids.army)
        };
        ensure(
            valid_ids(&siege.defending)
                && valid_ids(&siege.besieging)
                && !siege
                    .defending
                    .iter()
                    .any(|id| siege.besieging.contains(id)),
            "invalid historical siege partition",
        )
    }
}

fn ensure(valid: bool, message: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.sieges: {message}"))
    }
}
