//! Sparse validated physical fixtures, followed only by real accepted orders.

use super::*;
use kestrum::state::military::{Army, Formation};

pub(super) fn fixture(defending: bool, first: u32, second: u32) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign.people.clear();
    for (id, formation, count) in [(1, 1, first), (3, 7, second)] {
        let army = campaign.armies.get_mut(&ArmyId(id)).unwrap();
        army.slots = [Some(FormationId(formation)), None, None, None, None, None];
        army.commander = None;
        army.site = SiteId(if (id == 1) == defending { 9 } else { 8 });
        campaign
            .formations
            .get_mut(&FormationId(formation))
            .unwrap()
            .headcount = count;
    }
    let besieger = FactionId(if defending { 3 } else { 1 });
    let defender = FactionId(if defending { 1 } else { 3 });
    for site in [5, 6, 8] {
        campaign
            .set_site_control(&data, SiteId(site), Some(besieger), false)
            .unwrap();
    }
    campaign
        .set_site_control(&data, SiteId(9), Some(defender), false)
        .unwrap();
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn enter(armies: &[u32], from: u32, to: u32) -> Command {
    Command::Move(MoveOrder {
        armies: armies.iter().copied().map(ArmyId).collect(),
        path: vec![SiteId(from), SiteId(to)],
    })
}
pub(super) fn order(action: SiegeAction, armies: &[u32], destination: Option<u32>) -> Command {
    Command::Siege(SiegeOrder {
        site: SiteId(9),
        action,
        armies: armies.iter().copied().map(ArmyId).collect(),
        destination: destination.map(SiteId),
    })
}
pub(super) fn establish(campaign: &mut StrategicCampaign, data: &GameData, defending: bool) {
    if defending {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
        advance_npc(campaign, data).unwrap();
        apply(campaign, data, Actor::Npc(FactionId(3)), enter(&[3], 8, 9)).unwrap();
    } else {
        establish_group(campaign, data, &[1]);
    }
}
pub(super) fn establish_group(campaign: &mut StrategicCampaign, data: &GameData, armies: &[u32]) {
    apply(campaign, data, Actor::Player, enter(armies, 8, 9)).unwrap();
}
pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}
pub(super) fn fight(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    action: SiegeAction,
    armies: &[u32],
    destination: Option<u32>,
) -> BattleReport {
    let result = apply(campaign, data, actor, order(action, armies, destination)).unwrap();
    campaign.battles[&result.battle.unwrap()].clone()
}
pub(super) fn first_loss(report: &BattleReport, formation: u32) -> u32 {
    report.exchanges[0]
        .losses
        .iter()
        .find(|loss| loss.formation == FormationId(formation))
        .unwrap()
        .amount
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
pub(super) fn assert_rejected(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) {
    let before = campaign.clone();
    assert!(apply(campaign, data, actor, command).is_err());
    assert_eq!(*campaign, before);
}
pub(super) struct Force {
    pub id: u32,
    pub owner: u32,
    pub site: u32,
    pub formation: u32,
    pub kind: TroopKind,
    pub headcount: u32,
}

pub(super) fn add(campaign: &mut StrategicCampaign, data: &GameData, force: Force) {
    let Force {
        id,
        owner,
        site,
        formation,
        kind,
        headcount,
    } = force;
    campaign.armies.insert(
        ArmyId(id),
        Army {
            id: ArmyId(id),
            faction: FactionId(owner),
            site: SiteId(site),
            name: format!("Test army {id}"),
            slots: [Some(FormationId(formation)), None, None, None, None, None],
            commander: None,
        },
    );
    campaign.formations.insert(
        FormationId(formation),
        Formation {
            id: FormationId(formation),
            faction: FactionId(owner),
            kind,
            headcount,
            capacity: data.economy.formations[&kind].capacity,
            movement_spent: 0,
            created_round: 0,
            service: Default::default(),
        },
    );
    campaign.next_ids.army = ArmyId(campaign.next_ids.army.0.max(id + 1));
    campaign.next_ids.formation = FormationId(campaign.next_ids.formation.0.max(formation + 1));
}

pub(super) fn assert_adjacent_supply() {
    let (data, mut campaign) = fixture(false, 30, 30);
    for site in [8, 10] {
        campaign
            .world
            .sites
            .iter_mut()
            .find(|entry| entry.id == SiteId(site))
            .unwrap()
            .military = MilitaryLayer::Fort;
    }
    campaign
        .set_site_control(&data, SiteId(9), Some(FactionId(1)), false)
        .unwrap();
    for site in [8, 10] {
        campaign
            .set_site_control(&data, SiteId(site), Some(FactionId(3)), false)
            .unwrap();
    }
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(8);
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 1,
            site: 9,
            formation: 13,
            kind: TroopKind::Warriors,
            headcount: 30,
        },
    );
    add(
        &mut campaign,
        &data,
        Force {
            id: 6,
            owner: 3,
            site: 10,
            formation: 14,
            kind: TroopKind::Warriors,
            headcount: 30,
        },
    );
    apply(&mut campaign, &data, Actor::Player, enter(&[1], 6, 8)).unwrap();
    apply(&mut campaign, &data, Actor::Player, enter(&[5], 9, 10)).unwrap();
    assert!(campaign.army_is_supplied(ArmyId(1)));
    assert!(!campaign.army_is_supplied(ArmyId(5)));
    let previews = recovery_preview(&campaign, &data, FactionId(1)).unwrap();
    assert!(
        previews
            .iter()
            .find(|entry| entry.army == ArmyId(1))
            .unwrap()
            .restored
            > 0
    );
    assert_eq!(
        previews
            .iter()
            .find(|entry| entry.army == ArmyId(5))
            .unwrap()
            .restored,
        0
    );
    finish(&mut campaign, &data);
    assert!(campaign.formations[&FormationId(1)].headcount > 30);
    assert_eq!(campaign.formations[&FormationId(13)].headcount, 30);
}

