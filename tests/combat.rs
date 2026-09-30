//! Campaign encounters preserve their pending transaction and immutable playback.

use kestrum::{
    data::{
        economy::TroopKind,
        world::{FactionId, Geography, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{
        advance_npc, apply, battle_reports, movement_preview, project, Actor, Command, MoveOrder,
    },
    state::{
        battle::{
            simulation::{BattleResolutionReason, BattleUnitId},
            BattleEndReason, BattleId, BattleOutcome, BattleReport,
        },
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/combat.rs"]
mod support;
use support::*;

#[test]
fn committed_formation_receipts_drive_campaign_losses_and_keep_the_round_clock() {
    for named in [false, true] {
        let (data, mut campaign) = fixture(100, 100);
        if named {
            for (id, faction, formation) in [(1, 1, 1), (5, 1, 1), (3, 3, 7), (6, 3, 7)] {
                person(&mut campaign, id, faction, formation);
            }
        }
        let rounds = campaign.completed_rounds;
        let rng = campaign.rng.clone();
        let report = fight(&mut campaign, &data);
        let simulation = report.simulation.as_ref().unwrap();
        assert_eq!(simulation.reason, BattleResolutionReason::RoundLimit);
        assert_eq!(report.reason, BattleEndReason::ExchangeLimit);
        assert!(report.exchanges.len() <= 8);
        for id in [FormationId(1), FormationId(7)] {
            let final_count = simulation
                .units
                .iter()
                .find(|unit| unit.id == BattleUnitId::Formation(id))
                .unwrap()
                .headcount;
            let formation = report
                .faction_sides()
                .flat_map(|side| &side.armies)
                .flat_map(|army| &army.formations)
                .find(|formation| formation.id == id)
                .unwrap();
            assert_eq!(formation.combat_losses, 100 - final_count);
            assert_eq!(loss(&report, 0, id.0), formation.combat_losses);
        }
        assert_eq!(campaign.completed_rounds, rounds);
        assert_eq!(campaign.rng, rng); // no commanders or wiped people, hence no random damage
        assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(8));
        assert_eq!(campaign.armies[&ArmyId(3)].site, SiteId(10));
        assert!(campaign
            .formations
            .values()
            .all(|formation| formation.movement_spent <= 6));
        assert!(campaign
            .people
            .values()
            .all(|person| person.movement_spent == 6));
        assert!(report.structural_damage_added <= data.combat.field_damage);
    }
    let (data, mut campaign) = fixture(1, 1);
    let report = fight(&mut campaign, &data);
    assert_eq!(report.outcome, report.simulation.as_ref().unwrap().outcome);
    for id in [FormationId(1), FormationId(7)] {
        let resolved = report
            .simulation
            .as_ref()
            .unwrap()
            .units
            .iter()
            .find(|unit| unit.id == BattleUnitId::Formation(id))
            .unwrap();
        let formation = report
            .faction_sides()
            .flat_map(|side| &side.armies)
            .flat_map(|army| &army.formations)
            .find(|entry| entry.id == id)
            .unwrap();
        assert_eq!(formation.end, resolved.headcount);
    }
    assert!(campaign.battles.contains_key(&report.id));
}

#[test]
fn counters_terrain_and_living_people_change_exact_strength_without_hidden_preview_reads() {
    let (mut data, _) = fixture(100, 100);
    data.scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(10))
        .unwrap()
        .geography = Geography::Hill;
    let (_, mut campaign) = fixture(100, 100);
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(10))
        .unwrap()
        .geography = Geography::Hill;
    kind(&mut campaign, &data, 1, TroopKind::Spearmen, 58);
    kind(&mut campaign, &data, 7, TroopKind::Riders, 40);
    let report = fight(&mut campaign, &data);
    assert_eq!(report.terrain_permille, 1100);
    assert!(report.simulation.is_some());
    assert!(report.counters.is_empty());
    assert_eq!(
        data.combat.counter(TroopKind::Riders, TroopKind::Archers),
        1250
    );
    assert_eq!(
        data.combat.counter(TroopKind::Archers, TroopKind::Spearmen),
        1250
    );
    assert_eq!(
        data.combat.terrain(data.scenario.site(SiteId(8)).unwrap()),
        1200
    );
    assert_eq!(
        data.combat.terrain(data.scenario.site(SiteId(6)).unwrap()),
        1000
    );

    let (data, mut campaign) = fixture(100, 100);
    person(&mut campaign, 1, 1, 1);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    let report = fight(&mut campaign, &data);
    assert_eq!(report.attacker.armies[0].leadership_permille, 933);
    assert!(loss(&report, 0, 7) > 0);
    assert_eq!(report.attacker.armies[0].people[0].name, "Witness 1");
    let (_, initial) = fixture(100, 100);
    let preview =
        movement_preview(&initial, &data, FactionId(1), &[ArmyId(1)], SiteId(10)).unwrap();
    let mut hidden = initial.clone();
    hidden
        .formations
        .get_mut(&FormationId(7))
        .unwrap()
        .headcount = 1;
    person(&mut hidden, 3, 3, 7);
    assert_eq!(
        movement_preview(&hidden, &data, FactionId(1), &[ArmyId(1)], SiteId(10)).unwrap(),
        preview
    );
    for malformed in ["resistance", "attack", "exchanges", "terrain"] {
        let mut rules = data.clone();
        match malformed {
            "resistance" => {
                rules
                    .troops
                    .formations
                    .get_mut(&TroopKind::Warriors)
                    .unwrap()
                    .resistance = 0
            }
            "attack" => {
                rules
                    .troops
                    .formations
                    .get_mut(&TroopKind::Warriors)
                    .unwrap()
                    .attack = 0
            }
            "exchanges" => rules.combat.max_exchanges = 9,
            _ => rules.combat.forest_hill_permille = 0,
        }
        assert!(rules.validate().is_err());
        let mut attempted = initial.clone();
        assert!(apply(&mut attempted, &rules, Actor::Player, command()).is_err());
        assert_eq!(attempted, initial);
    }
    assert_living_leadership();
}

#[test]
fn selected_armies_and_all_defenders_keep_stable_slot_targeting_and_casualty_identity() {
    let (data, mut campaign) = fixture(100, 100);
    add_army(&mut campaign, 5, 3, 10, 13, 100);
    add_army(&mut campaign, 6, 1, 8, 14, 100);
    add_army(&mut campaign, 7, 1, 8, 15, 100); // deliberately not selected
    let outside = campaign.armies[&ArmyId(7)].clone();
    let outside_formation = campaign.formations[&FormationId(15)].clone();
    // Put a larger formation ID in an earlier slot: roster order is army then slot.
    let mut extra = campaign.formations[&FormationId(1)].clone();
    extra.id = FormationId(16);
    campaign.formations.insert(extra.id, extra);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots[0] = Some(FormationId(16));
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots[1] = Some(FormationId(1));
    let mut extra = campaign.formations[&FormationId(7)].clone();
    extra.id = FormationId(17);
    campaign.formations.insert(extra.id, extra);
    campaign.armies.get_mut(&ArmyId(5)).unwrap().slots[1] = Some(FormationId(17));
    campaign.next_ids.formation = FormationId(18);
    let result = apply(
        &mut campaign,
        &data,
        Actor::Player,
        order(&[6, 1], &[8, 10, 11]),
    )
    .unwrap();
    let movement = result.movement.clone().unwrap();
    let result = if result.battle_pending {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::StartPendingBattle,
        )
        .unwrap()
    } else {
        result
    };
    let report = &campaign.battles[&result.battle.unwrap()];
    assert_eq!(
        report
            .attacker
            .armies
            .iter()
            .map(|army| army.id)
            .collect::<Vec<_>>(),
        [ArmyId(1), ArmyId(6)]
    );
    assert_eq!(
        report
            .defender
            .armies()
            .iter()
            .map(|army| army.id)
            .collect::<Vec<_>>(),
        [ArmyId(3), ArmyId(5)]
    );
    assert_eq!(
        report.attacker.armies[0]
            .formations
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        [FormationId(16), FormationId(1)]
    );
    for id in [1, 7, 13, 14, 16, 17] {
        assert!(report
            .simulation
            .as_ref()
            .unwrap()
            .units
            .iter()
            .any(|unit| unit.id == BattleUnitId::Formation(FormationId(id))));
    }
    assert_eq!(movement.path, [SiteId(8), SiteId(10)]);
    assert_eq!(campaign.armies[&ArmyId(7)], outside);
    assert_eq!(campaign.formations[&FormationId(15)], outside_formation);
    assert_eq!(campaign.next_ids.formation, FormationId(18));
}

