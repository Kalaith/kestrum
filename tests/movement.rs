//! K06 physical costs, interruption, road access, and stable composition changes.

use kestrum::{
    data::{
        economy::TroopKind,
        world::{FactionId, RouteId, SiteId},
        GameData,
    },
    engine::{
        apply, army_remaining, formation_remaining, movement_preview, person_remaining, preview,
        Actor, Command, MoveOrder, MovementBlock,
    },
    state::{
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        Campaign, StrategicCampaign,
    },
};

#[path = "support/movement.rs"]
mod support;

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn order(armies: &[u32], path: &[u32]) -> Command {
    Command::Move(MoveOrder {
        armies: armies.iter().copied().map(ArmyId).collect(),
        path: path.iter().copied().map(SiteId).collect(),
    })
}

fn rejected(campaign: &mut StrategicCampaign, data: &GameData, command: Command) {
    let before = campaign.clone();
    assert!(apply(campaign, data, Actor::Player, command).is_err());
    assert_eq!(*campaign, before);
}

#[test]
fn cheapest_physical_routes_use_real_gates_and_stable_site_id_ties() {
    let (mut data, mut campaign) = fixture();
    let untouched = campaign.clone();
    let route =
        movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(7)).unwrap();
    assert_eq!(route.order.path, [1, 5, 6, 7].map(SiteId));
    assert_eq!(
        route
            .steps
            .iter()
            .map(|step| step.route)
            .collect::<Vec<_>>(),
        [13, 1, 2].map(RouteId)
    );
    assert_eq!(
        (route.total_cost, route.remaining, route.reachable_steps),
        (6, 6, 3)
    );
    assert!(route.stop.is_none());
    assert!(route.supplied_after && route.uncertain_contact);
    assert_eq!(campaign, untouched);
    let result = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(route.order),
    )
    .unwrap();
    assert_eq!(result.movement.unwrap().path, [1, 5, 6, 7].map(SiteId));
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(7));
    assert_eq!(campaign.people[&PersonId(1)].movement_spent, 6);
    assert!(campaign.armies[&ArmyId(1)]
        .formation_ids()
        .all(|id| campaign.formations[&id].movement_spent == 6));
    assert_eq!(
        campaign.world.site(SiteId(7)).unwrap().controller,
        Some(campaign.player)
    );
    assert_eq!(
        campaign.factions[&campaign.player].resources,
        untouched.factions[&campaign.player].resources
    );
    assert_eq!(campaign.rng, untouched.rng);

    // Two equal-cost complete routes: the lower site-ID branch wins.
    for id in [RouteId(2), RouteId(4)] {
        data.scenario
            .routes
            .iter_mut()
            .find(|route| route.id == id)
            .unwrap()
            .terrain_cost = 4;
    }
    let mut tied = StrategicCampaign::new(&data).unwrap();
    tied.set_site_control(&data, SiteId(9), Some(tied.player), false)
        .unwrap();
    let route = movement_preview(&tied, &data, tied.player, &[ArmyId(1)], SiteId(14)).unwrap();
    assert_eq!(route.order.path, [1, 5, 6, 7, 14].map(SiteId));
    assert_eq!(route.total_cost, 12);
    assert!(movement_preview(&tied, &data, tied.player, &[ArmyId(1)], SiteId(999)).is_err());
}

