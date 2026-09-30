use kestrum::{
    data::{
        economy::TroopKind, generation::ProductionSetup, rules::Emblem, world::PersonClass,
        GameData,
    },
    state::{
        people::{PersonAssignment, PersonId, PersonStatus},
        StrategicCampaign,
    },
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
fn each_new_kingdom_has_one_named_young_lord_commanding_its_starting_army() {
    let data = GameData::load().unwrap();
    for count in [4, 8] {
        let campaign = StrategicCampaign::new_production(&data, &setup(count, 19)).unwrap();
        for faction in campaign.factions.values() {
            let people: Vec<_> = campaign
                .people
                .values()
                .filter(|person| person.faction == faction.id)
                .collect();
            assert_eq!(people.len(), 1);
            let lord = people[0];
            assert!(lord.career.founding_lord);
            assert!((18..=24).contains(&lord.age_years(0)));
            assert_eq!(lord.class, PersonClass::Officer);
            assert!(data.human_names.given_names.iter().any(|given| data
                .human_names
                .family_names
                .iter()
                .any(|family| lord.name == format!("{given} {family}"))));
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == faction.id)
                .unwrap();
            assert_eq!(army.commander, Some(lord.id));
            assert_eq!(army.site, faction.headquarters);
            let warriors = army
                .formation_ids()
                .map(|id| &campaign.formations[&id])
                .find(|formation| formation.kind == TroopKind::Warriors)
                .unwrap();
            assert_eq!(
                lord.assignment,
                PersonAssignment::Formation {
                    formation: warriors.id
                }
            );
            assert_eq!((warriors.headcount, warriors.capacity), (100, 100));
            assert_eq!(army.formation_ids().count(), 3);
            assert!(lord.career.recognition.is_none());
            assert!(lord.career.traits.is_empty());
            assert!(lord.evidence.counts.is_empty());
        }
    }
}

#[test]
fn seeded_ages_cover_the_entire_young_range_and_replay_the_same_founders() {
    let data = GameData::load().unwrap();
    let mut ages = BTreeSet::new();
    for seed in 0..32 {
        let generated = data
            .production_layout
            .generate(&data, &setup(4, seed))
            .unwrap();
        assert_eq!(
            generated,
            data.production_layout
                .generate(&data, &setup(4, seed))
                .unwrap()
        );
        ages.extend(
            generated
                .scenario
                .factions
                .iter()
                .map(|faction| faction.founder.age_years),
        );
    }
    assert_eq!(ages, (18..=24).collect());
}

#[test]
fn founding_leadership_bonus_requires_a_fit_attached_commander() {
    let data = GameData::load().unwrap();
    let original = StrategicCampaign::new_production(&data, &setup(4, 43)).unwrap();
    let army = original
        .armies
        .values()
        .find(|army| army.faction == original.player)
        .unwrap()
        .id;
    let lord = original.armies[&army].commander.unwrap();
    assert_eq!(original.army_leadership_permille(army, &data), Some(983));
    for status in [
        PersonStatus::Fit,
        PersonStatus::Wounded {
            since_round: 0,
            remaining_steps: 1,
        },
        PersonStatus::Dead {
            completed_rounds: 0,
            site: original.armies[&army].site,
        },
    ] {
        for commands in [true, false] {
            let mut campaign = original.clone();
            campaign.people.get_mut(&lord).unwrap().status = status;
            if !commands {
                campaign.armies.get_mut(&army).unwrap().commander = None;
            }
            let titled = campaign.army_leadership_permille(army, &data).unwrap();
            campaign.people.get_mut(&lord).unwrap().career.founding_lord = false;
            let ordinary = campaign.army_leadership_permille(army, &data).unwrap();
            assert_eq!(
                titled - ordinary,
                if commands && status == PersonStatus::Fit {
                    50
                } else {
                    0
                }
            );
        }
    }
    let mut depleted = original.clone();
    let formation = match depleted.people[&lord].assignment {
        kestrum::state::people::PersonAssignment::Formation { formation } => formation,
        _ => panic!("founder must be attached to the army"),
    };
    depleted.formations.get_mut(&formation).unwrap().headcount = 0;
    assert_eq!(depleted.army_leadership_permille(army, &data), Some(500));
    let mut retired = original.clone();
    retired.people.get_mut(&lord).unwrap().career.retired = true;
    assert_eq!(retired.army_leadership_permille(army, &data), Some(500));
    let mut reassigned = original.clone();
    reassigned.people.get_mut(&lord).unwrap().assignment =
        kestrum::state::people::PersonAssignment::Site {
            site: original.armies[&army].site,
        };
    assert_eq!(reassigned.army_leadership_permille(army, &data), Some(500));
}

#[test]
fn saving_retains_the_lord_and_older_saves_do_not_receive_new_starting_grants() {
    let data = GameData::load().unwrap();
    let original = StrategicCampaign::new_production(&data, &setup(4, 91)).unwrap();
    let bytes = serde_json::to_vec(&original).unwrap();
    let restored: StrategicCampaign = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&data).unwrap();
    assert_eq!(original, restored);
    let mut old_save = serde_json::to_value(&original).unwrap();
    for person in old_save["people"].as_object_mut().unwrap().values_mut() {
        person["career"]
            .as_object_mut()
            .unwrap()
            .remove("founding_lord");
    }
    let earlier: StrategicCampaign = serde_json::from_value(old_save).unwrap();
    earlier.validate(&data).unwrap();
    assert_eq!(earlier.people.len(), original.people.len());
    assert!(earlier
        .people
        .values()
        .all(|person| !person.career.founding_lord));
    assert_eq!(
        earlier.people[&PersonId(1)].name,
        original.people[&PersonId(1)].name
    );
    assert_eq!(
        earlier.people[&PersonId(1)].birth_round,
        original.people[&PersonId(1)].birth_round
    );
    let prototype = StrategicCampaign::new(&data).unwrap();
    assert!(prototype
        .people
        .values()
        .all(|person| !person.career.founding_lord));
}

#[test]
fn founding_age_and_bonus_rules_reject_invalid_data() {
    let data = GameData::load().unwrap();
    for (minimum, maximum, bonus) in [(17, 24, 50), (24, 18, 50), (18, 25, 50), (18, 24, 101)] {
        let mut invalid = data.clone();
        invalid.rules.founder.minimum_age_years = minimum;
        invalid.rules.founder.maximum_age_years = maximum;
        invalid.rules.founder.commander_bonus_permille = bonus;
        assert!(invalid.rules.validate().unwrap_err().contains("founder"));
        assert!(invalid
            .production_layout
            .generate(&invalid, &setup(4, 1))
            .is_err());
    }
    let generated = data
        .production_layout
        .generate(&data, &setup(4, 1))
        .unwrap();
    for age in [17, 25] {
        let mut invalid = generated.scenario.clone();
        invalid.factions[0].founder.age_years = age;
        assert!(invalid
            .validate_generated(&data.rules, &data.economy)
            .is_err());
    }
}
