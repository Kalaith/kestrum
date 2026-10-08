//! Authored progression curves and ordinary career opportunities.

use super::{
    economy::TroopKind,
    validation::require,
    world::{Facility, PersonClass},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionRules {
    pub meaningful_opposition_permille: u32,
    pub meaningful_loss_permille: u32,
    pub outnumbered_permille: u32,
    pub battle_xp: u32,
    pub victory_xp: u32,
    pub outnumbered_xp: u32,
    pub round_xp_cap: u32,
    pub seasoned_xp: u32,
    pub veteran_xp: u32,
    pub ordinary_permille: u32,
    pub seasoned_permille: u32,
    pub veteran_permille: u32,
    pub emergence: EmergenceRules,
    pub traits: TraitRules,
    pub recognition: RecognitionRules,
    pub careers: CareerRules,
    pub specializations: BTreeMap<FormationSpecialization, SpecializationRule>,
    pub relationships_per_person: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmergenceRules {
    pub vacant_slot_engagements: u8,
    pub retrospective_rounds: u32,
    pub minimum_age_years: u32,
    pub maximum_age_years: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitRules {
    pub base_occasions: u32,
    pub minimum_occasions: u32,
    pub maximum_occasions: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecognitionRules {
    pub personal_engagements: u8,
    pub required_facts: Vec<EpithetFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpithetFact {
    SurvivedOutnumbered,
    DefendedAnchor,
    CapturedAnchor,
    TreatedWounded,
    AssumedCommand,
    CommandedVictory,
    BattleService,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CareerRules {
    pub training_focus_discount_percent: u32,
    pub course_steps: u32,
    pub courses: BTreeMap<PersonClass, CareerCourseRule>,
    pub infantry_battles: u32,
    pub archer_battles: u32,
    pub scout_routes: u32,
    pub mentored_scout_routes: u32,
    pub riding_seasons: u32,
    pub rider_battles: u32,
    pub treatment_occasions: u32,
    pub mentored_treatment_occasions: u32,
    pub officer_encounters: u32,
    pub officer_command_facts: u32,
    pub mentorship_seasons: u32,
    pub specialization_steps: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CareerCourseRule {
    pub gold_cost: i64,
    pub facility: Facility,
    pub requires_horse_access: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormationSpecialization {
    ShieldGuard,
    Pikemen,
    LightCavalry,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecializationRule {
    pub sources: Vec<TroopKind>,
    pub gold_cost: i64,
    pub course_steps: u32,
    pub requirement: SpecializationRequirement,
    pub defense_resistance_permille: Option<u32>,
    pub counter: Option<SpecializedCounter>,
    pub movement_allowance: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpecializationRequirement {
    DefendedAnchors { occasions: u32 },
    MeaningfulAgainst { troop: TroopKind, occasions: u32 },
    RoutesAndRetreatingVictories { routes: usize, victories: u32 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecializedCounter {
    pub target: TroopKind,
    pub permille: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingDiscipline {
    Infantry,
    Archery,
    Scouting,
    Riding,
    Medicine,
    Command,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanNamePool {
    pub given_names: Vec<String>,
    pub family_names: Vec<String>,
    pub epithets: BTreeMap<EpithetFact, String>,
}

impl HumanNamePool {
    pub fn validate(&self) -> Result<(), String> {
        if self.given_names.len() < 12
            || self.family_names.len() < 12
            || self
                .given_names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.given_names.len()
            || self
                .family_names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.family_names.len()
            || self.given_names.iter().any(|name| invalid_name(name))
            || self.family_names.iter().any(|name| invalid_name(name))
            || self.epithets.len() != 5
            || self.epithets.values().any(|epithet| invalid_name(epithet))
        {
            return Err("human_names.json: invalid names or incomplete epithet vocabulary".into());
        }
        Ok(())
    }
}

impl ProgressionRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.meaningful_opposition_permille > 1000
            || self.meaningful_loss_permille > 1000
            || !(1000..=10000).contains(&self.outnumbered_permille)
            || self.battle_xp == 0
            || self.round_xp_cap == 0
            || self.round_xp_cap > 100
            || self.battle_xp > self.round_xp_cap
            || self.victory_xp > self.round_xp_cap
            || self.outnumbered_xp > self.round_xp_cap
            || self.seasoned_xp == 0
            || self.veteran_xp <= self.seasoned_xp
            || self.ordinary_permille != 1000
            || self.seasoned_permille < self.ordinary_permille
            || self.veteran_permille < self.seasoned_permille
            || self.veteran_permille > 10000
        {
            return Err("progression.json: invalid significance, XP or veterancy rule".into());
        }
        self.validate_emergence()?;
        self.validate_traits()?;
        self.validate_careers()?;
        self.validate_specializations()?;
        require(
            "progression.json",
            "relationships_per_person",
            (1..=64).contains(&self.relationships_per_person),
            "must be 1..=64",
        )
    }

    fn validate_emergence(&self) -> Result<(), String> {
        let rules = &self.emergence;
        require(
            "progression.json",
            "emergence",
            (1..=10).contains(&rules.vacant_slot_engagements)
                && (1..=80).contains(&rules.retrospective_rounds)
                && rules.minimum_age_years >= 17
                && rules.maximum_age_years >= rules.minimum_age_years
                && rules.maximum_age_years <= 80,
            "invalid emergence curve, chronology or age range",
        )?;
        require(
            "progression.json",
            "recognition",
            (1..=10).contains(&self.recognition.personal_engagements)
                && self.recognition.personal_engagements == 3
                && self
                    .recognition
                    .required_facts
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    == [
                        EpithetFact::SurvivedOutnumbered,
                        EpithetFact::DefendedAnchor,
                        EpithetFact::TreatedWounded,
                        EpithetFact::AssumedCommand,
                    ]
                    .into_iter()
                    .collect(),
            "requires personal engagements and supported epithets",
        )
    }

    fn validate_traits(&self) -> Result<(), String> {
        require(
            "progression.json",
            "traits",
            self.traits.minimum_occasions > 0
                && self.traits.minimum_occasions <= self.traits.base_occasions
                && self.traits.base_occasions <= self.traits.maximum_occasions
                && self.traits.maximum_occasions <= 8
                && self.traits.base_occasions == 3
                && self.traits.minimum_occasions == 2
                && self.traits.maximum_occasions == 4,
            "invalid evidence threshold bounds",
        )
    }

    fn validate_careers(&self) -> Result<(), String> {
        require(
            "progression.json",
            "careers.training_focus_discount_percent",
            self.careers.training_focus_discount_percent <= 100,
            "must be at most 100",
        )?;
        use PersonClass::*;
        let rules = &self.careers;
        let expected = [Infantry, Archer, Scout, Cavalry, Medic, Officer];
        require(
            "progression.json",
            "careers",
            rules.course_steps == 2
                && rules.courses.len() == expected.len()
                && expected
                    .iter()
                    .all(|class| rules.courses.contains_key(class))
                && rules.courses.values().all(|course| course.gold_cost > 0)
                && rules.infantry_battles > 0
                && rules.archer_battles > 0
                && rules.scout_routes > 0
                && (1..=rules.scout_routes).contains(&rules.mentored_scout_routes)
                && rules.riding_seasons > 0
                && rules.rider_battles > 0
                && rules.treatment_occasions > 0
                && (1..=rules.treatment_occasions).contains(&rules.mentored_treatment_occasions)
                && rules.officer_encounters > 0
                && rules.officer_command_facts > 0
                && rules.mentorship_seasons > 0
                && rules.specialization_steps == 2,
            "incomplete ordinary classes, costs or thresholds",
        )?;
        for (class, cost, facility, horses) in [
            (Infantry, 20, Facility::TrainingGround, false),
            (Archer, 20, Facility::TrainingGround, false),
            (Scout, 20, Facility::TrainingGround, false),
            (Cavalry, 40, Facility::Stable, true),
            (Medic, 20, Facility::Infirmary, false),
            (Officer, 50, Facility::TrainingGround, false),
        ] {
            let course = &rules.courses[&class];
            require(
                "progression.json",
                "careers.courses",
                course.gold_cost == cost
                    && course.facility == facility
                    && course.requires_horse_access == horses,
                "ordinary course has an unsupported price or facility",
            )?;
        }
        require(
            "progression.json",
            "careers.courses.cavalry",
            rules.courses[&Cavalry].requires_horse_access
                && rules.courses[&Cavalry].facility == Facility::Stable,
            "Cavalry requires a horse-access Stable",
        )
    }

    fn validate_specializations(&self) -> Result<(), String> {
        use FormationSpecialization::*;
        require(
            "progression.json",
            "specializations",
            self.specializations.len() == 3
                && [ShieldGuard, Pikemen, LightCavalry]
                    .iter()
                    .all(|kind| self.specializations.contains_key(kind)),
            "requires all three ordinary formation specializations",
        )?;
        for (kind, rule) in &self.specializations {
            let valid_requirement = match rule.requirement {
                SpecializationRequirement::DefendedAnchors { occasions } => occasions > 0,
                SpecializationRequirement::MeaningfulAgainst { occasions, .. } => occasions > 0,
                SpecializationRequirement::RoutesAndRetreatingVictories { routes, victories } => {
                    routes > 0 && victories > 0
                }
            };
            require(
                "progression.json",
                "specializations.rule",
                !rule.sources.is_empty()
                    && rule.gold_cost > 0
                    && rule.course_steps == self.careers.specialization_steps
                    && valid_requirement
                    && rule
                        .defense_resistance_permille
                        .is_none_or(|value| (1000..=5000).contains(&value))
                    && rule
                        .counter
                        .as_ref()
                        .is_none_or(|counter| (1000..=5000).contains(&counter.permille))
                    && rule
                        .movement_allowance
                        .is_none_or(|value| (1..=100).contains(&value)),
                "invalid sources, course or effect",
            )?;
            let effects_match = match kind {
                ShieldGuard => {
                    rule.defense_resistance_permille == Some(1100)
                        && rule.sources == [TroopKind::Warriors]
                        && matches!(
                            rule.requirement,
                            SpecializationRequirement::DefendedAnchors { occasions: 3 }
                        )
                        && rule.counter.is_none()
                        && rule.movement_allowance.is_none()
                }
                Pikemen => {
                    rule.defense_resistance_permille.is_none()
                        && rule.sources == [TroopKind::Warriors, TroopKind::Spearmen]
                        && matches!(
                            rule.requirement,
                            SpecializationRequirement::MeaningfulAgainst {
                                troop: TroopKind::Riders,
                                occasions: 3
                            }
                        )
                        && rule.counter.as_ref().is_some_and(|counter| {
                            counter.target == TroopKind::Riders && counter.permille == 1750
                        })
                        && rule.movement_allowance.is_none()
                }
                LightCavalry => {
                    rule.defense_resistance_permille.is_none()
                        && rule.sources == [TroopKind::Riders]
                        && matches!(
                            rule.requirement,
                            SpecializationRequirement::RoutesAndRetreatingVictories {
                                routes: 6,
                                victories: 2
                            }
                        )
                        && rule.counter.is_none()
                        && rule.movement_allowance == Some(9)
                }
            };
            require(
                "progression.json",
                "specializations.effect",
                effects_match,
                "does not match its supported effect",
            )?;
        }
        Ok(())
    }
}

fn invalid_name(value: &str) -> bool {
    value.trim().is_empty()
        || value.trim() != value
        || value.chars().count() > 48
        || value.chars().any(char::is_control)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRules {
    pub recent_service_rounds: u32,
    pub detail_max_age_rounds: u32,
    pub detail_max_entries: usize,
    pub notable_max_age_rounds: u32,
    pub notable_max_entries: usize,
    pub knowledge_max_age_rounds: u32,
    pub knowledge_max_entries: usize,
    pub departed_max_age_rounds: u32,
    pub departed_max_entries: usize,
    pub page_size: usize,
}

impl HistoryRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.recent_service_rounds == 0
            || self.recent_service_rounds > 80
            || self.detail_max_age_rounds == 0
            || self.detail_max_age_rounds > self.notable_max_age_rounds
            || self.detail_max_entries == 0
            || self.detail_max_entries > 10000
            || self.notable_max_entries == 0
            || self.notable_max_entries > 12
            || self.notable_max_age_rounds > 80
            || self.knowledge_max_age_rounds == 0
            || self.knowledge_max_age_rounds > 80
            || self.knowledge_max_entries == 0
            || self.knowledge_max_entries > 10000
            || self.departed_max_age_rounds == 0
            || self.departed_max_age_rounds > 80
            || self.departed_max_entries == 0
            || self.departed_max_entries > 2000
            || self.page_size != 50
        {
            return Err("history_rules.json: invalid retention budget or page size".into());
        }
        Ok(())
    }
}
