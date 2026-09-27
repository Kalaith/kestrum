//! Durable army and logistics scenes, using real orders in isolated capture state.

use super::*;
use kestrum::{data::world::SiteId, state::military::ArmyId};

impl Game {
    pub(super) fn capture_progression(&mut self, scene: &str) {
        use kestrum::{
            data::{
                economy::TroopKind,
                progression::{EpithetFact, FormationSpecialization, TrainingDiscipline},
                world::{Facility, PersonClass, SiteId},
            },
            state::{
                evidence::{EvidenceKind, FormationCourse, Veterancy},
                mentorship::{Mentorship, MentorshipPauseReason, MentorshipStatus},
                military::FormationId,
                people::{
                    Disposition, EmergenceRecord, PersonAssignment, PersonCourse, PersonId,
                    PersonRelationship, PersonStatus, PersonTrait, Recognition, Tendency,
                },
            },
        };
        use std::collections::{BTreeMap, BTreeSet};

        self.capture_campaign();
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            let site = campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == SiteId(1))
                .expect("founding site");
            for facility in [
                Facility::TrainingGround,
                Facility::Stable,
                Facility::Infirmary,
            ] {
                if !site.facilities.contains(&facility) {
                    site.facilities.push(facility);
                }
            }

            let person = campaign.people.get_mut(&PersonId(1)).expect("founder");
            person.name = "Aveline Rose".into();
            person.class = PersonClass::Recruit;
            person.career.disposition = Disposition {
                courage: Tendency::Positive,
                care: Tendency::Positive,
                curiosity: Tendency::Positive,
            };
            person.career.emergence = Some(EmergenceRecord {
                completed_rounds: 0,
                source_formation: FormationId(1),
                source_troop: TroopKind::Warriors,
                site: SiteId(1),
                distinguishing_deed: Some(EpithetFact::SurvivedOutnumbered),
            });
            person.evidence.counts = BTreeMap::from([
                (EvidenceKind::Battle, 5),
                (EvidenceKind::MeaningfulEncounter, 5),
                (EvidenceKind::SurvivedOutnumbered, 2),
                (EvidenceKind::DefendedAnchor, 3),
                (EvidenceKind::TreatedWounded, 2),
                (EvidenceKind::AssumedCommand, 1),
                (EvidenceKind::CommandedVictory, 1),
            ]);
            person.evidence.service_by_troop = BTreeMap::from([
                (TroopKind::Warriors, 2),
                (TroopKind::Archers, 2),
                (TroopKind::Riders, 1),
            ]);
            person.evidence.traversed_routes = campaign
                .world
                .routes
                .iter()
                .take(6)
                .map(|route| route.id)
                .collect();
            person.career.notable_sites = BTreeMap::from([
                (EpithetFact::SurvivedOutnumbered, SiteId(6)),
                (EpithetFact::DefendedAnchor, SiteId(6)),
            ]);
            person.career.traits = BTreeSet::from([
                PersonTrait::Bold,
                PersonTrait::Protective,
                PersonTrait::NaturalCommander,
            ]);
            person.career.recognition = Some(Recognition {
                completed_rounds: 0,
                epithet: self.data.human_names.epithets[&EpithetFact::SurvivedOutnumbered].clone(),
                cause: EpithetFact::SurvivedOutnumbered,
                site: SiteId(6),
            });
            person.career.course = (scene.trim_end_matches("_minimum") == "career_training")
                .then_some(PersonCourse::Class {
                    target: PersonClass::Archer,
                    site: SiteId(1),
                    steps_completed: 1,
                });
            let mut companion = person.clone();
            companion.id = PersonId(4);
            companion.name = "Mira Reed".into();
            companion.birth_round = -80;
            companion.career = Default::default();
            companion.career.relationships.insert(
                PersonId(1),
                PersonRelationship {
                    shared_service_seasons: 1,
                    last_shared_service_round: Some(0),
                    mutual_combat_rounds: 0,
                    last_mutual_combat_round: None,
                },
            );
            companion.evidence = Default::default();
            companion.assignment = PersonAssignment::Formation {
                formation: FormationId(1),
            };
            campaign.people.insert(PersonId(4), companion);
            campaign.next_ids.person = PersonId(5);
            campaign
                .people
                .get_mut(&PersonId(1))
                .expect("founder")
                .career
                .relationships
                .insert(
                    PersonId(4),
                    PersonRelationship {
                        shared_service_seasons: 1,
                        last_shared_service_round: Some(0),
                        mutual_combat_rounds: 0,
                        last_mutual_combat_round: None,
                    },
                );

