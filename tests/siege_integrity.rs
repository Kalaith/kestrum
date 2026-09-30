//! Siege data, observer boundaries and persisted exceptions through real commands.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

use kestrum::{
    data::{
        siege::{SiegeRules, SOURCE},
        world::{FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{apply, project, siege_view, Actor, Command, MoveOrder},
    state::{
        battle::{BattleContext, BattleRoadDamage},
        military::ArmyId,
        siege::{SiegeAction, SiegeOrder},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::data_loader::parse_json_labeled;
use serde_json::{json, Value};

#[path = "support/siege_integrity.rs"]
mod support;
use support::*;

#[test]
fn typed_siege_rules_weaken_walls_without_overflow_or_automatic_victory() {
    let data = GameData::load().unwrap();
    let rules = &data.siege;
    for (steps, damage, engines, expected) in [
        (0, 0, false, 1500),
        (1, 5, false, 1410),
        (5, 25, false, 1050),
        (6, 30, false, 1000),
        (0, 0, true, 1300),
        (0, 100, false, 1000),
        (u32::MAX, 0, true, 1000),
    ] {
        assert_eq!(rules.wall_permille(steps, damage, engines), expected);
    }
    for (field, value) in [
        ("schema_version", 2),
        ("wall_minimum_permille", 999),
        ("wall_base_permille", 999),
        ("wall_step_reduction_permille", 0),
        ("wall_damage_reduction_permille", 10001),
        ("engine_wall_reduction_permille", 0),
        ("fort_damage_per_step", 101),
        ("assault_structural_damage", 101),
        ("assault_fort_damage", 101),
        ("assault_road_damage", 101),
        ("assault_road_loss_permille", 1001),
    ] {
        let mut raw = serde_json::to_value(rules).unwrap();
        raw[field] = json!(value);
        let parsed: SiegeRules = parse_json_labeled(SOURCE, &raw.to_string()).unwrap();
        let error = parsed.validate().unwrap_err();
        assert!(error.contains(SOURCE) && error.contains(field), "{error}");
    }
    let mut unknown = serde_json::to_value(rules).unwrap();
    unknown["automatic_capture_steps"] = json!(5);
    assert!(parse_json_labeled::<SiegeRules>(SOURCE, &unknown.to_string()).is_err());
    let (data, mut campaign) = siege_fixture(false);
    for _ in 0..7 {
        finish(&mut campaign, &data);
    }
    assert_eq!(campaign.sieges[&SiteId(5)].elapsed_steps, 7);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(FactionId(3))
    );
    assert_eq!(
        campaign.siege_wall_permille(SiteId(5), &data.siege),
        Some(1000)
    );
}

#[test]
fn live_siege_partitions_and_progress_reject_corrupt_save_references() {
    let (data, mut campaign) = siege_fixture(false);
    finish(&mut campaign, &data);
    assert_eq!(
        catalogue_load(&data, &serialized(&campaign)),
        Campaign::Strategic(Box::new(campaign.clone()))
    );
    for corruption in [
        "id",
        "counter",
        "participant",
        "duplicate",
        "side",
        "empty",
        "location",
        "control",
        "fort",
        "future",
        "progress",
        "damage",
        "unlisted",
        "inactive",
    ] {
        let mut invalid = campaign.clone();
        corrupt_partition(&mut invalid, corruption);
        let error = invalid.validate(&data).unwrap_err();
        if corruption == "inactive" {
            assert!(
                error.contains("inactive faction retains independent forces or work"),
                "{error}"
            );
        }
    }
    let (_, mut absent) = siege_fixture(false);
    let siege = absent.sieges.remove(&SiteId(5)).unwrap();
    absent.next_ids.siege = siege.id;
    assert!(
        absent.validate(&data).is_err(),
        "retained facts must protect monotonic identities"
    );
}

#[test]
fn complete_old_save_group_migrates_through_catalogue_without_replaying_reports() {
    let (data, original) = field_fixture();
    let mut old = serialized(&original);
    strip_siege(&mut old);
    let expected = Campaign::Strategic(Box::new(original));
    let decoded: Campaign = serde_json::from_value(old.clone()).unwrap();
    decoded.validate(&data).unwrap();
    assert_eq!(decoded, expected);
    let envelope = macroquad_toolkit::persistence::encode_slot("strategic_v2", &old, "2").unwrap();
    assert_eq!(
        kestrum::state::persistence::load_legacy(&envelope, &data).unwrap(),
        expected
    );
    assert_eq!(catalogue_load(&data, &old), expected);
    let mut partial = old.clone();
    partial["sieges"] = json!({});
    assert!(serde_json::from_value::<Campaign>(partial).is_err());
    assert_partial_report_groups(&old, &serialized(expected.strategic().unwrap()));
}

#[test]
fn retained_assault_context_validates_after_siege_lifts_and_rejects_forged_effects() {
    let (data, mut campaign) = siege_fixture(true);
    finish(&mut campaign, &data);
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Siege(SiegeOrder {
            site: SiteId(5),
            action: SiegeAction::Assault,
            armies: vec![ArmyId(1)],
            destination: None,
        }),
    )
    .unwrap();
    assert!(outcome.battle_pending);
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    let id = outcome.battle.unwrap();
    assert!(!campaign.sieges.contains_key(&SiteId(5)));
    assert!(matches!(
        campaign.battles[&id].context,
        BattleContext::Assault { .. }
    ));
    assert_eq!(
        catalogue_load(&data, &serialized(&campaign)),
        Campaign::Strategic(Box::new(campaign.clone()))
    );
    for corruption in [
        "field",
        "counter",
        "wall",
        "exchange",
        "fort",
        "road",
        "destination",
    ] {
        let mut invalid = campaign.clone();
        let report = invalid.battles.get_mut(&id).unwrap();
        match corruption {
            "field" => report.context = BattleContext::Field,
            "counter" => {
                report.context = BattleContext::Assault {
                    siege: invalid.next_ids.siege,
                }
            }
            "wall" => report.wall_permille = 999,
            "exchange" => report.exchanges[0].wall_permille = 999,
            "fort" => report.fort_damage_added = 101,
            "road" => {
                report.road_damage = Some(BattleRoadDamage {
                    route: kestrum::data::world::RouteId(999),
                    added: 10,
                })
            }
            "destination" => report.attacker.armies[0].final_site = Some(SiteId(2)),
            _ => unreachable!(),
        }
        assert!(invalid.validate(&data).is_err(), "accepted {corruption}");
    }
}

#[test]
fn siege_projection_and_choices_do_not_reveal_live_enemy_rosters_or_remote_progress() {
    let (data, mut campaign) = siege_fixture(false);
    let own = project(&campaign, FactionId(1)).unwrap();
    let own_view = siege_view(&campaign, &data, FactionId(1), SiteId(5)).unwrap();
    assert_eq!(own_view.own_armies, vec![ArmyId(1)]);
    assert!(own_view.supplied_armies.contains(&ArmyId(1)));
    assert_eq!(own.sieges.len(), 1);
    assert!(siege_view(&campaign, &data, FactionId(2), SiteId(5)).is_none());
    campaign.armies.get_mut(&ArmyId(3)).unwrap().name = "Unseen private garrison".into();
    for id in campaign.armies[&ArmyId(3)]
        .formation_ids()
        .collect::<Vec<_>>()
    {
        campaign.formations.get_mut(&id).unwrap().headcount = 1;
    }
    campaign.validate(&data).unwrap();
    assert_eq!(project(&campaign, FactionId(1)).unwrap(), own);
    assert_eq!(
        siege_view(&campaign, &data, FactionId(1), SiteId(5)).unwrap(),
        own_view
    );
    let remote = project(&campaign, FactionId(2)).unwrap();
    campaign.world.fort_damage.insert(SiteId(5), 80);
    assert_eq!(project(&campaign, FactionId(2)).unwrap(), remote);
    assert!(remote.sieges.is_empty() && !remote.world.fort_damage.contains_key(&SiteId(5)));
}
