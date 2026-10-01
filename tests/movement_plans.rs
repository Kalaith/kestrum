//! Long orders retain their remainder and recheck access on real faction turns.

use kestrum::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, ActionOutcome, Actor, Command, MoveOrder, MovementBlock, RuleError},
    state::{campaign::DomainFactKind, military::ArmyId, Campaign, StrategicCampaign},
};

fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn move_group(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    armies: &[ArmyId],
    path: &[u32],
) -> ActionOutcome {
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: armies.to_vec(),
            path: path.iter().copied().map(SiteId).collect(),
        }),
    )
    .unwrap()
}

fn season(campaign: &mut StrategicCampaign, data: &GameData) -> ActionOutcome {
    let round = campaign.completed_rounds;
    loop {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        let outcome = engine::apply(campaign, data, actor, Command::EndTurn).unwrap();
        if campaign.completed_rounds != round {
            return outcome;
        }
    }
}

fn exhaust(campaign: &mut StrategicCampaign, data: &GameData, army: ArmyId) {
    for id in campaign.armies[&army].formation_ids().collect::<Vec<_>>() {
        let formation = campaign.formations.get_mut(&id).unwrap();
        formation.movement_spent = formation.movement_allowance(data);
    }
}

#[test]
fn long_orders_keep_the_remainder_across_save_reload_and_resume_on_the_next_turn() {
    let (data, mut campaign) = fixture();
    let moved = move_group(&mut campaign, &data, &[ArmyId(1)], &[1, 5, 6, 7, 14]);
    assert_eq!(moved.movement.unwrap().path, [1, 5, 6, 7].map(SiteId));
    assert_eq!(campaign.movement_plans[0].path, [7, 14].map(SiteId));
    let raw = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    campaign = kestrum::state::persistence::load_legacy(&raw, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    let outcome = season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    assert!(campaign.movement_plans.is_empty());
    assert_eq!(outcome.continued_movements[0].spent, 3);
    assert!(outcome.facts.iter().any(|fact| matches!(&fact.kind,
        DomainFactKind::ArmiesMoved { path, spent: 3, .. } if *path == [7, 14].map(SiteId))));
    assert_eq!(
        engine::army_remaining(&campaign, &data, ArmyId(1)).unwrap(),
        3
    );
    assert!(
        engine::action_notices(&campaign, &data, campaign.player, &outcome)
            .iter()
            .any(|notice| notice.contains("Arrived"))
    );
    campaign.validate(&data).unwrap();

    let mut old = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    old.as_object_mut().unwrap().remove("movement_plans");
    let restored: Campaign = serde_json::from_value(old).unwrap();
    assert!(restored.strategic().unwrap().movement_plans.is_empty());
}

#[test]
fn exhausted_orders_are_confirmable_without_fake_movement_and_continue_over_multiple_turns() {
    let (data, mut campaign) = fixture();
    exhaust(&mut campaign, &data, ArmyId(1));
    let preview =
        engine::movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(14))
            .unwrap();
    assert_eq!(preview.reachable_steps, 0);
    assert!(preview.can_confirm());
    engine::preview(
        &campaign,
        &data,
        Actor::Player,
        Command::Move(preview.order.clone()),
    )
    .unwrap();
    let queued = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(preview.order),
    )
    .unwrap();
    assert!(queued.facts.is_empty());
    assert_eq!(queued.movement.unwrap().spent, 0);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(1));
    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(7));
    assert_eq!(campaign.movement_plans[0].path, [7, 14].map(SiteId));
    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    assert!(campaign.movement_plans.is_empty());
}

