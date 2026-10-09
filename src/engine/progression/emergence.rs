//! A season's most distinguished surviving formation may produce one new named recruit.

use crate::{
    data::{progression::EpithetFact, world::PersonClass, GameData},
    engine::RuleError,
    state::{
        evidence::{EvidenceKind, EvidenceLedger, FormationService, SeasonService},
        military::FormationId as MilitaryFormationId,
        people::{
            Disposition, EmergenceRecord, Person, PersonAssignment, PersonCareer, PersonId,
            PersonStatus, Recognition, Tendency,
        },
        StrategicCampaign,
    },
};

pub(super) fn advance(campaign: &mut StrategicCampaign, data: &GameData) -> Result<(), RuleError> {
    let threshold = data.progression.emergence.vacant_slot_engagements;
    let formations = campaign.formations.keys().copied().collect::<Vec<_>>();
    for formation in formations {
        let Some(source) = campaign.formations.get(&formation) else {
            continue;
        };
        if source.headcount == 0 || campaign.formation_person(formation).is_some() {
            super::reset_formation_vacancy_progress(campaign, formation);
            continue;
        }
        if source.service.vacancy_service_progress < threshold {
            continue;
        }
        let Some(season) = source.service.recent.last().filter(|season| {
            season.completed_rounds.saturating_add(1) == campaign.completed_rounds
                && season
                    .encounters
                    .iter()
                    .any(|encounter| encounter.meaningful)
        }) else {
            super::reset_formation_vacancy_progress(campaign, formation);
            continue;
        };
        let season = season.clone();
        create(campaign, data, formation, &season)?;
        super::reset_formation_vacancy_progress(campaign, formation);
    }
    Ok(())
}

pub(super) fn update_tracked(campaign: &mut StrategicCampaign, data: &GameData) {
    let ids = campaign.people.keys().copied().collect::<Vec<_>>();
    for id in ids {
        let person = campaign.people.get_mut(&id).expect("person id");
        add_traits(
            &mut person.career,
            &person.evidence,
            &data.progression.traits,
        );
        if person.career.recognition.is_none()
            && person.is_alive()
            && !person.career.retired
            && person.career.hero_service_progress
                >= data.progression.recognition.personal_engagements
        {
            person.career.recognition =
                recognition(data, campaign.completed_rounds, &mut person.career);
        }
    }
}

fn create(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    formation: MilitaryFormationId,
    season: &SeasonService,
) -> Result<(), RuleError> {
    let Some(source) = campaign.formations.get(&formation) else {
        return Ok(());
    };
    if source.headcount == 0 || campaign.formation_person(formation).is_some() {
        return Ok(());
    }
    let faction = source.faction;
    if campaign
        .armies
        .values()
        .all(|army| !army.formation_ids().any(|id| id == formation))
    {
        return Err(RuleError::InvalidState(
            "Emergence source has no army.".into(),
        ));
    }
    let id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "person identifiers",
    })?);
    let age = data.progression.emergence.minimum_age_years
        + campaign.rng.people.below(
            (data.progression.emergence.maximum_age_years
                - data.progression.emergence.minimum_age_years
                + 1) as usize,
        ) as u32;
    let birth_round = i64::from(campaign.completed_rounds) - i64::from(age) * 4;
    let service_start = campaign.formations[&formation]
        .created_round
        .max(
            campaign
                .completed_rounds
                .saturating_sub(data.progression.emergence.retrospective_rounds),
        )
        .max((birth_round + 68).clamp(0, i64::from(u32::MAX)) as u32);
    let name = crate::engine::succession::fresh_person_name(campaign, data);
    let disposition = Disposition {
        courage: tendency(&mut campaign.rng.people),
        care: tendency(&mut campaign.rng.people),
        curiosity: tendency(&mut campaign.rng.people),
    };
    let source = campaign.formations.get(&formation).expect("checked source");
    let assignment = PersonAssignment::Formation { formation };
    let evidence = retrospective(&source.service, source.kind, service_start);
    let mut career_state = PersonCareer {
        disposition,
        ..PersonCareer::default()
    };
    add_traits(&mut career_state, &evidence, &data.progression.traits);
    let (site, deed) = emergence_deed(season);
    career_state.notable_sites = notable_sites(&source.service, service_start);
    career_state.emergence = Some(EmergenceRecord {
        completed_rounds: campaign.completed_rounds,
        source_formation: formation,
        source_troop: source.kind,
        site,
        distinguishing_deed: deed,
    });
    let appearance = crate::engine::portraits::allocate_for_person(campaign, &data.portraits, id)
        .map_err(RuleError::InvalidState)?;
    campaign.people.insert(
        id,
        Person {
            appearance,
            career: career_state,
            evidence,
            id,
            faction,
            name,
            birth_round,
            service_start_round: service_start,
            class: PersonClass::Recruit,
            assignment,
            movement_spent: 0,
            status: PersonStatus::Fit,
        },
    );
    Ok(())
}

