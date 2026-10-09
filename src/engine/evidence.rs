//! Seasonal consumption of actual receipts. Viewing and pruning never call this.

mod participation;
mod treatment;
pub(crate) use treatment::{record_person_treatment, record_recovery, recovery_medics};

use super::{history, RuleError};
use crate::{
    data::{economy::TroopKind, world::SiteId, GameData},
    state::{
        battle::{BattleReport, BattleSideReport},
        campaign::{DomainFact, DomainFactKind},
        evidence::*,
        military::FormationId,
        people::{PersonId, PersonStatus},
        StrategicCampaign,
    },
};
use participation::{classify, Participation};
use std::collections::{BTreeMap, BTreeSet};

type PlaceKey = (SiteId, EncounterOpponent);

#[derive(Default)]
struct RoundCredit {
    formations: BTreeMap<(FormationId, PlaceKey), bool>,
    people: BTreeMap<(PersonId, PlaceKey), bool>,
    seasons: BTreeMap<FormationId, SeasonService>,
}

pub(crate) fn consume(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    facts: &[DomainFact],
) -> Result<(), RuleError> {
    let mut credit = RoundCredit::default();
    let consumed = campaign.consumed_sequence;
    let mut ordered: Vec<_> = facts
        .iter()
        .filter(|fact| fact.sequence > consumed)
        .collect();
    ordered.sort_by_key(|fact| fact.id);
    for fact in ordered {
        if let DomainFactKind::BattleResolved {
            movement: Some(receipt),
            ..
        }
        | DomainFactKind::ArmiesMoved {
            movement: Some(receipt),
            ..
        } = &fact.kind
        {
            record_movement(campaign, receipt, fact.completed_rounds, &mut credit);
        }
        if let DomainFactKind::BattleResolved { battle, .. } = fact.kind {
            let report = campaign
                .battles
                .get(&battle)
                .ok_or_else(|| {
                    RuleError::InvalidState("Pending battle receipt is missing.".into())
                })?
                .clone();
            use crate::state::battle::BattleDefender;
            use participation::Opponent;
            match &report.defender {
                BattleDefender::Faction(defender) => {
                    consume_side(
                        campaign,
                        data,
                        &report,
                        &report.attacker,
                        Opponent::Faction(defender),
                        true,
                        &mut credit,
                    )?;
                    consume_side(
                        campaign,
                        data,
                        &report,
                        defender,
                        Opponent::Faction(&report.attacker),
                        false,
                        &mut credit,
                    )?;
                }
                BattleDefender::Threat(threat) => {
                    consume_side(
                        campaign,
                        data,
                        &report,
                        &report.attacker,
                        Opponent::Threat(threat),
                        true,
                        &mut credit,
                    )?;
                }
            }
        }
    }
    finish_seasons(campaign, data, credit.seasons)
}

fn record_movement(
    campaign: &mut StrategicCampaign,
    receipt: &MovementService,
    round: u32,
    credit: &mut RoundCredit,
) {
    for id in &receipt.formations {
        if let Some(formation) = campaign.formations.get_mut(id) {
            formation
                .service
                .ledger
                .traversed_routes
                .extend(&receipt.routes);
            let season = credit.seasons.entry(*id).or_insert_with(|| SeasonService {
                completed_rounds: round,
                xp: 0,
                encounters: Vec::new(),
                routes: BTreeSet::new(),
            });
            season.routes.extend(&receipt.routes);
        }
    }
    for id in &receipt.people {
        if let Some(person) = campaign.people.get_mut(id) {
            person.evidence.traversed_routes.extend(&receipt.routes);
        }
    }
}

