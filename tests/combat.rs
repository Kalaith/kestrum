//! Five ordinary encounter contracts through the same transactional Move command.

use kestrum::{
    data::{
        economy::TroopKind,
        world::{FactionId, Geography, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{apply, battle_reports, movement_preview, project, Actor, Command, MoveOrder},
    state::{
        battle::{BattleEndReason, BattleId, BattleOutcome, BattleReport},
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        Campaign, StrategicCampaign,
    },
};

#[path = "support/combat.rs"]
mod support;
use support::*;

#[test]
fn exact_simultaneous_losses_use_named_contribution_and_a_bounded_exchange_clock() {
    for (named, first_loss) in [(false, 5), (true, 10)] {
        let (data, mut campaign) = fixture(100, 100);
        if named {
            for (id, faction, formation) in [(1, 1, 1), (5, 1, 1), (3, 3, 7), (6, 3, 7)] {
                person(&mut campaign, id, faction, formation);
            }
        }
        let rounds = campaign.completed_rounds;
        let rng = campaign.rng.clone();
        let report = fight(&mut campaign, &data);
        assert_eq!(loss(&report, 0, 1), first_loss);
        assert_eq!(loss(&report, 0, 7), first_loss);
        assert_eq!(report.exchanges.len(), 8);
        assert_eq!(report.reason, BattleEndReason::ExchangeLimit);
        assert_eq!(report.outcome, BattleOutcome::Stalemate);
        assert_eq!(campaign.completed_rounds, rounds);
        assert_eq!(campaign.rng, rng); // no commanders or wiped people, hence no random damage
        assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(8));
        assert_eq!(campaign.armies[&ArmyId(3)].site, SiteId(10));
        assert!(campaign
            .formations
            .values()
            .all(|formation| formation.movement_spent == 6));
        assert!(campaign
            .people
            .values()
            .all(|person| person.movement_spent == 6));
        assert_eq!(report.structural_damage_added, 5);
    }
    let (data, mut campaign) = fixture(1, 1);
    let report = fight(&mut campaign, &data);
    assert_eq!(report.outcome, BattleOutcome::MutualDestruction);
    assert_eq!(report.exchanges.len(), 1);
    assert_eq!(loss(&report, 0, 1), 1);
    assert_eq!(loss(&report, 0, 7), 1); // the killed defender still attacked
    assert!(campaign.armies.is_empty() && campaign.formations.is_empty());
    assert_eq!(campaign.world.site(SiteId(10)).unwrap().controller, None);
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
    assert_eq!(loss(&report, 0, 7), 1); // 391.5 / 198; prematurely rounding resistance to19 would give2
    assert_eq!(report.counters[0].permille, 1500);
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
    assert_eq!(loss(&report, 0, 7), 9);
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
            .armies
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
        assert_eq!(loss(report, 0, id), 5);
    }
    assert_eq!(result.movement.unwrap().path, [SiteId(8), SiteId(10)]); // never continues after combat
    assert_eq!(campaign.armies[&ArmyId(7)], outside);
    assert_eq!(campaign.formations[&FormationId(15)], outside_formation);
    assert_eq!(campaign.next_ids.formation, FormationId(18));
}

#[test]
fn rout_retreat_encirclement_and_capture_apply_physical_control_and_damage_once() {
    let (data, mut campaign) = fixture(100, 20);
    let report = fight(&mut campaign, &data);
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert_eq!(report.reason, BattleEndReason::Rout);
    assert_eq!(report.exchanges.len(), 3);
    assert_eq!(campaign.armies[&ArmyId(3)].site, SiteId(9)); // lowest eligible ID, origin8 forbidden
    assert_eq!(campaign.formations[&FormationId(7)].headcount, 7);
    assert_eq!(
        report.defender.armies[0].formations[0].encirclement_losses,
        0
    );
    assert_eq!(report.structural_damage_added, 15);
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
    assert_eq!(
        report.defender.armies[0].formations[0].encirclement_losses,
        7
    );
    assert!(report.defender.armies[0].final_site.is_none());
    assert!(
        !trapped.armies.contains_key(&ArmyId(3))
            && !trapped.formations.contains_key(&FormationId(7))
    );
    let (_, mut capped) = fixture(100, 20);
    capped.world.site_damage.insert(SiteId(10), 98);
    assert_eq!(fight(&mut capped, &data).structural_damage_added, 2);
    assert_eq!(capped.world.structural_damage(SiteId(10)), 100);

    let (mut high, mut both) = fixture(100, 100);
    high.troops
        .formations
        .get_mut(&TroopKind::Warriors)
        .unwrap()
        .attack = 120;
    let report = fight(&mut both, &high);
    assert_eq!(report.exchanges.len(), 1);
    assert_eq!(report.reason, BattleEndReason::Rout);
    assert_eq!(report.outcome, BattleOutcome::DefenderVictory); // both at exactly40%, attacker withdraws
    assert_eq!(both.armies[&ArmyId(1)].site, SiteId(8));
    let (_, mut neutral) = fixture(100, 100);
    neutral
        .set_site_control(&data, SiteId(10), None, false)
        .unwrap();
    let report = fight(&mut neutral, &data);
    assert_eq!(report.outcome, BattleOutcome::Stalemate);
    assert_eq!(report.control_after, None);
    assert_eq!(neutral.world.site(SiteId(10)).unwrap().controller, None);

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
