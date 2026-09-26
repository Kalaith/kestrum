//! K04 physical boundaries, secure supply, partial claims, and save compatibility.

use std::collections::{BTreeMap, BTreeSet};

use kestrum::{
    data::{
        world::{FactionId, MarkerId, RouteId, SiteId},
        GameData,
    },
    engine::project,
    state::{
        persistence::{load_legacy, SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE},
        world::RegionControl,
        Campaign, GameState, Overlay, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::{
    encode_slot, IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};

const REGION: MarkerId = MarkerId(5);
const PLAYER: FactionId = FactionId(1);
const RIVAL: FactionId = FactionId(3);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn hold(campaign: &mut StrategicCampaign, data: &GameData, faction: FactionId, sites: &[u32]) {
    for &site in sites {
        campaign
            .set_site_control(data, SiteId(site), Some(faction), false)
            .unwrap();
    }
}

#[test]
fn external_routes_resolve_the_same_entrance_and_reverse_exit() {
    let (_, campaign) = fixture();
    let world = &campaign.world;
    for (route, entrance, outside) in [(13, 5, 1), (14, 5, 2), (15, 11, 3), (16, 11, 4)] {
        let crossing = world.entrance(REGION, RouteId(route)).unwrap();
        assert_eq!(crossing.region, REGION);
        assert_eq!(crossing.route, RouteId(route));
        assert_eq!(crossing.entrance, SiteId(entrance));
        assert_eq!(crossing.external_site, SiteId(outside));
        assert_eq!(crossing.external_marker, MarkerId(outside));
        let edge = world.route(crossing.route).unwrap();
        assert_eq!(
            edge.other_endpoint(crossing.external_site),
            Some(crossing.entrance)
        );
        assert_eq!(
            edge.other_endpoint(crossing.entrance),
            Some(crossing.external_site)
        );
        assert_eq!(
            world.connected_route(crossing.entrance, crossing.external_site),
            Some(edge)
        );
    }
    assert!(world.entrance(REGION, RouteId(1)).is_none());
    assert!(world.entrance(MarkerId(1), RouteId(13)).is_none());
    assert!(world.entrance(REGION, RouteId(999)).is_none());
    assert!(world.connected_route(SiteId(5), SiteId(11)).is_none());
    assert!(world.connected_route(SiteId(1), SiteId(10)).is_none());
    assert_eq!(world.reachable_sites(SiteId(1)).len(), 14);
    assert!(world.reachable_sites(SiteId(999)).is_empty());
}

#[test]
fn region_markers_are_not_sites_and_invalid_geography_never_replaces_play() {
    let (data, mut campaign) = fixture();
    for id in 1..=4 {
        assert_eq!(campaign.world.physical_site(MarkerId(id)), Some(SiteId(id)));
    }
    assert!(campaign.world.physical_site(REGION).is_none());
    assert!(campaign.world.physical_site(MarkerId(999)).is_none());
    // Numeric values may overlap between typed namespaces; site 5 is West Gate,
    // never the occupiable version of the region marker with numeric value 5.
    assert_eq!(campaign.world.site(SiteId(5)).unwrap().key, "west_gate");
    let unchanged = campaign.clone();
    for (site, owner) in [
        (SiteId(999), Some(PLAYER)),
        (SiteId(5), Some(FactionId(999))),
    ] {
        assert!(campaign
            .set_site_control(&data, site, owner, false)
            .is_err());
        assert_eq!(campaign, unchanged);
    }
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(campaign.clone())), &data)
        .unwrap();
    state.overlay = Overlay::Menu;
    let live = state.campaign.clone();
    for variant in 0..7 {
        let mut invalid = campaign.clone();
        match variant {
            0 => invalid.world.routes[0].to = SiteId(999),
            1 => invalid.world.markers[0].position = [0.1, 0.1],
            2 => {
                invalid.world.contested_sites.insert(SiteId(999));
            }
            3 => {
                invalid
                    .world
                    .region_control
                    .get_mut(&REGION)
                    .unwrap()
                    .political_owner = Some(FactionId(999));
            }
            4 => {
                invalid
                    .world
                    .region_control
                    .get_mut(&REGION)
                    .unwrap()
                    .contested = false;
            }
            5 => {
                invalid.world.region_control.clear();
            }
            _ => {
                invalid.world.region_control.insert(
                    MarkerId(999),
                    RegionControl {
                        political_owner: None,
                        contested: true,
                    },
                );
            }
        }
        assert!(state
            .load_campaign(Campaign::Strategic(Box::new(invalid)), &data)
            .is_err());
        assert_eq!(state.campaign, live);
        assert_eq!(state.overlay, Overlay::Menu);
    }
}

