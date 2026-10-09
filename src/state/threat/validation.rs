use super::*;
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub(crate) fn validate_threats(&self, data: &GameData) -> Result<(), String> {
        let invalid = || {
            "campaign.threats: invalid identity, occupant, transition or cleared reward".to_owned()
        };
        if self.next_ids.threat.0 == 0 || !consistent_receipts(self) {
            return Err(invalid());
        }
        if self.scenario_kind == crate::data::world::ScenarioKind::Production
            && self.initial_threats.len() != self.factions.len() * 2
        {
            return Err(invalid());
        }
        if self.initial_threats.iter().any(|(site, kind)| {
            self.world.site(*site).is_none() || !data.threats.definitions.contains_key(kind)
        }) {
            return Err(invalid());
        }
        let mut active = BTreeSet::new();
        let mut origins = BTreeSet::new();
        let mut spawned_sites = BTreeSet::new();
        for (id, threat) in &self.threats {
            let definition = data
                .threats
                .definitions
                .get(&threat.kind)
                .ok_or_else(invalid)?;
            if id != &threat.id
                || id.0 == 0
                || *id >= self.next_ids.threat
                || self.world.site(threat.site).is_none()
                || threat.name != definition.name
                || threat.headcount > definition.headcount
                || threat.ruination == Some(0)
                || (threat.ruination.is_some() && threat.raid.is_some())
                || !origins.insert((threat.site, threat.ruination, threat.raid))
            {
                return Err(invalid());
            }
            let development = self
                .world
                .development
                .get(&threat.site)
                .ok_or_else(invalid)?;
            if let Some(transition) = threat.ruination {
                if !spawned_sites.insert(threat.site)
                    || threat.kind != crate::data::threats::ThreatKind::Bandits
                    || transition > development.ruination
                    || (transition == development.ruination && !development.threat_created)
                    || (threat.status == ThreatStatus::Active
                        && (!development.ruined || transition != development.ruination))
                {
                    return Err(invalid());
                }
            } else if let Some(round) = threat.raid {
                if threat.kind != crate::data::threats::ThreatKind::Bandits
                    || round > self.completed_rounds
                {
                    return Err(invalid());
                }
            } else if (self.initial_threats.get(&threat.site) != Some(&threat.kind)
                && !(self.scenario_kind == crate::data::world::ScenarioKind::RosemarchPrototype
                    && data
                        .threats
                        .initial
                        .iter()
                        .any(|initial| initial.site == threat.site && initial.kind == threat.kind)))
                || (development.ruination == 1 && development.ruined && !development.threat_created)
            {
                return Err(invalid());
            }
            match threat.status {
                ThreatStatus::Active => {
                    let pending_participation = self.pending_battle.as_ref().is_some_and(|pending| {
                        pending.report.site == threat.site
                            && matches!(&pending.report.defender,
                                crate::state::battle::BattleDefender::Threat(side) if side.id == threat.id)
                            && self.armies.values().filter(|army| army.site == threat.site)
                                .all(|army| pending.report.attacker.armies.iter().any(|entry| entry.id == army.id))
                    });
                    if threat.headcount == 0
                        || !active.insert(threat.site)
                        || self.sieges.contains_key(&threat.site)
                        || (self.armies.values().any(|army| army.site == threat.site)
                            && !pending_participation)
                    {
                        return Err(invalid());
                    }
                }
                ThreatStatus::Cleared { round, by, payout } => {
                    if threat.headcount != 0
                        || round > self.completed_rounds
                        || !self.factions.contains_key(&by)
                        || (payout != definition.reward
                            && payout
                                != Resources {
                                    gold: 0,
                                    wood: 0,
                                    stone: 0,
                                })
                    {
                        return Err(invalid());
                    }
                }
            }
        }
        Ok(())
    }
}

fn consistent_receipts(campaign: &StrategicCampaign) -> bool {
    use crate::state::battle::{BattleDefender, BattleOutcome};
    campaign.battles.values().all(|report| {
        let BattleDefender::Threat(side) = &report.defender else {
            return true;
        };
        let Some(threat) = campaign.threats.get(&side.id) else {
            return true;
        };
        if side.kind != threat.kind || report.site != threat.site || threat.headcount > side.end {
            return false;
        }
        if matches!(
            report.outcome,
            BattleOutcome::AttackerVictory | BattleOutcome::MutualDestruction
        ) {
            threat.status
                == ThreatStatus::Cleared {
                    round: report.completed_rounds,
                    by: report.attacker.faction,
                    payout: side.payout,
                }
        } else {
            true
        }
    })
}