#[test]
fn rout_retreat_encirclement_and_capture_apply_physical_control_and_damage_once() {
    let (data, mut campaign) = fixture(100, 20);
    let report = fight(&mut campaign, &data);
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert_eq!(report.simulation.as_ref().unwrap().outcome, report.outcome);
    assert!(matches!(
        report.reason,
        BattleEndReason::Rout | BattleEndReason::Annihilation
    ));
    assert!(report.exchanges.len() <= data.battle_tactics.max_rounds as usize);
    if campaign.formations.contains_key(&FormationId(7)) {
        assert_eq!(campaign.armies[&ArmyId(3)].site, SiteId(9));
    }
    let defender = &report.defender.armies()[0].formations[0];
    let resolved = report
        .simulation
        .as_ref()
        .unwrap()
        .units
        .iter()
        .find(|unit| unit.id == BattleUnitId::Formation(FormationId(7)))
        .unwrap();
    assert_eq!(
        defender.end + defender.encirclement_losses,
        resolved.headcount
    );
    assert!(
        report.structural_damage_added <= data.combat.field_damage + data.combat.capture_damage
    );
    assert_eq!(report.occupation_after, 100);
    assert_eq!(
        campaign.world.site(SiteId(10)).unwrap().controller,
        Some(FactionId(1))
    );
    let (_, mut trapped) = fixture(100, 20);
    for site in [9, 11, 12] {
        trapped
            .set_site_control(&data, SiteId(site), Some(FactionId(2)), false)
            .unwrap();
    }
    let report = fight(&mut trapped, &data);
    let defender = &report.defender.armies()[0].formations[0];
    let simulated = report
        .simulation
        .as_ref()
        .unwrap()
        .units
        .iter()
        .find(|unit| unit.id == BattleUnitId::Formation(FormationId(7)))
        .unwrap();
    assert_eq!(defender.end, 0);
    assert_eq!(defender.encirclement_losses, simulated.headcount);
    assert!(report.defender.armies()[0].final_site.is_none());
    assert!(
        !trapped.armies.contains_key(&ArmyId(3))
            && !trapped.formations.contains_key(&FormationId(7))
    );
    let (_, mut capped) = fixture(100, 20);
    capped.world.site_damage.insert(SiteId(10), 98);
    assert!(fight(&mut capped, &data).structural_damage_added <= 2);
    assert_eq!(capped.world.structural_damage(SiteId(10)), 100);

    let (mut high, mut both) = fixture(100, 100);
    high.troops
        .formations
        .get_mut(&TroopKind::Warriors)
        .unwrap()
        .attack = 120;
    let report = fight(&mut both, &high);
    assert_eq!(report.outcome, report.simulation.as_ref().unwrap().outcome);
    assert!(both.armies.contains_key(&ArmyId(1)) || !both.formations.contains_key(&FormationId(1)));
    let (_, mut neutral) = fixture(100, 100);
    neutral
        .set_site_control(&data, SiteId(10), None, false)
        .unwrap();
    let report = fight(&mut neutral, &data);
    assert_eq!(report.outcome, report.simulation.as_ref().unwrap().outcome);
    assert_eq!(
        report.control_after,
        neutral.world.site(SiteId(10)).unwrap().controller
    );

    let (_, mut empty) = fixture(100, 100);
    empty.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(3);
    assert!(apply(&mut empty, &data, Actor::Player, command())
        .unwrap()
        .battle
        .is_none());
    assert_eq!(empty.world.structural_damage(SiteId(10)), 10);
    apply(&mut empty, &data, Actor::Player, order(&[1], &[10, 8])).unwrap();
    apply(&mut empty, &data, Actor::Player, command()).unwrap();
    assert_eq!(empty.world.structural_damage(SiteId(10)), 10);
    assert!(empty.battles.is_empty());
    assert_peaceful_contact();
    assert_retreat_priorities();
}

