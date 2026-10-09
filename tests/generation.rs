//! Seeded production-world generation replays exactly and saves losslessly.

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, GameData},
    state::StrategicCampaign,
};
use std::collections::BTreeSet;

fn setup(factions: usize, seed: u64) -> ProductionSetup {
    ProductionSetup {
        kingdom_name: "Briarhold".into(),
        emblem: Emblem::Rose,
        factions,
        seed,
    }
}

#[test]
fn identical_seed_replays_every_authored_choice_and_other_seed_changes_contents() {
    let data = GameData::load().unwrap();
    let repeated = data
        .production_layout
        .generate(&data, &setup(6, 42017))
        .unwrap();
    let replay = data
        .production_layout
        .generate(&data, &setup(6, 42017))
        .unwrap();
    let changed = data
        .production_layout
        .generate(&data, &setup(6, 42018))
        .unwrap();
    assert_eq!(repeated, replay);
    assert_ne!(repeated, changed);
    assert_eq!(repeated.scenario.seed, 42017);
}

#[test]
fn setup_validation_and_saved_four_and_eight_faction_worlds_are_independent() {
    let data = GameData::load().unwrap();
    assert!(data
        .production_layout
        .generate(&data, &setup(3, 4))
        .is_err());
    assert!(data
        .production_layout
        .generate(&data, &setup(9, 4))
        .is_err());
    let mut untrimmed = setup(4, 4);
    untrimmed.kingdom_name = " Rose ".into();
    assert!(data.production_layout.generate(&data, &untrimmed).is_err());
    let mut too_long = setup(4, 4);
    too_long.kingdom_name = "Kestrum".repeat(5);
    assert!(data.production_layout.generate(&data, &too_long).is_err());
    let mut unicode = setup(4, 4);
    unicode.kingdom_name = "🌹".repeat(32);
    assert!(data.production_layout.generate(&data, &unicode).is_ok());
    for count in [4, 8] {
        let original =
            StrategicCampaign::new_production(&data, &setup(count, 811 + count as u64)).unwrap();
        assert_eq!(original.factions.len(), count);
        assert_eq!(original.armies.len(), count);
        assert_eq!(original.people.len(), count);
        assert_eq!(original.pending_facts.len(), 0);
        assert_eq!(
            original
                .factions
                .values()
                .map(|faction| faction.name.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            count
        );
        assert_eq!(
            original
                .factions
                .values()
                .map(|faction| faction.emblem)
                .collect::<BTreeSet<_>>()
                .len(),
            count
        );
        let json = serde_json::to_vec(&original).unwrap();
        let restored: StrategicCampaign = serde_json::from_slice(&json).unwrap();
        restored.validate(&data).unwrap();
        assert_eq!(original, restored);
    }
}
