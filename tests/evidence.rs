//! Five seasonal evidence contracts; all awards consume actual accepted receipts.

use kestrum::{
    data::{
        economy::TroopKind,
        world::{FactionId, FounderClass, SiteId},
        GameData,
    },
    engine::{advance_npc, apply, history_page, Actor, Command, HistoryFilter, MoveOrder},
    state::{
        evidence::{EvidenceKind, Veterancy},
        history::{HistoryKind, HistorySubject},
        military::{ArmyId, FormationId},
        people::{PersonId, PersonStatus},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/evidence.rs"]
mod support;
use support::*;

#[test]
fn first_meaningful_encounter_wins_dedup_and_consumption_survives_save_reload() {
    let (data, mut campaign) = fixture();
    campaign.people.remove(&PersonId(3));
    let trivial = encounter(&mut campaign, &data, 5, 6, 1);
    assert!(trivial.attacker.armies[0].formations[0].combat_losses < 10);
    encounter(&mut campaign, &data, 5, 6, 100);
    encounter(&mut campaign, &data, 5, 6, 100);
    assert_eq!(campaign.formations[&FormationId(1)].service.xp, 0);
    let mut resumed = reload(&campaign, &data);
    finish(&mut campaign, &data);
    finish(&mut resumed, &data);
    assert_eq!(campaign, resumed);
    assert_eq!(count(&campaign, 1, EvidenceKind::Battle), 1);
    assert_eq!(count(&campaign, 1, EvidenceKind::MeaningfulEncounter), 1);
    let xp = campaign.formations[&FormationId(1)].service.xp;
    assert!((2..=4).contains(&xp));
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::MeaningfulEncounter],
        1
    );
    finish(&mut campaign, &data);
    assert_eq!(campaign.formations[&FormationId(1)].service.xp, xp);
}

#[test]
fn round_cap_tiers_and_combat_factors_follow_surviving_stable_formations() {
    let (data, mut campaign) = fixture();
    campaign.people.clear();
    for round in 0..5 {
        encounter(&mut campaign, &data, 5, 6, 100);
        encounter(&mut campaign, &data, 8, 10, 100);
        finish(&mut campaign, &data);
        assert_eq!(
            campaign.formations[&FormationId(1)].service.xp,
            (round + 1) * 4
        );
    }
    let service = &campaign.formations[&FormationId(1)].service;
    assert_eq!(service.tier, Veterancy::Veteran);
    assert_eq!(service.recent.len(), 5);
    assert_eq!(
        service.ledger.counts[&EvidenceKind::MeaningfulEncounter],
        10
    );
    assert!(campaign.history.events.values().any(|entry| matches!(
        entry.kind,
        HistoryKind::VeterancyEarned {
            tier: Veterancy::Seasoned,
            ..
        }
    )));
    assert_veteran_recovery(&mut campaign, &data);
    let mut ordinary = campaign.clone();
    ordinary
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .service = Default::default();
    let ordinary_report = encounter(&mut ordinary, &data, 5, 6, 100);
    let report = encounter(&mut campaign, &data, 5, 6, 100);
    assert_eq!(
        report.attacker.armies[0].formations[0].veterancy_permille,
        1200
    );
    let loss = |report: &kestrum::state::battle::BattleReport, id| {
        report.exchanges[0]
            .losses
            .iter()
            .find(|entry| entry.formation == id)
            .unwrap()
            .amount
    };
    assert!(loss(&report, FormationId(1)) < loss(&ordinary_report, FormationId(1)));
    let enemy = report.defender.armies()[0].formations[0].id;
    assert!(loss(&report, enemy) > loss(&ordinary_report, enemy));
    let service = campaign.formations[&FormationId(1)].service.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(1),
        },
    )
    .unwrap();
    assert_eq!(campaign.formations[&FormationId(1)].service, service);
    assert_invalid_service(&campaign, &data);
}

#[test]
fn movement_and_starting_host_evidence_survive_transfer_and_destroyed_service_stays_gone() {
    assert_starting_host();
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
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
    assert_movement_receipt(&campaign, &data);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(2),
        },
    )
    .unwrap();
    let mut campaign = reload(&campaign, &data);
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)]
            .evidence
            .traversed_routes
            .len(),
        1
    );
    assert_eq!(
        campaign.formations[&FormationId(1)]
            .service
            .ledger
            .traversed_routes
            .len(),
        1
    );
    let page = history_page(
        &campaign,
        FactionId(1),
        &HistoryFilter {
            subject: Some(HistorySubject::Person(PersonId(1))),
            ..Default::default()
        },
    );
    assert!(page
        .entries
        .iter()
        .any(|entry| entry.kind == HistoryKind::Moved));
    let (data, mut campaign) = fixture();
    campaign.people.clear();
    encounter(&mut campaign, &data, 5, 6, 100);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(1),
        },
    )
    .unwrap();
    finish(&mut campaign, &data);
    assert!(!campaign.formations.contains_key(&FormationId(1)));
    assert!(history_page(
        &campaign,
        FactionId(1),
        &HistoryFilter {
            subject: Some(HistorySubject::Formation(FormationId(1))),
            ..Default::default()
        }
    )
    .entries
    .is_empty());
}

#[test]
fn personal_treatment_requires_survival_and_real_recovery_or_friendly_casualties() {
    assert_wiped_people();
    assert_battle_treatment_fitness();
    let (data, mut campaign) = fixture();
    campaign.people.get_mut(&PersonId(1)).unwrap().class = FounderClass::Medic;
    encounter(&mut campaign, &data, 5, 6, 100);
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::TreatedWounded],
        1
    );
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign.people.get_mut(&PersonId(1)).unwrap().class = FounderClass::Medic;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 99;
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::TreatedWounded],
        1
    );
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::TreatedWounded],
        1
    );
    campaign.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 1,
    };
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 90;
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::TreatedWounded],
        1
    );
}

#[test]
fn pruning_bounds_recent_service_and_narrative_without_erasing_gameplay_evidence() {
    assert_count_budgets();
    let (data, mut campaign) = fixture();
    encounter(&mut campaign, &data, 5, 6, 100);
    finish(&mut campaign, &data);
    let earned = campaign.formations[&FormationId(1)].service.clone();
    let previous_events: Vec<_> = campaign.history.events.keys().copied().collect();
    for _ in 0..41 {
        finish(&mut campaign, &data);
    }
    let service = &campaign.formations[&FormationId(1)].service;
    assert_eq!(service.xp, earned.xp);
    assert_eq!(service.ledger, earned.ledger);
    assert!(service.recent.is_empty());
    assert!(campaign.battles.is_empty());
    assert!(previous_events
        .iter()
        .all(|id| !campaign.history.events.contains_key(id)));
    assert!(!campaign.history.person_notables[&PersonId(1)].is_empty());
    assert_eq!(reload(&campaign, &data), campaign);
    for _ in 0..40 {
        finish(&mut campaign, &data);
    }
    assert!(campaign.history.person_notables[&PersonId(1)].is_empty());
    assert_eq!(
        campaign.formations[&FormationId(1)].service.ledger,
        earned.ledger
    );
}
