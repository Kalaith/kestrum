//! Controlled rosters exercise real commands and receipts without graphics or disk I/O.

use super::*;
use kestrum::state::{
    battle::BattleReport,
    military::{Army, Formation},
};

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign
        .people
        .retain(|id, _| [PersonId(1), PersonId(3)].contains(id));
    for (id, formation) in [(1, 1), (3, 7)] {
        let army = campaign.armies.get_mut(&ArmyId(id)).unwrap();
        army.commander = None;
        army.slots = [Some(FormationId(formation)), None, None, None, None, None];
    }
    campaign.validate(&data).unwrap();
    (data, campaign)
}

// Reset positions/headcounts as the next independent encounter's test input.
// The production resolver emits every report; no XP, tags, or narrative is injected.
pub(super) fn encounter(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    origin: u32,
    site: u32,
    enemy: u32,
) -> BattleReport {
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(origin);
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.headcount = formation.capacity;
    formation.movement_spent = 0;
    for person in campaign.people.values_mut() {
        person.movement_spent = 0;
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == FactionId(3))
        .map(|army| army.id);
    let (army, formation) = if let Some(id) = army {
        let army = campaign.armies.get_mut(&id).unwrap();
        army.site = SiteId(site);
        (id, army.formation_ids().next().unwrap())
    } else {
        let army = campaign.next_ids.army;
        let formation = campaign.next_ids.formation;
        campaign.next_ids.army.0 += 1;
        campaign.next_ids.formation.0 += 1;
        campaign.armies.insert(
            army,
            Army {
                id: army,
                faction: FactionId(3),
                name: "Fresh opposing host".into(),
                site: SiteId(site),
                slots: [Some(formation), None, None, None, None, None],
                commander: None,
            },
        );
        campaign.formations.insert(
            formation,
            Formation {
                id: formation,
                faction: FactionId(3),
                kind: TroopKind::Warriors,
                headcount: enemy,
                capacity: 100,
                movement_spent: 0,
                created_round: campaign.completed_rounds,
                service: Default::default(),
            },
        );
        (army, formation)
    };
    campaign.armies.get_mut(&army).unwrap().site = SiteId(site);
    let member = campaign.formations.get_mut(&formation).unwrap();
    member.headcount = enemy;
    member.movement_spent = 0;
    campaign
        .set_site_control(data, SiteId(site), Some(FactionId(3)), false)
        .unwrap();
    let result = apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(origin), SiteId(site)],
        }),
    )
    .unwrap();
    campaign.battles[&result.battle.unwrap()].clone()
}

pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn count(campaign: &StrategicCampaign, id: u32, kind: EvidenceKind) -> u32 {
    campaign.formations[&FormationId(id)]
        .service
        .ledger
        .counts
        .get(&kind)
        .copied()
        .unwrap_or(0)
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

pub(super) fn assert_invalid_service(campaign: &StrategicCampaign, data: &GameData) {
    for field in ["tier", "routes", "xp", "partial"] {
        let mut value =
            serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
        match field {
            "tier" => value["formations"]["1"]["service"]["tier"] = serde_json::json!("ordinary"),
            "routes" => {
                value["formations"]["1"]["service"]["ledger"]["traversed_routes"] =
                    serde_json::json!([999])
            }
            "xp" => value["formations"]["1"]["service"]["recent"][0]["xp"] = serde_json::json!(99),
            "partial" => {
                value.as_object_mut().unwrap().remove("history");
            }
            _ => unreachable!(),
        }
        let decoded = serde_json::from_value::<Campaign>(value);
        assert!(
            decoded.is_err() || decoded.unwrap().validate(data).is_err(),
            "accepted {field}"
        );
    }
}

pub(super) fn assert_starting_host() {
    let (data, mut campaign) = fixture();
    let id = campaign.next_ids.formation;
    campaign.next_ids.formation.0 += 1;
    let mut support = campaign.formations[&FormationId(1)].clone();
    support.id = id;
    support.kind = TroopKind::Spearmen;
    campaign.formations.insert(id, support);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots[1] = Some(id);
    encounter(&mut campaign, &data, 5, 6, 100);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: id,
        },
    )
    .unwrap();
    let mut campaign = reload(&campaign, &data);
    finish(&mut campaign, &data);
    let ledger = &campaign.people[&PersonId(1)].evidence;
    assert_eq!(ledger.service_by_troop.get(&TroopKind::Warriors), Some(&1));
    assert!(!ledger.service_by_troop.contains_key(&TroopKind::Spearmen));
}