#[test]
fn later_budget_or_blocked_edges_keep_the_legal_prefix_without_revealing_hidden_strength() {
    let (data, initial) = fixture();
    let mut campaign = initial.clone();
    let moved = apply(
        &mut campaign,
        &data,
        Actor::Player,
        order(&[1], &[1, 5, 6, 7, 14]),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(moved.path, [1, 5, 6, 7].map(SiteId));
    assert_eq!(moved.spent, 6);
    assert!(matches!(
        moved.stop.unwrap().reason,
        MovementBlock::InsufficientMovement {
            required: 3,
            remaining: 0
        }
    ));
    rejected(&mut campaign, &data, order(&[1], &[7, 14]));

    let mut changed = initial.clone();
    let route = movement_preview(&changed, &data, changed.player, &[ArmyId(1)], SiteId(7)).unwrap();
    changed
        .set_site_control(&data, SiteId(7), Some(FactionId(2)), false)
        .unwrap();
    let result = apply(
        &mut changed,
        &data,
        Actor::Player,
        Command::Move(route.order),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(result.path, [1, 5, 6].map(SiteId));
    assert_eq!(result.spent, 4);
    assert_eq!(result.stop.unwrap().reason, MovementBlock::PeaceBoundary);
    assert_eq!(
        changed.world.site(SiteId(7)).unwrap().controller,
        Some(FactionId(2))
    );

    let mut hostile = initial.clone();
    let result = apply(
        &mut hostile,
        &data,
        Actor::Player,
        order(&[1], &[1, 5, 6, 8, 10]),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(result.path, [1, 5, 6, 8].map(SiteId));
    assert!(matches!(
        result.stop.unwrap().reason,
        MovementBlock::InsufficientMovement {
            required: 2,
            remaining: 0
        }
    ));
    assert_eq!(
        hostile.world.site(SiteId(10)).unwrap().controller,
        Some(FactionId(3))
    );
    let mut neutral_fort = initial.clone();
    neutral_fort.world.sites[4].military = kestrum::data::world::MilitaryLayer::Fort;
    rejected(&mut neutral_fort, &data, order(&[1], &[1, 5]));

    let mut hidden = initial.clone();
    hidden.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(5);
    let expected =
        movement_preview(&initial, &data, initial.player, &[ArmyId(1)], SiteId(6)).unwrap();
    assert_eq!(
        movement_preview(&hidden, &data, hidden.player, &[ArmyId(1)], SiteId(6)).unwrap(),
        expected
    );
    assert_eq!(
        preview(&hidden, &data, Actor::Player, order(&[1], &[1, 5, 6])),
        preview(&initial, &data, Actor::Player, order(&[1], &[1, 5, 6]))
    );
    let contact = apply(&mut hidden, &data, Actor::Player, order(&[1], &[1, 5, 6])).unwrap();
    assert!(contact.battle.is_some());
    assert_eq!(contact.movement.unwrap().path, [SiteId(1), SiteId(5)]);
    assert_eq!(hidden.battles.len(), 1);
    let mut interrupted = initial.clone();
    let moved = apply(
        &mut interrupted,
        &data,
        Actor::Player,
        order(&[1], &[1, 5, 10]),
    )
    .unwrap()
    .movement
    .unwrap();
    assert_eq!(moved.path, [1, 5].map(SiteId));
    assert_eq!(moved.stop.unwrap().reason, MovementBlock::RouteUnavailable);
    rejected(&mut initial.clone(), &data, order(&[1, 1], &[1, 5]));
}

#[test]
fn roads_charge_both_directions_and_factions_equally_and_groups_pay_per_member() {
    let (data, initial) = fixture();
    for (owner, damage, cost) in [(1, 49, 1), (2, 49, 1), (1, 50, 2), (2, 100, 2)] {
        let mut campaign = initial.clone();
        let road = &mut campaign
            .world
            .routes
            .iter_mut()
            .find(|route| route.id == RouteId(13))
            .unwrap()
            .road;
        road.improved = true;
        road.damage = damage;
        let (actor, from, to) = if owner == 1 {
            (Actor::Player, 1, 5)
        } else {
            campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
            campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(5);
            campaign
                .set_site_control(&data, SiteId(1), None, false)
                .unwrap();
            apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
            (Actor::Npc(FactionId(2)), 5, 1)
        };
        let moved = apply(&mut campaign, &data, actor, order(&[owner], &[from, to]))
            .unwrap()
            .movement
            .unwrap();
        assert_eq!(moved.spent, cost);
        assert_eq!(campaign.people[&PersonId(owner)].movement_spent, cost);
        let back = apply(&mut campaign, &data, actor, order(&[owner], &[to, from]))
            .unwrap()
            .movement
            .unwrap();
        assert_eq!(back.spent, cost);
    }
    let mut group = initial;
    let split = apply(
        &mut group,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(2),
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    let slower = group.formations.get_mut(&FormationId(2)).unwrap();
    slower.kind = TroopKind::SiegeEngines;
    slower.capacity = data.economy.formations[&TroopKind::SiegeEngines].capacity;
    slower.headcount = slower.capacity;
    let preview =
        movement_preview(&group, &data, group.player, &[split, ArmyId(1)], SiteId(7)).unwrap();
    assert_eq!(preview.order.armies, [ArmyId(1), split]);
    assert_eq!((preview.remaining, preview.reachable_site), (4, SiteId(6)));
    apply(
        &mut group,
        &data,
        Actor::Player,
        Command::Move(preview.order),
    )
    .unwrap();
    assert_eq!(group.armies[&split].site, SiteId(6));
    assert_eq!(group.armies[&ArmyId(1)].site, SiteId(6));
    for formation in group
        .formations
        .values()
        .filter(|formation| formation.faction == group.player)
    {
        assert_eq!(formation.movement_spent, 4);
    }
    assert_eq!(group.people[&PersonId(1)].movement_spent, 4);
}

#[test]
fn transfers_require_own_members_empty_slots_and_actual_site_colocation() {
    let (data, mut campaign) = fixture();
    let split = apply(
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
    assert_existing_commander_retained(&campaign, &data, split);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferFormation {
            formation: FormationId(1),
            to_army: split,
            to_slot: 1,
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&split].commander, Some(PersonId(1)));
    assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(3),
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(PersonId(1)));
    assert_eq!(campaign.armies[&split].commander, None);
    for command in [
        Command::TransferFormation {
            formation: FormationId(3),
            to_army: split,
            to_slot: 0,
        },
        Command::TransferFormation {
            formation: FormationId(3),
            to_army: split,
            to_slot: 6,
        },
        Command::TransferFormation {
            formation: FormationId(4),
            to_army: split,
            to_slot: 2,
        },
        Command::TransferFormation {
            formation: FormationId(3),
            to_army: ArmyId(2),
            to_slot: 3,
        },
        Command::TransferPerson {
            person: PersonId(2),
            to_formation: FormationId(3),
        },
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(4),
        },
    ] {
        rejected(&mut campaign, &data, command);
    }
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign.armies.get_mut(&split).unwrap().site = SiteId(6);
    rejected(
        &mut campaign,
        &data,
        Command::TransferFormation {
            formation: FormationId(3),
            to_army: split,
            to_slot: 2,
        },
    );
    rejected(
        &mut campaign,
        &data,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(2),
        },
    );
    campaign.armies.get_mut(&split).unwrap().site = SiteId(5);
    campaign.world.contested_sites.insert(SiteId(5));
    campaign.reconcile_region_control();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferFormation {
            formation: FormationId(3),
            to_army: split,
            to_slot: 2,
        },
    )
    .unwrap();
    assert!(!campaign.armies.contains_key(&ArmyId(1)));
    assert_eq!(campaign.armies[&split].site, SiteId(5));
    campaign.validate(&data).unwrap();
}

