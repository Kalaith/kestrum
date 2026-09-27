//! Cross-system persistence, privacy and earned service for living places and threats.

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
        battle::{BattleDefender, BattleOutcome},
        development::DevelopmentReceipt,
        evidence::{EncounterOpponent, EvidenceKind},
        history::{HistoryKind, HistoryKindFilter},
        military::{ArmyId, FormationId},
        people::PersonId,
        threat::{ThreatId, ThreatStatus},
        Campaign, CampaignPhase, GameState, StrategicCampaign,
    },
};
use serde_json::Value;

#[path = "support/development_integrity.rs"]
mod support;
use support::*;

#[test]
fn old_campaigns_keep_actual_battles_without_retroactive_development_or_threats() {
    let (data, campaign) = field_campaign();
    let original_battles = campaign.battles.clone();
    let original_people = campaign.people.clone();
    let mut old = value(&campaign);
    strip_k11(&mut old);
    let restored = catalogue_load(&data, &old);
    let restored = restored.strategic().unwrap();
    assert_eq!(restored.battles, original_battles);
    assert_eq!(restored.people, original_people);
    assert_eq!(restored.history, campaign.history);
    assert_eq!(restored.rng, campaign.rng);
    assert_eq!(restored.world.population, campaign.world.population);
    assert!(restored.threats.is_empty());
    assert_eq!(restored.next_ids.threat, ThreatId(1));
    assert!(restored
        .world
        .development
        .values()
        .all(|state| *state == Default::default()));
    assert!(restored
        .factions
        .values()
        .all(|faction| faction.last_hq_relocation.is_none()));
    restored.validate(&data).unwrap();
}

#[test]
fn partial_or_invalid_conditions_cannot_replace_a_live_campaign() {
    let (data, campaign) = field_campaign();
    let modern = value(&campaign);
    for path in [
        "/threats",
        "/next_ids/threat",
        "/world/development",
        "/factions/1/last_hq_relocation",
    ] {
        let mut partial = modern.clone();
        let (parent, field) = path.rsplit_once('/').unwrap();
        partial
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            serde_json::from_value::<Campaign>(partial).is_err(),
            "{path}"
        );
    }
    let battle = campaign.battles.keys().next().unwrap().0.to_string();
    let mut partial = modern;
    partial["battles"][&battle]["exchanges"][0]
        .as_object_mut()
        .unwrap()
        .remove("threat_losses");
    assert!(serde_json::from_value::<Campaign>(partial).is_err());
    let mut live = GameState::default();
    live.load_campaign(Campaign::Strategic(Box::new(campaign.clone())), &data)
        .unwrap();
    for corruption in ["pressure", "displaced", "future", "threat"] {
        let mut invalid = campaign.clone();
        match corruption {
            "pressure" => {
                invalid
                    .world
                    .development
                    .get_mut(&SiteId(1))
                    .unwrap()
                    .pressure = 25
            }
            "displaced" => {
                invalid
                    .world
                    .development
                    .get_mut(&SiteId(1))
                    .unwrap()
                    .displaced = u32::MAX
            }
            "future" => {
                invalid
                    .factions
                    .get_mut(&FactionId(1))
                    .unwrap()
                    .last_hq_relocation = Some(1)
            }
            "threat" => invalid.next_ids.threat = ThreatId(1),
            _ => unreachable!(),
        }
        assert!(
            live.load_campaign(Campaign::Strategic(Box::new(invalid)), &data)
                .is_err(),
            "{corruption}"
        );
        assert_eq!(
            live.campaign.as_ref().unwrap().strategic().unwrap(),
            &campaign
        );
    }
}

