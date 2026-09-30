//! Evidence is compact gameplay state; it never depends on retained prose or reports.

use super::*;
use crate::{
    data::GameData,
    state::{campaign::DomainFactKind, StrategicCampaign},
};

impl StrategicCampaign {
    pub(crate) fn validate_evidence(&self, data: &GameData) -> Result<(), String> {
        for formation in self.formations.values() {
            let service = &formation.service;
            if let Some(specialization) = service.specialization {
                let rule = data.progression.specializations.get(&specialization);
                if rule.is_none_or(|rule| !rule.sources.contains(&formation.kind))
                    || service.course.is_some()
                {
                    return Err("formation.service: incompatible specialization".into());
                }
            }
            if let Some(course) = &service.course {
                let valid = data
                    .progression
                    .specializations
                    .get(&course.target)
                    .is_some_and(|rule| {
                        rule.sources.contains(&formation.kind)
                            && course.steps_completed < rule.course_steps
                    })
                    && course.paid_gold >= 0
                    && service.specialization.is_none()
                    && self.world.site(course.site).is_some();
                if !valid {
                    return Err("formation.service: invalid specialization course".into());
                }
            }
            if service.tier != Veterancy::from_xp(service.xp, &data.progression) {
                return Err("formation.service: tier disagrees with earned XP".into());
            }
            self.validate_ledger(&service.ledger)?;
            if !service
                .recent
                .windows(2)
                .all(|pair| pair[0].completed_rounds < pair[1].completed_rounds)
                || service.recent.len() > data.history.recent_service_rounds as usize
            {
                return Err("formation.service: invalid recent season order or count".into());
            }
            let mut recent_xp = 0_u64;
            for season in &service.recent {
                self.validate_season(season, formation.faction, data)?;
                recent_xp += u64::from(season.xp);
            }
            if recent_xp > u64::from(service.xp) {
                return Err("formation.service: recent XP exceeds total".into());
            }
        }
        for person in self.people.values() {
            self.validate_ledger(&person.evidence)?;
        }
        for fact in &self.pending_facts {
            self.validate_movement_fact(&fact.kind)?;
        }
        Ok(())
    }

    fn validate_ledger(&self, ledger: &EvidenceLedger) -> Result<(), String> {
        let meaningful = ledger
            .counts
            .get(&EvidenceKind::MeaningfulEncounter)
            .copied()
            .unwrap_or(0);
        let battles = ledger
            .counts
            .get(&EvidenceKind::Battle)
            .copied()
            .unwrap_or(0);
        let valid = meaningful <= battles
            && ledger.counts.values().all(|count| *count > 0)
            && ledger.counts.iter().all(|(kind, count)| {
                matches!(kind, EvidenceKind::Battle | EvidenceKind::TreatedWounded)
                    || *count <= meaningful
            })
            && ledger.meaningful_against.iter().all(|(kind, count)| {
                *count > 0 && *count <= meaningful && ledger.encountered_troops.contains(kind)
            })
            && ledger
                .service_by_troop
                .values()
                .all(|count| *count > 0 && *count <= meaningful)
            && ledger
                .service_by_troop
                .values()
                .map(|count| u64::from(*count))
                .sum::<u64>()
                == u64::from(meaningful)
            && ledger
                .traversed_routes
                .iter()
                .all(|id| self.world.route(*id).is_some());
        if valid {
            Ok(())
        } else {
            Err("evidence: inconsistent counters or unknown physical route".into())
        }
    }

    fn validate_season(
        &self,
        season: &SeasonService,
        owner: FactionId,
        data: &GameData,
    ) -> Result<(), String> {
        let valid = (season.completed_rounds < self.completed_rounds
            || (self.diplomacy.ending.is_some()
                && season.completed_rounds == self.completed_rounds))
            && season.completed_rounds
                >= self
                    .completed_rounds
                    .saturating_sub(data.history.recent_service_rounds)
            && season.xp <= data.progression.round_xp_cap
            && (!season.encounters.is_empty() || !season.routes.is_empty())
            && season
                .encounters
                .windows(2)
                .all(|pair| (pair[0].site, pair[0].opponent) < (pair[1].site, pair[1].opponent))
            && season
                .encounters
                .iter()
                .map(|entry| u64::from(entry.xp))
                .sum::<u64>()
                == u64::from(season.xp)
            && season.encounters.iter().all(|entry| {
                self.world.site(entry.site).is_some()
                    && match entry.opponent {
                        EncounterOpponent::Faction(faction) => {
                            faction != owner && self.factions.contains_key(&faction)
                        }
                        EncounterOpponent::Threat { threat } => {
                            threat.0 > 0
                                && threat < self.next_ids.threat
                                && entry.enemy_types.is_empty()
                        }
                    }
                    && entry.tags.contains(&EvidenceKind::Battle)
                    && entry.meaningful == entry.tags.contains(&EvidenceKind::MeaningfulEncounter)
                    && (entry.meaningful || entry.xp == 0)
                    && entry.xp <= data.progression.round_xp_cap
            })
            && season
                .routes
                .iter()
                .all(|route| self.world.route(*route).is_some());
        if valid {
            Ok(())
        } else {
            Err("formation.service: invalid seasonal receipt".into())
        }
    }