fn retrospective(
    service: &FormationService,
    source: crate::data::economy::TroopKind,
    start: u32,
) -> EvidenceLedger {
    let mut ledger = EvidenceLedger::default();
    for season in service
        .recent
        .iter()
        .filter(|season| season.completed_rounds >= start)
    {
        ledger.traversed_routes.extend(&season.routes);
        for encounter in season
            .encounters
            .iter()
            .filter(|encounter| encounter.meaningful)
        {
            bump(&mut ledger, EvidenceKind::Battle);
            bump(&mut ledger, EvidenceKind::MeaningfulEncounter);
            bump_troop(&mut ledger, source);
            for tag in &encounter.tags {
                if *tag != EvidenceKind::Battle
                    && *tag != EvidenceKind::MeaningfulEncounter
                    && !tag.is_personal_command_deed()
                {
                    bump(&mut ledger, *tag);
                }
            }
            for troop in &encounter.enemy_types {
                ledger.encountered_troops.insert(*troop);
                *ledger.meaningful_against.entry(*troop).or_default() += 1;
            }
        }
    }
    ledger
}

fn bump_troop(ledger: &mut EvidenceLedger, troop: crate::data::economy::TroopKind) {
    *ledger.service_by_troop.entry(troop).or_default() += 1;
}

fn bump(ledger: &mut EvidenceLedger, kind: EvidenceKind) {
    *ledger.counts.entry(kind).or_default() += 1;
}

fn add_traits(
    career: &mut PersonCareer,
    evidence: &EvidenceLedger,
    rules: &crate::data::progression::TraitRules,
) {
    let count = |kind| evidence.counts.get(&kind).copied().unwrap_or(0);
    let occasions = |adjustment: i32| {
        (i32::try_from(rules.base_occasions).unwrap_or(i32::MAX) - adjustment).clamp(
            rules.minimum_occasions as i32,
            rules.maximum_occasions as i32,
        ) as u32
    };
    if count(EvidenceKind::SurvivedOutnumbered)
        >= occasions(career.disposition.courage.adjustment())
    {
        career
            .traits
            .insert(crate::state::people::PersonTrait::Bold);
    }
    if count(EvidenceKind::DefendedAnchor).saturating_add(count(EvidenceKind::TreatedWounded))
        >= occasions(career.disposition.care.adjustment())
    {
        career
            .traits
            .insert(crate::state::people::PersonTrait::Protective);
    }
    if count(EvidenceKind::AssumedCommand).saturating_add(count(EvidenceKind::CommandedVictory))
        >= occasions(career.disposition.curiosity.adjustment())
    {
        career
            .traits
            .insert(crate::state::people::PersonTrait::NaturalCommander);
    }
}

fn recognition(data: &GameData, round: u32, career: &mut PersonCareer) -> Option<Recognition> {
    if career.hero_service_progress < data.progression.recognition.personal_engagements {
        return None;
    }
    let distinction = data
        .progression
        .recognition
        .required_facts
        .iter()
        .find_map(|fact| {
            career
                .hero_service_sites
                .get(fact)
                .copied()
                .map(|site| (*fact, site))
        })
        .or_else(|| {
            career
                .hero_service_sites
                .get(&EpithetFact::BattleService)
                .copied()
                .map(|site| (EpithetFact::BattleService, site))
        })?;
    let (cause, site) = distinction;
    let epithet = data.human_names.epithets.get(&cause)?.clone();
    career.notable_sites.insert(cause, site);
    Some(Recognition {
        completed_rounds: round,
        epithet,
        cause,
        site,
    })
}

fn notable_sites(
    service: &FormationService,
    start: u32,
) -> std::collections::BTreeMap<EpithetFact, crate::data::world::SiteId> {
    let mut sites = std::collections::BTreeMap::new();
    for season in service
        .recent
        .iter()
        .filter(|season| season.completed_rounds >= start)
    {
        for encounter in &season.encounters {
            for (fact, kind) in [
                (
                    EpithetFact::SurvivedOutnumbered,
                    EvidenceKind::SurvivedOutnumbered,
                ),
                (EpithetFact::DefendedAnchor, EvidenceKind::DefendedAnchor),
                (EpithetFact::CapturedAnchor, EvidenceKind::CapturedAnchor),
                (EpithetFact::TreatedWounded, EvidenceKind::TreatedWounded),
                (EpithetFact::AssumedCommand, EvidenceKind::AssumedCommand),
                (
                    EpithetFact::CommandedVictory,
                    EvidenceKind::CommandedVictory,
                ),
            ] {
                if !kind.is_personal_command_deed() && encounter.tags.contains(&kind) {
                    sites.entry(fact).or_insert(encounter.site);
                }
            }
        }
    }
    sites
}

fn emergence_deed(season: &SeasonService) -> (crate::data::world::SiteId, Option<EpithetFact>) {
    let facts = [
        (
            EpithetFact::SurvivedOutnumbered,
            EvidenceKind::SurvivedOutnumbered,
        ),
        (EpithetFact::DefendedAnchor, EvidenceKind::DefendedAnchor),
        (EpithetFact::CapturedAnchor, EvidenceKind::CapturedAnchor),
        (EpithetFact::TreatedWounded, EvidenceKind::TreatedWounded),
        (EpithetFact::AssumedCommand, EvidenceKind::AssumedCommand),
        (
            EpithetFact::CommandedVictory,
            EvidenceKind::CommandedVictory,
        ),
    ];
    for encounter in &season.encounters {
        for (deed, kind) in facts {
            if !kind.is_personal_command_deed() && encounter.tags.contains(&kind) {
                return (encounter.site, Some(deed));
            }
        }
    }
    (
        season
            .encounters
            .first()
            .expect("emergence requires a battle service encounter")
            .site,
        None,
    )
}

fn tendency(rng: &mut macroquad_toolkit::rng::SeededRng) -> Tendency {
    match rng.below(3) {
        0 => Tendency::Negative,
        1 => Tendency::Neutral,
        _ => Tendency::Positive,
    }
}
