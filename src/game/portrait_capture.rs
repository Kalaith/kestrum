//! Headless portrait review scenes built on the production capture fixtures.

use super::*;
use kestrum::state::people::PersonAssignment;

impl Game {
    pub(super) fn capture_portrait_review(&mut self, scene: &str) -> bool {
        match scene.trim_end_matches("_minimum") {
            "portraits_identity" => {
                self.capture_founder("formation_emerged");
                self.verify_portrait_identity_roundtrip();
                self.show_portrait_people();
                true
            }
            "portraits_dense" => {
                self.capture_founder("founder_dense");
                self.prepare_dense_portrait_roster();
                true
            }
            "portraits_career_dense" => {
                self.capture_founder("founder_dense");
                self.prepare_dense_portrait_roster();
                let founder = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .and_then(|campaign| {
                        campaign
                            .people
                            .values()
                            .find(|person| person.career.founding_lord)
                    })
                    .expect("dense founding lord")
                    .id;
                self.army.mode = ui::ArmyMode::ProgressionPerson(founder);
                true
            }
            "portraits_known" => {
                self.prepare_known_portrait_records();
                true
            }
            "portraits_missing_source" | "portraits_missing_sources" => {
                #[cfg(not(target_arch = "wasm32"))]
                self.capture_missing_portrait_source();
                cfg!(not(target_arch = "wasm32"))
            }
            _ => false,
        }
    }

    fn verify_portrait_identity_roundtrip(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            panic!("portrait identity capture must have a strategic campaign");
        };
        campaign
            .validate(&self.data)
            .expect("valid portrait identity capture");
        let identities = campaign
            .people
            .iter()
            .map(|(id, person)| (*id, person.appearance.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let founder = campaign
            .people
            .values()
            .find(|person| person.career.founding_lord)
            .expect("production founder");
        let emergent = campaign
            .people
            .values()
            .find(|person| person.career.emergence.is_some())
            .expect("deterministic emerged person");
        assert_ne!(founder.id, emergent.id);
        assert_ne!(founder.appearance.signature, emergent.appearance.signature);

        let encoded = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone())))
            .expect("serialize portrait compatibility capture");
        let decoded: Campaign =
            serde_json::from_value(encoded).expect("decode through campaign compatibility");
        let decoded = decoded
            .strategic()
            .expect("strategic campaign compatibility result");
        decoded
            .validate(&self.data)
            .expect("valid compatible portrait identity capture");
        assert_eq!(
            identities,
            decoded
                .people
                .iter()
                .map(|(id, person)| (*id, person.appearance.clone()))
                .collect(),
            "save compatibility preserves person IDs and assigned appearances"
        );
        assert_eq!(campaign.appearance_registry, decoded.appearance_registry);
    }

    fn prepare_dense_portrait_roster(&mut self) {
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("dense portrait capture must have a strategic campaign");
        };
        let founder = campaign
            .people
            .values()
            .find(|person| person.career.founding_lord)
            .expect("production founder")
            .id;
        campaign.people.get_mut(&founder).expect("founder").name = "W".repeat(64);
        let assigned = campaign
            .people
            .values()
            .filter(|person| {
                person.faction == campaign.player
                    && matches!(person.assignment, PersonAssignment::Formation { .. })
                    && person.is_alive()
            })
            .collect::<Vec<_>>();
        assert!(assigned.len() >= 6, "dense portrait roster has six people");
        assert_eq!(campaign.people[&founder].name.chars().count(), 64);
        assert_eq!(
            assigned
                .iter()
                .map(|person| person.appearance.signature.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            assigned.len(),
            "the appearance allocator supplies distinct stable identities"
        );
        campaign
            .validate(&self.data)
            .expect("valid dense portrait capture");
        self.show_portrait_people();
    }

    fn prepare_known_portrait_records(&mut self) {
        // The established history fixture observes this person through real combat.
        self.capture_history("history_known");
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("known portrait campaign");
        campaign
            .validate(&self.data)
            .expect("valid observed portrait history");
        let known = engine::person_knowledge(
            campaign,
            campaign.player,
            kestrum::state::people::PersonId(3),
        )
        .expect("observed enemy identity");
        assert!(matches!(
            known,
            engine::PersonKnowledge::LastEncountered { .. }
        ));
    }
    fn show_portrait_people(&mut self) {
        self.army.mode = ui::ArmyMode::People;
        self.invalidate_projection();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn capture_missing_portrait_source(&mut self) {
        self.capture_founder("founder_people");
        let appearance = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| {
                campaign
                    .people
                    .values()
                    .find(|person| person.faction == campaign.player)
                    .map(|person| person.appearance.clone())
            })
            .expect("founder appearance for missing-source capture");
        self.portraits
            .capture_missing_source(&appearance)
            .expect("hidden native missing-source cache verification");
        self.show_portrait_people();
    }
}
