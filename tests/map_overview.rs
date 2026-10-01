//! M01 summaries preserve secrecy while connecting current conditions to map objects.

use kestrum::{
    data::{
        economy::Habitation,
        world::{FactionId, MarkerId, SiteId},
        GameData,
    },
    engine::{self, Actor, ArmyMapStatus, AttentionKind, AttentionTarget, Command, MoveOrder},
    state::{
        military::{ArmyId, FormationId},
        movement::MovementPlan,
        CampaignPhase, StrategicCampaign,
    },
};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn overview(campaign: &StrategicCampaign) -> engine::MapOverview {
    engine::map_overview(&engine::project_map(campaign, campaign.player).unwrap())
}

fn discover_region(campaign: &mut StrategicCampaign) {
    campaign
        .knowledge
        .explored
        .entry(campaign.player)
        .or_default()
        .extend((5..=14).map(SiteId));
}

#[test]
fn summaries_never_reveal_hidden_land_enemy_strength_orders_or_capitals() {
    let (_, mut campaign) = fixture();
    let initial = overview(&campaign);
    assert_eq!(
        initial.armies.keys().copied().collect::<Vec<_>>(),
        [ArmyId(1)]
    );
    assert!(!initial.sites.contains_key(&SiteId(14)));
    assert!(!initial.markers[&MarkerId(5)].political_known);
    assert!(initial.sites[&SiteId(1)].capital);
    assert_eq!(
        engine::map_overview(&engine::project(&campaign, campaign.player).unwrap()),
        initial,
        "full projections must not let the overview bypass discovery"
    );
    let hidden = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(14))
        .unwrap();
    hidden.name = "Unobserved secret city".into();
    hidden.habitation = Habitation::MajorCity;
    hidden.controller = Some(FactionId(3));
    campaign.armies.get_mut(&ArmyId(3)).unwrap().name = "Hidden host".into();
    campaign
        .formations
        .get_mut(&FormationId(7))
        .unwrap()
        .headcount = 1;
    campaign.movement_plans.push(MovementPlan {
        armies: vec![ArmyId(3)],
        path: vec![SiteId(3), SiteId(11)],
    });
    campaign.factions.get_mut(&FactionId(3)).unwrap().capital = SiteId(14);
    campaign
        .world
        .region_control
        .get_mut(&MarkerId(5))
        .unwrap()
        .political_owner = Some(FactionId(3));
    assert_eq!(overview(&campaign), initial);
    assert_eq!(
        engine::map_overview(&engine::project(&campaign, campaign.player).unwrap()),
        initial
    );
}

#[test]
fn mixed_region_keeps_political_claim_occupation_contest_and_neutral_distinct() {
    let (data, mut campaign) = fixture();
    discover_region(&mut campaign);
    for id in [5, 6, 8, 9, 10] {
        campaign
            .set_site_control(&data, SiteId(id), Some(campaign.player), false)
            .unwrap();
    }
    campaign
        .set_site_control(&data, SiteId(11), Some(FactionId(3)), false)
        .unwrap();
    let view = overview(&campaign);
    let region = &view.markers[&MarkerId(5)];
    assert_eq!(region.political_owner, Some(campaign.player));
    assert!(region.political_known && region.mixed_control && region.occupied);
    assert!(!region.contested);
    assert!(view.sites[&SiteId(11)].occupied);
    assert_eq!(view.sites[&SiteId(11)].controller, Some(FactionId(3)));
    assert!(view
        .attention
        .iter()
        .any(|entry| entry.target == AttentionTarget::Site(SiteId(11))
            && entry.kind == AttentionKind::Occupied));
    assert_eq!(view.sites[&SiteId(7)].controller, None);
    assert!(!view.sites[&SiteId(7)].occupied);
    campaign
        .set_site_control(&data, SiteId(9), Some(campaign.player), true)
        .unwrap();
    let changed = overview(&campaign);
    assert_eq!(
        changed.markers[&MarkerId(5)].political_owner,
        Some(campaign.player)
    );
    assert!(changed.markers[&MarkerId(5)].contested);
    assert!(changed.sites[&SiteId(9)].contested);
    assert!(changed.attention.iter().any(|entry| {
        entry.target == AttentionTarget::Site(SiteId(9)) && entry.kind == AttentionKind::Contested
    }));
}

#[test]
fn region_danger_aggregates_only_observed_internal_contacts_and_threats() {
    let (data, mut campaign) = fixture();
    discover_region(&mut campaign);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(10);
    campaign.validate(&data).unwrap();
    let view = overview(&campaign);
    let danger = view.markers[&MarkerId(5)].danger;
    assert_eq!(
        (danger.threats, danger.hostile_contacts, danger.sieges),
        (1, 1, 0)
    );
    assert!(danger.any());
    assert_eq!(danger.count(), 2);
    assert_eq!(view.sites[&SiteId(13)].danger.threats, 1);
    assert_eq!(
        view.sites[&SiteId(14)].danger.threats,
        0,
        "discovery alone does not observe a threat"
    );
    assert_eq!(view.attention[0].target, AttentionTarget::Site(SiteId(10)));
    assert_eq!(view.attention[0].kind, AttentionKind::HostileContact);
    assert_eq!(view.attention[1].target, AttentionTarget::Site(SiteId(13)));
    assert_eq!(view.attention[1].kind, AttentionKind::LocalThreat);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(1);
    let left = overview(&campaign);
    assert!(!left.markers[&MarkerId(5)].danger.any());
    assert!(left.attention.is_empty());
}