pub(super) fn assert_assault_capture_and_roads() {
    let (data, mut campaign) = fixture(false, 100, 1);
    let route = campaign
        .world
        .connected_route(SiteId(8), SiteId(9))
        .unwrap()
        .id;
    campaign
        .world
        .routes
        .iter_mut()
        .find(|entry| entry.id == route)
        .unwrap()
        .road
        .improved = true;
    establish(&mut campaign, &data, false);
    finish(&mut campaign, &data);
    let report = fight(
        &mut campaign,
        &data,
        Actor::Player,
        SiegeAction::Assault,
        &[1],
        None,
    );
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert!(!campaign.formations.contains_key(&FormationId(7)));
    assert!(campaign.sieges.is_empty());
    assert_eq!(report.control_after, Some(FactionId(1)));
    assert_eq!(report.structural_damage_added, 20);
    assert_eq!(report.road_damage.as_ref().unwrap().added, 10);
    assert_eq!(campaign.world.route(route).unwrap().road.damage, 10);
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_escape(win: bool) {
    let (data, mut campaign) = fixture(true, 100, if win { 10 } else { 100 });
    establish(&mut campaign, &data, true);
    finish(&mut campaign, &data);
    let report = fight(
        &mut campaign,
        &data,
        Actor::Player,
        SiegeAction::Escape,
        &[1],
        Some(14),
    );
    if win {
        assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
        assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    } else {
        assert!(matches!(
            report.outcome,
            BattleOutcome::Stalemate | BattleOutcome::DefenderVictory
        ));
        assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(9));
    }
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_relief(win: bool) {
    let (data, mut campaign) = fixture(true, if win { 1 } else { 30 }, if win { 1 } else { 100 });
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 1,
            site: 14,
            formation: 13,
            kind: TroopKind::Warriors,
            headcount: if win { 100 } else { 20 },
        },
    );
    establish(&mut campaign, &data, true);
    finish(&mut campaign, &data);
    let result = apply(&mut campaign, &data, Actor::Player, enter(&[5], 14, 9)).unwrap();
    let report = &campaign.battles[&result.battle.unwrap()];
    assert!(
        matches!(&report.context,BattleContext::Relief{garrison,..} if garrison==&vec![ArmyId(1)])
    );
    assert_eq!(report.attacker.armies.len(), 2);
    if win {
        assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
        assert!(!campaign.armies.contains_key(&ArmyId(1)));
        assert_eq!(campaign.armies[&ArmyId(5)].site, SiteId(9));
        assert!(campaign.sieges.is_empty());
    } else {
        assert!(matches!(
            report.outcome,
            BattleOutcome::DefenderVictory | BattleOutcome::Stalemate
        ));
        if let Some(army) = campaign.armies.get(&ArmyId(1)) {
            assert_eq!(army.site, SiteId(9));
        }
        if let Some(army) = campaign.armies.get(&ArmyId(5)) {
            assert_eq!(army.site, SiteId(14));
        }
    }
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_third_faction() {
    let (data, mut campaign) = fixture(false, 1, 100);
    for relation in &mut campaign.relations {
        relation.state = DiplomaticState::War;
    }
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 2,
            site: 14,
            formation: 13,
            kind: TroopKind::Warriors,
            headcount: 100,
        },
    );
    establish(&mut campaign, &data, false);
    finish(&mut campaign, &data);
    let old = campaign.sieges[&SiteId(9)].id;
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let result = apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        enter(&[5], 14, 9),
    )
    .unwrap();
    assert_eq!(
        campaign.battles[&result.battle.unwrap()].outcome,
        BattleOutcome::AttackerVictory
    );
    let siege = &campaign.sieges[&SiteId(9)];
    assert!(siege.id > old);
    assert_eq!(siege.elapsed_steps, 0);
    assert_eq!(siege.besieger, FactionId(2));
    assert_eq!(siege.defending, vec![ArmyId(3)]);
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 5);
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_peace(trapped: bool) {
    let (data, mut campaign) = fixture(false, 100, 100);
    establish(&mut campaign, &data, false);
    if trapped {
        for site in [8, 10, 14] {
            campaign
                .set_site_control(&data, SiteId(site), Some(FactionId(2)), false)
                .unwrap();
        }
    }
    for relation in &mut campaign.relations {
        if relation.factions == [FactionId(1), FactionId(3)] {
            relation.state = DiplomaticState::Peace;
        }
    }
    let before = campaign.clone();
    let result = reconcile_sieges(&mut campaign, &data);
    if trapped {
        assert!(result.is_err());
        assert_eq!(campaign, before);
    } else {
        result.unwrap();
        assert!(campaign.sieges.is_empty());
        assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(8));
        assert_eq!(campaign.army_movement_remaining(ArmyId(1), &data), Some(0));
        assert_eq!(reload(&campaign, &data), campaign);
    }
}