fn assert_existing_commander_retained(
    campaign: &StrategicCampaign,
    data: &GameData,
    target: ArmyId,
) {
    let mut appointed = campaign.clone();
    let mut person = appointed.people[&PersonId(2)].clone();
    person.id = appointed.next_ids.person;
    person.faction = appointed.player;
    person.assignment = PersonAssignment::Formation {
        formation: FormationId(2),
    };
    let commander = person.id;
    appointed.next_ids.person = PersonId(commander.0 + 1);
    appointed.people.insert(commander, person);
    appointed.armies.get_mut(&target).unwrap().commander = Some(commander);
    apply(
        &mut appointed,
        data,
        Actor::Player,
        Command::TransferFormation {
            formation: FormationId(1),
            to_army: target,
            to_slot: 1,
        },
    )
    .unwrap();
    assert_eq!(appointed.armies[&target].commander, Some(commander));
    assert_eq!(appointed.armies[&ArmyId(1)].commander, None);
    assert_eq!(
        appointed.people[&PersonId(1)].assignment,
        PersonAssignment::Formation {
            formation: FormationId(1)
        }
    );
}

#[test]
fn split_recruit_and_paused_person_transfers_cannot_refresh_member_allowances() {
    support::assert_application_command_gates();
    let (data, mut campaign) = fixture();
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .movement_spent = 2;
    campaign
        .formations
        .get_mut(&FormationId(2))
        .unwrap()
        .movement_spent = 4;
    campaign
        .formations
        .get_mut(&FormationId(3))
        .unwrap()
        .movement_spent = 1;
    campaign
        .people
        .get_mut(&PersonId(1))
        .unwrap()
        .movement_spent = 5;
    let split = apply(
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
    assert_eq!(army_remaining(&campaign, &data, split).unwrap(), 2);
    assert_eq!(army_remaining(&campaign, &data, ArmyId(1)).unwrap(), 1);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(2),
        },
    )
    .unwrap();
    assert_eq!(army_remaining(&campaign, &data, split).unwrap(), 1);
    assert_eq!(army_remaining(&campaign, &data, ArmyId(1)).unwrap(), 4);
    let recruited = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Recruit {
            site: SiteId(1),
            army: Some(ArmyId(1)),
            kind: TroopKind::Warriors,
        },
    )
    .unwrap()
    .recruited
    .unwrap();
    assert_eq!(army_remaining(&campaign, &data, ArmyId(1)).unwrap(), 0);
    let exhausted = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: recruited.formation,
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    assert_eq!(army_remaining(&campaign, &data, exhausted).unwrap(), 0);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let transfer = Command::TransferFormation {
        formation: recruited.formation,
        to_army: split,
        to_slot: 1,
    };
    rejected(&mut campaign, &data, transfer.clone());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetNpcPaused(true),
    )
    .unwrap();
    apply(&mut campaign, &data, Actor::Player, transfer).unwrap();
    assert!(!campaign.armies.contains_key(&exhausted));
    assert_eq!(
        formation_remaining(&campaign, &data, recruited.formation).unwrap(),
        0
    );
    assert_eq!(person_remaining(&campaign, &data, PersonId(1)).unwrap(), 1);
    rejected(&mut campaign, &data, order(&[split.0], &[1, 5]));
    rejected(
        &mut campaign,
        &data,
        Command::Recruit {
            site: SiteId(1),
            army: None,
            kind: TroopKind::Warriors,
        },
    );

    campaign.armies.get_mut(&split).unwrap().commander = None;
    campaign.people.get_mut(&PersonId(1)).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(1) };
    let raw = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let Campaign::Strategic(mut restored) = serde_json::from_str::<Campaign>(&raw).unwrap() else {
        panic!("strategic")
    };
    let command = Command::TransferPerson {
        person: PersonId(1),
        to_formation: FormationId(3),
    };
    let outcome = apply(&mut campaign, &data, Actor::Player, command.clone()).unwrap();
    assert_eq!(
        apply(&mut restored, &data, Actor::Player, command).unwrap(),
        outcome
    );
    assert_eq!(*restored, campaign);
    assert_eq!(campaign.people[&PersonId(1)].movement_spent, 5);
    campaign.validate(&data).unwrap();
}
