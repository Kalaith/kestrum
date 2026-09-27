//! Actual orders and rounds exercise the shared history/save boundary.

use super::*;

pub(super) fn begin_stable_and_road(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(
        campaign,
        data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(1)),
            kind: ConstructionKind::Facility(Facility::Stable),
            builder: ArmyId(1),
        },
    )
    .unwrap();
    apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    apply(
        campaign,
        data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Route(RouteId(13)),
            kind: ConstructionKind::Road,
            builder: ArmyId(1),
        },
    )
    .unwrap();
}

pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let encoded = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    kestrum::state::persistence::load_legacy(&encoded, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

pub(super) fn records(
    campaign: &StrategicCampaign,
    observer: FactionId,
    site: SiteId,
) -> Vec<kestrum::state::history::HistoryRecord> {
    history_page(
        campaign,
        observer,
        &HistoryFilter {
            subject: Some(HistorySubject::Site(site)),
            kind: Some(HistoryKindFilter::Construction),
            ..Default::default()
        },
    )
    .entries
}
