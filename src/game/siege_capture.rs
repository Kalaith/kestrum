//! Authored starting armies establish and resolve every captured siege by commands.

use super::*;
use kestrum::{
    data::world::{FactionId, MarkerId, SiteId},
    engine::{Actor, MoveOrder},
    navigation::MapSelection,
    state::{
        legacy::LegacyItemCustody,
        military::{ArmyId, FormationId},
        people::PersonId,
        siege::{SiegeAction, SiegeOrder},
        CampaignPhase, StrategicCampaign,
    },
};

impl Game {
    pub(super) fn capture_siege_scene(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        if scene == "help_siege" {
            self.state.overlay = Overlay::Help;
            self.help_page = 7;
            return true;
        }
        if !scene.starts_with("siege_") {
            return false;
        }
        let defending = scene.contains("defender")
            || scene.contains("sortie")
            || scene.contains("escape")
            || scene.contains("relief")
            || scene == "siege_no_exit"
            || scene == "siege_damage_afterlift";
        let mut campaign = fixture(&self.data, defending);
        if scene == "siege_dense" {
            dense_armies(&mut campaign);
        }
        if scene.contains("relief") {
            relief_army(&mut campaign, &self.data);
        }
        if scene == "siege_no_exit" {
            for site in campaign.world.adjacent_sites(SiteId(9)) {
                campaign
                    .set_site_control(&self.data, site, Some(FactionId(3)), false)
                    .expect("blocked exit");
            }
        }
        if defending && scene != "siege_no_exit" {
            campaign
                .set_site_control(&self.data, SiteId(10), Some(FactionId(1)), false)
                .expect("exit");
        }
        campaign
            .validate(&self.data)
            .expect("valid authored siege scenario");
        establish(&mut campaign, &self.data, defending);
        if defending {
            finish_npcs(&mut campaign, &self.data);
        } else if !matches!(scene, "siege_besieger" | "siege_exhausted" | "siege_dense") {
            round(&mut campaign, &self.data);
        }
        if scene == "siege_progress" {
            round(&mut campaign, &self.data);
            round(&mut campaign, &self.data);
        }
        self.state
            .load_campaign(Campaign::Strategic(Box::new(campaign)), &self.data)
            .expect("load actual siege snapshot");
        self.invalidate_projection();
        if scene.contains("relief") {
            self.capture_relief_scene(scene);
        } else if scene == "siege_damage_afterlift" {
            self.capture_lifted_siege();
        } else {
            self.capture_siege_choice(scene);
        }
        true
    }

    fn capture_siege_choice(&mut self, scene: &str) {
        self.open_siege(SiteId(9));
        match scene {
            "siege_besieger" | "siege_defender" | "siege_progress" => {}
            "siege_dense" => self.apply_siege_action(UiAction::SiegePage(1)),
            "siege_orders" | "siege_defender_orders" => {
                self.apply_siege_action(UiAction::SiegeMode(ui::SiegeMode::Orders))
            }
            "siege_maintain" => {
                self.apply_siege_action(UiAction::ChooseSiegeAction(SiegeAction::Maintain))
            }
            "siege_assault"
            | "siege_exhausted"
            | "siege_assault_report"
            | "siege_assault_positions"
            | "siege_factors" => {
                self.apply_siege_action(UiAction::ChooseSiegeAction(SiegeAction::Assault))
            }
            "siege_withdraw" | "siege_withdraw_review" => {
                self.apply_siege_action(UiAction::ChooseSiegeAction(SiegeAction::Withdraw))
            }
            "siege_sortie" | "siege_sortie_report" | "siege_sortie_positions" => {
                self.apply_siege_action(UiAction::ChooseSiegeAction(SiegeAction::Sortie))
            }
            "siege_escape"
            | "siege_escape_review"
            | "siege_escape_report"
            | "siege_escape_positions"
            | "siege_no_exit" => {
                self.apply_siege_action(UiAction::ChooseSiegeAction(SiegeAction::Escape))
            }
            _ => panic!("unknown siege capture {scene}"),
        }
        if scene == "siege_withdraw_review" {
            self.apply_siege_action(UiAction::SiegeDestination(SiteId(8)));
        }
        if matches!(
            scene,
            "siege_escape_review" | "siege_escape_report" | "siege_escape_positions"
        ) {
            self.apply_siege_action(UiAction::SiegeDestination(SiteId(10)));
        }
        if scene.ends_with("_report") || scene.ends_with("_positions") || scene == "siege_factors" {
            self.apply_siege_action(UiAction::ConfirmSiege);
            assert_eq!(
                self.state.overlay,
                Overlay::Battle,
                "real siege report: {}",
                self.siege.status
            );
            if scene.ends_with("_positions") {
                self.battle.page = 1;
            }
            if scene == "siege_factors" {
                self.battle.tab = ui::BattleTab::Factors;
            }
        }
    }

    fn capture_relief_scene(&mut self, scene: &str) {
        self.begin_move(ArmyId(5));
        self.choose_move_destination();
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("campaign");
        self.navigation
            .enter_region(&campaign.world, MarkerId(5), &mut self.view)
            .expect("region");
        self.navigation
            .select(&campaign.world, MapSelection::Site(SiteId(9)))
            .expect("relief target");
        self.select_move_destination(SiteId(9));
        assert!(
            self.movement.preview.is_some(),
            "relief preview: {}",
            self.movement.status
        );
        if scene == "siege_relief_review" {
            self.review_move();
        }
        if scene == "siege_relief_report" || scene == "siege_relief_positions" {
            self.review_move();
            self.confirm_move();
            assert_eq!(self.state.overlay, Overlay::Battle, "real relief report");
            if scene == "siege_relief_positions" {
                self.battle.page = 1;
            }
        }
    }

