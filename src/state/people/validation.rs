//! Living assignments and dated battle injuries/deaths remain valid after removal.

use super::{
    Person, PersonAssignment, PersonCourse, PersonSiteRole, PersonStatus, PersonTrait,
    StrategicCampaign,
};
use crate::data::{progression::EpithetFact, world::PersonClass, GameData};

impl StrategicCampaign {
    pub(crate) fn validate_people(&self, data: &GameData) -> Result<(), String> {
        let mut occupied = std::collections::BTreeSet::new();
        for (id, person) in &self.people {
            require(*id == person.id && id.0 > 0, "id", "invalid identity")?;
            require(
                self.factions.contains_key(&person.faction),
                "faction",
                "unknown faction",
            )?;
            require(
                !person.name.is_empty()
                    && person.name.trim() == person.name
                    && person.name.chars().count() <= 64
                    && !person.name.chars().any(char::is_control),
                "name",
                "invalid person name",
            )?;
            let age = i64::from(self.completed_rounds).checked_sub(person.birth_round);
            require(
                age.is_some_and(|age| (0..=i64::from(u32::MAX) * 4).contains(&age))
                    && person.service_start_round <= self.completed_rounds
                    && person.birth_round <= i64::from(person.service_start_round),
                "birth_round/service_start_round",
                "invalid birth or service date",
            )?;
            require(
                person.movement_spent <= self.person_movement_allowance(person, data),
                "movement_spent",
                "exceeds seasonal allowance",
            )?;
            self.validate_person_status(person, data)?;
            if let PersonAssignment::Formation { formation } = person.assignment {
                require(
                    occupied.insert(formation),
                    "assignment",
                    "each formation slot can contain only one named person",
                )?;
            }
            let valid_assignment = match person.assignment {
                PersonAssignment::Formation { formation } => {
                    person.is_alive()
                        && person.age_years(self.completed_rounds) >= 17
                        && person.age_years(self.completed_rounds)
                            < data.lifecycle.automatic_retirement_age_years
                        && !person.career.retired
                        && self
                            .formations
                            .get(&formation)
                            .is_some_and(|entry| entry.faction == person.faction)
                }
                PersonAssignment::Site { site } => {
                    person.is_alive() && self.world.site(site).is_some()
                }
                PersonAssignment::Dependent { site } => {
                    person.is_alive()
                        && person.status == PersonStatus::Fit
                        && !person.career.retired
                        && person.career.course.is_none()
                        && self.world.site(site).is_some()
                }
                PersonAssignment::Trainee { site } => {
                    person.is_alive()
                        && person.status == PersonStatus::Fit
                        && !person.career.retired
                        && person.career.course.is_none()
                        && person.class == PersonClass::Recruit
                        && (data.households.trainee_minimum_age_years
                            ..data.households.service_minimum_age_years)
                            .contains(&person.age_years(self.completed_rounds))
                        && self.world.site(site).is_some()
                }
                PersonAssignment::Dead => !person.is_alive(),
            };
            if !valid_assignment {
                let assigned_formation_faction = match person.assignment {
                    PersonAssignment::Formation { formation } => {
                        self.formations.get(&formation).map(|entry| entry.faction)
                    }
                    _ => None,
                };
                return Err(format!(
                    "campaign.people.assignment: {:?} of faction {:?} has incompatible {:?} assignment at age {} (assigned formation faction {:?}, class {:?}, status {:?}, retired {}, course {:?})",
                    person.id,
                    person.faction,
                    person.assignment,
                    person.age_years(self.completed_rounds),
                    assigned_formation_faction,
                    person.class,
                    person.status,
                    person.career.retired,
                    person.career.course,
                ));
            }
            self.validate_career(person, data)?;
        }
        self.validate_mentorships(data)?;
        Ok(())
    }

