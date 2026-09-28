//! Observe real UI destinations; opening a review never issues its proposed order.

use super::*;
use kestrum::{
    navigation::{MapScope, MapSelection},
    state::tutorial::TutorialStep,
};

impl Game {
    pub(super) fn apply_tutorial_action(&mut self, action: UiAction) -> bool {
        if !matches!(
            action,
            UiAction::DismissTutorial | UiAction::ReopenTutorial | UiAction::TutorialHeadquarters
        ) {
            return false;
        }
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return true;
        };
        match action {
            UiAction::DismissTutorial => campaign.tutorial.dismiss(),
            UiAction::ReopenTutorial => {
                campaign.tutorial.reopen();
                self.state.screen = Screen::Campaign;
                self.state.overlay = Overlay::None;
            }
            UiAction::TutorialHeadquarters => {
                let site = campaign.factions[&campaign.player].headquarters;
                if let Some(marker) = campaign.world.site(site).map(|site| site.marker) {
                    self.navigation.reset(&mut self.view);
                    let selection = if campaign.world.physical_site(marker) == Some(site) {
                        MapSelection::Marker(marker)
                    } else {
                        self.navigation
                            .enter_region(&campaign.world, marker, &mut self.view)
                            .expect("a regional headquarters belongs to a region");
                        MapSelection::Site(site)
                    };
                    self.navigation
                        .select(&campaign.world, selection)
                        .expect("headquarters is selectable on its map");
                    self.movement = ui::MoveView::default();
                    self.state.overlay = Overlay::None;
                }
            }
            _ => unreachable!("tutorial action guard"),
        }
        true
    }

    pub(super) fn observe_tutorial(&mut self, action: UiAction) {
        if self.state.screen != Screen::Campaign {
            return;
        }
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        let step = match action {
            UiAction::OpenArmies(site)
                if self.state.overlay == Overlay::Armies
                    && self.army.site == Some(site)
                    && site == campaign.factions[&campaign.player].headquarters =>
            {
                Some(TutorialStep::Headquarters)
            }
            UiAction::EnterRegion(region)
                if self.navigation.scope() == MapScope::Region(region) =>
            {
                Some(TutorialStep::Region)
            }
            UiAction::WorldMap
                if self.navigation.scope() == MapScope::World
                    && campaign.tutorial.completed(TutorialStep::Region) =>
            {
                Some(TutorialStep::WorldMap)
            }
            UiAction::OpenPersonProgression(person)
                if self.state.overlay == Overlay::Armies
                    && campaign
                        .people
                        .get(&person)
                        .is_some_and(|person| person.faction == campaign.player)
                    && self.army.mode == ui::ArmyMode::ProgressionPerson(person) =>
            {
                Some(TutorialStep::Career)
            }
            UiAction::ReviewHousehold(_)
                if self.state.overlay == Overlay::Armies
                    && self.army.household_review.is_some() =>
            {
                Some(TutorialStep::Household)
            }
            UiAction::OpenRecords if self.state.overlay == Overlay::History => {
                Some(TutorialStep::Records)
            }
            _ => None,
        };
        if let Some(step) = step {
            campaign.tutorial.record(step);
        }
    }
}
