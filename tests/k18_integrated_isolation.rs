//! K18 keeps veteran recovery blocked across a real Outpost supply cut and save.

use kestrum::{
    data::{
        economy::{Habitation, TroopKind},
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{apply, recovery_preview, Actor, Command, MoveOrder},
    state::{
        construction::{ConstructionKind, ConstructionTarget},
        evidence::Veterancy,
        military::{ArmyId, FormationId},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

#[test]
fn isolated_outpost_veteran_stays_wounded_through_save_then_recovers_for_the_receipt_cost() {
    let mut data = GameData::load().expect("Rosemarch data");
    // Keep the authored checkpoint quiet so the case measures supply and recovery,
    // while all campaign boundaries still run through the normal command engine.
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    data.threats.initial.clear();
    let mut campaign = StrategicCampaign::new(&data).expect("Rosemarch campaign");
    let rose = FactionId(1);
    let rival = FactionId(2);
    let hq = SiteId(1);
    let cut = SiteId(6);
    let outpost = SiteId(8);
    let army = ArmyId(1);
    let veteran = FormationId(1);

    // Author a valid forward-control checkpoint on Rosemarch's real 1-5-6-8
    // corridor, then complete the Outpost through ordinary construction turns.
    for site in [SiteId(5), cut, outpost] {
        campaign
            .set_site_control(&data, site, Some(rose), false)
            .expect("valid Rose control checkpoint");
    }
    campaign.armies.get_mut(&army).unwrap().site = outpost;
    for formation in campaign.armies[&army].formation_ids() {
        campaign
            .formations
            .get_mut(&formation)
            .unwrap()
            .movement_spent = 0;
    }
    campaign.validate(&data).expect("valid outpost checkpoint");

    let relief_army = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(2),
        },
    )
    .expect("split a separate route-restoring force")
    .split_army
    .expect("new Rose army");

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(outpost),
            kind: ConstructionKind::Outpost,
            builder: army,
        },
    )
    .expect("place a legal Outpost order");
    for _ in 0..data.construction.outpost_steps {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(
        campaign.world.site(outpost).unwrap().habitation,
        Habitation::Outpost
    );
    assert!(campaign.army_is_supplied(army));

    // This depleted unit retains real Veteran service metadata while Oak legally
    // occupies West Gate, cutting the route to Milltown and the forward Outpost.
    let service = &mut campaign.formations.get_mut(&veteran).unwrap().service;
    service.xp = data.progression.veteran_xp;
    service.tier = Veterancy::Veteran;
    let formation = campaign.formations.get_mut(&veteran).unwrap();
    formation.headcount = 21;
    formation.movement_spent = 0;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar { faction: rival },
    )
    .expect("declare war before the hostile route cut");
    campaign.validate(&data).expect("valid isolated checkpoint");

    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).expect("end Rose's turn");
    assert_eq!(campaign.active_faction(), rival);
    let occupied = apply(
        &mut campaign,
        &data,
        Actor::Npc(rival),
        Command::Move(MoveOrder {
            armies: vec![ArmyId(2)],
            path: vec![SiteId(2), SiteId(5)],
        }),
    )
    .expect("Oak legally captures the supply link");
    assert_eq!(occupied.battle, None);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(rival)
    );
    let returned = apply(
        &mut campaign,
        &data,
        Actor::Npc(rival),
        Command::Move(MoveOrder {
            armies: vec![ArmyId(2)],
            path: vec![SiteId(5), SiteId(2)],
        }),
    )
    .expect("Oak returns its force without restoring control");
    assert_eq!(returned.battle, None);
    assert_eq!(campaign.armies[&ArmyId(2)].site, SiteId(2));
    finish_npc_round(&mut campaign, &data);
    assert_eq!(
        campaign.completed_rounds,
        data.construction.outpost_steps + 1
    );
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(rival)
    );
    assert!(campaign.world.supply_path(rose, hq, outpost).is_none());
    assert!(!campaign.army_is_supplied(army));

    let blocked = recovery_preview(&campaign, &data, rose)
        .expect("recovery forecast")
        .into_iter()
        .find(|entry| entry.formation == veteran)
        .expect("veteran recovery row");
    assert_eq!(blocked.site, outpost);
    assert_eq!(blocked.headcount_before, 21);
    assert_eq!(blocked.restored, 0);
    assert!(blocked
        .blocked
        .as_deref()
        .is_some_and(|reason| reason.contains("Cut off")));

    assert_eq!(campaign.formations[&veteran].headcount, 21);
    assert_eq!(
        campaign.formations[&veteran].service.tier,
        Veterancy::Veteran
    );
    let cut_receipt = campaign.factions[&rose]
        .last_recovery
        .as_ref()
        .expect("completed boundary recovery receipt");
    assert!(cut_receipt.entries.is_empty());
    assert_eq!(cut_receipt.gold_spent, 0);

    let mut resumed = save_and_load(&campaign, &data);
    assert_eq!(resumed, campaign, "isolation and its receipt round-trip");
    assert_eq!(resumed.formations[&veteran].headcount, 21);
    assert!(resumed.world.supply_path(rose, hq, outpost).is_none());

    // Restore the cut link in both branches. A preview is still only a forecast;
    // the next real boundary must create one identical paid recovery receipt.
    let retake = Command::Move(MoveOrder {
        armies: vec![relief_army],
        path: vec![outpost, cut, SiteId(5)],
    });
    let result = apply(&mut campaign, &data, Actor::Player, retake.clone())
        .expect("Rose retakes the empty supply link through normal movement");
    let replayed_result = apply(&mut resumed, &data, Actor::Player, retake)
        .expect("loaded Rose force retakes the same supply link");
    assert_eq!(result, replayed_result);
    assert_eq!(result.battle, None);
    assert_eq!(campaign.armies[&relief_army].site, SiteId(5));
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(rose)
    );
    assert_eq!(
        campaign.world.supply_path(rose, hq, outpost),
        Some(vec![hq, SiteId(5), cut, outpost])
    );
    assert!(campaign.army_is_supplied(army));

    let forecast = recovery_preview(&campaign, &data, rose)
        .expect("recovery forecast after supply returns")
        .into_iter()
        .find(|entry| entry.formation == veteran)
        .expect("veteran recovery row");
    assert_eq!((forecast.maximum, forecast.restored), (20, 20));
    assert_eq!(forecast.gold_cost, 6);
    finish_round(&mut campaign, &data);
    finish_round(&mut resumed, &data);
    assert_eq!(campaign, resumed, "loaded route restoration resolves once");
    assert_eq!(campaign.formations[&veteran].headcount, 41);
    assert_eq!(
        campaign.formations[&veteran].service.tier,
        Veterancy::Veteran
    );
    let recovery = campaign.factions[&rose]
        .last_recovery
        .as_ref()
        .expect("restored recovery receipt");
    assert_eq!(recovery.entries.len(), 1);
    let entry = &recovery.entries[0];
    assert_eq!(
        (entry.army, entry.formation, entry.site, entry.kind),
        (army, veteran, outpost, TroopKind::Warriors)
    );
    assert_eq!((entry.headcount_before, entry.restored), (21, 20));
    assert_eq!(entry.gold_cost, forecast.gold_cost);
    assert_eq!(recovery.gold_spent, 6);
    assert_eq!(
        campaign.factions[&rose].resources.gold,
        recovery.closing_gold
    );
    let economy = campaign.factions[&rose]
        .last_economy
        .as_ref()
        .expect("economy resolves before recovery");
    assert_eq!(recovery.opening_gold, economy.closing.gold);
    assert_eq!(
        recovery.closing_gold,
        recovery.opening_gold - recovery.gold_spent
    );
    assert_eq!(
        campaign.factions[&rose].resources.wood,
        economy.closing.wood
    );
    assert_eq!(
        campaign.factions[&rose].resources.stone,
        economy.closing.stone
    );
    campaign
        .validate(&data)
        .expect("recovered campaign remains valid");
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let opening_round = campaign.completed_rounds;
    if campaign.phase == CampaignPhase::PlayerTurn {
        apply(campaign, data, Actor::Player, Command::EndTurn).expect("end Rose turn");
    }
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).expect("pass NPC turn");
    }
    assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
    assert_eq!(campaign.completed_rounds, opening_round + 1);
}

fn finish_npc_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let opening_round = campaign.completed_rounds;
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).expect("pass NPC turn");
    }
    assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
    assert_eq!(campaign.completed_rounds, opening_round + 1);
}

fn save_and_load(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let raw = encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .expect("encode isolation checkpoint");
    kestrum::state::persistence::load_legacy(&raw, data)
        .expect("load isolation checkpoint")
        .strategic()
        .expect("strategic campaign")
        .clone()
}
