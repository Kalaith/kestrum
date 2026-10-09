//! Observe real UI destinations; opening a review never issues its proposed order.

use super::*;
use kestrum::{navigation::MapScope, state::tutorial::TutorialStep};

impl Game {
    pub(super) fn apply_tutorial_action(&mut self, action: UiAction) -> bool {
        if !matches!(
            action,
            UiAction::DismissTutorial | UiAction::ReopenTutorial | UiAction::TutorialHeadquarters
        ) {
            return false;
        }
        if self.is_observer() {
            return true;
        }
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return true;
        };
        match action {
            UiAction::DismissTutorial => campaign.tutorial.dismiss(),
            UiAction::ReopenTutorial => {
                campaign.tutorial.reopen();
                reconcile_lessons(campaign);
                self.state.screen = Screen::Campaign;
                self.state.overlay = Overlay::None;
            }
            UiAction::TutorialHeadquarters => {
                let site = campaign.factions[&campaign.player].capital;
                self.navigation
                    .focus_site(&campaign.world, site, &mut self.view);
                self.movement = ui::MoveView::default();
                self.state.overlay = Overlay::None;
            }
            _ => unreachable!("tutorial action guard"),
        }
        true
    }

    pub(super) fn observe_tutorial(&mut self, action: UiAction) {
        if self.is_observer() || self.state.screen != Screen::Campaign {
            return;
        }
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        reconcile_lessons(campaign);
        let capital = campaign.factions[&campaign.player].capital;
        let capital_marker = campaign.world.site(capital).map(|site| site.marker);
        let step = match action {
            UiAction::OpenSettlement(site)
                if self.state.overlay == Overlay::Settlement
                    && self.settlement.site == Some(site)
                    && site == capital =>
            {
                Some(TutorialStep::Headquarters)
            }
            UiAction::EnterRegion(region)
                if self.navigation.scope() == MapScope::Region(region)
                    && capital_marker == Some(region)
                    && campaign.world.is_region_available(region) =>
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

fn reconcile_lessons(campaign: &mut kestrum::state::StrategicCampaign) {
    reconcile_city_development(campaign);
    if campaign.tutorial.current() == Some(TutorialStep::ClearThreat)
        && kestrum::engine::project(campaign, campaign.player)
            .is_ok_and(|view| view.threats.is_empty())
    {
        campaign.tutorial.record(TutorialStep::ClearThreat);
    }
}

fn reconcile_city_development(campaign: &mut kestrum::state::StrategicCampaign) {
    let capital_marker = campaign
        .world
        .site(campaign.factions[&campaign.player].capital)
        .map(|capital| capital.marker);
    if capital_marker.is_some_and(|marker| campaign.world.is_region_available(marker)) {
        campaign.tutorial.record(TutorialStep::CityDevelopment);
    }
}
