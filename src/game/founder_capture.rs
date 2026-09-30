//! Founding lord verification uses the production new-campaign flow.

use super::*;
use kestrum::state::people::PersonId;

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
        ) {
            return false;
        }
        self.setup.seed = self.data.production_layout.default_seed;
        if name == "founder_long_name" {
            self.setup.kingdom_name = "W".repeat(self.data.rules.kingdom_name_max_chars);
        }
        self.start_game();
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
        if name == "founder_dense" {
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
        true
    }
}