#[test]
fn owned_development_force_orders_and_actual_seasonal_receipts_refresh() {
    let (mut data, _) = fixture();
    data.threats.initial.clear();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let initial = overview(&campaign);
    campaign
        .set_site_control(&data, SiteId(5), Some(campaign.player), false)
        .unwrap();
    campaign
        .set_site_control(&data, SiteId(6), Some(campaign.player), false)
        .unwrap();
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(6))
        .unwrap();
    site.habitation = Habitation::City;
    campaign.factions.get_mut(&campaign.player).unwrap().capital = SiteId(6);
    let army = campaign.armies.get_mut(&ArmyId(1)).unwrap();
    army.name = "Renamed home guard".into();
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 72;
    campaign.movement_plans.push(MovementPlan {
        armies: vec![ArmyId(1)],
        path: vec![SiteId(1), SiteId(5)],
    });
    let changed = overview(&campaign);
    assert_eq!(changed.sites[&SiteId(6)].habitation, Habitation::City);
    assert!(changed.sites[&SiteId(6)].capital && !changed.sites[&SiteId(1)].capital);
    assert!(changed.markers[&MarkerId(5)].capital);
    assert_eq!(changed.armies[&ArmyId(1)].name, "Renamed home guard");
    assert_eq!(
        changed.armies[&ArmyId(1)].troops,
        initial.armies[&ArmyId(1)].troops - 28
    );
    assert_eq!(changed.armies[&ArmyId(1)].status, ArmyMapStatus::Queued);
    campaign.validate(&data).unwrap();
    let before = engine::project_map(&campaign, campaign.player).unwrap();
    assert!(before
        .factions
        .iter()
        .find(|faction| faction.id == campaign.player)
        .unwrap()
        .last_economy
        .is_none());
    engine::apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        engine::apply(&mut campaign, &data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    let after = engine::project_map(&campaign, campaign.player).unwrap();
    let owned = after
        .factions
        .iter()
        .find(|faction| faction.id == campaign.player)
        .unwrap();
    let authoritative = &campaign.factions[&campaign.player];
    assert_eq!(owned.resources, Some(authoritative.resources));
    assert_eq!(owned.last_economy, authoritative.last_economy);
    let receipt = owned.last_economy.as_ref().unwrap();
    assert!(receipt.income.gold > 0 && receipt.upkeep_due > 0 && receipt.upkeep_paid > 0);
    assert!(after
        .factions
        .iter()
        .filter(|faction| faction.id != campaign.player)
        .all(|faction| faction.resources.is_none() && faction.last_economy.is_none()));
    assert_eq!(
        engine::map_overview(&after).armies[&ArmyId(1)].status,
        ArmyMapStatus::Idle
    );
}

#[test]
fn known_siege_warns_on_world_marker_and_uses_real_besieger_supply() {
    let (mut data, _) = fixture();
    data.threats.initial.clear();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign.people.clear();
    campaign.legacy_items.clear();
    for (army_id, formation_id, site) in [(1, 1, 8), (3, 7, 9)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).unwrap();
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
        army.commander = None;
        army.site = SiteId(site);
    }
    for site in [5, 6, 8] {
        campaign
            .set_site_control(&data, SiteId(site), Some(campaign.player), false)
            .unwrap();
    }
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(8), SiteId(9)],
        }),
    )
    .unwrap();
    let view = engine::project_map(&campaign, campaign.player).unwrap();
    assert!(!view.supplied_sites.contains(&SiteId(9)));
    let summary = engine::map_overview(&view);
    assert_eq!(summary.markers[&MarkerId(5)].danger.sieges, 1);
    assert_eq!(summary.armies[&ArmyId(1)].status, ArmyMapStatus::Siege);
    assert!(summary.armies[&ArmyId(1)].supplied);
    assert_eq!(summary.attention[0].kind, AttentionKind::Siege);
    assert_eq!(
        summary.attention[0].target,
        AttentionTarget::Site(SiteId(9))
    );
    assert!(!summary
        .attention
        .iter()
        .any(|entry| entry.target == AttentionTarget::Army(ArmyId(1))));
    let defender = engine::map_overview(&engine::project_map(&campaign, FactionId(3)).unwrap());
    assert!(!defender.armies[&ArmyId(3)].supplied);
    assert!(defender
        .attention
        .iter()
        .any(|entry| entry.target == AttentionTarget::Army(ArmyId(3))
            && entry.kind == AttentionKind::Unsupplied));
}
