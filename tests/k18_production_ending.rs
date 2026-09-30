//! K18 production terminal path through ordinary orders and actual NPC turns.

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, GameData},
    engine::{self, apply, Actor, Command},
    state::{
        diplomacy::EndingKind, persistence::load_legacy, Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

const SEED: u64 = 1_809_281;
const ROUND_CAP: u32 = 80;

#[test]
fn four_faction_production_campaign_reaches_and_keeps_a_real_defeat_ending() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new_production(
        &data,
        &ProductionSetup {
            kingdom_name: "Briarhold".into(),
            emblem: Emblem::Rose,
            factions: 4,
            seed: SEED,
        },
    )
    .unwrap();
    let player = campaign.player;
    let rivals: Vec<_> = campaign
        .factions
        .keys()
        .copied()
        .filter(|faction| *faction != player)
        .collect();
    assert_eq!(campaign.factions.len(), 4);
    assert_eq!(campaign.world.markers.len(), 80);
    assert_eq!(campaign.world.sites.len(), 152);

    // Put the ordinary AI policy at war with the unarmed player. The player
    // gives up its starting formations through the same accepted commands used
    // in play; ownership and terminal state remain engine-owned throughout.
    for rival in rivals {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::DeclareWar { faction: rival },
        )
        .unwrap();
    }
    let starting_formations: Vec<_> = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == player)
        .map(|formation| formation.id)
        .collect();
    assert_eq!(starting_formations.len(), 3);
    for formation in starting_formations {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::Disband { formation },
        )
        .unwrap();
    }
    assert!(campaign
        .armies
        .values()
        .all(|army| army.faction != player || army.is_empty()));

    let mut npc_actions = 0_u64;
    let mut completed_boundaries = 0_u32;
    while campaign.diplomacy.ending.is_none() && campaign.completed_rounds < ROUND_CAP {
        if let Some(offer) = campaign
            .diplomacy
            .pending_offers
            .iter()
            .find(|offer| offer.recipient == player)
            .cloned()
        {
            apply(
                &mut campaign,
                &data,
                Actor::Player,
                Command::RespondPeace {
                    proposer: offer.proposer,
                    accept: false,
                },
            )
            .unwrap();
            continue;
        }

        match campaign.phase {
            CampaignPhase::PlayerTurn => {
                apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
            }
            CampaignPhase::NpcTurn { .. } => {
                let before_round = campaign.completed_rounds;
                let before_sequence = campaign.accepted_sequence;
                let action = engine::advance_npc(&mut campaign, &data).unwrap();
                npc_actions += 1;
                if action.battle_pending {
                    apply(
                        &mut campaign,
                        &data,
                        Actor::Player,
                        Command::StartPendingBattle,
                    )
                    .unwrap();
                }
                assert_eq!(
                    campaign.accepted_sequence,
                    if action.battle_pending {
                        action.accepted_sequence + 1
                    } else {
                        action.accepted_sequence
                    },
                    "NPC action began at {before_sequence}, accepted through {}",
                    action.accepted_sequence
                );
                if campaign.completed_rounds > before_round {
                    completed_boundaries += campaign.completed_rounds - before_round;
                }
            }
        }
    }

    let ending = campaign
        .diplomacy
        .ending
        .as_ref()
        .expect("production campaign did not end within the 80-round cap");
    assert_eq!(ending.kind, EndingKind::Defeat);
    assert!(ending.completed_rounds <= ROUND_CAP);
    assert!(npc_actions > 0, "the NPC policy never advanced");
    assert!(
        completed_boundaries > 0,
        "no actual NPC phase reached a round boundary"
    );
    assert!(campaign.defeat_eligible(player));
    assert!(campaign.diplomacy.last_attackers.contains_key(&player));
    assert!(campaign
        .world
        .sites
        .iter()
        .all(|site| site.controller != Some(player)));
    assert!(campaign
        .armies
        .values()
        .all(|army| army.faction != player || army.is_empty()));
    campaign.validate(&data).unwrap();

    let encoded = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    let mut restored = load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(restored, campaign, "terminal save changed the ending state");

    let terminal = restored.clone();
    assert!(apply(&mut restored, &data, Actor::Player, Command::EndTurn).is_err());
    assert!(engine::advance_npc(&mut restored, &data).is_err());
    assert_eq!(
        restored, terminal,
        "terminal state replayed campaign effects"
    );

    println!(
        "K18_PRODUCTION_ENDING seed={SEED} rounds={} boundaries={completed_boundaries} npc_actions={npc_actions} captor={:?} save_bytes={}",
        ending.completed_rounds,
        campaign.diplomacy.last_attackers.get(&player),
        encoded.len(),
    );
}