#[test]
fn supplied_anchors_transfer_political_ownership_without_erasing_hostile_pockets() {
    let (data, mut campaign) = fixture();
    let initial_rng = campaign.rng.clone();
    assert_eq!(
        campaign.world.region_control(REGION),
        Some(&RegionControl {
            political_owner: None,
            contested: true,
        })
    );
    hold(&mut campaign, &data, PLAYER, &[12]);
    hold(&mut campaign, &data, RIVAL, &[11]);
    assert_eq!(
        campaign
            .world
            .region_control(REGION)
            .unwrap()
            .political_owner,
        Some(RIVAL)
    );
    assert!(!campaign.world.region_control(REGION).unwrap().contested);
    assert_eq!(
        campaign.world.site(SiteId(12)).unwrap().controller,
        Some(PLAYER)
    );
    hold(&mut campaign, &data, PLAYER, &[5, 6, 8, 9, 10]);
    assert_eq!(
        campaign
            .world
            .region_control(REGION)
            .unwrap()
            .political_owner,
        Some(PLAYER)
    );
    assert!(!campaign.world.region_control(REGION).unwrap().contested);
    assert_eq!(
        campaign.world.site(SiteId(11)).unwrap().controller,
        Some(RIVAL)
    );
    assert_eq!(
        campaign.world.controller_counts(REGION),
        BTreeMap::from([(None, 3), (Some(PLAYER), 6), (Some(RIVAL), 1),])
    );
    let supply = campaign.world.supplied_sites(PLAYER, SiteId(1));
    assert_eq!(supply, BTreeSet::from([1, 5, 6, 8, 9, 10, 12].map(SiteId)));
    assert_eq!(
        campaign.world.supply_path(PLAYER, SiteId(1), SiteId(10)),
        Some(vec![SiteId(1), SiteId(5), SiteId(6), SiteId(8), SiteId(10)])
    );
    assert!(campaign
        .world
        .supply_path(PLAYER, SiteId(1), SiteId(11))
        .is_none());
    assert_eq!(
        campaign.world.anchors_satisfied(REGION, PLAYER, SiteId(1)),
        Some(true)
    );
    assert!(campaign
        .world
        .anchors_satisfied(MarkerId(1), PLAYER, SiteId(1))
        .is_none());
    let view = project(&campaign, PLAYER).unwrap();
    assert_eq!(view.observer, PLAYER);
    assert_eq!(view.supplied_sites, supply);
    assert!(view
        .factions
        .iter()
        .filter(|faction| faction.id != PLAYER)
        .all(|faction| faction.headquarters.is_none() && faction.resources.is_none()));

    // Peace never creates foreign military access or a supply relay.
    campaign
        .set_site_control(&data, SiteId(6), Some(FactionId(2)), false)
        .unwrap();
    assert_eq!(
        campaign.world.supplied_sites(PLAYER, SiteId(1)),
        BTreeSet::from([SiteId(1), SiteId(5)])
    );
    assert!(campaign
        .world
        .supply_path(PLAYER, SiteId(1), SiteId(10))
        .is_none());
    hold(&mut campaign, &data, PLAYER, &[6]);
    campaign.world.routes[0].road.improved = true;
    campaign.world.routes[0].road.damage = 100;
    campaign.validate(&data).unwrap();
    assert_eq!(campaign.world.supplied_sites(PLAYER, SiteId(1)), supply);
    assert_eq!(campaign.rng, initial_rng);
    assert_eq!(
        (campaign.completed_rounds, campaign.accepted_sequence),
        (0, 0)
    );
}

