//! The five K06 recovery contracts use real seasonal resolution and saved state.

use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{advance_npc, apply, project, recovery_preview, Actor, Command},
    state::{
        military::{ArmyId, FormationId},
        persistence::load_legacy,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

#[path = "support/recovery.rs"]
mod support;
use support::*;

#[test]
fn supplied_recovery_uses_the_cap_missing_headcount_and_post_upkeep_gold() {
    for (kind, count, cost, one_cost) in [
        (TroopKind::Warriors, 20, 6, 1),
        (TroopKind::Spearmen, 20, 7, 1),
        (TroopKind::Archers, 16, 8, 1),
        (TroopKind::Riders, 8, 12, 2),
        (TroopKind::Medics, 8, 6, 1),
        (TroopKind::SiegeEngines, 4, 12, 3),
    ] {
        for missing in [1, count + 1] {
            let (data, mut campaign) = budget_fixture(100);
            let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
            formation.kind = kind;
            formation.capacity = data.economy.formations[&kind].capacity;
            formation.headcount = formation.capacity - missing;
            let before = formation.headcount;
            let expected = missing.min(count);
            let expected_cost = if missing == 1 { one_cost } else { cost };
            let previews = recovery_preview(&campaign, &data, campaign.player).unwrap();
            let preview = &previews[0];
            assert_eq!(
                (preview.maximum, preview.restored, preview.gold_cost),
                (expected, expected, expected_cost)
            );
            finish_round(&mut campaign, &data);
            assert_eq!(
                campaign.formations[&FormationId(1)].headcount,
                before + expected
            );
            let statement = campaign.factions[&campaign.player]
                .last_recovery
                .as_ref()
                .unwrap();
            assert_eq!(
                (statement.restored, statement.gold_spent),
                (u64::from(expected), expected_cost)
            );
            assert_eq!(statement.entries[0].kind, kind);
            assert_eq!(statement.closing_gold, 100 - expected_cost);
            campaign.validate(&data).unwrap();
        }
    }
    assert_income_then_upkeep_then_recovery();
    let (mut data, mut campaign) = budget_fixture(100);
    data.economy
        .formations
        .get_mut(&TroopKind::Warriors)
        .unwrap()
        .capacity = 103;
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.kind == TroopKind::Warriors)
    {
        formation.capacity = 103;
    }
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 21;
    assert_eq!(
        recovery_preview(&campaign, &data, campaign.player).unwrap()[0].maximum,
        20
    );
}

#[test]
fn partial_recovery_chooses_the_largest_affordable_headcount_with_one_ceil() {
    for (gold, restored, cost) in [
        (0, 0, 0),
        (1, 3, 1),
        (2, 6, 2),
        (3, 10, 3),
        (5, 16, 5),
        (6, 20, 6),
        (100, 20, 6),
    ] {
        let (data, mut campaign) = budget_fixture(gold);
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = 21;
        let original = campaign.clone();
        let preview = recovery_preview(&campaign, &data, campaign.player).unwrap();
        assert_eq!(campaign, original);
        assert_eq!(
            (preview[0].restored, preview[0].gold_cost),
            (restored, cost)
        );
        assert_eq!(preview[0].blocked.is_some(), restored == 0);
        finish_round(&mut campaign, &data);
        assert_eq!(
            campaign.formations[&FormationId(1)].headcount,
            21 + restored
        );
        assert_eq!(
            campaign.factions[&campaign.player].resources.gold,
            gold - cost
        );
        assert_eq!(campaign.factions[&campaign.player].resources.wood, 123);
        assert_eq!(campaign.factions[&campaign.player].resources.stone, 88);
    }
    let (mut data, mut campaign) = budget_fixture(i64::MAX);
    data.economy
        .formations
        .get_mut(&TroopKind::Warriors)
        .unwrap()
        .recruit_cost
        .gold = i64::MAX;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 21;
    let preview = recovery_preview(&campaign, &data, campaign.player).unwrap();
    assert_eq!(preview[0].restored, 20);
    assert_eq!(preview[0].gold_cost, 922_337_203_685_477_581);
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.factions[&campaign.player]
            .last_recovery
            .as_ref()
            .unwrap()
            .gold_spent,
        preview[0].gold_cost
    );
}