#[test]
fn renamed_places_keep_dated_labels_and_population_transfers_stay_private() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for site in [5, 6] {
        campaign
            .set_site_control(&data, SiteId(site), Some(FactionId(1)), false)
            .unwrap();
    }
    campaign
        .world
        .development
        .get_mut(&SiteId(1))
        .unwrap()
        .displaced = 60;
    let population = campaign
        .world
        .population
        .values()
        .copied()
        .map(u64::from)
        .sum::<u64>();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RenameSite {
            site: SiteId(1),
            name: "Firsthaven".into(),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Resettle {
            from: SiteId(1),
            to: SiteId(6),
        },
    )
    .unwrap();
    let earlier = campaign.history.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RenameSite {
            site: SiteId(1),
            name: "Newhaven".into(),
        },
    )
    .unwrap();
    assert_eq!(
        population,
        campaign
            .world
            .population
            .values()
            .copied()
            .map(u64::from)
            .sum::<u64>()
    );
    for (id, record) in &earlier.events {
        assert_eq!(&campaign.history.events[id], record);
    }
    assert_development_history(&data, &campaign);
    assert_eq!(reload(&data, &campaign), campaign);
}

#[test]
fn actual_threat_combat_earns_service_without_inventing_enemy_people_or_troops() {
    let (data, mut campaign) = threat_campaign();
    let id = campaign.active_threat(SiteId(13)).unwrap().id;
    let before = campaign.clone();
    let command = Command::ClearThreat {
        armies: vec![ArmyId(1)],
        threat: id,
    };
    kestrum::engine::preview(&campaign, &data, Actor::Player, command.clone()).unwrap();
    assert_eq!(campaign, before);
    let outcome = apply(&mut campaign, &data, Actor::Player, command.clone()).unwrap();
    let report = campaign.battles[&outcome.battle.unwrap()].clone();
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    let BattleDefender::Threat(defender) = &report.defender else {
        panic!("typed local opponent")
    };
    assert_eq!(defender.id, id);
    assert!(report.defender.armies().is_empty());
    assert_eq!(campaign.factions.len(), before.factions.len());
    assert_eq!(campaign.next_ids.army, before.next_ids.army);
    assert_eq!(campaign.next_ids.formation, before.next_ids.formation);
    assert!(campaign.knowledge.observers.is_empty());
    round(&mut campaign, &data);
    let service = &campaign.formations[&FormationId(1)].service;
    assert!(service.xp > 0);
    assert_eq!(service.ledger.counts[&EvidenceKind::EncounteredBandits], 1);
    assert_eq!(service.ledger.counts[&EvidenceKind::ClearedThreat], 1);
    assert!(service.ledger.encountered_troops.is_empty());
    assert_eq!(
        service.recent[0].encounters[0].opponent,
        EncounterOpponent::Threat { threat: id }
    );
    let mut restored = reload(&data, &campaign);
    let frozen = restored.clone();
    assert!(apply(&mut restored, &data, Actor::Player, command).is_err());
    assert_eq!(restored, frozen);
    assert_eq!(
        restored.people[&PersonId(1)].evidence.counts[&EvidenceKind::EncounteredBandits],
        1
    );
}

#[test]
fn forgetting_threat_reports_cannot_restore_the_occupant_or_repeat_its_reward() {
    let (mut data, mut campaign) = threat_campaign();
    data.history.detail_max_entries = 1;
    let id = campaign.active_threat(SiteId(13)).unwrap().id;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::ClearThreat {
            armies: vec![ArmyId(1)],
            threat: id,
        },
    )
    .unwrap();
    let terminal = campaign.threats[&id].clone();
    assert!(matches!(terminal.status, ThreatStatus::Cleared { .. }));
    let population = campaign.world.population[&SiteId(13)];
    for _ in 0..41 {
        round(&mut campaign, &data);
    }
    assert!(campaign.battles.is_empty());
    assert_eq!(campaign.threats[&id], terminal);
    assert_eq!(campaign.world.population[&SiteId(13)], population);
    assert!(campaign.site_is_ruined(SiteId(13)));
    let restored = reload(&data, &campaign);
    assert_eq!(restored, campaign);
    assert!(restored.active_threat(SiteId(13)).is_none());
}
