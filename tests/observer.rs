//! Observer campaigns keep all factions under deterministic AI control.

use kestrum::{
    data::{
        generation::ProductionSetup,
        rules::Emblem,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, Actor, Command, MoveOrder},
    state::{
        battle::BattleId,
        military::{ArmyId, FormationId},
        people::PersonAssignment,
        Campaign, CampaignPhase, GameState, StrategicCampaign,
    },
};

fn production_observer(data: &GameData, seed: u64, factions: usize) -> StrategicCampaign {
    StrategicCampaign::new_production_observer(
        data,
        &ProductionSetup {
            kingdom_name: "Observer court".into(),
            emblem: Emblem::Rose,
            factions,
            seed,
        },
    )
    .unwrap()
}

#[test]
fn observer_setup_starts_ai_control_and_keeps_regular_campaigns_human() {
    let data = GameData::load().unwrap();
    let regular = StrategicCampaign::new(&data).unwrap();
    let observer = StrategicCampaign::new_observer(&data).unwrap();

    assert!(!regular.is_observer());
    assert_eq!(regular.phase, CampaignPhase::PlayerTurn);
    assert!(observer.is_observer());
    assert!(matches!(
        observer.phase,
        CampaignPhase::NpcTurn {
            faction,
            paused: false
        } if faction == observer.player
    ));
    assert_eq!(
        observer.ai.factions.len(),
        observer.independent_order().len()
    );
    assert!(observer
        .independent_order()
        .iter()
        .all(|faction| observer.ai.factions.contains_key(faction)));
    assert!(observer.tutorial.current().is_none());
    regular.validate(&data).unwrap();
    observer.validate(&data).unwrap();
}

#[test]
fn observer_rejects_faction_orders_but_can_pause_and_take_a_step() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new_observer(&data).unwrap();
    let unchanged = campaign.clone();
    assert!(matches!(
        engine::apply(&mut campaign, &data, Actor::Player, Command::EndTurn),
        Err(kestrum::engine::RuleError::ObserverControlOnly)
    ));
    assert_eq!(campaign, unchanged);
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(campaign)), &data)
        .unwrap();
    assert_eq!(
        state.command(&data, Command::EndTurn),
        Err(kestrum::engine::RuleError::ObserverControlOnly)
    );
    let before = state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .accepted_sequence;
    state.command(&data, Command::SetNpcPaused(true)).unwrap();
    state.step_observer(&data).unwrap();
    let campaign = state.campaign.as_ref().unwrap().strategic().unwrap();
    assert!(campaign.accepted_sequence > before);
    assert!(matches!(
        campaign.phase,
        CampaignPhase::NpcTurn { paused: true, .. }
    ));
    campaign.validate(&data).unwrap();
}

#[test]
fn same_seed_observer_steps_produce_identical_campaigns() {
    let data = GameData::load().unwrap();
    let mut first = production_observer(&data, 71_003, 8);
    let mut second = production_observer(&data, 71_003, 8);
    for _ in 0..48 {
        if first.observer_finished() {
            break;
        }
        engine::advance_observer(&mut first, &data).unwrap();
        engine::advance_observer(&mut second, &data).unwrap();
        assert_eq!(first, second);
    }
}

#[test]
fn observer_finishes_a_round_without_returning_to_a_player_phase() {
    let data = GameData::load().unwrap();
    let mut campaign = production_observer(&data, 71_004, 8);
    let mut steps = 0;
    while campaign.completed_rounds == 0 && !campaign.observer_finished() {
        engine::advance_observer(&mut campaign, &data).unwrap();
        steps += 1;
        assert!(steps <= 2_000, "observer did not reach a season boundary");
    }
    assert_eq!(campaign.completed_rounds, 1);
    assert!(!campaign.observer_finished());
    assert!(matches!(campaign.phase, CampaignPhase::NpcTurn { .. }));
    assert!(campaign.acted.is_empty());
    campaign.validate(&data).unwrap();
}