            let capture_kind = scene.trim_end_matches("_minimum");
            if matches!(
                capture_kind,
                "lifecycle" | "lifecycle_wounded" | "mentorship" | "mentorship_paused"
            ) {
                campaign.completed_rounds = 4;
                let mentor = campaign.people.get_mut(&PersonId(1)).expect("founder");
                mentor.class = PersonClass::Officer;
                mentor.career.course = None;
                mentor.career.emergence = None;
                mentor.birth_round = if capture_kind == "lifecycle" {
                    -220
                } else {
                    -120
                };
                mentor
                    .career
                    .discipline_service_seasons
                    .insert(TrainingDiscipline::Command, 4);
                if capture_kind == "mentorship_paused" {
                    mentor.status = PersonStatus::Wounded {
                        since_round: 4,
                        remaining_steps: 2,
                    };
                    campaign.mentorships.insert(
                        PersonId(4),
                        Mentorship {
                            mentor: PersonId(1),
                            discipline: TrainingDiscipline::Command,
                            started_round: 3,
                            seasons_completed: 0,
                            status: MentorshipStatus::Paused {
                                reason: MentorshipPauseReason::Wounded,
                            },
                        },
                    );
                }
                if capture_kind == "lifecycle_wounded" {
                    mentor.assignment = PersonAssignment::Site { site: SiteId(1) };
                    mentor.status = PersonStatus::Wounded {
                        since_round: 4,
                        remaining_steps: 2,
                    };
                }
            }

