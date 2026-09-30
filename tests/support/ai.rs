use super::*;
use kestrum::{engine::MoveOrder, state::military::FormationId};

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), FactionId(2));
    (data, campaign)
}
pub(super) fn no_resources(campaign: &mut StrategicCampaign) {
    campaign.factions.get_mut(&FactionId(2)).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    for site in campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(FactionId(2)))
    {
        campaign
            .world
            .focus
            .insert(site.id, kestrum::state::construction::Focus::Gold);
    }
}

fn rose_oak_war(campaign: &mut StrategicCampaign) {
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .unwrap()
        .state = kestrum::data::world::DiplomaticState::War;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == [FactionId(1), FactionId(2)])
        .unwrap();
    pair.peace_since = None;
    pair.truce_until = None;
}
pub(super) fn next_oak_turn(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    for _ in 0..5 {
        if campaign.completed_rounds > round && campaign.active_faction() == FactionId(2) {
            return;
        }
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
    panic!("one cycle returns to Oak");
}

pub(super) fn invalid_rules_and_state(data: &GameData, campaign: &StrategicCampaign) {
    for mutate in [0, 1, 2] {
        let mut invalid = data.ai.clone();
        match mutate {
            0 => invalid.max_commands_per_phase = 65,
            1 => invalid.recruitment_order[0] = TroopKind::Medics,
            _ => invalid.attack_advantage_percent = 100,
        }
        assert!(invalid.validate().is_err());
    }
    let mut invalid = campaign.clone();
    invalid
        .ai
        .factions
        .get_mut(&FactionId(2))
        .unwrap()
        .accepted_commands = 65;
    assert!(invalid.validate(data).is_err());
    let mut invalid = campaign.clone();
    invalid
        .ai
        .factions
        .get_mut(&FactionId(2))
        .unwrap()
        .objective = Some(AiObjective {
        site: SiteId(999),
        chosen_round: 0,
        kind: AiObjectiveKind::Expand,
    });
    assert!(invalid.validate(data).is_err());
    let raw = serde_json::to_string(campaign).unwrap();
    let restored: StrategicCampaign = serde_json::from_str(&raw).unwrap();
    restored.validate(data).unwrap();
    assert_eq!(*campaign, restored);
}

pub(super) fn unknown_enemy_and_empty_capture(data: &GameData) {
    let (_, mut occupied) = fixture();
    rose_oak_war(&mut occupied);
    no_resources(&mut occupied);
    occupied.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    occupied
        .set_site_control(data, SiteId(5), Some(FactionId(1)), false)
        .unwrap();
    let decision = ai::propose(&occupied, data, FactionId(2)).unwrap();
    assert!(
        matches!(&decision.command, Command::Move(order)
        if order.path == [SiteId(2), SiteId(5)]),
        "{decision:?}"
    );
    let mut weak = occupied.clone();
    for formation in weak
        .formations
        .values_mut()
        .filter(|f| f.faction == FactionId(1))
    {
        formation.headcount = 1;
    }
    assert_eq!(
        decision,
        ai::propose(&weak, data, FactionId(2)).unwrap(),
        "AI cannot see tiny live enemy headcounts"
    );
    let mut cautious = weak.clone();
    for formation in cautious
        .formations
        .values_mut()
        .filter(|f| f.faction == FactionId(2))
    {
        formation.headcount = formation.capacity / 2;
    }
    assert_eq!(
        ai::propose(&cautious, data, FactionId(2)).unwrap().command,
        Command::EndTurn,
        "an understrength army cannot blindly probe"
    );
    let restored: StrategicCampaign =
        serde_json::from_str(&serde_json::to_string(&weak).unwrap()).unwrap();
    assert_eq!(
        decision,
        ai::propose(&restored, data, FactionId(2)).unwrap()
    );
    let encounter = advance_npc(&mut weak, data).unwrap();
    assert!(encounter.battle_pending);
    assert!(encounter.battle.is_none());
    assert!(weak.pending_battle.is_some());
    assert!(
        apply(&mut weak, data, Actor::Player, Command::StartPendingBattle)
            .unwrap()
            .battle
            .is_some()
    );
    occupied.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(1);
    let decision = ai::propose(&occupied, data, FactionId(2)).unwrap();
    assert!(
        matches!(decision.command,Command::Move(ref order) if order.path==vec![SiteId(2),SiteId(5)])
    );
    let result = advance_npc(&mut occupied, data).unwrap();
    assert!(result.battle.is_none());
    assert_eq!(
        occupied.world.site(SiteId(5)).unwrap().controller,
        Some(FactionId(2))
    );
    assert_eq!(occupied.armies[&ArmyId(2)].site, SiteId(5));
}

pub(super) fn actual_last_known_attack(data: &GameData) {
    let mut campaign = StrategicCampaign::new(data).unwrap();
    rose_oak_war(&mut campaign);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    campaign
        .set_site_control(data, SiteId(5), Some(FactionId(1)), false)
        .unwrap();
    campaign
        .set_site_control(data, SiteId(6), Some(FactionId(2)), false)
        .unwrap();
    let result = apply(
        &mut campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(5), SiteId(6)],
        }),
    )
    .unwrap();
    assert!(result.battle_pending);
    let resolved = apply(
        &mut campaign,
        data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    let report = campaign.battles[&resolved.battle.unwrap()].clone();
    let target = report.attacker.armies[0]
        .final_site
        .expect("surviving recorded enemy");
    apply(&mut campaign, data, Actor::Player, Command::EndTurn).unwrap();
    for formation in campaign
        .formations
        .values_mut()
        .filter(|f| f.faction == FactionId(2))
    {
        formation.headcount = formation.capacity;
        formation.movement_spent = 0;
    }
    for person in campaign
        .people
        .values_mut()
        .filter(|p| p.faction == FactionId(2))
    {
        person.movement_spent = 0;
    }
    no_resources(&mut campaign);
    campaign.ai.factions.insert(
        FactionId(2),
        AiFactionState {
            objective: Some(AiObjective {
                site: target,
                chosen_round: 0,
                kind: AiObjectiveKind::Attack,
            }),
            ..Default::default()
        },
    );
    campaign.validate(data).unwrap();
    let decision = ai::propose(&campaign, data, FactionId(2)).unwrap();
    assert!(
        matches!(decision.command,Command::Move(ref order) if order.path.last()==Some(&target)),
        "real recorded losses permit a sufficiently stronger attack: {decision:?}"
    );
    let mut changed = campaign.clone();
    for formation in changed
        .formations
        .values_mut()
        .filter(|f| f.faction == FactionId(1))
    {
        formation.headcount = formation.capacity;
    }
    assert_eq!(
        decision,
        ai::propose(&changed, data, FactionId(2)).unwrap(),
        "live recovery does not rewrite the dated enemy snapshot"
    );
}