pub(super) fn assert_admission() {
    let (data, mut campaign) = fixture(false, 100, 100);
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 2,
            site: 7,
            formation: 13,
            kind: TroopKind::Warriors,
            headcount: 100,
        },
    );
    establish(&mut campaign, &data, false);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let command = Command::Move(MoveOrder {
        armies: vec![ArmyId(5)],
        path: vec![SiteId(7), SiteId(14), SiteId(9)],
    });
    let result = apply(&mut campaign, &data, Actor::Npc(FactionId(2)), command).unwrap();
    let moved = result.movement.unwrap();
    assert_eq!(moved.path, vec![SiteId(7), SiteId(14)]);
    assert_eq!(moved.stop.unwrap().site, SiteId(9));
    assert_eq!(campaign.armies[&ArmyId(5)].site, SiteId(14));
    assert_eq!(
        campaign.world.site(SiteId(14)).unwrap().controller,
        Some(FactionId(2))
    );
    assert_eq!(campaign.sieges[&SiteId(9)].besieging, vec![ArmyId(1)]);
    assert_rejected(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        enter(&[5], 14, 9),
    );
    let (data, mut neutral) = fixture(false, 100, 100);
    neutral
        .set_site_control(&data, SiteId(9), None, false)
        .unwrap();
    for relation in &mut neutral.relations {
        relation.state = DiplomaticState::Peace;
    }
    let result = apply(&mut neutral, &data, Actor::Player, enter(&[1], 8, 9)).unwrap();
    assert!(result.battle.is_none());
    assert!(neutral.sieges.is_empty());
    assert_eq!(neutral.world.site(SiteId(9)).unwrap().controller, None);
    assert_eq!(neutral.armies[&ArmyId(1)].site, SiteId(9));
    assert_eq!(neutral.armies[&ArmyId(3)].site, SiteId(9));
}

