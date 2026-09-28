//! Durable introductory states and assertions through the actual action dispatcher.

use super::*;
use kestrum::{
    data::world::MarkerLocation,
    engine::HouseholdAction,
    navigation::MapSelection,
    state::tutorial::{TutorialProgress, TutorialStep},
};

impl Game {
    pub(super) fn capture_tutorial(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        if !scene.starts_with("tutorial_") {
            return false;
        }
        self.setup.seed = self.data.production_layout.default_seed;
        self.start_game();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("tutorial capture requires a production campaign");
        };
        campaign.tutorial = TutorialProgress::new();
        let headquarters = campaign.factions[&campaign.player].headquarters;
        let headquarters_marker = campaign.world.site(headquarters).unwrap().marker;
        if scene == "tutorial_long_name" {
            let site = campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == headquarters)
                .unwrap();
            site.name = "The Riverward Settlement of Silver Hawthorns".into();
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
        let region = campaign
            .world
            .markers
            .iter()
            .find(|marker| matches!(marker.location, MarkerLocation::Region { .. }))
            .unwrap()
            .id;
        let destination = campaign
            .world
            .adjacent_sites(headquarters)
            .into_iter()
            .find(|site| {
                engine::movement_preview(campaign, &self.data, campaign.player, &[army], *site)
                    .is_ok_and(|preview| preview.stop.is_none() && preview.reachable_steps > 0)
            })
            .expect("production headquarters has a reachable neighbor");
        let destination_marker = campaign.world.site(destination).unwrap().marker;
        if scene == "tutorial_help" {
            self.apply(UiAction::DismissTutorial);
            self.apply(UiAction::Open(Overlay::Help));
            return true;
        }
        self.apply(UiAction::TutorialHeadquarters);
        if matches!(scene, "tutorial_headquarters" | "tutorial_long_name") {
            return true;
        }
        self.apply(UiAction::OpenArmies(headquarters));
        self.assert_tutorial(TutorialStep::Movement);
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
            self.apply(UiAction::SelectMap(MapSelection::Marker(
                headquarters_marker,
            )));
            self.apply(UiAction::ReviewMove);
            assert!(self.movement.preview.is_none());
            self.assert_tutorial(TutorialStep::Movement);
            return true;
        }
        self.apply(UiAction::SelectMap(MapSelection::Marker(
            destination_marker,
        )));
        self.apply(UiAction::ReviewMove);
        assert_eq!(self.state.overlay, Overlay::MoveReview);
        if scene == "tutorial_route" {
            return true;
        }
        self.apply(UiAction::ConfirmMove);
        self.assert_tutorial(TutorialStep::Region);
        self.apply(UiAction::Back);
        self.apply(UiAction::EnterRegion(region));
        self.assert_tutorial(TutorialStep::WorldMap);
        if scene == "tutorial_region" {
            return true;
        }
        self.apply(UiAction::WorldMap);
        self.assert_tutorial(TutorialStep::Career);
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
