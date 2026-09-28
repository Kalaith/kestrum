use kestrum::{
    data::{
        economy::Habitation,
        generation::ProductionSetup,
        rules::Emblem,
        threats::ThreatKind,
        world::{AnchorExpression, Facility, MarkerLocation, PersonClass, SiteId, SiteTag},
        GameData,
    },
    engine,
    state::{people::PersonId, StrategicCampaign},
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
fn authored_production_map_has_eighty_major_and_152_physical_sites() {
    let data = GameData::load().unwrap();
    let layout = &data.production_layout;
    assert_eq!(layout.markers.len(), 80);
    assert_eq!(layout.sites.len(), 152);
    assert_eq!(layout.headquarters_candidates.len(), 8);
    assert_eq!(layout.routes.len(), 191);
    let regions: Vec<_> = layout
        .markers
        .iter()
        .filter_map(|marker| match &marker.location {
            MarkerLocation::Region {
                sites,
                entrances,
                anchors,
            } => Some((sites, entrances, anchors)),
            MarkerLocation::Site { .. } => None,
        })
        .collect();
    assert_eq!(regions.len(), 8);
    for (sites, entrances, anchors) in regions {
        assert_eq!(sites.len(), 10);
        assert!(entrances.len() >= 2);
        assert!(
            entrances
                .iter()
                .map(|entry| entry.site)
                .collect::<BTreeSet<_>>()
                .len()
                >= 2
        );
        assert!(matches!(anchors, AnchorExpression::All { .. }));
    }
    let reached = layout.reachable_sites(layout.sites[0].id);
    assert_eq!(reached.len(), layout.sites.len());
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
fn four_through_eight_factions_get_connected_separated_starts_and_local_threats() {
    let data = GameData::load().unwrap();
    for count in 4..=8 {
        let generated = data
            .production_layout
            .generate(&data, &setup(count, 9000 + count as u64))
            .unwrap();
        let scenario = &generated.scenario;
        assert_eq!(scenario.factions.len(), count);
        assert_eq!(scenario.relations.len(), count * (count - 1) / 2);
        let mut starts = Vec::new();
        for faction in &scenario.factions {
            starts.push(faction.headquarters);
            let hq = scenario.site(faction.headquarters).unwrap();
            assert_eq!(hq.controller, Some(faction.id));
            assert_eq!(hq.habitation, Habitation::Village);
            assert!(hq.facilities.contains(&Facility::TrainingGround));
            assert!(hq.tags.contains(&SiteTag::HorseAccess));
            assert!(matches!(
                scenario.marker(hq.marker).unwrap().location,
                MarkerLocation::Region { .. }
            ));
            assert_eq!(faction.capital, faction.headquarters);
            assert_eq!(faction.founder.age_years, 24);
            assert_eq!(faction.founder.class, PersonClass::Officer);
            assert_eq!(
                faction.resources.resolve(&data.economy),
                data.economy.starting_resources
            );
            assert_eq!(scenario.reachable_sites(faction.headquarters).len(), 152);
        }
        for (index, left) in starts.iter().enumerate() {
            for right in starts.iter().skip(index + 1) {
                assert!(distance(scenario, *left, *right).unwrap() >= 4);
            }
        }
        assert_eq!(generated.initial_threats.len(), count * 2);
        let mut threatened = BTreeSet::new();
        for faction in &scenario.factions {
            let pair: Vec<_> = generated
                .initial_threats
                .iter()
                .filter(|threat| {
                    distance(scenario, faction.headquarters, threat.site)
                        .is_some_and(|steps| steps <= 2)
                })
                .collect();
            assert_eq!(pair.len(), 2);
            assert!(scenario
                .routes
                .iter()
                .filter_map(|route| route.other_endpoint(faction.headquarters))
                .any(|site| !generated
                    .initial_threats
                    .iter()
                    .any(|threat| threat.site == site)));
            assert_eq!(
                pair.iter()
                    .map(|threat| threat.kind)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([ThreatKind::Bandits, ThreatKind::Wildlife])
            );
            for threat in pair {
                assert!(threatened.insert(threat.site));
                assert!(scenario.site(threat.site).unwrap().controller.is_none());
            }
        }
    }
}

#[test]
fn ordinary_courses_have_real_sites_and_no_starting_battle_credit() {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production(&data, &setup(4, 82)).unwrap();
    let options = engine::career_options(&campaign, &data, PersonId(1)).unwrap();
    assert_eq!(
        options
            .iter()
            .map(|option| option.class)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            PersonClass::Infantry,
            PersonClass::Archer,
            PersonClass::Scout,
            PersonClass::Cavalry,
            PersonClass::Medic,
            PersonClass::Officer,
        ])
    );
    assert!(options.iter().all(|option| !option.eligible));
    let hq = campaign.factions[&campaign.player].headquarters;
    let hq_site = campaign.world.site(hq).unwrap();
    assert!(hq_site.facilities.contains(&Facility::TrainingGround));
    assert!(data.threats.definitions.contains_key(&ThreatKind::Bandits));
    assert!(data.threats.definitions.contains_key(&ThreatKind::Wildlife));
    assert!(campaign.people[&PersonId(1)].evidence.counts.is_empty());
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

fn distance(
    scenario: &kestrum::data::world::Scenario,
    start: SiteId,
    target: SiteId,
) -> Option<usize> {
    let mut visited = BTreeSet::from([start]);
    let mut queue = std::collections::VecDeque::from([(start, 0usize)]);
    while let Some((site, steps)) = queue.pop_front() {
        if site == target {
            return Some(steps);
        }
        for next in scenario
            .routes
            .iter()
            .filter_map(|route| route.other_endpoint(site))
        {
            if visited.insert(next) {
                queue.push_back((next, steps + 1));
            }
        }
    }
    None
}
