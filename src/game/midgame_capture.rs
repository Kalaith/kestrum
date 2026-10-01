//! Review the reproducible development save through the ordinary load and UI paths.

#[path = "../../examples/prepare_midgame/campaign.rs"]
mod campaign;

use super::*;
use kestrum::{navigation::MapSelection, state::history::HistorySubject};

impl Game {
    pub(super) fn capture_midgame(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "midgame_map" | "midgame_frontier" | "midgame_heroes" | "midgame_hero" | "midgame_army"
        ) {
            return false;
        }
        let prepared = campaign::generate(&self.data).expect("valid developed review campaign");
        self.finish_load(Ok(Campaign::Strategic(Box::new(prepared))));
        self.navigation.show_world(&mut self.view);
        self.navigation.clear_selection();
        self.notice = None;
        match scene {
            "midgame_army" => {
                let campaign = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .unwrap();
                let army = campaign
                    .armies
                    .values()
                    .filter(|army| army.faction == campaign.player)
                    .max_by_key(|army| {
                        army.formation_ids()
                            .filter(|id| campaign.formation_person(*id).is_some())
                            .count()
                    })
                    .expect("staffed review army");
                let (id, site) = (army.id, army.site);
                self.open_armies(site);
                self.army.page = self
                    .local_armies()
                    .iter()
                    .position(|army| *army == id)
                    .unwrap();
                self.refresh_army();
            }
            "midgame_frontier" => {
                let world = &self.projection.as_ref().expect("loaded projection").world;
                let frontier = world
                    .sites
                    .iter()
                    .find(|site| {
                        site.controller == Some(self.projection.as_ref().unwrap().observer)
                            && world.physical_site(site.marker) == Some(site.id)
                            && world.adjacent_sites(site.id).iter().any(|id| {
                                world
                                    .site(*id)
                                    .and_then(|site| site.controller)
                                    .is_some_and(|owner| {
                                        owner != self.projection.as_ref().unwrap().observer
                                    })
                            })
                    })
                    .expect("visible frontier");
                let marker = frontier.marker;
                self.view.focus(world.marker(marker).unwrap().position, 1.5);
                self.apply(UiAction::SelectMap(MapSelection::Marker(marker)));
            }
            "midgame_heroes" => self.apply(UiAction::OpenRecords),
            "midgame_hero" => {
                let person = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .unwrap()
                    .people
                    .values()
                    .find(|person| {
                        person.faction == self.projection.as_ref().unwrap().observer
                            && person.is_alive()
                            && !person.career.founding_lord
                            && person.class != kestrum::data::world::PersonClass::Recruit
                    })
                    .expect("developed hero")
                    .id;
                self.apply(UiAction::OpenRecords);
                self.apply(UiAction::OpenHistory(HistorySubject::Person(person)));
            }
            _ => {}
        }
        true
    }
}
