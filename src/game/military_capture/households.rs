//! Review scenes preserve the ordinary selected-list and confirmation paths.
use super::*;
use kestrum::{engine::HouseholdAction, state::people::PersonId};

impl Game {
    pub(in crate::game) fn capture_household_review(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        let action = match scene {
            "household_blocked" => HouseholdAction::Form,
            "household_ready" => HouseholdAction::End,
            "succession_review" => HouseholdAction::Designate,
            "household_dense" => HouseholdAction::Form,
            _ => return false,
        };
        self.capture_progression(if scene == "succession_review" {
            "succession"
        } else {
            "households"
        });
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            campaign.people.get_mut(&PersonId(1)).expect("founder").name =
                "Aveline of the Long Western River Crossing".into();
            if scene == "household_dense" {
                for index in 0..10 {
                    let mut person = campaign.people[&PersonId(1)].clone();
                    person.id = campaign.next_ids.person;
                    person.name = format!("Household resident {} of the Western River", index + 1);
                    person.career = Default::default();
                    person.evidence = Default::default();
                    campaign.next_ids.person.0 += 1;
                    person.appearance = engine::portraits::allocate_for_person(
                        campaign,
                        &self.data.portraits,
                        person.id,
                    )
                    .expect("capture household appearance");
                    campaign.people.insert(person.id, person);
                }
            }
            campaign
                .validate(&self.data)
                .expect("valid family review capture");
        }
        self.army.household_review = (scene != "household_dense").then_some(action);
        self.refresh_army();
        true
    }
}
