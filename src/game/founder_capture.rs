//! Founding lord verification uses the production new-campaign flow.

use super::*;
use kestrum::{
    data::{economy::TroopKind, progression::EpithetFact, world::PersonClass},
    state::{
        evidence::EvidenceKind,
        people::{EmergenceRecord, PersonAssignment, PersonId, PersonStatus, Recognition},
        StrategicCampaign,
    },
};

impl Game {
    pub(super) fn capture_founder(&mut self, scene: &str) -> bool {
        let name = scene.trim_end_matches("_minimum");
        if !matches!(
            name,
            "founder_army"
                | "founder_people"
                | "founder_career"
                | "founder_dense"
                | "founder_long_name"
                | "formation_members"
                | "formation_emerged"
                | "formation_transfer"
                | "formation_transfer_blocked"
                | "formation_long_name"
        ) {
            return false;
        }
        self.setup.seed = self.data.production_layout.default_seed;
        if name == "founder_long_name" {
            self.setup.kingdom_name = "W".repeat(self.data.rules.kingdom_name_max_chars);
        }
        self.start_game();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return true;
        };
        let site = campaign.factions[&campaign.player].headquarters;
        let founder = campaign
            .people
            .values()
            .find(|person| person.faction == campaign.player)
            .expect("production founding lord")
            .id;
        if name == "formation_long_name" {
            campaign.people.get_mut(&founder).unwrap().name = "W".repeat(64);
        }
        if name.starts_with("formation_") {
            roster_archer(&self.data, campaign, name);
        }
        if matches!(
            name,
            "founder_dense" | "formation_members" | "formation_long_name"
        ) {
            // Later named companions share the army without inheriting noble status.
            for index in 0..5 {
                let mut companion = campaign.people[&founder].clone();
                companion.id = campaign.next_ids.person;
                campaign.next_ids.person = PersonId(companion.id.0 + 1);
                companion.name = format!(
                    "{} {}",
                    self.data.human_names.given_names[index],
                    self.data.human_names.family_names[index]
                );
                companion.career.founding_lord = false;
                if campaign
                    .available_person_formation(campaign.player, site)
                    .is_none()
                {
                    let army = campaign
                        .armies
                        .values()
                        .filter(|army| {
                            army.faction == campaign.player
                                && army.site == site
                                && army.first_empty_slot().is_some()
                        })
                        .map(|army| army.id)
                        .min();
                    engine::apply(
                        campaign,
                        &self.data,
                        engine::Actor::Player,
                        Command::Recruit {
                            site,
                            army,
                            kind: TroopKind::Warriors,
                        },
                    )
                    .expect("funded capture formation for each companion");
                }
                companion.assignment = PersonAssignment::Formation {
                    formation: campaign
                        .available_person_formation(campaign.player, site)
                        .expect("separate companion slot"),
                };
                if name == "formation_members" && index == 0 {
                    companion.status = PersonStatus::Wounded {
                        since_round: 0,
                        remaining_steps: 2,
                    };
                }
                campaign.people.insert(companion.id, companion);
            }
        }
        campaign
            .validate(&self.data)
            .expect("valid founding capture");
        self.invalidate_projection();
        self.open_armies(site);
        self.army.mode = match name {
            "founder_people" | "founder_dense" => ui::ArmyMode::People,
            "founder_career" | "founder_long_name" => ui::ArmyMode::ProgressionPerson(founder),
            _ => ui::ArmyMode::Roster,
        };
        if name == "formation_transfer_blocked" {
            self.capture_staffed_transfer(founder);
        }
        true
    }

    fn capture_staffed_transfer(&mut self, founder: PersonId) {
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        let formation = campaign
            .people
            .values()
            .find_map(|person| match person.assignment {
                PersonAssignment::Formation { formation }
                    if person.id != founder && person.faction == campaign.player =>
                {
                    Some(formation)
                }
                _ => None,
            })
            .expect("staffed capture formation");
        let army = campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .unwrap()
            .id;
        self.begin_transfer(ui::TransferSubject::Person(founder));
        self.apply(UiAction::SelectTransferArmy(army));
        self.apply(UiAction::SelectTransferFormation(formation));
        assert!(self.army.transfer.blocked.is_some());
    }
}

fn roster_archer(data: &GameData, campaign: &mut StrategicCampaign, scene: &str) {
    let founder = campaign
        .people
        .values()
        .find(|person| person.faction == campaign.player && person.career.founding_lord)
        .expect("founding lord")
        .clone();
    let archers = campaign
        .formations
        .values()
        .find(|formation| {
            formation.faction == campaign.player && formation.kind == TroopKind::Archers
        })
        .expect("starting archers")
        .id;
    let site = campaign.factions[&campaign.player].headquarters;
    let mut archer = founder.clone();
    archer.id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(archer.id.0 + 1);
    archer.name = if matches!(scene, "formation_long_name" | "formation_transfer_blocked") {
        "W".repeat(64)
    } else {
        format!(
            "{} {}",
            data.human_names.given_names[5], data.human_names.family_names[5]
        )
    };
    archer.class = PersonClass::Archer;
    archer.assignment = PersonAssignment::Formation { formation: archers };
    archer.career = Default::default();
    archer.career.emergence = Some(EmergenceRecord {
        completed_rounds: 0,
        source_formation: archers,
        source_troop: TroopKind::Archers,
        site,
        distinguishing_deed: Some(EpithetFact::SurvivedOutnumbered),
    });
    let encounters = if scene == "formation_emerged" {
        data.progression
            .recognition
            .meaningful_encounters
            .saturating_sub(1)
            .max(1)
    } else {
        data.progression.recognition.meaningful_encounters
    };
    archer
        .evidence
        .counts
        .insert(EvidenceKind::Battle, encounters);
    archer
        .evidence
        .counts
        .insert(EvidenceKind::MeaningfulEncounter, encounters);
    archer
        .evidence
        .counts
        .insert(EvidenceKind::SurvivedOutnumbered, 1);
    archer
        .evidence
        .service_by_troop
        .insert(TroopKind::Archers, encounters);
    if scene != "formation_emerged" {
        archer
            .career
            .notable_sites
            .insert(EpithetFact::SurvivedOutnumbered, site);
        archer.career.recognition = Some(Recognition {
            completed_rounds: 0,
            epithet: data.human_names.epithets[&EpithetFact::SurvivedOutnumbered].clone(),
            cause: EpithetFact::SurvivedOutnumbered,
            site,
        });
    }
    if scene != "formation_transfer" {
        campaign.people.insert(archer.id, archer);
    }
    if scene == "formation_transfer" {
        engine::apply(
            campaign,
            data,
            engine::Actor::Player,
            Command::TransferPerson {
                person: founder.id,
                to_formation: archers,
            },
        )
        .expect("transfer the actual founding lord to Archers");
    }
}
