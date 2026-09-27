//! Completion effects, compatibility and parity at real round boundaries.

use super::*;

pub(in super::super) fn assert_facilities(owner: u32) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let site = SiteId(owner);
    let faction = FactionId(owner);
    campaign.factions.get_mut(&faction).unwrap().resources = Resources {
        gold: 5000,
        wood: 5000,
        stone: 5000,
    };
    campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .unwrap()
        .facilities
        .clear();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == site)
        .unwrap()
        .military = MilitaryLayer::None;
    for facility in [
        Facility::Stable,
        Facility::Infirmary,
        Facility::Workshop,
        Facility::TrainingGround,
        Facility::Temple,
    ] {
        build_facility(&mut campaign, &data, owner, facility);
    }
    owner_turn(&mut campaign, &data, owner);
    let fort = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(site),
        ConstructionKind::Fort,
        owner,
    );
    for _ in 0..3 {
        finish(&mut campaign, &data);
    }
    assert!(!campaign.construction[&fort].is_open());
    assert_eq!(
        campaign.world.site(site).unwrap().military,
        MilitaryLayer::Fort
    );
    owner_turn(&mut campaign, &data, owner);
    for kind in [
        TroopKind::Riders,
        TroopKind::Medics,
        TroopKind::SiegeEngines,
    ] {
        apply(
            &mut campaign,
            &data,
            actor(faction),
            Command::Recruit {
                site,
                army: None,
                kind,
            },
        )
        .unwrap();
    }
    campaign.world.site_damage.insert(site, 75);
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        actor(faction),
        Command::Recruit {
            site,
            army: None,
            kind: TroopKind::Medics
        }
    )
    .is_err());
    assert_eq!(campaign, before);
}

fn build_facility(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: u32,
    facility: Facility,
) {
    let site = SiteId(owner);
    let faction = FactionId(owner);
    owner_turn(campaign, data, owner);
    let kind = ConstructionKind::Facility(facility);
    let option = construction_options(campaign, data, faction, ConstructionTarget::Site(site))
        .into_iter()
        .find(|option| option.kind == kind)
        .unwrap();
    assert!(option.blocked.is_none());
    assert!(option.builders.contains(&ArmyId(owner)));
    let order = start(campaign, data, ConstructionTarget::Site(site), kind, owner);
    assert_eq!(campaign.construction[&order].builder, None);
    assert!(!campaign
        .world
        .site(site)
        .unwrap()
        .facilities
        .contains(&facility));
    for _ in 0..option.steps {
        finish(campaign, data);
    }
    assert!(campaign
        .world
        .site(site)
        .unwrap()
        .facilities
        .contains(&facility));
}

pub(in super::super) fn assert_road(owner: u32) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(owner)), false)
        .unwrap();
    let route = RouteId(if owner == 1 { 13 } else { 14 });
    let edge = campaign.world.route(route).unwrap().clone();
    let original = route_cost(&edge, &data);
    owner_turn(&mut campaign, &data, owner);
    let order = start(
        &mut campaign,
        &data,
        ConstructionTarget::Route(route),
        ConstructionKind::Road,
        owner,
    );
    finish(&mut campaign, &data);
    assert!(!campaign.world.route(route).unwrap().road.improved);
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&order].progress, 2);
    assert!(route_cost(campaign.world.route(route).unwrap(), &data) < original);
    let mut reverse = campaign.world.route(route).unwrap().clone();
    std::mem::swap(&mut reverse.from, &mut reverse.to);
    assert_eq!(
        route_cost(&reverse, &data),
        route_cost(campaign.world.route(route).unwrap(), &data)
    );
    campaign
        .world
        .routes
        .iter_mut()
        .find(|entry| entry.id == route)
        .unwrap()
        .road
        .damage = 100;
    owner_turn(&mut campaign, &data, owner);
    let repair = start(
        &mut campaign,
        &data,
        ConstructionTarget::Route(route),
        ConstructionKind::RoadRepair,
        owner,
    );
    assert_eq!(
        campaign.construction[&repair].paid,
        Resources {
            gold: 20,
            wood: 15,
            stone: 0
        }
    );
    finish(&mut campaign, &data);
    assert_eq!(campaign.world.route(route).unwrap().road.damage, 0);
    assert_eq!(campaign.world.route(route).unwrap().from, edge.from);
    assert_eq!(campaign.world.route(route).unwrap().to, edge.to);
}

pub(in super::super) fn assert_income_timing() {
    let (data, mut campaign) = builder();
    let baseline = kestrum::engine::recovery_preview(&campaign, &data, FactionId(1)).unwrap();
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 79;
    start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    finish(&mut campaign, &data);
    assert!(campaign.factions[&FactionId(1)]
        .last_recovery
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .any(|entry| entry.site == SiteId(5) && entry.restored > 0));
    let income = campaign.factions[&FactionId(1)]
        .last_economy
        .as_ref()
        .unwrap()
        .income;
    finish(&mut campaign, &data);
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.factions[&FactionId(1)]
            .last_economy
            .as_ref()
            .unwrap()
            .income,
        income
    );
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Outpost
    );
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.factions[&FactionId(1)]
            .last_economy
            .as_ref()
            .unwrap()
            .income
            .gold,
        income.gold + data.economy.settlement_income[&Habitation::Outpost].gold
    );
    assert!(baseline.iter().all(|entry| entry.maximum == 0));
}

pub(in super::super) fn assert_migration(data: &GameData) {
    let original = StrategicCampaign::new(data).unwrap();
    let mut old = serde_json::to_value(Campaign::Strategic(Box::new(original.clone()))).unwrap();
    old.as_object_mut().unwrap().remove("construction");
    old["next_ids"].as_object_mut().unwrap().remove("order");
    old["world"].as_object_mut().unwrap().remove("population");
    old["world"].as_object_mut().unwrap().remove("focus");
    let migrated = serde_json::from_value::<Campaign>(old.clone()).unwrap();
    migrated.validate(data).unwrap();
    assert_eq!(migrated.strategic().unwrap(), &original);
    let raw = macroquad_toolkit::persistence::encode_slot("strategic_v2", &old, "2").unwrap();
    assert_eq!(
        kestrum::state::persistence::load_legacy(&raw, data).unwrap(),
        migrated
    );
    old["world"]["focus"] = serde_json::json!({});
    assert!(serde_json::from_value::<Campaign>(old).is_err());
}

pub(in super::super) fn assert_invalid(campaign: &StrategicCampaign, data: &GameData, id: OrderId) {
    for kind in ["counter", "future", "elapsed", "target", "population"] {
        let mut invalid = campaign.clone();
        match kind {
            "counter" => invalid.next_ids.order = id,
            "future" => {
                invalid.construction.get_mut(&id).unwrap().status = ConstructionStatus::Completed {
                    completed_rounds: campaign.completed_rounds + 1,
                }
            }
            "elapsed" => {
                invalid
                    .construction
                    .get_mut(&id)
                    .unwrap()
                    .last_progress_round = Some(0)
            }
            "target" => {
                invalid.construction.get_mut(&id).unwrap().target =
                    ConstructionTarget::Site(SiteId(999))
            }
            "population" => {
                invalid.world.population.remove(&SiteId(1));
            }
            _ => unreachable!(),
        }
        assert!(invalid.validate(data).is_err(), "accepted {kind}");
    }
}