    fn capture_lifted_siege(&mut self) {
        let campaign = match self.state.campaign.as_mut().expect("campaign") {
            Campaign::Strategic(campaign) => campaign,
            Campaign::Shell(_) => unreachable!(),
        };
        round(campaign, &self.data);
        engine::apply(campaign, &self.data, Actor::Player, Command::EndTurn)
            .expect("turn to besieger");
        engine::advance_npc(campaign, &self.data).expect("pass Oak");
        engine::apply(
            campaign,
            &self.data,
            Actor::Npc(FactionId(3)),
            Command::Siege(SiegeOrder {
                site: SiteId(9),
                action: SiegeAction::Withdraw,
                armies: vec![ArmyId(3)],
                destination: Some(SiteId(8)),
            }),
        )
        .expect("withdraw actual besieger");
        finish_npcs(campaign, &self.data);
        assert!(!campaign.sieges.contains_key(&SiteId(9)));
        assert!(campaign.world.fort_damage[&SiteId(9)] > 0);
        self.open_settlement(SiteId(9));
        self.invalidate_projection();
    }
}

fn fixture(data: &GameData, defending: bool) -> StrategicCampaign {
    let mut campaign = StrategicCampaign::new(data).expect("campaign");
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign
        .people
        .retain(|id, _| [PersonId(1), PersonId(3)].contains(id));
    for item in campaign.legacy_items.values_mut() {
        let retained_custodian = match item.custody {
            LegacyItemCustody::Person(person) => campaign
                .people
                .get(&person)
                .is_some_and(|person| person.is_alive() && person.faction == item.faction),
            LegacyItemCustody::SiteEstate(_) => true,
        };
        if !retained_custodian {
            let estate = campaign
                .factions
                .get(&item.faction)
                .expect("item faction is retained")
                .headquarters;
            item.custody = LegacyItemCustody::SiteEstate(estate);
        }
    }
    for (army_id, formation_id) in [(1, 1), (3, 7)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).expect("army");
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
        army.commander = None;
        army.site = if (army_id == 1) == defending {
            SiteId(9)
        } else {
            SiteId(8)
        };
    }
    let defender = if defending {
        FactionId(1)
    } else {
        FactionId(3)
    };
    let besieger = if defending {
        FactionId(3)
    } else {
        FactionId(1)
    };
    for site in [5, 6, 8] {
        campaign
            .set_site_control(data, SiteId(site), Some(besieger), false)
            .expect("approach");
    }
    campaign
        .set_site_control(data, SiteId(9), Some(defender), false)
        .expect("fort ownership");
    campaign
}

fn establish(campaign: &mut StrategicCampaign, data: &GameData, defending: bool) {
    if defending {
        engine::apply(campaign, data, Actor::Player, Command::EndTurn)
            .expect("defender turn ended");
        while matches!(campaign.phase, CampaignPhase::NpcTurn { faction, .. } if faction != FactionId(3))
        {
            engine::advance_npc(campaign, data).expect("earlier rival action");
        }
        assert!(matches!(
            campaign.phase,
            CampaignPhase::NpcTurn {
                faction: FactionId(3),
                paused: false
            }
        ));
    }
    let actor = if defending {
        Actor::Npc(FactionId(3))
    } else {
        Actor::Player
    };
    let owner = if defending {
        FactionId(3)
    } else {
        FactionId(1)
    };
    let armies = campaign
        .armies
        .values()
        .filter(|army| army.faction == owner && army.site == SiteId(8))
        .map(|army| army.id)
        .collect();
    engine::apply(
        campaign,
        data,
        actor,
        Command::Move(MoveOrder {
            armies,
            path: vec![SiteId(8), SiteId(9)],
        }),
    )
    .expect("real defended-fort arrival");
    assert!(campaign.sieges.contains_key(&SiteId(9)));
}

fn round(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(campaign, data, Actor::Player, Command::EndTurn).expect("end siege season");
    finish_npcs(campaign, data);
}

fn finish_npcs(campaign: &mut StrategicCampaign, data: &GameData) {
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        engine::advance_npc(campaign, data).expect("rival turn");
    }
}

fn dense_armies(campaign: &mut StrategicCampaign) {
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(9))
        .expect("fort")
        .name = "The Hillfort of the Silver Hawthorns and Western Crossing".into();
    for number in 5..=10 {
        let mut army = campaign.armies[&ArmyId(1)].clone();
        let mut formation = campaign.formations[&FormationId(1)].clone();
        army.id = ArmyId(number);
        formation.id = FormationId(number + 8);
        army.name = format!("The Silver Hawthorn Regiment of the Northern Borderlands {number}");
        army.slots = [Some(formation.id), None, None, None, None, None];
        campaign.formations.insert(formation.id, formation);
        campaign.armies.insert(army.id, army);
    }
    campaign.next_ids.army = ArmyId(11);
    campaign.next_ids.formation = FormationId(19);
}

fn relief_army(campaign: &mut StrategicCampaign, data: &GameData) {
    let mut army = campaign.armies[&ArmyId(1)].clone();
    let mut formation = campaign.formations[&FormationId(1)].clone();
    army.id = ArmyId(5);
    army.site = SiteId(10);
    army.name = "Rose Relief from Milltown".into();
    formation.id = FormationId(13);
    army.slots = [Some(formation.id), None, None, None, None, None];
    campaign.formations.insert(formation.id, formation);
    campaign.armies.insert(army.id, army);
    campaign.next_ids.army = ArmyId(6);
    campaign.next_ids.formation = FormationId(14);
    campaign
        .set_site_control(data, SiteId(10), Some(FactionId(1)), false)
        .expect("relief start");
}