#[test]
fn peaceful_territory_rejects_the_whole_order_even_beyond_the_current_budget() {
    let (data, mut campaign) = fixture();
    campaign
        .set_site_control(&data, SiteId(14), Some(FactionId(2)), false)
        .unwrap();
    let preview =
        engine::movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(14))
            .unwrap();
    assert!(!preview.can_confirm());
    assert_eq!(
        preview.blocked.as_ref().unwrap().reason,
        MovementBlock::PeaceBoundary
    );
    let before = campaign.clone();
    for checking in [true, false] {
        let result = if checking {
            engine::preview(
                &campaign,
                &data,
                Actor::Player,
                Command::Move(preview.order.clone()),
            )
            .map(|_| ())
        } else {
            engine::apply(
                &mut campaign,
                &data,
                Actor::Player,
                Command::Move(preview.order.clone()),
            )
            .map(|_| ())
        };
        assert!(matches!(
            result,
            Err(RuleError::MovementBlocked {
                reason: MovementBlock::PeaceBoundary,
                ..
            })
        ));
        assert_eq!(campaign, before);
    }
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert!(
        engine::movement_preview(&campaign, &data, campaign.player, &[ArmyId(1)], SiteId(14))
            .unwrap()
            .can_confirm()
    );
    move_group(&mut campaign, &data, &[ArmyId(1)], &[1, 5, 6, 7, 14]);
    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));

    let (data, mut changed) = fixture();
    move_group(&mut changed, &data, &[ArmyId(1)], &[1, 5, 6, 7, 14]);
    changed
        .set_site_control(&data, SiteId(14), Some(FactionId(2)), false)
        .unwrap();
    let halted = season(&mut changed, &data);
    assert_eq!(changed.armies[&ArmyId(1)].site, SiteId(7));
    assert!(changed.movement_plans.is_empty());
    assert_eq!(
        halted.continued_movements[0].stop.as_ref().unwrap().reason,
        MovementBlock::PeaceBoundary
    );
}

#[test]
fn group_plans_keep_the_slowest_budget_and_new_orders_or_cancellation_replace_them() {
    let (data, mut campaign) = fixture();
    let formation = campaign.armies[&ArmyId(1)].formation_ids().nth(1).unwrap();
    let other = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy { formation },
    )
    .unwrap()
    .split_army
    .unwrap();
    campaign
        .formations
        .get_mut(&formation)
        .unwrap()
        .movement_spent = 2;
    let moved = move_group(&mut campaign, &data, &[ArmyId(1), other], &[1, 5, 6, 7, 14]);
    assert_eq!(moved.movement.unwrap().spent, 4);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(6));
    assert_eq!(campaign.armies[&other].site, SiteId(6));
    assert_eq!(campaign.movement_plans[0].armies, [ArmyId(1), other]);
    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    assert_eq!(campaign.armies[&other].site, SiteId(14));

    exhaust(&mut campaign, &data, ArmyId(1));
    move_group(&mut campaign, &data, &[ArmyId(1)], &[14, 7, 6]);
    move_group(&mut campaign, &data, &[ArmyId(1)], &[14, 7]);
    assert_eq!(campaign.movement_plans.len(), 1);
    assert_eq!(campaign.movement_plans[0].path, [14, 7].map(SiteId));
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelMovementPlan { army: ArmyId(1) },
    )
    .unwrap();
    assert!(campaign.movement_plans.is_empty());
    season(&mut campaign, &data);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
}

#[test]
fn resumed_orders_stop_at_contact_and_removed_armies_leave_no_stale_plans() {
    let (data, mut campaign) = fixture();
    let formation = campaign.armies[&ArmyId(1)].formation_ids().nth(1).unwrap();
    let other = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy { formation },
    )
    .unwrap()
    .split_army
    .unwrap();
    move_group(&mut campaign, &data, &[ArmyId(1)], &[1, 5, 6, 7, 14]);
    exhaust(&mut campaign, &data, other);
    move_group(&mut campaign, &data, &[other], &[1, 5]);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(14);
    let contact = season(&mut campaign, &data);
    assert!(contact.battle_pending);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(14));
    assert_eq!(campaign.movement_plans.len(), 1);
    assert_eq!(campaign.armies[&other].site, SiteId(1));
    assert!(campaign.pending_battle.as_ref().unwrap().movement.is_some());
    campaign.validate(&data).unwrap();
    let resolved = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    assert!(resolved.battle.is_some());
    assert_eq!(campaign.armies[&other].site, SiteId(5));
    assert!(campaign.movement_plans.is_empty());

    let (data, mut removed) = fixture();
    exhaust(&mut removed, &data, ArmyId(1));
    move_group(&mut removed, &data, &[ArmyId(1)], &[1, 5, 6]);
    for formation in removed.armies[&ArmyId(1)]
        .formation_ids()
        .collect::<Vec<_>>()
    {
        engine::apply(
            &mut removed,
            &data,
            Actor::Player,
            Command::Disband { formation },
        )
        .unwrap();
    }
    assert!(removed.movement_plans.is_empty());
    removed.validate(&data).unwrap();
}