    fn validate_movement_fact(&self, fact: &DomainFactKind) -> Result<(), String> {
        let (receipt, faction) = match fact {
            DomainFactKind::ArmiesMoved {
                faction,
                path,
                movement: Some(receipt),
                ..
            } => {
                let routes = path
                    .windows(2)
                    .filter_map(|pair| self.world.connected_route(pair[0], pair[1]))
                    .map(|route| route.id)
                    .collect::<BTreeSet<_>>();
                if routes.iter().copied().collect::<Vec<_>>() != receipt.routes {
                    return Err(
                        "movement evidence: receipt route differs from accepted path".into(),
                    );
                }
                (receipt, *faction)
            }
            DomainFactKind::BattleResolved {
                battle,
                movement: Some(receipt),
            } => {
                let report = self
                    .battles
                    .get(battle)
                    .ok_or("movement evidence: missing battle")?;
                let formations = report
                    .attacker
                    .armies
                    .iter()
                    .filter(|army| travelled_in_report(report, army.id))
                    .flat_map(|army| &army.formations)
                    .map(|entry| entry.id)
                    .collect::<BTreeSet<_>>();
                let people = report
                    .attacker
                    .armies
                    .iter()
                    .filter(|army| travelled_in_report(report, army.id))
                    .flat_map(|army| &army.people)
                    .map(|entry| entry.id)
                    .collect::<BTreeSet<_>>();
                // Attached travellers outside field service remain in movement receipts.
                let travellers = receipt.people.iter().copied().collect::<BTreeSet<_>>();
                if formations.iter().copied().collect::<Vec<_>>() != receipt.formations
                    || !people.is_subset(&travellers)
                {
                    return Err(format!(
                        "movement evidence: battle participants are absent from the movement receipt (battle formations {formations:?}, receipt formations {:?}, battle people {people:?}, receipt people {:?})",
                        receipt.formations, receipt.people
                    ));
                }
                (receipt, report.attacker.faction)
            }
            _ => return Ok(()),
        };
        self.validate_movement_service(receipt)?;
        if receipt.formations.iter().any(|id| {
            self.formations
                .get(id)
                .is_some_and(|member| member.faction != faction)
        }) || receipt.people.iter().any(|id| {
            self.people
                .get(id)
                .is_some_and(|person| person.faction != faction)
        }) {
            return Err("movement evidence: foreign members in accepted group".into());
        }
        Ok(())
    }

    fn validate_movement_service(&self, receipt: &MovementService) -> Result<(), String> {
        let valid = !receipt.formations.is_empty()
            && !receipt.routes.is_empty()
            && receipt.formations.windows(2).all(|pair| pair[0] < pair[1])
            && receipt.people.windows(2).all(|pair| pair[0] < pair[1])
            && receipt.routes.windows(2).all(|pair| pair[0] < pair[1])
            && receipt
                .formations
                .iter()
                .all(|id| id.0 > 0 && *id < self.next_ids.formation)
            && receipt
                .people
                .iter()
                .all(|id| id.0 > 0 && *id < self.next_ids.person)
            && receipt
                .routes
                .iter()
                .all(|id| self.world.route(*id).is_some());
        if valid {
            Ok(())
        } else {
            Err("pending movement evidence: invalid member or route snapshot".into())
        }
    }
}

fn travelled_in_report(
    report: &crate::state::battle::BattleReport,
    army: crate::state::military::ArmyId,
) -> bool {
    match &report.context {
        crate::state::battle::BattleContext::Relief { garrison, .. } => !garrison.contains(&army),
        _ => true,
    }
}
