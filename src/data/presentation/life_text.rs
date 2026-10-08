//! The same life-event wording appears in dated history and accepted-action notices.
use super::GameTextData;
use crate::{
    data::{
        economy::TroopKind,
        progression::{EpithetFact, TrainingDiscipline},
        world::PersonClass,
    },
    state::{
        history::LifeEvent,
        relationships::{FamilyOrigin, HouseholdEndReason},
    },
};

impl GameTextData {
    pub fn life_event_text(&self, event: &LifeEvent) -> String {
        let (key, detail) = match event {
            LifeEvent::Emerged { troop } => {
                ("life_emerged", self.text(troop_key(*troop)).to_owned())
            }
            LifeEvent::Arrived { origin } => (
                match origin {
                    FamilyOrigin::Birth => "life_born",
                    FamilyOrigin::AdoptedWard => "life_adopted",
                    FamilyOrigin::LocalApprentice => "life_apprentice",
                },
                String::new(),
            ),
            LifeEvent::Recognized { epithet, cause } => (
                "life_recognized",
                format!("{epithet} / {}", self.text(deed_key(*cause))),
            ),
            LifeEvent::ClassCompleted { class } => {
                ("life_class", self.text(class_key(*class)).to_owned())
            }
            LifeEvent::MentorshipStarted { discipline, .. } => (
                "life_mentorship_started",
                self.text(discipline_key(*discipline)).to_owned(),
            ),
            LifeEvent::MentorshipCompleted { discipline, .. } => (
                "life_mentorship_completed",
                self.text(discipline_key(*discipline)).to_owned(),
            ),
            LifeEvent::HouseholdFormed { .. } => ("life_household_formed", String::new()),
            LifeEvent::HouseholdEnded { reason, .. } => (
                "life_household_ended",
                self.text(match reason {
                    HouseholdEndReason::Chosen => "life_chosen",
                    HouseholdEndReason::PartnerDied => "life_partner_died",
                    HouseholdEndReason::SiteCaptured => "life_site_captured",
                    HouseholdEndReason::SiteAbandoned => "life_site_abandoned",
                    HouseholdEndReason::FactionDefeated => "life_faction_defeated",
                })
                .to_owned(),
            ),
            LifeEvent::ServiceEntered => ("life_service", String::new()),
            LifeEvent::Retired => ("life_retired", String::new()),
            LifeEvent::NaturalDeath => ("life_natural_death", String::new()),
        };
        self.text(key).replace("{detail}", &detail)
    }
}

fn class_key(class: PersonClass) -> &'static str {
    match class {
        PersonClass::Recruit => "class_recruit",
        PersonClass::Infantry => "class_infantry",
        PersonClass::Archer => "class_archer",
        PersonClass::Scout => "class_scout",
        PersonClass::Cavalry => "class_cavalry",
        PersonClass::Medic => "class_medic",
        PersonClass::Officer => "class_officer",
    }
}
fn discipline_key(discipline: TrainingDiscipline) -> &'static str {
    match discipline {
        TrainingDiscipline::Infantry => "discipline_infantry",
        TrainingDiscipline::Archery => "discipline_archery",
        TrainingDiscipline::Scouting => "discipline_scouting",
        TrainingDiscipline::Riding => "discipline_riding",
        TrainingDiscipline::Medicine => "discipline_medicine",
        TrainingDiscipline::Command => "discipline_command",
    }
}
fn troop_key(troop: TroopKind) -> &'static str {
    match troop {
        TroopKind::Warriors => "troop_warriors",
        TroopKind::Spearmen => "troop_spearmen",
        TroopKind::Archers => "troop_archers",
        TroopKind::Riders => "troop_riders",
        TroopKind::Medics => "troop_medics",
        TroopKind::SiegeEngines => "troop_siege_engines",
    }
}
fn deed_key(deed: EpithetFact) -> &'static str {
    match deed {
        EpithetFact::SurvivedOutnumbered => "deed_outnumbered",
        EpithetFact::DefendedAnchor => "deed_defended_anchor",
        EpithetFact::CapturedAnchor => "deed_captured_anchor",
        EpithetFact::TreatedWounded => "deed_treated_wounded",
        EpithetFact::AssumedCommand => "deed_assumed_command",
        EpithetFact::CommandedVictory => "deed_commanded_victory",
        EpithetFact::BattleService => "deed_battle_service",
    }
}
