//! Living assignments and dated battle injuries/deaths remain valid after removal.

use super::{Person, PersonAssignment, PersonCourse, PersonStatus, PersonTrait, StrategicCampaign};
use crate::data::{progression::EpithetFact, world::PersonClass, GameData};

impl StrategicCampaign {
    pub(crate) fn validate_people(&self, data: &GameData) -> Result<(), String> {
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
                person.movement_spent <= data.rules.leadership.officer_movement_allowance,
                "movement_spent",
                "exceeds seasonal allowance",
            )?;
            self.validate_person_status(person, data)?;
            let valid_assignment = match person.assignment {
                PersonAssignment::Formation { formation } => {
                    person.is_alive()
                        && self
                            .formations
                            .get(&formation)
                            .is_some_and(|entry| entry.faction == person.faction)
                }
                PersonAssignment::Site { site } => {
                    person.is_alive() && self.world.site(site).is_some()
                }
                PersonAssignment::Dead => !person.is_alive(),
            };
            require(
                valid_assignment,
                "assignment",
                "unknown, foreign or incompatible assignment",
            )?;
            self.validate_career(person, data)?;
        }
        Ok(())
    }

    fn validate_career(&self, person: &Person, data: &GameData) -> Result<(), String> {
        let career = &person.career;
        require(
            career.notable_sites.len() <= 6
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
            career.mentorship_seasons.iter().all(|(discipline, count)| {
                let _ = discipline;
                *count > 0 && *count <= self.completed_rounds
            }),
            "career.mentorship_seasons",
            "invalid mentorship total",
        )?;
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
            require(
                recognition.completed_rounds <= self.completed_rounds
                    && self.world.site(recognition.site).is_some()
                    && career.notable_sites.get(&recognition.cause) == Some(&recognition.site)
                    && person
                        .evidence
                        .counts
                        .get(&crate::state::evidence::EvidenceKind::MeaningfulEncounter)
                        .copied()
                        .unwrap_or(0)
                        >= data.progression.recognition.meaningful_encounters
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
                    steps_completed,
                } => {
                    *target != PersonClass::Recruit
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
                    && !self.is_independent(person.faction)
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
    }
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.people.{field}: {reason}"))
    }
}