            let formation = campaign
                .formations
                .get_mut(&FormationId(1))
                .expect("founding formation");
            formation.service.xp = 12;
            formation.service.tier = Veterancy::Seasoned;
            formation.service.ledger.counts = BTreeMap::from([
                (EvidenceKind::Battle, 5),
                (EvidenceKind::MeaningfulEncounter, 5),
                (EvidenceKind::DefendedAnchor, 3),
                (EvidenceKind::RetreatingEnemyVictory, 2),
            ]);
            formation.service.ledger.service_by_troop = BTreeMap::from([(TroopKind::Warriors, 5)]);
            formation.service.ledger.meaningful_against = BTreeMap::from([(TroopKind::Riders, 3)]);
            formation
                .service
                .ledger
                .encountered_troops
                .insert(TroopKind::Riders);
            formation.service.ledger.traversed_routes = campaign
                .world
                .routes
                .iter()
                .take(6)
                .map(|route| route.id)
                .collect();
            formation.service.course = Some(FormationCourse {
                target: FormationSpecialization::ShieldGuard,
                site: SiteId(1),
                steps_completed: 1,
            });
            for army in campaign.armies.values_mut() {
                army.commander = None;
            }
            campaign
                .validate(&self.data)
                .expect("valid progression capture");
        }
        self.open_armies(SiteId(1));
        self.army.mode = match scene.trim_end_matches("_minimum") {
            "mentorship" | "mentorship_paused" => ui::ArmyMode::Mentorship(PersonId(4)),
            "lifecycle" | "lifecycle_wounded" | "career" | "career_training" => {
                ui::ArmyMode::ProgressionPerson(PersonId(1))
            }
            _ => ui::ArmyMode::ProgressionFormation(FormationId(1)),
        };
        self.army.selected = Some(FormationId(1));
    }

    pub(super) fn capture_logistics(&mut self, scene: &str) {
        use kestrum::{
            data::{economy::TroopKind, world::MarkerId},
            engine::Actor,
        };
        self.capture_campaign();
        let mut army = ArmyId(1);
        let mut added = None;
        let scene = scene.trim_end_matches("_minimum");
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            let recruits = if scene == "move_group_dense" {
                7
            } else if scene.starts_with("transfer") || scene == "move_exhausted" {
                1
            } else {
                0
            };
            for _ in 0..recruits {
                added = engine::apply(
                    campaign,
                    &self.data,
                    Actor::Player,
                    Command::Recruit {
                        site: SiteId(1),
                        army: None,
                        kind: TroopKind::Warriors,
                    },
                )
                .expect("affordable separate capture army")
                .recruited
                .map(|result| result.army);
            }
            if scene == "move_exhausted" {
                army = added.expect("new army");
            }
            if matches!(scene, "army_recovery" | "army_recovered" | "army_cutoff") {
                let formation = campaign.armies[&army]
                    .formation_ids()
                    .next()
                    .expect("founding force");
                campaign
                    .formations
                    .get_mut(&formation)
                    .expect("formation")
                    .headcount = 21;
                if scene == "army_cutoff" {
                    campaign
                        .set_site_control(&self.data, SiteId(1), Some(campaign.player), true)
                        .expect("contested supply root");
                }
            }
            if matches!(scene, "army_recovered" | "army_paused") {
                engine::apply(campaign, &self.data, Actor::Player, Command::EndTurn)
                    .expect("capture ends player turn");
                if scene == "army_recovered" {
                    while !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
                        engine::advance_npc(campaign, &self.data).expect("capture completes round");
                    }
                }
            }
        }
        self.open_armies(SiteId(1));
        if scene == "army_people" {
            self.army.mode = ui::ArmyMode::People;
        }
        if matches!(
            scene,
            "army_recovery" | "army_recovered" | "army_cutoff" | "army_paused"
        ) {
            self.army.mode = ui::ArmyMode::Orders;
        }
        if scene.starts_with("move_") {
            self.begin_move(army);
            if scene == "move_group_dense" {
                self.movement.armies = self.local_armies();
            } else if scene != "move_group" {
                self.choose_move_destination();
                self.enter_region(MarkerId(5));
                if scene != "move_map" {
                    self.select_map(kestrum::navigation::MapSelection::Site(
                        if scene == "move_blocked" {
                            SiteId(9)
                        } else {
                            SiteId(6)
                        },
                    ));
                    if scene == "move_review" {
                        self.review_move();
                    }
                }
            }
        } else if scene.starts_with("transfer") {
            let subject = if scene == "transfer_person" {
                ui::TransferSubject::Person(kestrum::state::people::PersonId(1))
            } else {
                ui::TransferSubject::Formation(self.army.selected.expect("founding formation"))
            };
            self.begin_transfer(subject);
            if matches!(scene, "transfer_slots" | "transfer_person") {
                self.army.transfer.army = added;
                if scene == "transfer_slots" {
                    self.army.transfer.slot = Some(1);
                } else if let Some(Campaign::Strategic(campaign)) = &self.state.campaign {
                    self.army.transfer.formation =
                        added.and_then(|army| campaign.armies[&army].formation_ids().next());
                }
            }
            self.refresh_transfer();
        }
    }

    pub(super) fn capture_army(&mut self, scene: &str) {
        use kestrum::{data::economy::TroopKind, engine::Actor};
        self.capture_campaign();
        let scene = scene.trim_end_matches("_minimum");
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player)
                .expect("capture starts with the founding army")
                .id;
            if matches!(scene, "army_full" | "recruit_full") {
                for _ in 0..3 {
                    engine::apply(
                        campaign,
                        &self.data,
                        Actor::Player,
                        Command::Recruit {
                            site: SiteId(1),
                            army: Some(army),
                            kind: TroopKind::Warriors,
                        },
                    )
                    .expect("three affordable founding army slots");
                }
            }
            if matches!(scene, "army_empty" | "disband_last") {
                let ids: Vec<_> = campaign.armies[&army].formation_ids().collect();
                for (index, formation) in ids.into_iter().enumerate() {
                    if scene == "disband_last" && index == 0 {
                        continue;
                    }
                    engine::apply(
                        campaign,
                        &self.data,
                        Actor::Player,
                        Command::Disband { formation },
                    )
                    .expect("owned formation can be disbanded");
                }
            }
            if matches!(scene, "army_long_name" | "army_dense") {
                campaign.armies.get_mut(&army).expect("founding army").name =
                    "The Riverward Silver Hawthorn Regiment of the Northern Marches I".into();
                campaign
                    .people
                    .values_mut()
                    .find(|person| person.faction == campaign.player)
                    .expect("founder")
                    .name =
                    "Alexandria of the Silver Hawthorns and Northern River Marches II".into();
            }
            if matches!(scene, "army_economy" | "army_deficit" | "army_dense") {
                if matches!(scene, "army_deficit" | "army_dense") {
                    campaign
                        .factions
                        .get_mut(&campaign.player)
                        .expect("player")
                        .resources
                        .gold = 0;
                    campaign
                        .set_site_control(&self.data, SiteId(1), Some(campaign.player), true)
                        .expect("contested headquarters fixture");
                }
                engine::apply(campaign, &self.data, Actor::Player, Command::EndTurn)
                    .expect("player finishes capture round");
                while !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
                    engine::advance_npc(campaign, &self.data).expect("rivals finish capture round");
                }
            }
        }
        self.open_armies(SiteId(1));
        if matches!(scene, "recruit" | "recruit_blocked" | "recruit_full") {
            let army = self.local_armies().first().copied();
            self.begin_recruit(army);
            self.army.mode = ui::ArmyMode::Recruit {
                army,
                kind: Some(if scene == "recruit_blocked" {
                    TroopKind::Riders
                } else {
                    TroopKind::Warriors
                }),
            };
        }
        if matches!(scene, "disband" | "disband_last") {
            if let Some(formation) = self.army.selected {
                self.army.mode = ui::ArmyMode::Disband(formation);
            }
        }
    }
}
