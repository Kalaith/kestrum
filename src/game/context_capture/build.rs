//! Compact construction choices and builder pages stay tied to the selected city.

use super::*;
use kestrum::{
    data::{economy::Resources, world::SiteId},
    navigation::MapSelection,
    state::construction::{ConstructionKind, ConstructionTarget},
};

impl Game {
    pub(super) fn capture_context_build_choices(&mut self) {
        let site = self.begin_context_build(4);
        let pages = self.build_choice_count(site).div_ceil(3).max(1);
        assert!(pages > 1, "capture offers multiple construction pages");
        let target_page = 1;
        self.visit_context_build_page(target_page);
        let visible: Vec<_> = self
            .settlement
            .options
            .iter()
            .filter(|choice| !matches!(choice.target, ConstructionTarget::Route(_)))
            .skip(target_page * 3)
            .take(3)
            .collect();
        assert!(
            !visible.is_empty(),
            "requested construction page has choices"
        );
        self.assert_region_place_context(site);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Build);
        assert_eq!(self.settlement.page, target_page);
    }

    pub(super) fn capture_context_builders(&mut self) {
        let site = self.begin_context_build(4);
        let choices: Vec<_> = self
            .settlement
            .options
            .iter()
            .filter(|choice| !matches!(choice.target, ConstructionTarget::Route(_)))
            .map(|choice| {
                (
                    choice.target,
                    choice.option.kind,
                    choice.option.blocked.is_none() && !choice.option.builders.is_empty(),
                )
            })
            .collect();
        let index = choices
            .iter()
            .position(|(target, kind, available)| {
                *target == ConstructionTarget::Site(site)
                    && *available
                    && !matches!(kind, ConstructionKind::Facility(_))
            })
            .expect("a builder-backed improvement is available at the capture city");
        let (target, kind, _) = choices[index];
        self.visit_context_build_page(index / 3);
        let choice = self.panel_point(16.0 + 163.0, 138.0 + (index % 3) as f32 * 120.0 + 58.0);
        let action = self.pointer_tap(choice);
        assert!(
            matches!(action, UiAction::SelectConstruction(found, found_kind) if found == target && found_kind == kind)
        );
        self.apply(action);
        self.refresh_settlement();

        let choose_builder = self.panel_point(16.0 + 108.0 + 109.0, 476.0 + 24.0);
        let action = self.pointer_tap(choose_builder);
        assert!(matches!(action, UiAction::ChooseBuilder));
        self.apply(action);
        self.refresh_settlement();
        let pages = self.settlement.builders.len().div_ceil(3).max(1);
        assert!(pages > 1, "capture offers multiple builder pages");
        self.visit_context_builder_page(pages - 1);
        assert!(self
            .settlement
            .builders
            .iter()
            .skip((pages - 1) * 3)
            .any(|builder| builder.blocked.is_none()));
        self.assert_region_place_context(site);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Builders);
        assert_eq!(self.settlement.page, pages - 1);
    }

    fn begin_context_build(&mut self, builders: usize) -> SiteId {
        assert!(self.capture_city("production_region"));
        let site = match self.navigation.selection() {
            Some(MapSelection::Site(site)) => site,
            selection => {
                panic!("production city capture should select a regional site: {selection:?}")
            }
        };
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.prepare_selected_site(site);
        self.ensure_local_builders(site, builders);
        self.refresh_projection();
        self.refresh_settlement();
        let action = self.pointer_tap(self.inspector_action_center(1));
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Build) if id == site)
        );
        self.apply(action);
        self.assert_region_place_context(site);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Build);
        site
    }

    fn build_choice_count(&self, site: SiteId) -> usize {
        self.settlement
            .options
            .iter()
            .filter(|choice| choice.target == ConstructionTarget::Site(site))
            .count()
    }

    fn visit_context_build_page(&mut self, target: usize) {
        while self.settlement.page < target {
            let next = self.panel_point(16.0 + 218.0 + 54.0, 552.0 + 24.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::SettlementPage(1)));
            self.apply(action);
            self.refresh_settlement();
        }
        while self.settlement.page > target {
            let previous = self.panel_point(16.0 + 109.0 + 50.0, 552.0 + 24.0);
            let action = self.pointer_tap(previous);
            assert!(matches!(action, UiAction::SettlementPage(-1)));
            self.apply(action);
            self.refresh_settlement();
        }
    }

    fn visit_context_builder_page(&mut self, target: usize) {
        while self.settlement.page < target {
            let next = self.panel_point(16.0 + 218.0 + 54.0, 552.0 + 24.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::SettlementPage(1)));
            self.apply(action);
            self.refresh_settlement();
        }
    }
}