fn consume_side(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
    own: &BattleSideReport,
    enemy: participation::Opponent<'_>,
    attacking: bool,
    credit: &mut RoundCredit,
) -> Result<(), RuleError> {
    for army in &own.armies {
        for formation in &army.formations {
            let participation = classify(
                campaign,
                data,
                participation::BattleContext {
                    report,
                    own,
                    enemy,
                    attacking,
                },
                army,
                formation,
            )?;
            if campaign.formations.contains_key(&formation.id) && formation.end > 0 {
                credit_formation(
                    campaign,
                    data,
                    formation.id,
                    report.completed_rounds,
                    report.sequence,
                    &participation,
                    credit,
                )?;
            }
            for person in army
                .people
                .iter()
                .filter(|person| person.starting_formation == formation.id)
            {
                if !campaign.people.contains_key(&person.id) {
                    continue;
                }
                let mut personal = participation.clone();
                personal.tags.remove(&EvidenceKind::CommanderWounded);
                personal.tags.remove(&EvidenceKind::AssumedCommand);
                personal.tags.remove(&EvidenceKind::CommandedVictory);
                participation::personal_tags(report, army, person, &mut personal);
                let key = (person.id, (report.site, enemy.identity()));
                let first = !credit.people.contains_key(&key);
                let meaningful = (personal.meaningful
                    || personal.tags.contains(&EvidenceKind::TreatedWounded))
                    && !credit.people.get(&key).copied().unwrap_or(false);
                let tracked = campaign.people.get_mut(&person.id).expect("present person");
                record(&mut tracked.evidence, &personal, first, meaningful)?;
                if meaningful {
                    for tag in &personal.tags {
                        if let Some(fact) = epithet_fact(*tag) {
                            tracked
                                .career
                                .notable_sites
                                .entry(fact)
                                .or_insert(report.site);
                        }
                    }
                    if person.starting_status == Some(PersonStatus::Fit)
                        && tracked.is_alive()
                        && !tracked.career.retired
                        && tracked.career.recognition.is_none()
                    {
                        let threshold = data.progression.recognition.personal_engagements;
                        tracked.career.hero_service_progress = tracked
                            .career
                            .hero_service_progress
                            .saturating_add(1)
                            .min(threshold);
                        tracked
                            .career
                            .hero_service_sites
                            .entry(crate::data::progression::EpithetFact::BattleService)
                            .or_insert(report.site);
                        for tag in &personal.tags {
                            if let Some(fact) = epithet_fact(*tag) {
                                if data.progression.recognition.required_facts.contains(&fact) {
                                    tracked
                                        .career
                                        .hero_service_sites
                                        .entry(fact)
                                        .or_insert(report.site);
                                }
                            }
                        }
                    }
                }
                credit
                    .people
                    .entry(key)
                    .and_modify(|done| *done |= meaningful)
                    .or_insert(meaningful);
            }
        }
    }
    Ok(())
}

fn epithet_fact(kind: EvidenceKind) -> Option<crate::data::progression::EpithetFact> {
    use crate::data::progression::EpithetFact;
    Some(match kind {
        EvidenceKind::SurvivedOutnumbered => EpithetFact::SurvivedOutnumbered,
        EvidenceKind::DefendedAnchor => EpithetFact::DefendedAnchor,
        EvidenceKind::CapturedAnchor => EpithetFact::CapturedAnchor,
        EvidenceKind::TreatedWounded => EpithetFact::TreatedWounded,
        EvidenceKind::AssumedCommand => EpithetFact::AssumedCommand,
        EvidenceKind::CommandedVictory => EpithetFact::CommandedVictory,
        _ => return None,
    })
}

