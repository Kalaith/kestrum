//! Siege facts retain real context without revealing unencountered enemy rosters.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

use kestrum::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{apply, history_page, Actor, Command, HistoryFilter, MoveOrder},
    state::{
        battle::BattleContext,
        evidence::EvidenceKind,
        history::{HistoryKind, HistoryKindFilter, HistoryRecord, HistorySubject},
        military::{ArmyId, FormationId},
        people::PersonId,
        siege::{SiegeAction, SiegeChange, SiegeOrder},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/siege_history.rs"]
mod support;
use support::*;

#[test]
fn establishing_siege_records_a_known_place_without_foreign_rosters() {
    assert_authored_target();
    let (data, mut campaign) = fixture(false);
    establish(&mut campaign, &data, false);
    let rose = records(&campaign, FactionId(1));
    let hawthorn = records(&campaign, FactionId(3));
    assert_eq!(rose, hawthorn);
    assert_eq!(rose.len(), 1);
    assert!(records(&campaign, FactionId(2)).is_empty());
    let first = &rose[0];
    assert!(matches!(
        first.kind,
        HistoryKind::Siege {
            change: SiegeChange::Established,
            elapsed_steps: 0,
            ..
        }
    ));
    assert!(first.armies.is_empty() && first.people.is_empty() && first.formations.is_empty());
    let original_name = first.sites[0].name.clone();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(9))
        .unwrap()
        .name = "Renamed after the arrival".into();
    assert_eq!(
        records(&reload(&campaign, &data), FactionId(1))[0].sites[0].name,
        original_name
    );
    assert!(campaign.battles.is_empty());
    assert!(campaign.formations.values().all(|f| f.service.xp == 0));
}

#[test]
fn forgotten_siege_progress_does_not_end_the_siege_or_repair_its_walls() {
    let (mut data, mut campaign) = fixture(false);
    data.history.detail_max_entries = 1;
    establish(&mut campaign, &data, false);
    for _ in 0..41 {
        finish(&mut campaign, &data);
    }
    assert_eq!(campaign.sieges[&SiteId(9)].elapsed_steps, 41);
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 100);
    assert_eq!(campaign.history.events.len(), 1);
    let mut campaign = reload(&campaign, &data);
    assert_eq!(campaign.sieges[&SiteId(9)].elapsed_steps, 41);
    order(&mut campaign, &data, SiegeAction::Withdraw, Some(SiteId(8)));
    assert!(!campaign.sieges.contains_key(&SiteId(9)));
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 100);
    assert!(records(&campaign, FactionId(1))
        .iter()
        .any(|entry| matches!(
            entry.kind,
            HistoryKind::Siege {
                change: SiegeChange::Lifted,
                ..
            }
        )));
    let mut restored = reload(&campaign, &data);
    finish(&mut restored, &data);
    assert!(!restored.sieges.contains_key(&SiteId(9)));
    assert_eq!(restored.world.fort_damage[&SiteId(9)], 100);
}

#[test]
fn assault_evidence_requires_actual_combat_and_is_consumed_once() {
    let (mut data, mut campaign) = fixture(false);
    data.combat.max_exchanges = 1;
    establish(&mut campaign, &data, false);
    finish(&mut campaign, &data);
    assert_eq!(count(&campaign, 1, EvidenceKind::AssaultedFort), 0);
    let outcome = order(&mut campaign, &data, SiegeAction::Assault, None);
    let report = campaign.battles[&outcome.battle.unwrap()].clone();
    assert!(matches!(report.context, BattleContext::Assault { .. }));
    assert_eq!(report.attacker.armies.len(), 1);
    assert!(report.fort_damage_added > 0);
    finish(&mut campaign, &data);
    assert_eq!(count(&campaign, 1, EvidenceKind::AssaultedFort), 1);
    assert_eq!(count(&campaign, 7, EvidenceKind::DefendedFort), 1);
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::AssaultedFort],
        1
    );
    let xp = campaign.formations[&FormationId(1)].service.xp;
    let mut restored = reload(&campaign, &data);
    let _ = history_page(&restored, FactionId(1), &HistoryFilter::default());
    finish(&mut restored, &data);
    assert_eq!(count(&restored, 1, EvidenceKind::AssaultedFort), 1);
    assert_eq!(restored.formations[&FormationId(1)].service.xp, xp);
}

#[test]
fn relief_records_joint_combat_but_only_incoming_route_service() {
    let (mut data, mut campaign) = fixture(true);
    data.combat.max_exchanges = 1;
    add_relief(&mut campaign);
    establish(&mut campaign, &data, true);
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(&mut campaign, &data).unwrap();
    }
    let result = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(5)],
            path: vec![SiteId(10), SiteId(9)],
        }),
    )
    .unwrap();
    let report = campaign.battles[&result.battle.unwrap()].clone();
    assert!(
        matches!(report.context, BattleContext::Relief { ref garrison, .. }
        if *garrison == vec![ArmyId(1)])
    );
    assert_eq!(report.attacker.armies.len(), 2);
    let route = campaign
        .world
        .connected_route(SiteId(10), SiteId(9))
        .unwrap()
        .id;
    let mut restored = reload(&campaign, &data);
    finish(&mut restored, &data);
    assert!(restored.formations[&FormationId(13)]
        .service
        .ledger
        .traversed_routes
        .contains(&route));
    assert!(!restored.formations[&FormationId(1)]
        .service
        .ledger
        .traversed_routes
        .contains(&route));
    assert!(!restored.people[&PersonId(1)]
        .evidence
        .traversed_routes
        .contains(&route));
    assert_eq!(count(&restored, 1, EvidenceKind::Relief), 1);
    assert_eq!(count(&restored, 13, EvidenceKind::Relief), 1);
}

#[test]
fn siege_history_rejects_forged_observers_rosters_and_progress() {
    let (data, mut campaign) = fixture(false);
    establish(&mut campaign, &data, false);
    let id = records(&campaign, FactionId(1))[0].id;
    for corruption in 0..4 {
        let mut broken = campaign.clone();
        let record = broken.history.events.get_mut(&id).unwrap();
        match corruption {
            0 => {
                record.visible_to.insert(FactionId(2));
            }
            1 => {
                record.sites[0].id = SiteId(8);
            }
            2 => {
                if let HistoryKind::Siege { elapsed_steps, .. } = &mut record.kind {
                    *elapsed_steps = 1;
                }
            }
            _ => {
                record.armies.push(kestrum::state::history::EntityLabel {
                    id: ArmyId(3),
                    name: "Undiscovered foreign roster".into(),
                });
            }
        }
        assert!(
            broken.validate(&data).is_err(),
            "accepted corruption {corruption}"
        );
    }
    let before = campaign.clone();
    let _ = records(&campaign, FactionId(1));
    assert_eq!(campaign, before);
}
