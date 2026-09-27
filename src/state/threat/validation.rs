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
                || !origins.insert((threat.site, threat.ruination))
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
            } else if !data
                .threats
                .initial
                .iter()
                .any(|initial| initial.site == threat.site && initial.kind == threat.kind)
                || (development.ruination == 1 && development.ruined && !development.threat_created)
            {
                return Err(invalid());
            }
            match threat.status {
                ThreatStatus::Active => {
                    if threat.headcount == 0
                        || !active.insert(threat.site)
                        || self.sieges.contains_key(&threat.site)
                        || self.armies.values().any(|army| army.site == threat.site)
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