fn credit_formation(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: FormationId,
    round: u32,
    sequence: u64,
    participation: &Participation,
    credit: &mut RoundCredit,
) -> Result<(), RuleError> {
    let place = (participation.site, participation.opponent);
    let key = (id, place);
    let first = !credit.formations.contains_key(&key);
    let meaningful =
        participation.meaningful && !credit.formations.get(&key).copied().unwrap_or(false);
    let service = &mut campaign
        .formations
        .get_mut(&id)
        .expect("surviving formation")
        .service;
    match participation.named_slot_vacant {
        Some(false) => {
            service.vacancy_service_progress = 0;
            service.vacancy_service_after_sequence =
                service.vacancy_service_after_sequence.max(sequence);
        }
        Some(true)
            if (meaningful || participation.first_threat_victory)
                && sequence > service.vacancy_service_after_sequence =>
        {
            service.vacancy_service_progress = service
                .vacancy_service_progress
                .saturating_add(1)
                .min(data.progression.emergence.vacant_slot_engagements);
        }
        Some(true) | None => {}
    }
    record(&mut service.ledger, participation, first, meaningful)?;
    let season = credit.seasons.entry(id).or_insert_with(|| SeasonService {
        completed_rounds: round,
        xp: 0,
        encounters: Vec::new(),
        routes: BTreeSet::new(),
    });
    let xp = if meaningful {
        participation
            .xp
            .min(data.progression.round_xp_cap.saturating_sub(season.xp))
    } else {
        0
    };
    service.xp = service.xp.checked_add(xp).ok_or(RuleError::Overflow {
        field: "formation service XP",
    })?;
    season.xp += xp;
    if let Some(entry) = season
        .encounters
        .iter_mut()
        .find(|entry| (entry.site, entry.opponent) == place)
    {
        entry.enemy_types.extend(&participation.enemy_types);
        if meaningful {
            entry.tags.extend(&participation.tags);
            entry.meaningful = true;
            entry.xp = xp;
        }
    } else {
        season.encounters.push(EncounterService {
            site: participation.site,
            opponent: participation.opponent,
            tags: if meaningful {
                participation.tags.clone()
            } else {
                [EvidenceKind::Battle].into_iter().collect()
            },
            enemy_types: participation.enemy_types.clone(),
            meaningful,
            xp,
        });
    }
    credit
        .formations
        .entry(key)
        .and_modify(|done| *done |= meaningful)
        .or_insert(meaningful);
    Ok(())
}

fn record(
    ledger: &mut EvidenceLedger,
    part: &Participation,
    first: bool,
    meaningful: bool,
) -> Result<(), RuleError> {
    ledger.encountered_troops.extend(&part.enemy_types);
    if first {
        increment(&mut ledger.counts, EvidenceKind::Battle)?;
    }
    if meaningful {
        for tag in part.tags.iter().filter(|tag| **tag != EvidenceKind::Battle) {
            increment(&mut ledger.counts, *tag)?;
        }
        for troop in &part.enemy_types {
            increment(&mut ledger.meaningful_against, *troop)?;
        }
        increment(&mut ledger.service_by_troop, part.troop)?;
    }
    Ok(())
}

fn increment<K: Ord>(map: &mut BTreeMap<K, u32>, key: K) -> Result<(), RuleError> {
    let count = map.entry(key).or_default();
    *count = count.checked_add(1).ok_or(RuleError::Overflow {
        field: "participation counter",
    })?;
    Ok(())
}

fn finish_seasons(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    seasons: BTreeMap<FormationId, SeasonService>,
) -> Result<(), RuleError> {
    let mut promotions = Vec::new();
    for (id, mut season) in seasons {
        season
            .encounters
            .sort_by_key(|entry| (entry.site, entry.opponent));
        let service = &mut campaign
            .formations
            .get_mut(&id)
            .expect("surviving formation")
            .service;
        service.recent.push(season);
        let tier = Veterancy::from_xp(service.xp, &data.progression);
        if tier != service.tier {
            promotions.push((id, tier, service.xp));
            service.tier = tier;
        }
    }
    let oldest = campaign.completed_rounds.saturating_sub(
        data.history
            .recent_service_rounds
            .saturating_sub(u32::from(campaign.diplomacy.ending.is_some())),
    );
    for formation in campaign.formations.values_mut() {
        formation
            .service
            .recent
            .retain(|season| season.completed_rounds >= oldest);
    }
    for (id, tier, xp) in promotions {
        history::record_veterancy(campaign, id, tier, xp)?;
    }
    Ok(())
}
