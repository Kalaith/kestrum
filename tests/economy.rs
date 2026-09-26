//! K05's five contracts: grants, recruitment, rejection, seasonal accounts, removal.

use kestrum::{
    data::{
        economy::{Habitation, Resources, TroopKind},
        world::{Facility, FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{advance_npc, apply, preview, project, recruit_options, Actor, Command, RuleError},
    state::{
        campaign::DomainFactKind,
        military::{ArmyId, FormationId},
        people::PersonAssignment,
        persistence::load_legacy,
        Campaign, CampaignPhase, GameState, Overlay, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

#[path = "support/economy.rs"]
mod support;
use support::*;

#[test]
fn authored_grants_create_full_fresh_armies_and_compatible_saves_do_not_invent_them() {
    let (data, campaign) = fixture();
    assert_eq!(campaign.armies.len(), data.scenario.factions.len());
    assert_eq!(campaign.people.len(), data.scenario.factions.len());
    assert!(campaign.pending_facts.is_empty());
    for setup in &data.scenario.factions {
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == setup.id)
            .unwrap();
        assert_eq!(army.name, setup.army_name);
        assert_eq!(army.site, setup.headquarters);
        assert_eq!(army.slots.len(), 6);
        assert_eq!(
            army.first_empty_slot(),
            Some(setup.starting_formations.len())
        );
        let formations: Vec<_> = army
            .formation_ids()
            .map(|id| &campaign.formations[&id])
            .collect();
        assert_eq!(
            formations
                .iter()
                .map(|formation| formation.kind)
                .collect::<Vec<_>>(),
            setup.starting_formations
        );
        for formation in formations {
            assert_eq!(
                formation.headcount,
                data.economy.formations[&formation.kind].capacity
            );
            assert_eq!(formation.capacity, formation.headcount);
            assert_eq!(formation.movement_spent, 0);
            assert_eq!(formation.created_round, 0);
        }
        let founder = &campaign.people[&army.commander.unwrap()];
        assert_eq!(founder.name, setup.founder.name);
        assert_eq!(founder.age_years(0), setup.founder.age_years);
        assert_eq!(founder.class, setup.founder.class);
        assert_eq!(founder.service_start_round, 0);
        assert_eq!(founder.movement_spent, 0);
        assert_eq!(
            founder.assignment,
            PersonAssignment::Formation {
                formation: army.slots[0].unwrap()
            }
        );
        assert_eq!(campaign.army_leadership_permille(army.id, &data), Some(933));
        assert_eq!(
            campaign.factions[&setup.id].resources,
            setup.resources.resolve(&data.economy)
        );
    }
    let visible = project(&campaign, campaign.player).unwrap();
    assert_eq!(
        (
            visible.armies.len(),
            visible.formations.len(),
            visible.people.len()
        ),
        (1, 3, 1)
    );
    let mut hidden = campaign.clone();
    hidden
        .formations
        .get_mut(&FormationId(4))
        .unwrap()
        .headcount = 1;
    hidden
        .people
        .get_mut(&kestrum::state::people::PersonId(2))
        .unwrap()
        .name = "Unseen commander".into();
    hidden.factions.get_mut(&FactionId(2)).unwrap().deficit = true;
    assert_eq!(visible, project(&hidden, campaign.player).unwrap());
    assert_legacy_military_migration(&data, campaign);
}

#[test]
fn affordable_recruitment_creates_fresh_exhausted_formations_for_player_and_npc() {
    let (data, initial) = fixture();
    for kind in data.economy.formations.keys().copied() {
        let mut campaign = initial.clone();
        specialists(&mut campaign);
        for destination in [Some(ArmyId(1)), None] {
            let before = campaign.clone();
            let command = recruit(kind, destination);
            preview(&campaign, &data, Actor::Player, command).unwrap();
            assert_eq!(campaign, before);
            let options =
                recruit_options(&campaign, &data, campaign.player, SiteId(1), destination);
            assert!(options
                .iter()
                .find(|option| option.kind == kind)
                .unwrap()
                .blocked
                .is_none());
            let outcome = apply(&mut campaign, &data, Actor::Player, command).unwrap();
            let result = outcome.recruited.unwrap();
            let formation = &campaign.formations[&result.formation];
            let definition = &data.economy.formations[&kind];
            assert_eq!(formation.kind, kind);
            assert_eq!(formation.headcount, definition.capacity);
            assert_eq!(formation.movement_spent, definition.movement_allowance);
            assert_eq!(result.formation, before.next_ids.formation);
            assert_eq!(campaign.armies[&result.army].site, SiteId(1));
            assert_eq!(campaign.armies[&result.army].faction, campaign.player);
            assert_eq!(campaign.rng, before.rng);
            let was = before.factions[&campaign.player].resources;
            let cost = definition.recruit_cost;
            assert_eq!(
                campaign.factions[&campaign.player].resources,
                Resources {
                    gold: was.gold - cost.gold,
                    wood: was.wood - cost.wood,
                    stone: was.stone - cost.stone
                }
            );
            assert!(
                matches!(outcome.facts[0].kind, DomainFactKind::FormationRecruited { formation, .. } if formation == result.formation)
            );
            if destination.is_none() {
                assert_eq!(
                    campaign.army_leadership_permille(result.army, &data),
                    Some(500)
                );
            }
        }
    }
    let mut npc = initial.clone();
    apply(&mut npc, &data, Actor::Player, Command::EndTurn).unwrap();
    let command = Command::Recruit {
        site: SiteId(2),
        army: Some(ArmyId(2)),
        kind: TroopKind::Warriors,
    };
    let recruited = apply(&mut npc, &data, Actor::Npc(FactionId(2)), command)
        .unwrap()
        .recruited
        .unwrap();
    assert_eq!(npc.formations[&recruited.formation].movement_spent, 6);
    assert_eq!(npc.factions[&FactionId(2)].resources.gold, 440);
    assert_eq!(npc.rng, initial.rng);
    assert_army_panel_command_boundary(&data, initial);
}

#[test]
fn invalid_cost_site_facility_and_slot_orders_preserve_the_entire_campaign() {
    let (data, initial) = fixture();
    assert_unaffordable(&data, &initial);
    let command = recruit(TroopKind::Warriors, Some(ArmyId(1)));
    for site in [SiteId(2), SiteId(999)] {
        rejected(
            &mut initial.clone(),
            &data,
            Actor::Player,
            Command::Recruit {
                site,
                army: None,
                kind: TroopKind::Warriors,
            },
            if site.0 == 2 {
                "controls"
            } else {
                "unavailable"
            },
        );
    }
    let mut campaign = initial.clone();
    campaign
        .set_site_control(&data, SiteId(6), Some(campaign.player), false)
        .unwrap();
    rejected(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Recruit {
            site: SiteId(6),
            army: None,
            kind: TroopKind::Warriors,
        },
        "supply path",
    );
    let mut campaign = initial.clone();
    campaign.world.sites[0].habitation = Habitation::Camp;
    rejected(&mut campaign, &data, Actor::Player, command, "Outpost");
    let mut campaign = initial.clone();
    campaign
        .set_site_control(&data, SiteId(1), Some(campaign.player), true)
        .unwrap();
    rejected(&mut campaign, &data, Actor::Player, command, "contested");
    let mut campaign = initial.clone();
    campaign.factions.get_mut(&campaign.player).unwrap().deficit = true;
    rejected(&mut campaign, &data, Actor::Player, command, "shortfall");
    for (army, reason) in [(ArmyId(999), "unavailable"), (ArmyId(2), "own armies")] {
        rejected(
            &mut initial.clone(),
            &data,
            Actor::Player,
            recruit(TroopKind::Warriors, Some(army)),
            reason,
        );
    }
    let mut campaign = initial.clone();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    rejected(
        &mut campaign,
        &data,
        Actor::Player,
        command,
        "receiving army",
    );
    let mut campaign = initial.clone();
    for _ in 0..3 {
        apply(&mut campaign, &data, Actor::Player, command).unwrap();
    }
    rejected(
        &mut campaign,
        &data,
        Actor::Player,
        command,
        "six formation",
    );
    rejected(
        &mut initial.clone(),
        &data,
        Actor::Npc(FactionId(2)),
        command,
        "Wait",
    );
    assert_invalid_facilities(&data, &initial);
    let mut campaign = initial.clone();
    campaign.next_ids.formation = FormationId(u32::MAX);
    rejected(&mut campaign, &data, Actor::Player, command, "identifiers");
}

#[test]
fn seasonal_income_precedes_full_upkeep_and_shortfall_clears_only_at_a_paid_boundary() {
    let (data, mut campaign) = fixture();
    prepare_shortfall(&mut campaign, &data);
    let rng = campaign.rng.clone();
    finish_round(&mut campaign, &data);
    let owner = &campaign.factions[&campaign.player];
    let statement = owner.last_economy.as_ref().unwrap();
    assert_eq!(
        statement.income,
        Resources {
            gold: 50,
            wood: 19,
            stone: 12
        }
    );
    assert_eq!(
        (
            statement.upkeep_due,
            statement.upkeep_paid,
            statement.shortfall
        ),
        (60, 50, 10)
    );
    assert_eq!(owner.resources.gold, 0);
    assert!(owner.deficit);
    assert_deficit_statement_contract(&data, &campaign);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 1);
    assert!(campaign
        .formations
        .values()
        .all(|formation| formation.movement_spent == 0));
    assert!(campaign
        .people
        .values()
        .all(|person| person.movement_spent == 0));
    assert_eq!(campaign.rng, rng);
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 500;
    rejected(
        &mut campaign,
        &data,
        Actor::Player,
        recruit(TroopKind::Warriors, None),
        "shortfall",
    );
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 0;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(13),
        },
    )
    .unwrap();
    assert!(campaign.factions[&campaign.player].deficit);
    finish_round(&mut campaign, &data);
    let owner = &campaign.factions[&campaign.player];
    let statement = owner.last_economy.as_ref().unwrap();
    assert_eq!(
        (
            statement.upkeep_due,
            statement.upkeep_paid,
            statement.shortfall
        ),
        (50, 50, 0)
    );
    assert!(!owner.deficit);
    assert_eq!(owner.resources.gold, 0);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 1);
    let encoded = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let restored: Campaign = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.strategic(), Some(&campaign));
    restored.validate(&data).unwrap();
    assert_income_eligibility_and_overflow(&data);
}