#[test]
fn save_reload_replay_and_observed_reports_never_repeat_effects_or_reveal_live_enemies() {
    let (data, mut campaign) = fixture(100, 100);
    person(&mut campaign, 1, 1, 1);
    person(&mut campaign, 3, 3, 7);
    let serialized =
        serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let restored: Campaign = serde_json::from_str(&serialized).unwrap();
    let mut restored = restored.strategic().unwrap().clone();
    let expected = fight(&mut campaign, &data);
    assert_eq!(fight(&mut restored, &data), expected);
    assert_eq!(restored, campaign);
    assert_eq!(campaign.pending_facts.len(), 1);
    assert!(
        matches!(campaign.pending_facts[0].kind,kestrum::state::campaign::DomainFactKind::BattleResolved {battle,..} if battle == expected.id)
    );
    let frozen = campaign.clone();
    for _ in 0..5 {
        assert_eq!(
            battle_reports(&campaign, FactionId(1)),
            vec![expected.clone()]
        );
        assert_eq!(
            battle_reports(&campaign, FactionId(3)),
            vec![expected.clone()]
        );
        assert!(battle_reports(&campaign, FactionId(2)).is_empty());
        assert!(project(&campaign, FactionId(2)).unwrap().battles.is_empty());
    }
    assert_eq!(campaign, frozen);
    campaign
        .formations
        .get_mut(&FormationId(7))
        .unwrap()
        .headcount = 1;
    campaign.people.get_mut(&PersonId(3)).unwrap().name = "Later name".into();
    assert_eq!(battle_reports(&campaign, FactionId(1)), vec![expected]);
    campaign.validate(&data).unwrap();
    assert_catalogue(&data, &campaign);
    assert_bad_reports(&data, &campaign);
    assert_earlier_save(&data);
    let (_, mut fortified) = fixture(100, 100);
    fortified
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(10))
        .unwrap()
        .military = MilitaryLayer::Fort;
    let outcome = apply(&mut fortified, &data, Actor::Player, command()).unwrap();
    assert!(outcome.battle.is_none());
    assert!(fortified.battles.is_empty());
    assert_eq!(fortified.sieges[&SiteId(10)].defending, vec![ArmyId(3)]);
    assert_eq!(fortified.sieges[&SiteId(10)].besieging, vec![ArmyId(1)]);
    for formation in fortified.formations.values() {
        assert_eq!(formation.headcount, 100);
    }
    fortified.validate(&data).unwrap();
}