pub(super) fn headquarters_emergency(data: &GameData) {
    let (_, mut campaign) = fixture();
    rose_oak_war(&mut campaign);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    campaign
        .set_site_control(data, SiteId(5), Some(FactionId(1)), false)
        .unwrap();
    campaign
        .set_site_control(data, SiteId(6), Some(FactionId(2)), false)
        .unwrap();
    campaign.ai.factions.insert(
        FactionId(2),
        AiFactionState {
            objective: Some(AiObjective {
                site: SiteId(13),
                chosen_round: 0,
                kind: AiObjectiveKind::Expand,
            }),
            ..Default::default()
        },
    );
    let decision = ai::propose(&campaign, data, FactionId(2)).unwrap();
    assert_eq!(
        decision.objective.as_ref().unwrap().kind,
        AiObjectiveKind::Defend
    );
    assert_eq!(decision.objective.as_ref().unwrap().site, SiteId(2));
    assert!(matches!(decision.command, Command::Move(ref order)
        if order.path == [SiteId(6), SiteId(5)]));
    let (_, mut weak) = fixture();
    no_resources(&mut weak);
    weak.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    weak.set_site_control(data, SiteId(6), Some(FactionId(2)), false)
        .unwrap();
    for id in [FormationId(4), FormationId(5), FormationId(6)] {
        weak.formations.get_mut(&id).unwrap().headcount = 1;
    }
    let decision = ai::propose(&weak, data, FactionId(2)).unwrap();
    assert!(
        matches!(decision.command,Command::Move(ref order) if order.path==vec![SiteId(6),SiteId(5)]),
        "damaged unsupplied army heads toward headquarters supply"
    );
}