#[test]
fn disband_and_zero_headcount_remove_identity_without_refund_or_losing_people() {
    let (data, mut campaign) = fixture();
    let founder = campaign.armies[&ArmyId(1)].commander.unwrap();
    let original = campaign.people[&founder].clone();
    let balance = campaign.factions[&campaign.player].resources;
    let counters = campaign.next_ids.clone();
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(1),
        },
    )
    .unwrap();
    assert_eq!(outcome.disbanded, Some(FormationId(1)));
    assert!(matches!(
        outcome.facts[0].kind,
        DomainFactKind::FormationDisbanded {
            formation: FormationId(1),
            ..
        }
    ));
    assert_eq!(campaign.factions[&campaign.player].resources, balance);
    assert!(!campaign.formations.contains_key(&FormationId(1)));
    assert_eq!(campaign.next_ids.formation, counters.formation);
    assert_eq!(
        campaign.people[&founder].assignment,
        PersonAssignment::Formation {
            formation: FormationId(2)
        }
    );
    assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(founder));
    assert_zero_headcount_cleanup(&data, &mut campaign, founder);
    let replacement = apply(
        &mut campaign,
        &data,
        Actor::Player,
        recruit(TroopKind::Warriors, None),
    )
    .unwrap()
    .recruited
    .unwrap();
    assert_eq!(replacement.formation, counters.formation);
    assert_ne!(replacement.army, ArmyId(1));
    let spent = campaign.factions[&campaign.player].resources;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(3),
        },
    )
    .unwrap();
    assert!(!campaign.armies.contains_key(&ArmyId(1)));
    assert_eq!(
        campaign.people[&founder].assignment,
        PersonAssignment::Formation {
            formation: replacement.formation
        }
    );
    assert_eq!(campaign.armies[&replacement.army].commander, None);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: replacement.formation,
        },
    )
    .unwrap();
    assert_eq!(campaign.factions[&campaign.player].resources, spent);
    assert_eq!(
        campaign.people[&founder].assignment,
        PersonAssignment::Site { site: SiteId(1) }
    );
    assert_eq!(campaign.people[&founder].name, original.name);
    assert_eq!(campaign.people[&founder].birth_round, original.birth_round);
    assert_fresh_after_removal(&data, &mut campaign, replacement);
}
