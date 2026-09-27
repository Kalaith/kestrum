//! K18's integrated Rosemarch route keeps real armies and people continuous.

use kestrum::{
    data::world::{FactionId, SiteId},
    engine::{apply, history_page, Actor, Command, HistoryFilter, MoveOrder},
    state::{
        evidence::EvidenceKind,
        history::{HistoryKind, HistoryKindFilter, HistorySubject},
        military::{ArmyId, FormationId},
        people::PersonId,
        persistence::load_legacy,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

#[test]
fn rosemarch_counterattack_retreat_and_rematch_survive_save_reload_once() {
    let data = kestrum::data::GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();

    assert_eq!(campaign.world.site(SiteId(5)).unwrap().key, "west_gate");
    assert_eq!(campaign.world.site(SiteId(6)).unwrap().key, "milltown");
    assert_eq!(campaign.world.site(SiteId(9)).unwrap().key, "high_fort");
    assert_eq!(campaign.armies[&ArmyId(1)].faction, FactionId(1));
    assert_eq!(campaign.armies[&ArmyId(3)].faction, FactionId(3));
    assert_eq!(campaign.people[&PersonId(1)].name, "Aveline Rose");
    assert_eq!(campaign.people[&PersonId(3)].name, "Mara Hawthorn");

    // Keep Rose's two-unit force at Milltown and send the commander's force ahead.
    let split = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(1),
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    assert_eq!(split, ArmyId(5));
    move_army(&mut campaign, &data, Actor::Player, ArmyId(1), &[1, 5, 6]);
    move_army(&mut campaign, &data, Actor::Player, split, &[1, 5, 6, 8]);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(6));
    assert_eq!(campaign.armies[&split].site, SiteId(8));
    assert_eq!(campaign.armies[&split].commander, Some(PersonId(1)));
    for site in [SiteId(5), SiteId(6)] {
        assert_eq!(
            campaign.world.site(site).unwrap().controller,
            Some(FactionId(1)),
            "Rose must capture the authored route through {site:?}"
        );
    }
    finish_npc_turns(&mut campaign, &data);

    let advance = move_army(&mut campaign, &data, Actor::Player, split, &[8, 9]);
    assert!(advance.battle.is_none());
    assert_eq!(campaign.armies[&split].site, SiteId(9));
    assert_eq!(
        campaign.world.site(SiteId(9)).unwrap().controller,
        Some(FactionId(1))
    );

    // Hawthorn legally splits Mara's force and counterattacks Milltown by road.
    finish_npc_turns_with_approach(&mut campaign, &data);
    let first_battle = finish_npc_turns_with_counterattack(&mut campaign, &data);
    let first = campaign.battles[&first_battle].clone();
    assert_eq!(first.site, SiteId(6));
    assert_eq!(first.attacker.faction, FactionId(3));
    assert_eq!(first.attacker.armies[0].id, ArmyId(6));
    assert!(first.attacker.armies[0]
        .people
        .iter()
        .any(|person| person.id == PersonId(3)));
    let retreat = first.attacker.armies[0]
        .final_site
        .expect("the defeated counterattacker must retreat legally");
    assert_ne!(retreat, SiteId(6));
    assert!(campaign.people[&PersonId(3)].is_alive());
    assert_eq!(campaign.armies[&ArmyId(6)].site, retreat);
    assert_eq!(campaign.completed_rounds, 3);
    campaign.validate(&data).unwrap();

    let mut resumed = reload(&campaign, &data);
    assert_eq!(resumed, campaign);
    assert_eq!(battle_history_count(&campaign, first_battle), 1);
    assert_eq!(battle_history_count(&resumed, first_battle), 1);
    assert_battle_is_visible_through_each_subject(
        &campaign,
        first_battle,
        [
            HistorySubject::Army(ArmyId(6)),
            HistorySubject::Person(PersonId(3)),
            HistorySubject::Site(SiteId(6)),
        ],
    );
    assert_battle_is_visible_through_each_subject(
        &resumed,
        first_battle,
        [
            HistorySubject::Army(ArmyId(6)),
            HistorySubject::Person(PersonId(3)),
            HistorySubject::Site(SiteId(6)),
        ],
    );

    let second_battle = rematch(&mut campaign, &data, retreat);
    let resumed_battle = rematch(&mut resumed, &data, retreat);
    assert_eq!(second_battle, resumed_battle);
    let second = campaign.battles[&second_battle].clone();
    assert_eq!(second.site, retreat);
    assert_eq!(second.attacker.faction, FactionId(1));
    assert!(second
        .defender
        .faction_side()
        .unwrap()
        .armies
        .iter()
        .any(|army| {
            army.id == ArmyId(6) && army.people.iter().any(|person| person.id == PersonId(3))
        }));
    assert!(campaign.people[&PersonId(3)].is_alive());
    finish_npc_turns(&mut campaign, &data);
    finish_npc_turns(&mut resumed, &data);
    assert_eq!(
        campaign, resumed,
        "reload branch consumed different receipts"
    );

    assert_eq!(campaign.battles.len(), 2);
    assert_eq!(battle_history_count(&campaign, first_battle), 1);
    assert_eq!(battle_history_count(&campaign, second_battle), 1);
    assert_eq!(battle_history_count(&resumed, first_battle), 1);
    assert_eq!(battle_history_count(&resumed, second_battle), 1);
    for subject in [
        HistorySubject::Army(ArmyId(6)),
        HistorySubject::Person(PersonId(3)),
    ] {
        for observer in [FactionId(1), FactionId(3)] {
            let page = history_page(
                &campaign,
                observer,
                &HistoryFilter {
                    subject: Some(subject),
                    kind: Some(HistoryKindFilter::Battle),
                    ..Default::default()
                },
            );
            assert_eq!(
                page.entries
                    .iter()
                    .filter(|entry| matches!(entry.kind, HistoryKind::Battle { .. }))
                    .count(),
                2,
                "each real battle appears once in the {subject:?} view"
            );
        }
    }
    assert_eq!(campaign.pending_facts.len(), 0);
    assert_eq!(campaign.consumed_sequence, campaign.accepted_sequence);
    assert_eq!(
        campaign.people[&PersonId(3)].evidence.counts[&EvidenceKind::MeaningfulEncounter],
        2,
        "the same person earned one actual encounter at each site"
    );
    campaign.validate(&data).unwrap();
}

fn move_army(
    campaign: &mut StrategicCampaign,
    data: &kestrum::data::GameData,
    actor: Actor,
    army: ArmyId,
    path: &[u32],
) -> kestrum::engine::ActionOutcome {
    apply(
        campaign,
        data,
        actor,
        Command::Move(MoveOrder {
            armies: vec![army],
            path: path.iter().copied().map(SiteId).collect(),
        }),
    )
    .unwrap()
}

fn finish_npc_turns(campaign: &mut StrategicCampaign, data: &kestrum::data::GameData) {
    let starting_round = campaign.completed_rounds;
    if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, starting_round + 1);
}

