//! A season's most distinguished surviving formation may produce one new named recruit.

use crate::{
    data::{
        progression::EpithetFact,
        world::{FactionId, PersonClass},
        GameData,
    },
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
    let mut factions = campaign.factions.keys().copied().collect::<Vec<_>>();
    factions.sort();
    for faction in factions {
        let Some((formation, season)) = candidate(campaign, faction) else {
            continue;
        };
        if season.xp == 0 || !campaign.is_independent(faction) {
            continue;
        }
        let people = campaign
            .people
            .values()
            .filter(|person| {
                person.faction == faction
                    && person.is_alive()
                    && !person.career.retired
                    && person.age_years(campaign.completed_rounds) >= 18
            })
            .count() as u64;
        let curve = u64::from(data.progression.emergence.roster_base)
            .saturating_mul(u64::from(data.progression.emergence.roster_factor))
            / (u64::from(data.progression.emergence.roster_square_factor)
                .saturating_mul(people.saturating_mul(people))
                .saturating_add(u64::from(data.progression.emergence.roster_factor))
                .max(1));
        let roll = campaign.rng.people.below(1000) as u64;
        let mut chance = curve;
        let tier = campaign.formations[&formation].service.tier;
        chance = chance.saturating_mul(match tier {
            crate::state::evidence::Veterancy::Ordinary => 1000,
            crate::state::evidence::Veterancy::Seasoned => {
                data.progression.emergence.seasoned_multiplier_permille
            }
            crate::state::evidence::Veterancy::Veteran => {
                data.progression.emergence.veteran_multiplier_permille
            }
        } as u64)
            / 1000;
        if exceptional(&season) {
            chance = chance.saturating_mul(u64::from(
                data.progression.emergence.exceptional_multiplier_permille,
            )) / 1000;
        }
        chance = chance.min(u64::from(
            data.progression.emergence.maximum_chance_permille,
        ));
        if roll >= chance {
            continue;
        }
        create(campaign, data, faction, formation, &season)?;
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
        if person.career.recognition.is_none() {
            person.career.recognition = recognition(
                data,
                campaign.completed_rounds,
                &person.evidence,
                &person.career.notable_sites,
            );
        }
    }
}

fn candidate(
    campaign: &StrategicCampaign,
    faction: FactionId,
) -> Option<(MilitaryFormationId, SeasonService)> {
    campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction && formation.headcount > 0)
        .filter_map(|formation| {
            formation
                .service
                .recent
                .last()
                .filter(|season| {
                    season.completed_rounds.saturating_add(1) == campaign.completed_rounds
                })
                .cloned()
                .map(|season| (formation, season))
        })
        .max_by_key(|(formation, season)| {
            (
                season.xp,
                formation.service.tier,
                std::cmp::Reverse(formation.id),
            )
        })
        .map(|(formation, season)| (formation.id, season))
}

fn exceptional(season: &SeasonService) -> bool {
    let outnumbered = season
        .encounters
        .iter()
        .any(|encounter| encounter.tags.contains(&EvidenceKind::SurvivedOutnumbered));
    let anchor = season.encounters.iter().any(|encounter| {
        encounter.tags.contains(&EvidenceKind::CapturedAnchor)
            || encounter.tags.contains(&EvidenceKind::DefendedAnchor)
    });
    outnumbered && anchor
}

fn create(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    formation: MilitaryFormationId,
    season: &SeasonService,
) -> Result<(), RuleError> {
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
    let given = &data.human_names.given_names[campaign
        .rng
        .people
        .below(data.human_names.given_names.len())];
    let family = &data.human_names.family_names[campaign
        .rng
        .people
        .below(data.human_names.family_names.len())];
    let name = format!("{given} {family}");
    let disposition = Disposition {
        courage: tendency(&mut campaign.rng.people),
        care: tendency(&mut campaign.rng.people),
        curiosity: tendency(&mut campaign.rng.people),
    };
    let source = &campaign.formations[&formation];
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
    career_state.recognition = recognition(
        data,
        campaign.completed_rounds,
        &evidence,
        &career_state.notable_sites,
    );
    campaign.people.insert(
        id,
        Person {
            career: career_state,
            evidence,
            id,
            faction,
            name,
            birth_round,
            service_start_round: service_start,
            class: PersonClass::Recruit,
            assignment: PersonAssignment::Formation { formation },
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

fn recognition(
    data: &GameData,
    round: u32,
    evidence: &EvidenceLedger,
    sites: &std::collections::BTreeMap<EpithetFact, crate::data::world::SiteId>,
) -> Option<Recognition> {
    if evidence
        .counts
        .get(&EvidenceKind::MeaningfulEncounter)
        .copied()
        .unwrap_or(0)
        < data.progression.recognition.meaningful_encounters
    {
        return None;
    }
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
    facts
        .into_iter()
        .filter(|(fact, _)| data.progression.recognition.required_facts.contains(fact))
        .find_map(|(cause, kind)| {
            if evidence.counts.get(&kind).copied().unwrap_or(0) == 0 {
                return None;
            }
            let site = *sites.get(&cause)?;
            Some(Recognition {
                completed_rounds: round,
                epithet: data.human_names.epithets[&cause].clone(),
                cause,
                site,
            })
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
