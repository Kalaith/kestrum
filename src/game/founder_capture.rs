//! Founding lord verification uses the production new-campaign flow.

use super::*;
use kestrum::{
    data::{
        economy::TroopKind,
        progression::EpithetFact,
        world::{PersonClass, SiteId},
    },
    state::{
        evidence::EvidenceKind,
        military::{Army, ArmyId, Formation, FormationId},
        people::{EmergenceRecord, PersonAssignment, PersonId, PersonStatus, Recognition},
        CampaignPhase, StrategicCampaign,
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
                | "formation_hero_earned"
                | "formation_hero_progress"
                | "formation_hero_detail"
        ) {
            return false;
        }
        self.setup.seed = self.data.production_layout.default_seed;
        if name == "founder_long_name" {
            self.setup.kingdom_name = "W".repeat(self.data.rules.kingdom_name_max_chars);
        }
        self.start_game();
        if matches!(
            name,
            "formation_hero_earned" | "formation_hero_progress" | "formation_hero_detail"
        ) {
            self.capture_earned_hero(name);
            return true;
        }
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
                companion.appearance = engine::portraits::allocate_for_person(
                    campaign,
                    &self.data.portraits,
                    companion.id,
                )
                .expect("capture companion appearance");
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

    fn capture_earned_hero(&mut self, scene: &str) {
        let person = {
            let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
                return;
            };
            let personal_engagements = if scene == "formation_hero_progress" {
                self.data
                    .progression
                    .recognition
                    .personal_engagements
                    .saturating_sub(1)
                    .max(1)
            } else {
                self.data.progression.recognition.personal_engagements
            };
            earn_capture_hero(&self.data, campaign, personal_engagements)
        };
        self.invalidate_projection();
        let site = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.armies.get(&ArmyId(1)))
            .map(|army| army.site)
            .expect("capture Hero army");
        self.open_armies(site);
        if scene != "formation_hero_earned" {
            self.apply(UiAction::ArmyPeople);
            self.apply(UiAction::OpenPersonProgression(person));
            assert_eq!(self.army.mode, ui::ArmyMode::ProgressionPerson(person));
        }
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

fn earn_capture_hero(
    data: &GameData,
    campaign: &mut StrategicCampaign,
    personal_engagements: u8,
) -> PersonId {
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign
        .people
        .retain(|id, _| [PersonId(1), PersonId(3)].contains(id));
    campaign.legacy_items.retain(|_, item| {
        !matches!(item.custody, kestrum::state::legacy::LegacyItemCustody::Person(id)
            if !campaign.people.contains_key(&id))
    });
    for (army_id, formation_id) in [(1, 1), (3, 7)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).unwrap();
        army.commander = None;
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
    }
    let founder = campaign
        .people
        .values()
        .find(|person| person.faction == campaign.player)
        .expect("capture founding lord")
        .id;
    campaign.people.get_mut(&founder).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(1) };
    campaign.validate(data).expect("valid Hero capture setup");
    engine::apply(
        campaign,
        data,
        engine::Actor::Player,
        Command::DeclareWar {
            faction: kestrum::data::world::FactionId(3),
        },
    )
    .expect("accepted capture war declaration");

    for _ in 0..2 {
        capture_accepted_encounter(campaign, data);
        finish_capture_round(campaign, data);
    }
    let apprentice = campaign
        .people
        .values()
        .find(|person| person.career.emergence.is_some())
        .expect("accepted formation service creates an apprentice")
        .id;
    assert_eq!(campaign.people[&apprentice].career.hero_service_progress, 0);
    for _ in 0..personal_engagements {
        capture_accepted_encounter(campaign, data);
        finish_capture_round(campaign, data);
    }
    let hero = &campaign.people[&apprentice];
    assert_eq!(
        hero.career.recognition.is_some(),
        personal_engagements >= data.progression.recognition.personal_engagements
    );
    assert_eq!(
        hero.career.hero_service_progress,
        personal_engagements.min(data.progression.recognition.personal_engagements)
    );
    assert_eq!(
        hero.assignment,
        PersonAssignment::Formation {
            formation: FormationId(1)
        }
    );
    apprentice
}

fn capture_accepted_encounter(campaign: &mut StrategicCampaign, data: &GameData) {
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.headcount = formation.capacity;
    formation.movement_spent = 0;
    let formation_ids: Vec<_> = campaign.armies[&ArmyId(1)].formation_ids().collect();
    for id in formation_ids {
        campaign.formations.get_mut(&id).unwrap().movement_spent = 0;
    }
    for person in campaign.people.values_mut() {
        person.movement_spent = 0;
    }
    let opponent = campaign
        .armies
        .values()
        .find(|army| army.faction == kestrum::data::world::FactionId(3))
        .map(|army| army.id);
    let (army, formation) = if let Some(id) = opponent {
        let army = campaign.armies.get_mut(&id).unwrap();
        army.site = SiteId(6);
        (id, army.formation_ids().next().unwrap())
    } else {
        let army = campaign.next_ids.army;
        let formation = campaign.next_ids.formation;
        campaign.next_ids.army.0 += 1;
        campaign.next_ids.formation.0 += 1;
        campaign.armies.insert(
            army,
            Army {
                id: army,
                faction: kestrum::data::world::FactionId(3),
                name: "Fresh opposing host".into(),
                site: SiteId(6),
                slots: [Some(formation), None, None, None, None, None],
                commander: None,
                battle_doctrine: None,
            },
        );
        campaign.formations.insert(
            formation,
            Formation {
                battle_leader: None,
                tactics: None,
                tactics_override: Some(false),
                id: formation,
                faction: kestrum::data::world::FactionId(3),
                kind: TroopKind::Warriors,
                headcount: 100,
                capacity: 100,
                movement_spent: 0,
                created_round: campaign.completed_rounds,
                service: Default::default(),
            },
        );
        (army, formation)
    };
    campaign.armies.get_mut(&army).unwrap().site = SiteId(6);
    campaign.formations.get_mut(&formation).unwrap().headcount = 100;
    campaign
        .formations
        .get_mut(&formation)
        .unwrap()
        .movement_spent = 0;
    campaign
        .set_site_control(
            data,
            SiteId(6),
            Some(kestrum::data::world::FactionId(3)),
            false,
        )
        .expect("set capture enemy site");
    let moved = engine::apply(
        campaign,
        data,
        engine::Actor::Player,
        Command::Move(kestrum::engine::MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(5), SiteId(6)],
        }),
    )
    .expect("accepted capture movement");
    assert!(moved.battle_pending);
    engine::apply(
        campaign,
        data,
        engine::Actor::Player,
        Command::StartPendingBattle,
    )
    .expect("accepted capture battle");
}

fn finish_capture_round(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(campaign, data, engine::Actor::Player, Command::EndTurn)
        .expect("end capture player turn");
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        engine::apply(
            campaign,
            data,
            engine::Actor::Npc(faction),
            Command::EndTurn,
        )
        .expect("pass capture NPC turn");
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
        u32::from(
            data.progression
                .recognition
                .personal_engagements
                .saturating_sub(1)
                .max(1),
        )
    } else {
        u32::from(data.progression.recognition.personal_engagements)
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
        archer.appearance =
            engine::portraits::allocate_for_person(campaign, &data.portraits, archer.id)
                .expect("capture archer appearance");
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