#[test]
fn anchor_or_supply_loss_retains_historical_claim_and_saves_restore_it() {
    let (data, mut campaign) = fixture();
    hold(&mut campaign, &data, PLAYER, &[5, 6, 8, 9, 10]);
    assert_earlier_v2_compatibility(&data, &campaign);
    let controlled = campaign.world.sites.clone();
    campaign
        .set_site_control(&data, SiteId(9), Some(PLAYER), true)
        .unwrap();
    assert_eq!(campaign.world.sites, controlled);
    assert_eq!(
        campaign.world.region_control(REGION),
        Some(&RegionControl {
            political_owner: Some(PLAYER),
            contested: true,
        })
    );
    assert!(!campaign.world.is_secure(SiteId(9), PLAYER));
    assert_eq!(
        campaign.world.anchors_satisfied(REGION, PLAYER, SiteId(1)),
        Some(false)
    );
    let wrapped = Campaign::Strategic(Box::new(campaign.clone()));
    let restored = load_legacy(&encode_slot("claim", &wrapped, "2").unwrap(), &data).unwrap();
    assert_eq!(restored, wrapped);

    hold(&mut campaign, &data, PLAYER, &[9]);
    campaign
        .set_site_control(&data, SiteId(5), Some(PLAYER), true)
        .unwrap();
    assert_eq!(
        campaign.world.supplied_sites(PLAYER, SiteId(1)),
        BTreeSet::from([SiteId(1)])
    );
    assert!(campaign.world.region_control(REGION).unwrap().contested);
    hold(&mut campaign, &data, PLAYER, &[5]);
    campaign
        .set_site_control(&data, SiteId(1), Some(RIVAL), false)
        .unwrap();
    assert!(campaign.world.supplied_sites(PLAYER, SiteId(1)).is_empty());
    assert_eq!(
        campaign
            .world
            .region_control(REGION)
            .unwrap()
            .political_owner,
        Some(PLAYER)
    );
    assert!(campaign.world.region_control(REGION).unwrap().contested);
    hold(&mut campaign, &data, RIVAL, &[9, 10, 11]);
    assert_eq!(
        campaign
            .world
            .region_control(REGION)
            .unwrap()
            .political_owner,
        Some(RIVAL)
    );
    assert!(!campaign.world.region_control(REGION).unwrap().contested);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(PLAYER)
    );
}

fn assert_earlier_v2_compatibility(data: &GameData, campaign: &StrategicCampaign) {
    let wrapped = Campaign::Strategic(Box::new(campaign.clone()));
    let mut old = serde_json::to_value(&wrapped).unwrap();
    let world = old["world"].as_object_mut().unwrap();
    world.remove("region_control");
    world.remove("contested_sites");
    let envelope = encode_slot("earlier", &old, "2").unwrap();
    assert_eq!(load_legacy(&envelope, data).unwrap(), wrapped);
    let raw = serde_json::to_string(&old).unwrap();
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let order = catalogue.reserve_identity(&mut store).unwrap();
    let metadata = SaveMetadata {
        name: "Earlier geography".into(),
        kind: SaveKind::Manual,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        schema_version: 2,
        content_version: 1,
        success_order: order,
    };
    // This writes the exact pre-K04 payload with its original version labels.
    let saved = catalogue
        .write(
            &mut store,
            "old_v2",
            None,
            metadata.clone(),
            &raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    assert_eq!(library.entries()[0].metadata, metadata);
    assert_eq!(
        library.load(&mut store, data, saved.entry_id).unwrap(),
        wrapped
    );
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);

    for field in ["region_control", "contested_sites"] {
        let mut partial = serde_json::to_value(&wrapped).unwrap();
        partial["world"].as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Campaign>(partial).is_err());
        let mut malformed = serde_json::to_value(&wrapped).unwrap();
        malformed["world"][field] = serde_json::Value::Null;
        assert!(serde_json::from_value::<Campaign>(malformed).is_err());
    }
    let mut invalid = serde_json::to_value(&wrapped).unwrap();
    invalid["world"]["region_control"] = serde_json::json!({});
    assert!(load_legacy(&encode_slot("invalid", &invalid, "2").unwrap(), data).is_err());
}

#[derive(Default)]
struct MemoryStore(BTreeMap<String, String>);

impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        self.0.insert(key.to_owned(), content.to_owned());
        Ok(())
    }
}

impl IndexedSaveStore for MemoryStore {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(WriterStatus::Ready)
    }

    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.0.remove(key);
        Ok(())
    }
}