#[test]
fn field_contact_waits_as_a_saved_transaction_then_commits_once() {
    let (data, mut campaign) = fixture(80, 80);
    let opening = campaign.clone();
    let contact = apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    assert!(contact.battle_pending);
    assert!(contact.battle.is_none());
    assert!(campaign.battles.is_empty());
    let movement = contact.movement.unwrap();
    assert_eq!(movement.path, [SiteId(8), SiteId(10)]);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(10));
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 80);
    assert_eq!(
        campaign.formations[&FormationId(1)].movement_spent,
        movement.spent
    );
    let opening_damage = campaign.world.structural_damage(SiteId(10));
    let receipt = campaign
        .pending_battle
        .as_ref()
        .unwrap()
        .report
        .simulation
        .clone()
        .unwrap();
    campaign.validate(&data).unwrap();

    let saved = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let reloaded: Campaign = serde_json::from_str(&saved).unwrap();
    let mut reloaded = reloaded.strategic().unwrap().clone();
    assert_eq!(reloaded, campaign);
    assert!(apply(&mut campaign, &data, Actor::Player, Command::EndTurn).is_err());
    assert_eq!(
        campaign.pending_battle.as_ref().unwrap().report.simulation,
        Some(receipt.clone())
    );

    let resolved = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    let reloaded_result = apply(
        &mut reloaded,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    assert_eq!(resolved.battle, reloaded_result.battle);
    let id = resolved.battle.unwrap();
    assert!(campaign.pending_battle.is_none());
    assert_eq!(campaign.battles.len(), 1);
    let report = &campaign.battles[&id];
    assert_eq!(report.simulation, Some(receipt));
    assert!(matches!(
        campaign.pending_facts.last().unwrap().kind,
        kestrum::state::campaign::DomainFactKind::BattleResolved { battle, movement: Some(_) }
            if battle == id
    ));
    assert_eq!(campaign, reloaded);
    campaign.validate(&data).unwrap();

    let frozen = campaign.clone();
    for _ in 0..4 {
        assert_eq!(
            battle_reports(&campaign, FactionId(1)),
            vec![report.clone()]
        );
        let resolution = report.simulation.as_ref().unwrap();
        let _ = kestrum::state::battle::playback::battle_presentation_at(resolution, usize::MAX);
    }
    assert_eq!(campaign, frozen);
    assert!(campaign.world.structural_damage(SiteId(10)) >= opening_damage);
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle
    )
    .is_err());
    assert_eq!(campaign, frozen);
    assert_eq!(opening.battles.len(), 0);
}

