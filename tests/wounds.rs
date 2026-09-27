//! K07 person consequences and the two supplied recovery steps are real state.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/relations.rs"]
mod relations;
use relations::sync_relations;

use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{
        apply, person_site, resolve_person_combat, Actor, Command, PersonCombatContext,
        PersonCombatSide,
    },
    state::{
        military::{ArmyId, FormationId},
        people::{
            PersonAssignment, PersonCombatOutcome, PersonDeathReason, PersonId, PersonStatus,
            WoundCause,
        },
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;

#[path = "support/wounds.rs"]
mod support;
use support::*;

#[test]
fn wipes_draw_once_per_person_in_global_id_order_and_preserve_surviving_identity() {
    let (data, mut campaign) = fixture();
    add_person(&mut campaign, 5, 1, 1, 30);
    campaign.people.get_mut(&PersonId(5)).unwrap().status = wounded(1);
    let context = context(&campaign);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 0;
    campaign
        .formations
        .get_mut(&FormationId(4))
        .unwrap()
        .headcount = 0;
    campaign.rng.combat = SeededRng::new(2); // 96, 13, 71: survive, die, survive.
    let before = campaign.clone();
    let events = resolve_person_combat(&mut campaign, &data, &context).unwrap();
    assert_eq!(
        events.iter().map(|event| event.person).collect::<Vec<_>>(),
        [PersonId(1), PersonId(2), PersonId(5)]
    );
    for id in [PersonId(1), PersonId(5)] {
        assert_eq!(
            campaign.people[&id].assignment,
            PersonAssignment::Formation {
                formation: FormationId(2)
            }
        );
        assert_eq!(campaign.people[&id].status, wounded(2));
        assert_eq!(campaign.people[&id].name, before.people[&id].name);
        assert_eq!(
            campaign.people[&id].birth_round,
            before.people[&id].birth_round
        );
        assert_eq!(
            campaign.people[&id].movement_spent,
            data.rules.leadership.officer_movement_allowance
        );
    }
    assert_eq!(
        campaign.people[&PersonId(2)].assignment,
        PersonAssignment::Dead
    );
    assert!(matches!(
        events[1].outcome,
        PersonCombatOutcome::Died {
            reason: PersonDeathReason::FormationDestroyed
        }
    ));
    assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
    assert_eq!(campaign.armies[&ArmyId(2)].commander, None);
    assert_eq!(campaign.next_ids, before.next_ids);
    assert_only_combat_draws(&before, &campaign, 3);
    let mut reordered = before;
    let mut reversed = context.clone();
    reversed.sides.swap(0, 1);
    assert_eq!(
        resolve_person_combat(&mut reordered, &data, &reversed).unwrap(),
        events
    );
    assert_eq!(reordered, campaign);
    clean_zeros(&mut campaign, &data);
    assert!(!campaign.formations.contains_key(&FormationId(1)));
    assert!(!campaign.formations.contains_key(&FormationId(4)));
    assert_eq!(person_site(&campaign, PersonId(2)), None);
    assert_wipe_probability_boundary();
}

#[test]
fn wounded_wipe_survivors_use_only_encounter_survivors_or_legal_inhabited_refuges() {
    for (mode, destination) in [
        ("lowest", Some(SiteId(1))),
        ("contested", Some(SiteId(6))),
        ("none", None),
    ] {
        let (data, mut campaign) = fixture();
        split_formation(&mut campaign, FormationId(3), ArmyId(5), None);
        campaign.armies.get_mut(&ArmyId(5)).unwrap().site = SiteId(5);
        campaign
            .set_site_control(&data, SiteId(6), Some(FactionId(1)), false)
            .unwrap();
        if mode == "contested" {
            campaign
                .set_site_control(&data, SiteId(1), Some(FactionId(1)), true)
                .unwrap();
        } else if mode == "none" {
            campaign
                .set_site_control(&data, SiteId(1), Some(FactionId(2)), false)
                .unwrap();
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == SiteId(6))
                .unwrap()
                .habitation = Habitation::Unsettled;
        }
        let mut context = context(&campaign);
        context.sides[0].refuges = vec![SiteId(11), SiteId(6), SiteId(2), SiteId(1)];
        for id in [FormationId(1), FormationId(2)] {
            campaign.formations.get_mut(&id).unwrap().headcount = 0;
        }
        campaign.rng.combat = SeededRng::new(22); // 25 survives the death roll.
        let before = campaign.clone();
        let events = resolve_person_combat(&mut campaign, &data, &context).unwrap();
        assert_only_combat_draws(&before, &campaign, 1);
        if let Some(site) = destination {
            assert_eq!(
                campaign.people[&PersonId(1)].assignment,
                PersonAssignment::Site { site }
            );
            assert_eq!(campaign.people[&PersonId(1)].status, wounded(2));
            assert!(matches!(
                events[0].outcome,
                PersonCombatOutcome::Wounded {
                    cause: WoundCause::FormationDestroyed,
                    ..
                }
            ));
        } else {
            assert!(matches!(
                events[0].outcome,
                PersonCombatOutcome::Died {
                    reason: PersonDeathReason::NoRefuge
                }
            ));
            assert!(!campaign.people[&PersonId(1)].is_alive());
        }
        clean_zeros(&mut campaign, &data);
        assert!(!campaign.armies.contains_key(&ArmyId(1)));
        assert!(
            campaign.armies.contains_key(&ArmyId(5)),
            "nonparticipant army is never an escape carrier"
        );
        assert_eq!(campaign.formations[&FormationId(3)].headcount, 80);
    }
}