pub(super) fn assert_engine_destruction() {
    let (data, mut campaign) = fixture(false, 100, 100);
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 1,
            site: 8,
            formation: 13,
            kind: TroopKind::SiegeEngines,
            headcount: 1,
        },
    );
    establish_group(&mut campaign, &data, &[1, 5]);
    campaign
        .set_site_control(&data, SiteId(1), None, false)
        .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(campaign.formations[&FormationId(13)].headcount, 1);
    let report = fight(
        &mut campaign,
        &data,
        Actor::Player,
        SiegeAction::Assault,
        &[1, 5],
        None,
    );
    assert_eq!(report.exchanges[0].wall_permille, 1210);
    assert!(!campaign.formations.contains_key(&FormationId(13)));
    assert!(report
        .exchanges
        .iter()
        .skip(2)
        .any(|exchange| exchange.wall_permille == 1410));
}

pub(super) fn assert_civilian_supply() {
    use kestrum::state::construction::{
        ConstructionKind, ConstructionPause, ConstructionStatus, ConstructionTarget,
    };
    let (data, mut campaign) = fixture(false, 100, 100);
    for site in [7, 14] {
        campaign
            .set_site_control(&data, SiteId(site), Some(FactionId(1)), false)
            .unwrap();
    }
    for site in &campaign.world.sites {
        if site.controller == Some(FactionId(1)) {
            campaign.world.population.insert(
                site.id,
                data.construction.population.minimum[&site.habitation],
            );
        }
    }
    campaign.world.population.insert(SiteId(9), 200);
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 1,
            site: 14,
            formation: 13,
            kind: TroopKind::Warriors,
            headcount: 100,
        },
    );
    establish(&mut campaign, &data, false);
    let order = campaign.next_ids.order;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(14)),
            kind: ConstructionKind::Outpost,
            builder: ArmyId(5),
        },
    )
    .unwrap();
    for _ in 0..3 {
        finish(&mut campaign, &data);
    }
    assert_eq!(campaign.world.population[&SiteId(9)], 200);
    assert_eq!(campaign.world.population[&SiteId(14)], 0);
    assert_eq!(
        campaign.construction[&order].status,
        ConstructionStatus::Paused {
            reason: ConstructionPause::NoSettlers
        }
    );
}

pub(super) fn assert_elimination() {
    let (data, mut campaign) = fixture(false, 100, 100);
    establish(&mut campaign, &data, false);
    campaign.factions.get_mut(&FactionId(3)).unwrap().status =
        kestrum::state::FactionStatus::Eliminated;
    let before = campaign.clone();
    assert!(reconcile_sieges(&mut campaign, &data).is_err());
    assert_eq!(campaign, before);
    campaign.remove_formation(FormationId(7)).unwrap();
    reconcile_sieges(&mut campaign, &data).unwrap();
    assert!(campaign.sieges.is_empty());
    assert_eq!(
        campaign.world.site(SiteId(9)).unwrap().controller,
        Some(FactionId(1))
    );
    assert_eq!(reload(&campaign, &data), campaign);
}
