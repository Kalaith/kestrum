//! A controlled Rosemarch places thread links settlement, safe growth and administration.
//!
//! Siege-caused fort damage/reclaim, refugee conservation and lost-HQ recovery remain
//! covered by the focused development tests. This scenario keeps its initial authored
//! changes to a peaceful relation baseline and cleared starting threats so the ordinary
//! command and boundary sequence stays deterministic without writing live site state.

use kestrum::{
    data::{
        economy::Habitation,
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{apply, development_view, Actor, Command, MoveOrder},
    state::{
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, Focus},
        history::HistoryKind,
        military::ArmyId,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

#[test]
fn rosemarch_outpost_safe_growth_and_capital_orders_survive_reload() {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    let mut campaign = StrategicCampaign::new(&data).unwrap();

    // The authored start is otherwise unchanged; ordinary movement captures the
    // unsettled West Gate and an ordinary construction order establishes its Outpost.
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(FactionId(1))
    );
    let order = campaign.next_ids.order;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(5)),
            kind: ConstructionKind::Outpost,
            builder: ArmyId(1),
        },
    )
    .unwrap();
    for _ in 0..3 {
        finish_round(&mut campaign, &data);
    }
    assert!(matches!(
        campaign.construction[&order].status,
        ConstructionStatus::Completed { .. }
    ));
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Outpost
    );
    assert_eq!(campaign.world.population[&SiteId(5)], 50);

    // Milltown is reached over the connected route and develops while safe and supplied.
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(5), SiteId(6)],
        }),
    )
    .unwrap();
    let before_growth = campaign.world.population[&SiteId(6)];
    let view = development_view(&campaign, &data, FactionId(1), SiteId(6)).unwrap();
    assert_eq!(view.safe, Some(true));
    assert!(view.supplied);
    assert!(!view.ruined);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFocus {
            site: SiteId(6),
            focus: Focus::Growth,
        },
    )
    .unwrap();

    // Renaming the capital site and moving the capital are separate real orders;
    // the faction HQ remains at Rose Headquarters.
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RenameSite {
            site: SiteId(6),
            name: "Milltown Market".into(),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::MoveCapital { site: SiteId(6) },
    )
    .unwrap();
    assert_eq!(campaign.factions[&FactionId(1)].capital, SiteId(6));
    assert_eq!(campaign.factions[&FactionId(1)].headquarters, SiteId(1));

    finish_round(&mut campaign, &data);
    assert!(campaign.world.population[&SiteId(6)] > before_growth);
    assert_eq!(campaign.world.focus[&SiteId(6)], Focus::Growth);
    assert_eq!(
        campaign.world.site(SiteId(6)).unwrap().name,
        "Milltown Market"
    );
    campaign.validate(&data).unwrap();

    let raw = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    let resumed = kestrum::state::persistence::load_legacy(&raw, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(resumed, campaign);
    assert_eq!(resumed.factions[&FactionId(1)].capital, SiteId(6));
    assert_eq!(resumed.factions[&FactionId(1)].headquarters, SiteId(1));
    assert!(resumed.history.events.values().any(|event| matches!(
        &event.kind,
        HistoryKind::Development {
            receipt: kestrum::state::development::DevelopmentReceipt::CapitalMoved {
                owner: FactionId(1),
                from: SiteId(1),
                to: SiteId(6),
            }
        }
    )));
    assert!(resumed.history.events.values().any(|event| matches!(
        &event.kind,
        HistoryKind::Development {
            receipt: kestrum::state::development::DevelopmentReceipt::SiteRenamed {
                owner: FactionId(1),
                site: SiteId(6),
                new_name,
                ..
            }
        } if new_name == "Milltown Market"
    )));
    resumed.validate(&data).unwrap();
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let start_round = campaign.completed_rounds;
    if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        // A peaceful, controlled boundary uses each faction's ordinary legal pass order.
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, start_round + 1);
}