pub(super) fn assert_wiped_people() {
    for (seed, alive) in [(2, true), (2, false)] {
        let (data, mut campaign) = fixture();
        campaign.people.get_mut(&PersonId(1)).unwrap().class = FounderClass::Medic;
        campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
        campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(6);
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = 1;
        campaign.rng.combat = macroquad_toolkit::rng::SeededRng::new(seed);
        if !alive {
            campaign.rng.combat.below(100);
        }
        campaign
            .set_site_control(&data, SiteId(5), Some(FactionId(1)), false)
            .unwrap();
        campaign
            .set_site_control(&data, SiteId(7), Some(FactionId(1)), false)
            .unwrap();
        campaign
            .set_site_control(&data, SiteId(6), Some(FactionId(3)), false)
            .unwrap();
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
        assert_eq!(
            campaign.people[&PersonId(1)].is_alive(),
            alive,
            "seed {seed}"
        );
        finish(&mut campaign, &data);
        let ledger = &campaign.people[&PersonId(1)].evidence;
        assert_eq!(
            ledger
                .counts
                .get(&EvidenceKind::Retreated)
                .copied()
                .unwrap_or(0),
            u32::from(alive)
        );
        assert_eq!(
            ledger
                .counts
                .get(&EvidenceKind::TreatedWounded)
                .copied()
                .unwrap_or(0),
            u32::from(alive)
        );
        assert!(!campaign.formations.contains_key(&FormationId(1)));
    }
}

pub(super) fn assert_count_budgets() {
    let (mut data, mut campaign) = fixture();
    data.history.detail_max_entries = 2;
    data.history.notable_max_entries = 2;
    for _ in 0..3 {
        encounter(&mut campaign, &data, 5, 6, 100);
    }
    assert_eq!(campaign.battles.len(), 3);
    finish(&mut campaign, &data);
    assert_eq!(campaign.history.events.len(), 2);
    assert_eq!(campaign.battles.len(), 2);
    assert_eq!(campaign.history.person_notables[&PersonId(1)].len(), 2);
    assert_eq!(count(&campaign, 1, EvidenceKind::MeaningfulEncounter), 1);
}

pub(super) fn assert_battle_treatment_fitness() {
    for (supervised, wounded) in [(true, false), (false, true)] {
        let (data, mut campaign) = fixture();
        if supervised {
            let member = campaign.formations.get_mut(&FormationId(1)).unwrap();
            member.kind = TroopKind::Medics;
            member.capacity = data.economy.formations[&TroopKind::Medics].capacity;
            member.headcount = member.capacity;
        } else {
            campaign.people.get_mut(&PersonId(1)).unwrap().class = FounderClass::Medic;
        }
        if wounded {
            campaign.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
                since_round: 0,
                remaining_steps: 2,
            };
        }
        let report = encounter(
            &mut campaign,
            &data,
            5,
            6,
            if supervised { 30 } else { 100 },
        );
        assert!(report.attacker.armies[0].formations[0].combat_losses > 0);
        assert!(campaign.people[&PersonId(1)].is_alive());
        finish(&mut campaign, &data);
        assert_eq!(
            campaign.people[&PersonId(1)]
                .evidence
                .counts
                .get(&EvidenceKind::TreatedWounded)
                .copied()
                .unwrap_or(0),
            u32::from(!wounded)
        );
    }
}

pub(super) fn assert_veteran_recovery(campaign: &mut StrategicCampaign, data: &GameData) {
    let earned = campaign.formations[&FormationId(1)].service.clone();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(1);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 21;
    finish(campaign, data);
    let recovered = &campaign.formations[&FormationId(1)];
    assert!(recovered.headcount > 21);
    assert_eq!(recovered.service.xp, earned.xp);
    assert_eq!(recovered.service.tier, Veterancy::Veteran);
    assert_eq!(recovered.service.ledger, earned.ledger);
    assert!(campaign.factions[&FactionId(1)]
        .last_recovery
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .any(|entry| entry.formation == FormationId(1)
            && entry.headcount_before == 21
            && entry.restored > 0));
}

pub(super) fn assert_movement_receipt(campaign: &StrategicCampaign, data: &GameData) {
    for foreign in [false, true] {
        let mut invalid = campaign.clone();
        let kestrum::state::campaign::DomainFactKind::ArmiesMoved {
            movement: Some(receipt),
            ..
        } = &mut invalid.pending_facts[0].kind
        else {
            panic!("expected actual movement receipt")
        };
        if foreign {
            receipt.people.push(PersonId(2));
        } else {
            receipt.routes = vec![kestrum::data::world::RouteId(2)];
        }
        assert!(invalid
            .validate(data)
            .unwrap_err()
            .contains("movement evidence"));
    }
}
