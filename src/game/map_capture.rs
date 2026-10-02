//! Real map selection and orders, captured without opening management overlays.

use super::*;
use kestrum::{navigation::MapSelection, state::military::ArmyId};
mod borders;
mod world_orders;

impl Game {
    pub(super) fn capture_map_fog(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(scene, "fog_north" | "fog_revealed") {
            return false;
        }
        self.setup.factions = 8;
        self.setup.seed = self.data.production_layout.default_seed;
        self.start_game();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("fog capture requires a production campaign");
        };
        campaign.tutorial.dismiss();
        let sites: Vec<_> = campaign
            .world
            .sites
            .iter()
            .filter(|site| {
                scene == "fog_revealed"
                    || campaign.world.marker(site.marker).unwrap().position[1] <= 0.4
            })
            .map(|site| site.id)
            .collect();
        for site in sites {
            campaign
                .set_site_control(&self.data, site, Some(campaign.player), false)
                .expect("valid control after northern conquest");
        }
        self.invalidate_projection();
        self.refresh_projection();
        self.navigation.show_world(&mut self.view);
        self.view.reset();
        self.navigation.clear_selection();
        self.notice = None;
        true
    }

    pub(super) fn capture_map_movement(&mut self, scene: &str) -> bool {
        if self.capture_border_access(scene) {
            return true;
        }
        let scene = scene.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "move_map"
                | "move_preview"
                | "move_arrived"
                | "move_blocked"
                | "move_exhausted"
                | "move_stack"
        ) {
            return false;
        }
        self.start_game();
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("capture campaign");
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == campaign.player)
            .expect("founding army")
            .id;
        let origin = campaign.armies[&army].site;
        let destination = campaign
            .world
            .adjacent_sites(origin)
            .into_iter()
            .find(|site| {
                engine::map_movement_preview(campaign, &self.data, campaign.player, &[army], *site)
                    .is_ok_and(|preview| preview.can_confirm() && preview.reachable_steps > 0)
            })
            .expect("production home has an open exit");
        if scene == "move_stack" {
            let formation = campaign.armies[&army]
                .formation_ids()
                .nth(1)
                .expect("two founding formations");
            let result = self
                .state
                .command(&self.data, Command::SplitArmy { formation })
                .expect("legal local split");
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                campaign
                    .armies
                    .get_mut(&result.split_army.unwrap())
                    .unwrap()
                    .name = "The Riverward Guard of the Silver Hawthorns".into();
            }
        }
        if scene == "move_blocked" {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                let neighbor = *campaign
                    .factions
                    .keys()
                    .find(|id| **id != campaign.player)
                    .unwrap();
                campaign
                    .set_site_control(&self.data, destination, Some(neighbor), false)
                    .expect("capture owns a peaceful foreign border");
            }
        }
        self.refresh_projection();
        self.capture_army_tap(army);
        if scene == "move_stack" {
            let next = *self.movement.remaining.keys().last().unwrap();
            self.apply(UiAction::BeginMove(next));
            assert_eq!(self.movement.armies, vec![next]);
            assert_eq!(self.state.overlay, Overlay::None);
            return true;
        }
        if scene == "move_map" {
            return true;
        }
        if scene == "move_exhausted" {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                for id in campaign.armies[&army].formation_ids().collect::<Vec<_>>() {
                    let formation = campaign.formations.get_mut(&id).unwrap();
                    formation.movement_spent = formation.movement_allowance(&self.data);
                }
            }
            self.begin_move(army);
            return true;
        }
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .unwrap();
        let site = campaign.world.site(destination).unwrap();
        let selection = if campaign.world.physical_site(site.marker) == Some(destination) {
            MapSelection::Marker(site.marker)
        } else {
            if self.navigation.scope() != kestrum::navigation::MapScope::Region(site.marker) {
                self.navigation.show_world(&mut self.view);
                self.navigation
                    .enter_region(&campaign.world, site.marker, &mut self.view)
                    .unwrap();
            }
            MapSelection::Site(destination)
        };
        self.apply(UiAction::SelectMap(selection));
        assert_eq!(self.state.overlay, Overlay::None);
        if scene == "move_blocked" {
            assert!(self.movement.preview.is_some());
            assert_eq!(self.movement.site, Some(origin));
        } else {
            assert!(self.movement.preview.is_none());
            assert_eq!(self.movement.site, Some(destination));
        }
        true
    }

    fn capture_army_tap(&mut self, army: ArmyId) {
        let campaign = self.projection.as_ref().unwrap();
        let target = self
            .navigation
            .army_targets(&campaign.world, &self.view, &campaign.armies)
            .into_iter()
            .find(|target| target.armies.contains(&army))
            .expect("home banner in view");
        let position = target.bounds.center();
        self.origin = Some(position);
        self.map_gesture = true;
        let action = self.map_selection_action(Pointer {
            position,
            hovering: false,
            down: false,
            released: true,
        });
        assert!(matches!(action, Some(UiAction::BeginMove(id)) if id == army));
        self.apply(action.unwrap());
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.movement.armies, vec![army]);
        self.origin = None;
        self.map_gesture = false;
    }
}
