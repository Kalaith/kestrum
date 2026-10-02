//! Political state loads strictly, while dated receipts reference historical identities.

use super::*;
use crate::{data::GameData, state::campaign::FactionStatus};
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub(crate) fn validate_diplomacy(&self, data: &GameData) -> Result<(), String> {
        data.diplomacy.validate()?;
        data.diplomacy.validate_economy(&data.economy)?;
        require(
            self.diplomacy.pairs.len() == self.relations.len(),
            "missing relation timers",
        )?;
        for (pair, relation) in self.diplomacy.pairs.iter().zip(&self.relations) {
            require(
                pair.factions == relation.factions,
                "unordered or unknown timer pair",
            )?;
            require(
                pair.peace_since.is_some() == (relation.state == DiplomaticState::Peace)
                    && pair
                        .peace_since
                        .is_none_or(|round| round <= self.completed_rounds)
                    && pair
                        .last_offer_round
                        .is_none_or(|round| round <= self.completed_rounds)
                    && pair
                        .war_started_round
                        .is_none_or(|round| round <= self.completed_rounds)
                    && pair
                        .war_ended_round
                        .is_none_or(|round| round <= self.completed_rounds)
                    && pair.war_ended_round.is_none_or(|end| {
                        pair.war_started_round.is_none_or(|start| {
                            start <= end || relation.state == DiplomaticState::War
                        })
                    }),
                "invalid peace/offer dates",
            )?;
            require(
                pair.truce_until.is_none_or(|until| {
                    pair.peace_since.is_some_and(|since| {
                        since.checked_add(data.diplomacy.truce_rounds) == Some(until)
                            && pair.last_offer_round == Some(since)
                    })
                }),
                "invalid truce duration",
            )?;
        }
        self.validate_decisions()?;
        self.validate_losses(data)?;
        self.validate_ending()?;
        for (victim, attacker) in &self.diplomacy.last_attackers {
            require(
                self.valid_pair(*victim, *attacker),
                "invalid lasting defeat attribution",
            )?;
        }
        for faction in self
            .factions
            .values()
            .filter(|faction| faction.status != FactionStatus::Independent)
        {
            require(
                !self.armies.values().any(|army| army.faction == faction.id)
                    && !self
                        .formations
                        .values()
                        .any(|formation| formation.faction == faction.id)
                    && !self
                        .construction
                        .values()
                        .any(|order| order.owner == faction.id && order.is_open()),
                "inactive faction retains independent forces or work",
            )?;
        }
        Ok(())
    }

    fn validate_decisions(&self) -> Result<(), String> {
        let mut proposers = BTreeSet::new();
        for offer in &self.diplomacy.pending_offers {
            require(
                offer.recipient == self.player
                    && self.valid_pair(offer.proposer, offer.recipient)
                    && self.is_independent(offer.proposer)
                    && self.is_independent(offer.recipient)
                    && offer.completed_rounds == self.completed_rounds
                    && proposers.insert(offer.proposer)
                    && self
                        .diplomacy
                        .pair(offer.proposer, offer.recipient)
                        .is_some_and(|pair| pair.last_offer_round == Some(offer.completed_rounds))
                    && self.relations.iter().any(|pair| {
                        pair.factions == ordered_pair(offer.proposer, offer.recipient)
                            && pair.state == DiplomaticState::War
                    }),
                "invalid pending peace offer",
            )?;
        }
        let mut previous = None;
        for pending in &self.diplomacy.pending_defeats {
            require(
                pending.victor == self.player
                    && self.valid_pair(pending.faction, pending.victor)
                    && self.factions[&pending.faction].status == FactionStatus::Independent
                    && pending.completed_rounds == self.completed_rounds
                    && self.defeat_eligible(pending.faction)
                    && previous.is_none_or(|id| id < pending.faction),
                "invalid pending defeated faction",
            )?;
            previous = Some(pending.faction);
        }
        Ok(())
    }
    fn validate_losses(&self, data: &GameData) -> Result<(), String> {
        let mut previous = None;
        for loss in &self.diplomacy.losses {
            let key = (loss.completed_rounds, loss.faction, loss.victor, loss.site);
            require(
                self.valid_pair(loss.faction, loss.victor)
                    && self.world.site(loss.site).is_some()
                    && loss.completed_rounds <= self.completed_rounds
                    && self.completed_rounds - loss.completed_rounds
                        < data.diplomacy.recent_loss_rounds
                    && previous.is_none_or(|old| old < key),
                "invalid durable site-loss record",
            )?;
            previous = Some(key);
        }
        Ok(())
    }
    fn validate_ending(&self) -> Result<(), String> {
        let Some(ending) = &self.diplomacy.ending else {
            return Ok(());
        };
        require(
            !self.diplomacy.has_pending_decision()
                && ending.completed_rounds == self.completed_rounds,
            "terminal campaign has pending choice or advanced calendar",
        )?;
        let valid = match ending.kind {
            EndingKind::Defeat => {
                self.factions[&self.player].status == FactionStatus::Eliminated
                    && self.defeat_eligible(self.player)
            }
            EndingKind::Victory => {
                if self.observer_mode {
                    self.factions
                        .keys()
                        .filter(|faction| self.is_independent(**faction))
                        .count()
                        <= 1
                } else {
                    self.is_independent(self.player)
                        && self
                            .factions
                            .values()
                            .filter(|faction| faction.id != self.player)
                            .all(|faction| {
                                faction.status == FactionStatus::Eliminated
                                    || faction.status
                                        == FactionStatus::Vassal {
                                            sovereign: self.player,
                                        }
                            })
                }
            }
        };
        require(valid, "terminal outcome contradicts faction status")
    }
    fn valid_pair(&self, a: FactionId, b: FactionId) -> bool {
        a != b && self.factions.contains_key(&a) && self.factions.contains_key(&b)
    }

    pub(crate) fn validate_diplomacy_receipt(
        &self,
        receipt: &DiplomacyReceipt,
        recorded_round: u32,
    ) -> Result<(), String> {
        let valid = match receipt {
            DiplomacyReceipt::WarDeclared { factions } => {
                factions[0] < factions[1] && self.valid_pair(factions[0], factions[1])
            }
            DiplomacyReceipt::PeaceAgreed {
                factions,
                truce_until,
            } => {
                factions[0] < factions[1]
                    && self.valid_pair(factions[0], factions[1])
                    && *truce_until > recorded_round
            }
            DiplomacyReceipt::PeaceOffered {
                proposer,
                recipient,
            }
            | DiplomacyReceipt::PeaceRejected {
                proposer,
                recipient,
            } => self.valid_pair(*proposer, *recipient),
            DiplomacyReceipt::ArmyWithdrawn {
                faction,
                army,
                from,
                to,
            } => {
                self.factions.contains_key(faction)
                    && army.0 > 0
                    && *army < self.next_ids.army
                    && self.world.connected_route(*from, *to).is_some()
            }
            DiplomacyReceipt::DefeatPending { faction, victor } => {
                self.valid_pair(*faction, *victor)
            }
            DiplomacyReceipt::FactionResolved {
                faction,
                victor,
                resolution,
            } => {
                self.factions.contains_key(faction)
                    && victor.is_none_or(|victor| self.valid_pair(*faction, victor))
                    && (*resolution == DefeatResolution::Annex || victor.is_some())
            }
            DiplomacyReceipt::CampaignEnded { ending } => {
                ending.completed_rounds == recorded_round
                    && self.diplomacy.ending.as_ref() == Some(ending)
            }
        };
        require(valid, "invalid historical diplomatic receipt")
    }
}
fn require(valid: bool, message: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.diplomacy: {message}"))
    }
}