    fn validate_career(&self, person: &Person, data: &GameData) -> Result<(), String> {
        let career = &person.career;
        require(
            career.notable_sites.len() <= 7
                && career.notable_sites.iter().all(|(fact, site)| {
                    self.world.site(*site).is_some()
                        && person
                            .evidence
                            .counts
                            .get(&evidence_for(*fact))
                            .copied()
                            .unwrap_or(0)
                            > 0
                }),
            "career.notable_sites",
            "unknown site or unsupported distinction",
        )?;
        require(
            career.hero_service_progress <= data.progression.recognition.personal_engagements
                && u32::from(career.hero_service_progress)
                    <= person
                        .evidence
                        .counts
                        .get(&crate::state::evidence::EvidenceKind::MeaningfulEncounter)
                        .copied()
                        .unwrap_or(0)
                && career.hero_service_sites.len() <= 5
                && (career.hero_service_progress == 0
                    || career
                        .hero_service_sites
                        .contains_key(&EpithetFact::BattleService))
                && career.hero_service_sites.iter().all(|(fact, site)| {
                    self.world.site(*site).is_some()
                        && (*fact == EpithetFact::BattleService
                            || data.progression.recognition.required_facts.contains(fact))
                        && person
                            .evidence
                            .counts
                            .get(&evidence_for(*fact))
                            .copied()
                            .unwrap_or(0)
                            > 0
                }),
            "career.hero_service_progress",
            "invalid personal Hero service or epithet sites",
        )?;
        require(
            career.mentorship_seasons.iter().all(|(discipline, count)| {
                let _ = discipline;
                *count > 0 && *count <= data.lifecycle.apprenticeship_seasons
            }),
            "career.mentorship_seasons",
            "invalid mentorship total",
        )?;
        require(
            career
                .discipline_service_seasons
                .values()
                .all(|count| *count > 0 && *count <= self.completed_rounds),
            "career.discipline_service_seasons",
            "invalid service total",
        )?;
        require(
            career
                .automatic_retirement_round
                .is_none_or(|round| career.retired && round <= self.completed_rounds),
            "career.automatic_retirement_round",
            "automatic retirement must be dated and retired",
        )?;
        require(
            career.completed_mentors.iter().all(|record| {
                self.people.get(&record.mentor).is_some_and(|mentor| {
                    mentor.id != person.id
                        && mentor.faction == person.faction
                        && record.started_round < record.completed_round
                        && record.completed_round <= self.completed_rounds
                        && record.completed_round - record.started_round
                            >= data.lifecycle.apprenticeship_seasons
                        && career
                            .mentorship_seasons
                            .get(&record.discipline)
                            .copied()
                            .unwrap_or(0)
                            >= data.lifecycle.apprenticeship_seasons
                })
            }),
            "career.completed_mentors",
            "apprenticeship must reference its mentor and supported completed dates",
        )?;
        if career.site_role == Some(PersonSiteRole::Governor) {
            let valid = matches!(person.assignment, PersonAssignment::Site { site }
            if self.world.site(site).is_some_and(|site| {
                site.controller == Some(person.faction)
                    && site.habitation != crate::data::economy::Habitation::Unsettled
                    && !self.sieges.contains_key(&site.id)
            })) && person.is_alive()
                && person.status == PersonStatus::Fit
                && person.age_years(self.completed_rounds)
                    >= data.rules.leadership.field_min_age_years;
            require(
                valid,
                "career.site_role",
                "governor must be a fit adult at an owned, inhabited, unbesieged settlement",
            )?;
        }
        let occasions = |tendency: super::Tendency| {
            (data.progression.traits.base_occasions as i32 - tendency.adjustment()).clamp(
                data.progression.traits.minimum_occasions as i32,
                data.progression.traits.maximum_occasions as i32,
            ) as u32
        };
        let count = |kind| person.evidence.counts.get(&kind).copied().unwrap_or(0);
        require(
            (!career.traits.contains(&PersonTrait::Bold)
                || count(crate::state::evidence::EvidenceKind::SurvivedOutnumbered)
                    >= occasions(career.disposition.courage))
                && (!career.traits.contains(&PersonTrait::Protective)
                    || count(crate::state::evidence::EvidenceKind::DefendedAnchor).saturating_add(
                        count(crate::state::evidence::EvidenceKind::TreatedWounded),
                    ) >= occasions(career.disposition.care))
                && (!career.traits.contains(&PersonTrait::NaturalCommander)
                    || count(crate::state::evidence::EvidenceKind::AssumedCommand).saturating_add(
                        count(crate::state::evidence::EvidenceKind::CommandedVictory),
                    ) >= occasions(career.disposition.curiosity)),
            "career.traits",
            "earned trait lacks its disposition-adjusted evidence",
        )?;
        if let Some(record) = &career.emergence {
            let age = i64::from(record.completed_rounds).saturating_sub(person.birth_round);
            require(
                record.completed_rounds <= self.completed_rounds
                    && record.source_formation.0 > 0
                    && record.source_formation < self.next_ids.formation
                    && self.world.site(record.site).is_some()
                    && data.economy.formations.contains_key(&record.source_troop)
                    && (i64::from(person.service_start_round)
                        >= i64::from(
                            record
                                .completed_rounds
                                .saturating_sub(data.progression.emergence.retrospective_rounds),
                        )
                        && i64::from(person.service_start_round) >= person.birth_round + 68)
                    && (18..=30).contains(&(age / 4)),
                "career.emergence",
                "invalid source, chronology, site or adult age",
            )?;
            if let Some(source) = self.formations.get(&record.source_formation) {
                require(
                    source.created_round <= person.service_start_round,
                    "career.emergence",
                    "service predates its source formation",
                )?;
            }
        }
        if let Some(recognition) = &career.recognition {
            let personal_award = career.hero_service_progress
                >= data.progression.recognition.personal_engagements
                && career.hero_service_sites.get(&recognition.cause) == Some(&recognition.site);
            let legacy_award = recognition.cause != EpithetFact::BattleService
                && career.hero_service_progress == 0
                && person
                    .evidence
                    .counts
                    .get(&crate::state::evidence::EvidenceKind::MeaningfulEncounter)
                    .copied()
                    .unwrap_or(0)
                    >= u32::from(data.progression.recognition.personal_engagements);
            require(
                recognition.completed_rounds <= self.completed_rounds
                    && self.world.site(recognition.site).is_some()
                    && career.notable_sites.get(&recognition.cause) == Some(&recognition.site)
                    && (personal_award || legacy_award)
                    && person
                        .evidence
                        .counts
                        .get(&evidence_for(recognition.cause))
                        .copied()
                        .unwrap_or(0)
                        > 0
                    && data.human_names.epithets.get(&recognition.cause)
                        == Some(&recognition.epithet),
                "career.recognition",
                "unsupported or inconsistent distinction",
            )?;
        }
        if let Some(course) = &career.course {
            let valid = match course {
                PersonCourse::Class {
                    target,
                    site,
                    paid_gold,
                    steps_completed,
                } => {
                    *paid_gold >= 0
                        && *target != PersonClass::Recruit
                        && data.progression.careers.courses.contains_key(target)
                        && *steps_completed < data.progression.careers.course_steps
                        && self.world.site(*site).is_some()
                }
                PersonCourse::RidingPractice {
                    site,
                    steps_completed,
                } => *steps_completed == 0 && self.world.site(*site).is_some(),
            };
            require(valid, "career.course", "invalid active course")?;
        }
        require(
            career.relationships.len() <= data.progression.relationships_per_person
                && career.relationships.iter().all(|(other_id, relation)| {
                    let Some(other) = self.people.get(other_id) else {
                        return false;
                    };
                    let symmetric = other.career.relationships.get(&person.id);
                    *other_id != person.id
                        && person.is_alive()
                        && other.is_alive()
                        && (relation.shared_service_seasons > 0)
                            == relation.last_shared_service_round.is_some()
                        && (relation.mutual_combat_rounds > 0)
                            == relation.last_mutual_combat_round.is_some()
                        && relation
                            .last_shared_service_round
                            .is_none_or(|round| round <= self.completed_rounds)
                        && relation
                            .last_mutual_combat_round
                            .is_none_or(|round| round <= self.completed_rounds)
                        && symmetric == Some(relation)
                }),
            "career.relationships",
            "invalid, asymmetric, stale or oversized relationship ledger",
        )?;
        Ok(())
    }

