//! Political compatibility, pending decisions and immutable historical privacy.
use kestrum::{
    data::{world::FactionId, GameData},
    engine::{apply, history_page, Actor, Command, HistoryFilter},
    state::{
        history::{HistoryKind, HistoryKindFilter},
        Campaign, CampaignPhase, GameState, Overlay, StrategicCampaign,
    },
};
use serde_json::json;

#[test]
fn earlier_payloads_preserve_relations_without_inventing_truces_or_ai_history() {
    let data = GameData::load().unwrap();
    let original = StrategicCampaign::new(&data).unwrap();
    let modern = serde_json::to_value(&original).unwrap();
    let mut old = modern.clone();
    old.as_object_mut().unwrap().remove("diplomacy");
    old.as_object_mut().unwrap().remove("ai");
    let decoded: Campaign = serde_json::from_value(old.clone()).unwrap();
    decoded.validate(&data).unwrap();
    assert_eq!(decoded.strategic().unwrap(), &original);
    for field in ["diplomacy", "ai"] {
        let mut partial = modern.clone();
        partial.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Campaign>(partial).is_err());
    }
    old["diplomacy"] = json!(null);
    old["ai"] = json!({"factions":{}});
    assert!(serde_json::from_value::<Campaign>(old).is_err());
}

#[test]
fn forged_timers_budgets_and_endings_cannot_replace_a_live_campaign() {
    let data = GameData::load().unwrap();
    let mut state = GameState::default();
    state.new_game(&data).unwrap();
    let before = serde_json::to_value(state.campaign.as_ref().unwrap()).unwrap();
    for (path, value) in [
        ("/diplomacy/pairs/0/truce_until", json!(4)),
        (
            "/diplomacy/ending",
            json!({"kind":"victory","completed_rounds":0}),
        ),
        (
            "/ai/factions",
            json!({"2":{"phase_round":0,"accepted_commands":65,"objective":null,"rejected_at_sequence":0,"rejected":[]}}),
        ),
        ("/factions/2/status", json!("eliminated")),
    ] {
        let mut invalid = before.clone();
        *invalid.pointer_mut(path).unwrap() = value;
        let candidate: Campaign = serde_json::from_value(invalid).unwrap();
        assert!(state.load_campaign(candidate, &data).is_err(), "{path}");
        assert_eq!(
            serde_json::to_value(state.campaign.as_ref().unwrap()).unwrap(),
            before
        );
    }
}

#[test]
fn incoming_offer_freezes_orders_and_resumes_the_same_npc_after_reload() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::EndTurn,
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        Command::OfferPeace {
            faction: FactionId(1),
        },
    )
    .unwrap();
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        Command::EndTurn
    )
    .is_err());
    assert_eq!(campaign, before);
    let restored: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    let mut state = GameState::default();
    state.load_campaign(restored, &data).unwrap();
    state.overlay = Overlay::Kingdom;
    state
        .command(
            &data,
            Command::RespondPeace {
                proposer: FactionId(3),
                accept: false,
            },
        )
        .unwrap();
    let current = state.campaign.as_ref().unwrap().strategic().unwrap();
    assert!(current.diplomacy.pending_offers.is_empty());
    assert_eq!(current.completed_rounds, 0);
    assert_eq!(
        current.phase,
        CampaignPhase::NpcTurn {
            faction: FactionId(3),
            paused: false
        }
    );
    assert_eq!(current.accepted_sequence, before.accepted_sequence + 1);
}

#[test]
fn public_war_and_private_offers_keep_distinct_factual_history() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap();
    let before = campaign.clone();
    let filter = HistoryFilter {
        kind: Some(HistoryKindFilter::Diplomacy),
        ..Default::default()
    };
    let own = history_page(&campaign, FactionId(1), &filter);
    let third = history_page(&campaign, FactionId(3), &filter);
    assert_eq!(own.entries.len(), 3);
    assert_eq!(third.entries.len(), 1);
    for event in own.entries {
        assert!(event.armies.is_empty() && event.people.is_empty() && event.formations.is_empty());
        assert!(matches!(event.kind, HistoryKind::Diplomacy { .. }));
    }
    assert_eq!(campaign, before);
}