#[test]
fn scarce_gold_follows_faction_army_then_slot_order_without_cross_funding() {
    let (data, mut campaign) = budget_fixture(11);
    for id in [1, 2, 3, 4] {
        campaign
            .formations
            .get_mut(&FormationId(id))
            .unwrap()
            .headcount = 1;
    }
    // Slot order intentionally differs from formation-ID order.
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots = [
        Some(FormationId(3)),
        Some(FormationId(1)),
        None,
        None,
        None,
        None,
    ];
    let mut second = campaign.armies[&ArmyId(1)].clone();
    second.id = ArmyId(10);
    second.commander = None;
    second.slots = [Some(FormationId(2)), None, None, None, None, None];
    campaign.armies.insert(second.id, second);
    campaign.next_ids.army = ArmyId(11);
    campaign
        .factions
        .get_mut(&FactionId(2))
        .unwrap()
        .resources
        .gold = 1;
    campaign.validate(&data).unwrap();
    let previews = recovery_preview(&campaign, &data, campaign.player).unwrap();
    assert_eq!(
        previews
            .iter()
            .map(|entry| (entry.army, entry.formation, entry.restored, entry.gold_cost))
            .collect::<Vec<_>>(),
        vec![
            (ArmyId(1), FormationId(3), 16, 8),
            (ArmyId(1), FormationId(1), 10, 3),
            (ArmyId(10), FormationId(2), 0, 0)
        ]
    );
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.formations[&FormationId(3)].headcount, 17);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 11);
    assert_eq!(campaign.formations[&FormationId(2)].headcount, 1);
    assert_eq!(campaign.formations[&FormationId(4)].headcount, 4);
    let statement = campaign.factions[&campaign.player]
        .last_recovery
        .as_ref()
        .unwrap();
    assert_eq!(
        statement
            .entries
            .iter()
            .map(|entry| entry.formation)
            .collect::<Vec<_>>(),
        [FormationId(3), FormationId(1)]
    );
    assert_eq!(statement.closing_gold, 0);
    assert_eq!(campaign.factions[&FactionId(2)].resources.gold, 0);
    let visible = project(&campaign, campaign.player).unwrap();
    assert!(visible
        .factions
        .iter()
        .filter(|faction| faction.id != campaign.player)
        .all(|faction| faction.last_recovery.is_none()));
}

#[test]
fn cut_supply_deficit_and_destroyed_formations_cannot_recover() {
    for obstruction in [
        "neutral_link",
        "foreign_peace_link",
        "contested_link",
        "lost_hq",
    ] {
        let (data, mut campaign) = budget_fixture(100);
        for site in [SiteId(5), SiteId(6)] {
            campaign
                .set_site_control(&data, site, Some(campaign.player), false)
                .unwrap();
        }
        campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = 21;
        match obstruction {
            "neutral_link" => campaign
                .set_site_control(&data, SiteId(5), None, false)
                .unwrap(),
            "foreign_peace_link" => campaign
                .set_site_control(&data, SiteId(5), Some(FactionId(2)), false)
                .unwrap(),
            "contested_link" => campaign
                .set_site_control(&data, SiteId(5), Some(campaign.player), true)
                .unwrap(),
            _ => campaign
                .set_site_control(&data, SiteId(1), Some(FactionId(2)), false)
                .unwrap(),
        }
        let previews = recovery_preview(&campaign, &data, campaign.player).unwrap();
        assert!(previews[0].blocked.as_ref().unwrap().contains("Cut off"));
        finish_round(&mut campaign, &data);
        assert_eq!(campaign.formations[&FormationId(1)].headcount, 21);
        assert!(campaign.factions[&campaign.player]
            .last_recovery
            .as_ref()
            .unwrap()
            .entries
            .is_empty());
    }
    assert_deficit_blocks_and_paid_boundary_clears();
    assert_inactive_factions_cannot_recover();
    let (data, mut campaign) = budget_fixture(100);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 0;
    let before = campaign.clone();
    assert!(recovery_preview(&campaign, &data, campaign.player).is_err());
    assert!(apply(&mut campaign, &data, Actor::Player, Command::EndTurn).is_err());
    assert_eq!(campaign, before);
    campaign.remove_formation(FormationId(1)).unwrap();
    finish_round(&mut campaign, &data);
    assert!(!campaign.formations.contains_key(&FormationId(1)));
    assert!(campaign.factions[&campaign.player]
        .last_recovery
        .as_ref()
        .unwrap()
        .entries
        .is_empty());
}

#[test]
fn recovery_preserves_formation_identity_metadata_and_replays_through_saves() {
    let (data, mut campaign) = budget_fixture(100);
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.headcount = 21;
    formation.movement_spent = 4;
    let mut expected = formation.clone();
    expected.headcount = 41;
    expected.movement_spent = 0; // The seasonal boundary independently resets movement.
    let before = campaign.clone();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    advance_npc(&mut campaign, &data).unwrap();
    let saved = Campaign::Strategic(Box::new(campaign.clone()));
    let restored: Campaign = serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
    let mut resumed = restored.strategic().unwrap().clone();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        assert_eq!(
            advance_npc(&mut campaign, &data).unwrap(),
            advance_npc(&mut resumed, &data).unwrap()
        );
    }
    assert_eq!(campaign, resumed);
    assert_eq!(campaign.formations[&FormationId(1)], expected);
    assert_eq!(campaign.armies, before.armies);
    assert_eq!(campaign.people, before.people);
    assert_eq!(campaign.rng, before.rng);
    assert_eq!(campaign.next_ids.army, before.next_ids.army);
    assert_eq!(campaign.next_ids.formation, before.next_ids.formation);
    assert_recovery_save_validation(&data, &campaign);
    assert_pre_recovery_save_migration(&data, &before);
    campaign.remove_formation(FormationId(1)).unwrap();
    let raw = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    restored.validate(&data).unwrap(); // Historical receipt does not require the removed entity.
}