    fn validate_mentorships(&self, data: &GameData) -> Result<(), String> {
        let mut mentors = std::collections::BTreeSet::new();
        for (learner_id, mentorship) in &self.mentorships {
            let Some(learner) = self.people.get(learner_id) else {
                return Err("campaign.mentorships: unknown learner".into());
            };
            let Some(mentor) = self.people.get(&mentorship.mentor) else {
                return Err("campaign.mentorships: unknown mentor".into());
            };
            let valid = *learner_id != mentorship.mentor
                && learner.is_alive()
                && mentor.is_alive()
                && learner.faction == mentor.faction
                && !learner.career.retired
                && learner.career.course.is_none()
                && learner.age_years(self.completed_rounds)
                    >= data.lifecycle.learner_minimum_age_years
                && mentor.age_years(self.completed_rounds)
                    >= data.lifecycle.mentor_minimum_age_years
                && mentor
                    .career
                    .discipline_service_seasons
                    .get(&mentorship.discipline)
                    .copied()
                    .unwrap_or(0)
                    >= data.lifecycle.mentor_service_seasons
                && (learner.assignment != PersonAssignment::Dead)
                && (mentor.assignment != PersonAssignment::Dead)
                && mentorship.started_round <= self.completed_rounds
                && mentorship.seasons_completed < data.lifecycle.apprenticeship_seasons
                && mentors.insert(mentorship.mentor);
            require(
                valid,
                "mentorships",
                "invalid identity, qualification, chronology or one-learner capacity",
            )?;
        }
        let mut governors = std::collections::BTreeSet::new();
        for person in self
            .people
            .values()
            .filter(|person| person.career.site_role == Some(PersonSiteRole::Governor))
        {
            let PersonAssignment::Site { site } = person.assignment else {
                return Err("campaign.people.career.site_role: governor has no settlement".into());
            };
            require(
                governors.insert(site),
                "people.career.site_role",
                "a settlement has more than one governor",
            )?;
        }
        Ok(())
    }