fn finish_npc_turns_with_approach(
    campaign: &mut StrategicCampaign,
    data: &kestrum::data::GameData,
) {
    let starting_round = campaign.completed_rounds;
    if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        if faction == FactionId(3) {
            let split = apply(
                campaign,
                data,
                Actor::Npc(faction),
                Command::SplitArmy {
                    formation: FormationId(7),
                },
            )
            .unwrap()
            .split_army
            .unwrap();
            assert_eq!(split, ArmyId(6));
            let advance = move_army(campaign, data, Actor::Npc(faction), split, &[3, 11, 10, 8]);
            assert!(
                advance.battle.is_none(),
                "Hawthorn's force should reach the bridge before Milltown: {advance:?}"
            );
            assert_eq!(campaign.armies[&split].site, SiteId(8));
        }
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, starting_round + 1);
}

fn finish_npc_turns_with_counterattack(
    campaign: &mut StrategicCampaign,
    data: &kestrum::data::GameData,
) -> kestrum::state::battle::BattleId {
    let starting_round = campaign.completed_rounds;
    if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    let mut battle = None;
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        if faction == FactionId(3) {
            let result = move_army(campaign, data, Actor::Npc(faction), ArmyId(6), &[8, 6]);
            battle = result.battle;
            assert!(
                battle.is_some(),
                "the NPC counterattack must meet Milltown's garrison: {result:?}, army={:?}",
                campaign.armies[&ArmyId(6)]
            );
        }
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, starting_round + 1);
    battle.expect("Hawthorn's legal counterattack should resolve one battle")
}

fn rematch(
    campaign: &mut StrategicCampaign,
    data: &kestrum::data::GameData,
    retreat: SiteId,
) -> kestrum::state::battle::BattleId {
    let path: &[u32] = match retreat {
        SiteId(8) => &[9, 8],
        SiteId(7) => &[9, 8, 6, 7],
        SiteId(5) => &[9, 8, 6, 5],
        other => panic!("unexpected legal Rosemarch retreat site {other:?}"),
    };
    let result = move_army(campaign, data, Actor::Player, ArmyId(5), path);
    let battle = result
        .battle
        .expect("the advancing Rose force must meet the same rival");
    assert_eq!(campaign.battles[&battle].site, retreat);
    battle
}

fn reload(campaign: &StrategicCampaign, data: &kestrum::data::GameData) -> StrategicCampaign {
    let encoded = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    load_legacy(&encoded, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

fn battle_history_count(
    campaign: &StrategicCampaign,
    battle: kestrum::state::battle::BattleId,
) -> usize {
    campaign
        .history
        .events
        .values()
        .filter(
            |record| matches!(record.kind, HistoryKind::Battle { battle: id, .. } if id == battle),
        )
        .count()
}

fn assert_battle_is_visible_through_each_subject<const N: usize>(
    campaign: &StrategicCampaign,
    battle: kestrum::state::battle::BattleId,
    subjects: [HistorySubject; N],
) {
    for subject in subjects {
        let page = history_page(
            campaign,
            FactionId(3),
            &HistoryFilter {
                subject: Some(subject),
                kind: Some(HistoryKindFilter::Battle),
                ..Default::default()
            },
        );
        assert_eq!(
            page.entries
                .iter()
                .filter(|entry| matches!(entry.kind, HistoryKind::Battle { battle: id, .. } if id == battle))
                .count(),
            1,
            "the same battle must appear once in the {subject:?} view"
        );
    }
}
