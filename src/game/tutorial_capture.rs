//! Durable introductory states and assertions through the actual action dispatcher.

use super::*;
use kestrum::{
    data::economy::{Habitation, Resources},
    engine::HouseholdAction,
    navigation::{MapScope, MapSelection},
    state::tutorial::{TutorialProgress, TutorialStep},
};

impl Game {
    pub(super) fn capture_tutorial(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !scene.starts_with("tutorial_") {
            return false;
        }
        self.setup.seed = self.data.production_layout.default_seed;
        self.start_game();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("tutorial capture requires a production campaign");
        };
        campaign.tutorial = TutorialProgress::new();
        let capital = campaign.factions[&campaign.player].capital;
        let headquarters = campaign.factions[&campaign.player].headquarters;
        let capital_marker = campaign.world.site(capital).unwrap().marker;
        if scene == "tutorial_long_name" {
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == capital)
                .unwrap()
                .name = "The Riverward Settlement of Silver Hawthorns".into();
        }
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == campaign.player)
            .unwrap()
            .id;
        let person = campaign
            .people
            .values()
            .find(|person| person.faction == campaign.player)
            .unwrap()
            .id;
        let home_marker = campaign.world.site(headquarters).unwrap().marker;
        let (destination_marker, destination) = campaign
            .world
            .routes
            .iter()
            .filter_map(|route| route.major_connection)
            .filter_map(|[from, to]| {
                if from == home_marker {
                    Some(to)
                } else if to == home_marker {
                    Some(from)
                } else {
                    None
                }
            })
            .find_map(|marker| {
                let preview = engine::world_movement_preview(
                    campaign,
                    &self.data,
                    campaign.player,
                    &[army],
                    marker,
                )
                .ok()?;
                let destination = *preview.order.path.last()?;
                (preview.stop.is_none()
                    && preview.can_confirm()
                    && preview.reachable_site == destination)
                    .then_some((marker, destination))
            })
            .expect("production headquarters has a reachable neighbor");
        if scene == "tutorial_help" {
            self.apply(UiAction::DismissTutorial);
            self.apply(UiAction::Open(Overlay::Help));
            return true;
        }
        if matches!(scene, "tutorial_headquarters" | "tutorial_long_name") {
            return true;
        }

        self.apply(UiAction::TutorialHeadquarters);
        self.apply(UiAction::OpenSettlement(capital));
        self.assert_tutorial(TutorialStep::CityDevelopment);
        if scene == "tutorial_city_overview" {
            return true;
        }
        if scene == "tutorial_city_actions" {
            self.apply(UiAction::SettlementTab(ui::SettlementMode::LocalActions));
            return true;
        }
        if scene == "tutorial_city_blocked" {
            let cost = self.data.development.city_development.cost;
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().unwrap() else {
                unreachable!()
            };
            let player = campaign.player;
            campaign.factions.get_mut(&player).unwrap().resources = Resources {
                gold: cost.gold.saturating_sub(1),
                wood: cost.wood,
                stone: cost.stone,
            };
            self.invalidate_projection();
            self.refresh_projection();
        }
        self.apply(UiAction::SelectLocalAction(ui::LocalAction::DevelopCity));
        assert_eq!(self.settlement.site, Some(capital));
        assert_eq!(self.settlement.mode, ui::SettlementMode::LocalReview);
        let before_payment = {
            let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
            campaign.factions[&campaign.player].resources
        };
        let cost = self.data.development.city_development.cost;
        assert_eq!(
            self.settlement.city_development.as_ref().unwrap().cost,
            cost
        );
        if scene == "tutorial_city_review" {
            assert!(self.settlement.blocked.is_none());
            return true;
        }
        if scene == "tutorial_city_blocked" {
            assert!(self.settlement.blocked.is_some());
            let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
            assert_eq!(
                campaign.factions[&campaign.player].resources,
                before_payment
            );
            return true;
        }

        self.apply(UiAction::ConfirmLocalAction);
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        assert_eq!(
            campaign.world.site(capital).unwrap().habitation,
            Habitation::City
        );
        assert_eq!(
            campaign.factions[&campaign.player].resources,
            Resources {
                gold: before_payment.gold - cost.gold,
                wood: before_payment.wood - cost.wood,
                stone: before_payment.stone - cost.stone,
            }
        );
        assert!(campaign.tutorial.completed(TutorialStep::CityDevelopment));
        self.assert_tutorial(TutorialStep::Region);
        assert_eq!(self.state.overlay, Overlay::Settlement);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Overview);
        if matches!(scene, "tutorial_city_success" | "tutorial_region_return") {
            return true;
        }
        self.apply(UiAction::SettlementBack);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.navigation.scope(), MapScope::World);
        if scene == "tutorial_region_select" {
            assert_eq!(
                self.navigation.selection(),
                Some(MapSelection::Marker(capital_marker))
            );
            return true;
        }
        self.apply(UiAction::EnterRegion(capital_marker));
        assert_eq!(self.navigation.scope(), MapScope::Region(capital_marker));
        self.assert_tutorial(TutorialStep::WorldMap);
        if matches!(scene, "tutorial_city_region" | "tutorial_region") {
            return true;
        }
        self.apply(UiAction::WorldMap);
        self.assert_tutorial(TutorialStep::Movement);
        if scene == "tutorial_world" {
            return true;
        }

        self.apply(UiAction::OpenArmies(headquarters));
        assert_eq!(self.army.site, Some(headquarters));
        if scene == "tutorial_roster" {
            return true;
        }
        self.apply(UiAction::ArmyOrders);
        self.apply(UiAction::BeginMove(army));
        if scene == "tutorial_group" {
            return true;
        }
        self.apply(UiAction::ChooseMoveDestination);
        if scene == "tutorial_blocked" {
            self.apply(UiAction::EnterRegion(home_marker));
            assert_eq!(self.navigation.scope(), MapScope::Region(home_marker));
            self.apply(UiAction::SelectMap(MapSelection::Site(headquarters)));
            self.apply(UiAction::ReviewMove);
            assert!(self.movement.preview.is_none());
            self.assert_tutorial(TutorialStep::Movement);
            return true;
        }
        self.apply(UiAction::SelectMap(MapSelection::Marker(
            destination_marker,
        )));
        assert_eq!(self.movement.site, Some(destination));
        assert_eq!(self.state.overlay, Overlay::None);
        self.assert_tutorial(TutorialStep::Career);
        if scene == "tutorial_route" {
            return true;
        }
        self.apply(UiAction::OpenArmies(destination));
        self.apply(UiAction::ArmyOrders);
        self.apply(UiAction::ArmyPeople);
        if scene == "tutorial_people" {
            return true;
        }
        self.apply(UiAction::OpenPersonProgression(person));
        self.assert_tutorial(TutorialStep::Household);
        if scene == "tutorial_career" {
            return true;
        }
        self.apply(UiAction::ArmyPeople);
        self.apply(UiAction::ArmyHouseholds);
        if scene == "tutorial_households" {
            return true;
        }
        self.apply(UiAction::ReviewHousehold(HouseholdAction::Invite));
        self.assert_tutorial(TutorialStep::FirstTurn);
        if scene == "tutorial_review" {
            return true;
        }
        self.apply(UiAction::CancelHouseholdReview);
        self.apply(UiAction::CancelArmyAction);
        self.apply(UiAction::Back);
        self.apply(UiAction::EndTurn);
        self.assert_tutorial(TutorialStep::Records);
        self.apply(UiAction::PauseNpcs(true));
        if scene == "tutorial_turn" {
            return true;
        }
        self.apply(UiAction::OpenRecords);
        assert!(self
            .state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .tutorial
            .is_complete());
        assert!(ui::tutorial_bounds(&self.state).is_none());
        self.apply(UiAction::Open(Overlay::Help));
        assert_eq!(scene, "tutorial_complete");
        true
    }

    fn assert_tutorial(&self, step: TutorialStep) {
        assert_eq!(
            self.state
                .campaign
                .as_ref()
                .unwrap()
                .strategic()
                .unwrap()
                .tutorial
                .current(),
            Some(step)
        );
    }
}
