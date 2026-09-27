//! History captures derive evidence through real orders, combat, and round boundaries.

use super::*;
use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, FactionId, SiteId},
    },
    engine::{Actor, MoveOrder},
    navigation::MapSelection,
    state::{
        evidence::Veterancy,
        history::HistorySubject,
        military::{ArmyId, FormationId},
        people::PersonAssignment,
        CampaignPhase, StrategicCampaign,
    },
};

impl Game {
    pub(super) fn capture_history(&mut self, scene: &str) {
        let scene = scene.trim_end_matches("_minimum");
        if scene == "history_presence" {
            self.capture_history_presence();
            return;
        }
        if matches!(scene, "history_seasoned" | "history_veteran") {
            self.capture_service_tier(scene == "history_veteran");
            return;
        }
        if scene == "history_dense" {
            self.capture_dense_history();
            return;
        }
        self.capture_battle("battle_people");
        if matches!(scene, "history_pruned" | "history_person") {
            let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
                return;
            };
            for _ in 0..if scene == "history_pruned" { 45 } else { 1 } {
                complete_round(campaign, &self.data);
            }
            if scene == "history_pruned" {
                assert!(campaign.battles.is_empty(), "old detailed reports expire");
            }
        }
        self.state.overlay = Overlay::Menu;
        match scene {
            "history_records" | "history_search" => {
                self.apply_history_action(UiAction::OpenRecords);
                if scene == "history_search" {
                    self.history.mode = ui::HistoryMode::Search;
                    self.history.search = "Mara".into();
                }
            }
            "history_known" | "history_pruned" => {
                self.open_history(HistorySubject::Person(kestrum::state::people::PersonId(3)))
            }
            "history_person" => {
                self.open_history(HistorySubject::Person(kestrum::state::people::PersonId(1)))
            }
            "history_empty" => {
                self.open_history(HistorySubject::Site(SiteId(4)));
                self.history.mode = ui::HistoryMode::Events;
            }
            "history_events" | "history_filters" => {
                self.open_history(HistorySubject::Army(ArmyId(1)));
                self.history.mode = if scene == "history_filters" {
                    ui::HistoryMode::Filters
                } else {
                    ui::HistoryMode::Events
                };
            }
            _ => panic!("Unknown history capture: {scene}"),
        }
    }

    fn capture_dense_history(&mut self) {
        self.capture_campaign();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        // Authored opening resources and a long label stress a real recruitment ledger.
        campaign
            .factions
            .get_mut(&campaign.player)
            .expect("player")
            .resources
            .gold = 100_000;
        campaign
            .factions
            .get_mut(&campaign.player)
            .expect("player")
            .resources
            .wood = 100_000;
        campaign.armies.get_mut(&ArmyId(1)).expect("army").name =
            "The Rose Ward of the Long Western March and Winter Crossing".into();
        campaign.validate(&self.data).expect("valid capture setup");
        for _ in 0..30 {
            let formation = campaign.next_ids.formation;
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::Recruit {
                    site: SiteId(1),
                    army: Some(ArmyId(1)),
                    kind: TroopKind::Warriors,
                },
            )
            .expect("real recruitment");
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::Disband { formation },
            )
            .expect("real disband");
        }
        self.state.overlay = Overlay::Menu;
        self.open_history(HistorySubject::Army(ArmyId(1)));
        self.history.mode = ui::HistoryMode::Events;
        self.apply_history_action(UiAction::HistoryPage(10));
        assert!(self
            .history
            .result
            .as_ref()
            .is_some_and(|page| page.page == 1));
    }

    fn capture_history_presence(&mut self) {
        self.capture_campaign();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        occupy_gate(campaign, &self.data);
        campaign
            .validate(&self.data)
            .expect("valid nearby hostile setup");
        let marker = campaign.world.site(SiteId(5)).expect("gate").marker;
        self.navigation
            .enter_region(&campaign.world, marker, &mut self.view)
            .expect("region");
        self.navigation
            .select(&campaign.world, MapSelection::Site(SiteId(5)))
            .expect("gate selection");
    }

    fn capture_service_tier(&mut self, veteran: bool) {
        self.capture_campaign();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        prepare_service_scenario(campaign, &self.data);
        let target = if veteran {
            Veterancy::Veteran
        } else {
            Veterancy::Seasoned
        };
        for _ in 0..12 {
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::Move(MoveOrder {
                    armies: vec![ArmyId(1)],
                    path: vec![SiteId(1), SiteId(5)],
                }),
            )
            .expect("real service battle");
            assert!(
                campaign.formations.contains_key(&FormationId(1)),
                "formation survives"
            );
            for _ in 0..5 {
                complete_round(campaign, &self.data);
            }
            if campaign.formations[&FormationId(1)].service.tier == target {
                break;
            }
            assert_eq!(
                campaign.armies[&ArmyId(1)].site,
                SiteId(1),
                "attacker retreats home"
            );
        }
        assert_eq!(campaign.formations[&FormationId(1)].service.tier, target);
        self.state.overlay = Overlay::Menu;
        self.open_history(HistorySubject::Formation(FormationId(1)));
    }
}

fn occupy_gate(campaign: &mut StrategicCampaign, data: &GameData) {
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .expect("relation")
        .state = DiplomaticState::War;
    campaign.armies.get_mut(&ArmyId(2)).expect("Oak army").site = SiteId(5);
    campaign
        .set_site_control(data, SiteId(5), Some(FactionId(2)), false)
        .expect("occupied gate");
}

fn prepare_service_scenario(campaign: &mut StrategicCampaign, data: &GameData) {
    occupy_gate(campaign, data);
    // This validated starting scenario has one anonymous formation on each side.
    // Every later loss, replenishment, and XP award comes from ordinary game commands.
    for (army_id, formation_id, site) in [
        (ArmyId(1), FormationId(1), SiteId(1)),
        (ArmyId(2), FormationId(4), SiteId(2)),
    ] {
        let army = campaign.armies.get_mut(&army_id).expect("army");
        let faction = army.faction;
        army.slots = [Some(formation_id), None, None, None, None, None];
        army.commander = None;
        campaign
            .formations
            .retain(|id, formation| formation.faction != faction || *id == formation_id);
        for person in campaign
            .people
            .values_mut()
            .filter(|person| person.faction == faction)
        {
            person.assignment = PersonAssignment::Site { site };
        }
        campaign
            .factions
            .get_mut(&faction)
            .expect("faction")
            .resources
            .gold = 100_000;
    }
    campaign
        .validate(data)
        .expect("valid service capture scenario");
}

fn complete_round(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(campaign, data, Actor::Player, Command::EndTurn).expect("end capture round");
    while !matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        engine::advance_npc(campaign, data).expect("capture rival phase");
    }
}