    fn validate_person_status(&self, person: &Person, data: &GameData) -> Result<(), String> {
        let valid = match person.status {
            PersonStatus::Fit => self.is_independent(person.faction),
            PersonStatus::Displaced {
                completed_rounds,
                site,
            } => {
                completed_rounds >= person.service_start_round
                    && completed_rounds <= self.completed_rounds
                    && self.world.site(site).is_some()
                    && person.assignment == (PersonAssignment::Site { site })
                    && person.movement_spent == 0
                    && (!self.is_independent(person.faction)
                        || person.career.retired
                            && person.career.automatic_retirement_round == Some(completed_rounds))
            }
            PersonStatus::Wounded {
                since_round,
                remaining_steps,
            } => {
                self.is_independent(person.faction)
                    && since_round >= person.service_start_round
                    && since_round <= self.completed_rounds
                    && remaining_steps > 0
                    && remaining_steps <= data.combat.wound_recovery_steps
            }
            PersonStatus::Dead {
                completed_rounds,
                site,
            } => {
                completed_rounds >= person.service_start_round
                    && completed_rounds <= self.completed_rounds
                    && self.world.site(site).is_some()
                    && person.movement_spent == 0
            }
        };
        require(valid, "status", "invalid injury progress or death record")
    }
}

fn evidence_for(fact: EpithetFact) -> crate::state::evidence::EvidenceKind {
    use crate::state::evidence::EvidenceKind;
    match fact {
        EpithetFact::SurvivedOutnumbered => EvidenceKind::SurvivedOutnumbered,
        EpithetFact::DefendedAnchor => EvidenceKind::DefendedAnchor,
        EpithetFact::CapturedAnchor => EvidenceKind::CapturedAnchor,
        EpithetFact::TreatedWounded => EvidenceKind::TreatedWounded,
        EpithetFact::AssumedCommand => EvidenceKind::AssumedCommand,
        EpithetFact::CommandedVictory => EvidenceKind::CommandedVictory,
        EpithetFact::BattleService => EvidenceKind::MeaningfulEncounter,
    }
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.people.{field}: {reason}"))
    }
}