#[test]
fn eliminated_former_player_does_not_end_the_observer_campaign() {
    let data = GameData::load().unwrap();
    let mut campaign = production_observer(&data, 71_005, 8);
    let former_player = campaign.player;
    let site = campaign.factions[&former_player].headquarters;
    campaign.world.sites.iter_mut().for_each(|site| {
        if site.controller == Some(former_player) {
            site.controller = None;
        }
    });
    campaign
        .armies
        .retain(|_, army| army.faction != former_player);
    campaign
        .formations
        .retain(|_, formation| formation.faction != former_player);
    for person in campaign
        .people
        .values_mut()
        .filter(|person| person.faction == former_player)
    {
        if person.is_alive() {
            person.assignment = PersonAssignment::Site { site };
        }
        person.career.site_role = None;
    }
    campaign.reconcile_region_control();
    assert!(campaign.defeat_eligible(former_player));
    campaign.validate(&data).unwrap();

    engine::apply(
        &mut campaign,
        &data,
        Actor::Npc(former_player),
        Command::EndTurn,
    )
    .unwrap();
    assert_eq!(
        campaign.factions[&former_player].status,
        kestrum::state::FactionStatus::Eliminated
    );
    assert!(campaign.diplomacy.ending.is_none());
    assert!(!campaign.observer_finished());
    assert!(matches!(
        campaign.phase,
        CampaignPhase::NpcTurn { faction, .. } if faction != former_player
    ));
    engine::advance_observer(&mut campaign, &data).unwrap();
    campaign.validate(&data).unwrap();

    let mut finale = production_observer(&data, 71_007, 8);
    let winner = FactionId(2);
    for site in &mut finale.world.sites {
        if site.controller.is_some_and(|faction| faction != winner) {
            site.controller = None;
        }
    }
    finale.armies.retain(|_, army| army.faction == winner);
    finale
        .formations
        .retain(|_, formation| formation.faction == winner);
    for person in finale
        .people
        .values_mut()
        .filter(|person| person.faction != winner)
    {
        if person.is_alive() {
            person.assignment = PersonAssignment::Site {
                site: finale.factions[&person.faction].headquarters,
            };
        }
        person.career.site_role = None;
    }
    finale.reconcile_region_control();
    assert!(finale
        .factions
        .keys()
        .filter(|faction| **faction != winner)
        .all(|faction| finale.defeat_eligible(*faction)));
    finale.validate(&data).unwrap();
    let acting_faction = finale.player;
    engine::apply(
        &mut finale,
        &data,
        Actor::Npc(acting_faction),
        Command::EndTurn,
    )
    .unwrap();
    assert_eq!(
        finale.diplomacy.ending.as_ref().map(|ending| ending.kind),
        Some(kestrum::state::diplomacy::EndingKind::Victory)
    );
    assert!(finale.is_independent(winner));
    assert!(finale.observer_finished());
    finale.validate(&data).unwrap();
}

#[test]
fn observer_automatically_commits_battles_that_include_the_former_player() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new_observer(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign.people.clear();
    campaign.legacy_items.clear();
    for (army_id, formation_id, site_id) in [(1, 1, 8), (3, 7, 10)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).unwrap();
        army.site = SiteId(site_id);
        army.commander = None;
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
    }
    campaign.validate(&data).unwrap();
    let player = campaign.player;
    let outcome = engine::apply(
        &mut campaign,
        &data,
        Actor::Npc(player),
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(8), SiteId(10)],
        }),
    )
    .unwrap();
    assert!(outcome.battle_pending);
    assert_eq!(campaign.battles.len(), 0);

    let resolved = engine::advance_observer(&mut campaign, &data).unwrap();
    assert!(resolved.battle.is_some());
    assert!(campaign.pending_battle.is_none());
    assert_eq!(campaign.battles.len(), 1);
    let report = &campaign.battles[&BattleId(1)];
    assert!(report
        .participant_factions()
        .any(|faction| faction == campaign.player));
    campaign.validate(&data).unwrap();
}

#[test]
fn observer_save_round_trips_and_old_saves_default_to_regular_mode() {
    let data = GameData::load().unwrap();
    let observer = production_observer(&data, 71_006, 8);
    let saved = serde_json::to_string(&Campaign::Strategic(Box::new(observer.clone()))).unwrap();
    let restored: Campaign = serde_json::from_str(&saved).unwrap();
    assert_eq!(restored.strategic(), Some(&observer));

    let mut invalid = observer;
    invalid.player = FactionId(u32::MAX);
    assert!(invalid.validate(&data).unwrap_err().contains("player"));

    let regular = StrategicCampaign::new(&data).unwrap();
    let mut old_save = serde_json::to_value(Campaign::Strategic(Box::new(regular))).unwrap();
    old_save.as_object_mut().unwrap().remove("observer_mode");
    let restored: Campaign = serde_json::from_value(old_save).unwrap();
    assert!(!restored.strategic().unwrap().is_observer());
    restored.validate(&data).unwrap();
}

#[test]
fn playback_speed_and_pause_reset_the_clock_without_catch_up_bursts() {
    let mut playback = kestrum::state::observer::ObserverPlayback::default();
    assert!(!playback.due_step(0.25, 1.0));
    assert!(playback.set_speed(4));
    assert!(playback.due_step(0.25, 1.0));
    assert!(playback.due_step(10.0, 1.0));
    assert!(!playback.due_step(0.01, 1.0));
    playback.set_paused(true);
    assert!(!playback.due_step(10.0, 1.0));
    assert!(!playback.set_speed(3));
    playback.set_paused(false);
    assert!(!playback.due_step(0.24, 1.0));
    assert!(playback.due_step(0.02, 1.0));
}