#[test]
fn an_npc_contact_waits_for_player_acceptance_without_advancing_its_phase() {
    let (data, mut campaign) = fixture(90, 90);
    campaign.acted.extend([FactionId(1), FactionId(2)]);
    campaign.accepted_sequence = 1;
    campaign.next_ids.fact = kestrum::state::campaign::FactId(3);
    campaign.pending_facts = [FactionId(1), FactionId(2)]
        .into_iter()
        .enumerate()
        .map(|(index, faction)| kestrum::state::campaign::DomainFact {
            id: kestrum::state::campaign::FactId(index as u64 + 1),
            sequence: 1,
            completed_rounds: campaign.completed_rounds,
            kind: kestrum::state::campaign::DomainFactKind::FactionPassed { faction },
        })
        .collect();
    campaign.phase = CampaignPhase::NpcTurn {
        faction: FactionId(3),
        paused: false,
    };
    campaign.validate(&data).unwrap();
    let contact = apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        order(&[3], &[10, 8]),
    )
    .unwrap();
    assert!(contact.battle_pending);
    assert_eq!(campaign.active_faction(), FactionId(3));
    assert!(matches!(
        campaign.phase,
        CampaignPhase::NpcTurn {
            faction: FactionId(3),
            paused: false
        }
    ));
    let waiting = campaign.clone();
    assert!(advance_npc(&mut campaign, &data).is_err());
    assert_eq!(campaign, waiting);
    let resolved = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    assert!(resolved.battle.is_some());
    assert_eq!(campaign.active_faction(), FactionId(3));
    assert!(matches!(
        campaign.phase,
        CampaignPhase::NpcTurn {
            faction: FactionId(3),
            paused: false
        }
    ));
    assert_eq!(campaign.acted, waiting.acted);
    campaign.validate(&data).unwrap();
}