#[test]
fn commander_wound_uses_one_qualifying_roll_and_lowest_fit_adult_successor() {
    for first_remaining in [81, 80] {
        let (data, mut campaign) = fixture();
        for (id, age) in [(5, 30), (6, 16), (7, 30), (8, 17), (9, 40)] {
            add_person(&mut campaign, id, 1, 2, age);
        }
        campaign.people.get_mut(&PersonId(7)).unwrap().status = wounded(1);
        split_formation(&mut campaign, FormationId(2), ArmyId(5), Some(PersonId(5)));
        let mut context = context(&campaign);
        context.sides[0] = side(&campaign, FactionId(1), vec![ArmyId(5), ArmyId(1)]);
        context.sides[0].commanders = vec![PersonId(5), PersonId(1)];
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = first_remaining;
        campaign
            .formations
            .get_mut(&FormationId(2))
            .unwrap()
            .headcount = 80;
        campaign.rng.combat = SeededRng::new(44); // 9 wounds at the 10% boundary.
        let before = campaign.clone();
        let events = resolve_person_combat(&mut campaign, &data, &context).unwrap();
        let expected = if first_remaining == 81 {
            PersonId(5)
        } else {
            PersonId(1)
        };
        assert_eq!(events[0].person, expected);
        assert!(matches!(
            events[0].outcome,
            PersonCombatOutcome::Wounded {
                cause: WoundCause::CommandCasualty,
                ..
            }
        ));
        assert_eq!(campaign.people[&expected].status, wounded(2));
        assert_only_combat_draws(&before, &campaign, 1);
        assert_eq!(
            campaign.formations, before.formations,
            "person consequences cannot recalculate attacks"
        );
        if expected == PersonId(5) {
            assert_eq!(campaign.armies[&ArmyId(5)].commander, Some(PersonId(8)));
            assert_eq!(events[1].person, PersonId(8));
            assert_eq!(
                events[1].outcome,
                PersonCombatOutcome::AssumedCommand {
                    army: ArmyId(5),
                    previous: PersonId(5)
                }
            );
            assert!(!campaign.people[&PersonId(6)].is_fit_for_field(0, 17));
            assert!(!campaign.people[&PersonId(7)].is_fit_for_field(0, 17));
        } else {
            assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
            assert_eq!(campaign.people[&PersonId(5)].status, PersonStatus::Fit);
        }
        campaign.validate(&data).unwrap();
    }
    assert_commander_thresholds_and_no_per_hit_rolls();
    assert_real_commander_battle();
}

#[test]
fn wounds_need_two_supplied_steps_without_currency_headcount_or_rng_effects() {
    for at_site in [false, true] {
        let (mut data, mut campaign) = fixture();
        wound_first_commander(&data, &mut campaign);
        if at_site {
            campaign.people.get_mut(&PersonId(1)).unwrap().assignment =
                PersonAssignment::Site { site: SiteId(1) };
        }
        for income in data.economy.settlement_income.values_mut() {
            *income = Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            };
        }
        data.economy.headquarters_income_bonus = Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        };
        campaign
            .factions
            .get_mut(&FactionId(1))
            .unwrap()
            .resources
            .gold = 0;
        assert_eq!(
            campaign.army_leadership_permille(ArmyId(1), &data),
            Some(500)
        );
        finish_without_healing_effects(&mut campaign, &data);
        assert!(
            campaign.factions[&FactionId(1)].deficit,
            "wound care is free despite upkeep deficit"
        );
        assert_eq!(campaign.people[&PersonId(1)].status, wounded(1));
        campaign
            .set_site_control(&data, SiteId(1), Some(FactionId(1)), true)
            .unwrap();
        finish_without_healing_effects(&mut campaign, &data);
        assert_eq!(
            campaign.people[&PersonId(1)].status,
            wounded(1),
            "cut-off seasons do not advance healing"
        );
        campaign
            .set_site_control(&data, SiteId(1), Some(FactionId(1)), false)
            .unwrap();
        finish_without_healing_effects(&mut campaign, &data);
        assert_eq!(campaign.people[&PersonId(1)].status, PersonStatus::Fit);
        assert_eq!(
            campaign.armies[&ArmyId(1)].commander,
            None,
            "healing does not displace an appointment"
        );
        assert_eq!(
            campaign.army_leadership_permille(ArmyId(1), &data),
            Some(if at_site { 500 } else { 833 })
        );
    }
    assert_dead_people_never_heal_or_transfer();
}

#[test]
fn person_status_migrates_and_saved_consequences_do_not_repeat_or_revive() {
    let (data, mut campaign) = fixture();
    assert_fit_migration(&data, &campaign);
    let context = context(&campaign);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 0;
    campaign
        .formations
        .get_mut(&FormationId(4))
        .unwrap()
        .headcount = 0;
    campaign.rng.combat = SeededRng::new(2);
    let mut resumed: StrategicCampaign =
        serde_json::from_str(&serde_json::to_string(&campaign).unwrap()).unwrap();
    let expected = resolve_person_combat(&mut campaign, &data, &context).unwrap();
    assert_eq!(
        resolve_person_combat(&mut resumed, &data, &context).unwrap(),
        expected
    );
    assert_eq!(resumed, campaign);
    clean_zeros(&mut campaign, &data);
    let saved = Campaign::Strategic(Box::new(campaign.clone()));
    for _ in 0..3 {
        let restored: Campaign =
            serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
        restored.validate(&data).unwrap();
        assert_eq!(restored.strategic(), Some(&campaign));
        let copied: Vec<kestrum::state::people::PersonCombatEvent> =
            serde_json::from_str(&serde_json::to_string(&expected).unwrap()).unwrap();
        assert_eq!(copied, expected);
    }
    assert_status_validation(&data, &campaign);
    assert_rejected_snapshot_is_atomic(&data);
}
