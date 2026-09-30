//! Small fixtures and actual command helpers for K09 acceptance.

use super::*;
#[path = "construction_world.rs"]
mod world;
pub(super) use world::{
    assert_facilities, assert_income_timing, assert_invalid, assert_migration, assert_road,
};

pub(super) fn builder() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, move_order(1, &[1, 5])).unwrap();
    (data, campaign)
}
pub(super) fn move_order(army: u32, path: &[u32]) -> Command {
    Command::Move(MoveOrder {
        armies: vec![ArmyId(army)],
        path: path.iter().copied().map(SiteId).collect(),
    })
}
pub(super) fn build(target: ConstructionTarget, kind: ConstructionKind, builder: u32) -> Command {
    Command::StartConstruction {
        target,
        kind,
        builder: ArmyId(builder),
    }
}
pub(super) fn start(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    target: ConstructionTarget,
    kind: ConstructionKind,
    builder: u32,
) -> OrderId {
    let id = campaign.next_ids.order;
    let owner = campaign.armies[&ArmyId(builder)].faction;
    apply(campaign, data, actor(owner), build(target, kind, builder)).unwrap();
    id
}
pub(super) fn actor(owner: FactionId) -> Actor {
    if owner == FactionId(1) {
        Actor::Player
    } else {
        Actor::Npc(owner)
    }
}
pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    if campaign.phase == CampaignPhase::PlayerTurn {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}
pub(super) fn owner_turn(campaign: &mut StrategicCampaign, data: &GameData, owner: u32) {
    if owner != 1 {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.active_faction(), FactionId(owner));
}
pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let raw = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    kestrum::state::persistence::load_legacy(&raw, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}
pub(super) fn subtract(a: Resources, b: Resources) -> Resources {
    Resources {
        gold: a.gold - b.gold,
        wood: a.wood - b.wood,
        stone: a.stone - b.stone,
    }
}
pub(super) fn population(campaign: &StrategicCampaign) -> u64 {
    campaign
        .world
        .population
        .values()
        .map(|count| u64::from(*count))
        .sum()
}
pub(super) fn topology(campaign: &StrategicCampaign) -> Vec<(u32, u32, u32)> {
    campaign
        .world
        .routes
        .iter()
        .map(|route| (route.id.0, route.from.0, route.to.0))
        .collect()
}

pub(super) fn assert_combat(capture: bool) {
    let (data, mut campaign) = builder();
    campaign
        .set_site_control(&data, SiteId(6), Some(FactionId(1)), false)
        .unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(8);
    let weakened = if capture { ArmyId(1) } else { ArmyId(3) };
    let keep = campaign.armies[&weakened].formation_ids().next().unwrap();
    let remove: Vec<_> = campaign.armies[&weakened]
        .formation_ids()
        .filter(|id| *id != keep)
        .collect();
    for id in remove {
        campaign.remove_formation(id).unwrap();
    }
    campaign.formations.get_mut(&keep).unwrap().headcount = 1;
    let order = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(6)),
        ConstructionKind::Fort,
        1,
    );
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    pass_npc(&mut campaign, &data).unwrap();
    let balance = campaign.factions[&FactionId(1)].resources;
    let contact = apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        move_order(3, &[8, 6]),
    )
    .unwrap();
    assert!(contact.battle_pending);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    if capture {
        assert_eq!(
            campaign.construction[&order].status,
            ConstructionStatus::Cancelled {
                completed_rounds: 0,
                reason: CancellationReason::ControlLost
            }
        );
        assert_eq!(campaign.factions[&FactionId(1)].resources, balance);
    } else {
        assert_eq!(
            campaign.construction[&order].status,
            ConstructionStatus::Paused {
                reason: ConstructionPause::Combat
            }
        );
        finish(&mut campaign, &data);
        assert_eq!(campaign.construction[&order].progress, 0);
        assert_eq!(
            campaign.construction[&order].status,
            ConstructionStatus::Active
        );
        finish(&mut campaign, &data);
        assert_eq!(campaign.construction[&order].progress, 1);
    }
    campaign.validate(&data).unwrap();
}

pub(super) fn assert_missing_builder() {
    let (data, mut campaign) = builder();
    let order = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    // The kingdom retains its HQ; this case isolates a lost builder from defeat.
    for formation in [1, 2, 3] {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::Disband {
                formation: FormationId(formation),
            },
        )
        .unwrap();
    }
    assert_eq!(campaign.construction[&order].builder, None);
    assert_eq!(
        campaign.construction[&order].status,
        ConstructionStatus::Paused {
            reason: ConstructionPause::BuilderMissing
        }
    );
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_frozen_donors() {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    data.scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(13))
        .unwrap()
        .tags
        .retain(|tag| *tag != kestrum::data::world::SiteTag::Ruins);
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, move_order(1, &[1, 5])).unwrap();
    for id in [6, 8, 10, 12, 13] {
        campaign
            .set_site_control(&data, SiteId(id), Some(FactionId(1)), false)
            .unwrap();
    }
    campaign.world.population.insert(SiteId(1), 250);
    let second = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(2),
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    campaign.armies.get_mut(&second).unwrap().site = SiteId(13);
    let camp = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(12)),
        ConstructionKind::Outpost,
        1,
    );
    let empty = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(13)),
        ConstructionKind::Outpost,
        second.0,
    );
    for _ in 0..2 {
        finish(&mut campaign, &data);
    }
    // This acceptance snapshot has exactly one eligible50-person donor before either final step.
    for site in &campaign.world.sites {
        if site.controller == Some(FactionId(1)) {
            campaign.world.population.insert(
                site.id,
                data.construction.population.minimum[&site.habitation],
            );
        }
    }
    campaign.world.population.insert(SiteId(1), 250);
    let total = population(&campaign);
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&camp].progress, 3);
    assert_eq!(campaign.construction[&empty].progress, 2);
    assert_eq!(campaign.world.population[&SiteId(12)], 70);
    assert_eq!(campaign.world.population[&SiteId(13)], 0);
    assert_eq!(
        population(&campaign),
        total + 23,
        "four HQs, the Town and City grow after construction"
    );
    finish(&mut campaign, &data);
    assert_eq!(campaign.world.population[&SiteId(12)], 51);
    assert_eq!(campaign.world.population[&SiteId(13)], 20);
    assert_eq!(
        population(&campaign),
        total + 47,
        "the completed Outpost also grows on the following boundary"
    );
}
