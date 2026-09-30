//! P15 local combat uses real forces, fixed geography and single-use rewards.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/threats.rs"]
mod support;
use kestrum::{
    data::{
        economy::{Habitation, Resources},
        threats::ThreatKind,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{apply, project, threat_preview, Actor, Command, MoveOrder},
    state::{
        battle::{BattleDefender, BattleOutcome},
        construction::{ConstructionKind, ConstructionTarget},
        military::{ArmyId, FormationId},
        people::{PersonId, PersonStatus},
        threat::{ThreatId, ThreatStatus},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use support::*;

#[test]
fn authored_ordinary_encounter_uses_shared_exact_casualties_and_real_identity() {
    let (data, mut campaign) = fixture(false);
    assert_eq!(campaign.factions.len(), 4);
    assert_eq!(campaign.threats.len(), 2);
    let before = campaign.clone();
    let preview =
        threat_preview(&campaign, &data, FactionId(1), &[ArmyId(1)], ThreatId(1)).unwrap();
    assert_eq!(preview.cost, 3);
    assert_eq!(campaign, before, "preview is pure, including RNG");
    let mut repeated = campaign.clone();
    let report = clear(&mut campaign, &data, 1);
    clear(&mut repeated, &data, 1);
    assert_eq!(
        campaign, repeated,
        "same state and command yield the same complete campaign"
    );
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert_eq!(report.exchanges[0].threat_losses, 5);
    assert_eq!(report.exchanges[0].losses[0].amount, 4);
    assert_eq!(report.exchanges[0].leadership.len(), 1);
    assert_eq!(report.exchanges[0].leadership[0].permille, 500);
    let BattleDefender::Threat(threat) = report.defender else {
        panic!("typed local defender")
    };
    assert_eq!(threat.kind, ThreatKind::Bandits);
    assert_eq!(
        threat.payout,
        Resources {
            gold: 30,
            wood: 10,
            stone: 0
        }
    );
    assert_eq!(
        campaign.factions[&FactionId(1)].resources.gold,
        before.factions[&FactionId(1)].resources.gold + 30
    );
    assert_eq!(campaign.next_ids.army, before.next_ids.army);
    assert_eq!(campaign.next_ids.formation, before.next_ids.formation);
    assert_eq!(campaign.factions.len(), 4);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(13));
    assert!(campaign.formations[&FormationId(1)].movement_spent > 0);
    assert!(campaign.knowledge.observers.is_empty());
    assert_invalid_content(&data);
}

#[test]
fn defeat_and_last_strike_resolve_real_losses_and_rewards_once() {
    for death in [0, 100] {
        let (mut data, mut campaign) = fixture(true);
        data.combat.wipe_death_percent = death;
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = 1;
        let resources = campaign.factions[&FactionId(1)].resources;
        let report = clear(&mut campaign, &data, 1);
        assert_eq!(report.outcome, BattleOutcome::DefenderVictory);
        assert!(!campaign.armies.contains_key(&ArmyId(1)));
        assert_eq!(campaign.threats[&ThreatId(1)].headcount, 59);
        assert_eq!(campaign.threats[&ThreatId(1)].status, ThreatStatus::Active);
        assert_eq!(campaign.factions[&FactionId(1)].resources, resources);
        assert_eq!(report.person_events.len(), 1);
        assert_eq!(
            matches!(
                campaign.people[&PersonId(1)].status,
                PersonStatus::Dead { .. }
            ),
            death == 100
        );
        if death == 0 {
            assert!(matches!(
                campaign.people[&PersonId(1)].status,
                PersonStatus::Wounded {
                    remaining_steps: 2,
                    ..
                }
            ));
        }
        assert_eq!(
            reload(&data, &campaign),
            Campaign::Strategic(Box::new(campaign))
        );
    }
    let (data, mut last_strike) = fixture(false);
    last_strike
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 1;
    last_strike.threats.get_mut(&ThreatId(1)).unwrap().headcount = 1;
    let balance = last_strike.factions[&FactionId(1)].resources;
    let report = clear(&mut last_strike, &data, 1);
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert!(last_strike.active_threat(SiteId(13)).is_none());
    let BattleDefender::Threat(threat) = report.defender else {
        unreachable!()
    };
    assert_eq!(threat.payout, data.threats.definitions[&threat.kind].reward);
    assert_eq!(
        last_strike.factions[&FactionId(1)].resources,
        Resources {
            gold: balance.gold + threat.payout.gold,
            wood: balance.wood + threat.payout.wood,
            stone: balance.stone + threat.payout.stone,
        }
    );
    last_strike.validate(&data).unwrap();
}

#[test]
fn clearance_catalogue_reload_and_rejected_replay_never_duplicate_reward() {
    let (data, mut campaign) = fixture(false);
    let mut overflow = campaign.clone();
    overflow
        .factions
        .get_mut(&FactionId(1))
        .unwrap()
        .resources
        .gold = i64::MAX;
    let before_overflow = overflow.clone();
    assert!(apply(&mut overflow, &data, Actor::Player, command(1)).is_err());
    assert_eq!(
        overflow, before_overflow,
        "reward overflow preserves the entire encounter and RNG"
    );
    clear(&mut campaign, &data, 1);
    let loaded = reload(&data, &campaign);
    let mut loaded = loaded.strategic().unwrap().clone();
    let before = loaded.clone();
    assert!(apply(&mut loaded, &data, Actor::Player, command(1)).is_err());
    assert_eq!(loaded, before);
    let mut resurrected = loaded.clone();
    let threat = resurrected.threats.get_mut(&ThreatId(1)).unwrap();
    threat.status = ThreatStatus::Active;
    threat.headcount = 60;
    resurrected.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    assert!(
        resurrected.validate(&data).is_err(),
        "a retained clearance receipt cannot become a paying active threat again"
    );
    assert_eq!(project(&loaded, FactionId(3)).unwrap().battles.len(), 0);
    assert_eq!(project(&loaded, FactionId(1)).unwrap().battles.len(), 1);
    for kind in ["id", "loss", "payout", "context", "faction_loss"] {
        let mut bad = loaded.clone();
        let report = bad.battles.values_mut().next().unwrap();
        let BattleDefender::Threat(threat) = &mut report.defender else {
            unreachable!()
        };
        match kind {
            "id" => threat.id = bad.next_ids.threat,
            "loss" => threat.combat_losses += 1,
            "payout" => threat.payout.gold += 1,
            "context" => {
                report.context = kestrum::state::battle::BattleContext::Assault {
                    siege: kestrum::state::siege::SiegeId(1),
                }
            }
            "faction_loss" => report.exchanges[0].threat_losses = 0,
            _ => unreachable!(),
        }
        assert!(bad.validate(&data).is_err(), "{kind}");
    }
}

#[test]
fn explicit_clearance_unlocks_reclamation_without_inventing_land_or_population() {
    let (data, mut campaign) = fixture(false);
    for site in [5, 6, 8, 10, 12] {
        campaign
            .set_site_control(&data, SiteId(site), Some(FactionId(1)), false)
            .unwrap();
    }
    campaign
        .set_site_control(&data, SiteId(13), Some(FactionId(1)), false)
        .unwrap();
    assert!(campaign
        .world
        .supplied_sites(FactionId(1), SiteId(1))
        .contains(&SiteId(13)));
    assert!(
        !campaign.supplied_sites(FactionId(1)).contains(&SiteId(13)),
        "nominally controlled threat cannot relay supply"
    );
    let development =
        kestrum::engine::development_view(&campaign, &data, FactionId(1), SiteId(13)).unwrap();
    assert!(development.safe != Some(true) && !development.supplied);
    let fixed = campaign
        .world
        .sites
        .iter()
        .map(|site| site.id)
        .collect::<Vec<_>>();
    let population = campaign.world.population.clone();
    let contact = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(12), SiteId(13)],
        }),
    )
    .unwrap();
    assert!(contact.battle_pending);
    assert!(contact.battle.is_none());
    assert_eq!(contact.movement.unwrap().path, [SiteId(12), SiteId(13)]);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(13));
    let started = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    assert!(started.battle.is_some());
    assert!(campaign.pending_battle.is_none());
    assert_eq!(campaign.world.population, population);
    assert_eq!(
        campaign
            .world
            .sites
            .iter()
            .map(|site| site.id)
            .collect::<Vec<_>>(),
        fixed
    );
    assert!(campaign.site_is_ruined(SiteId(13)));
    assert_eq!(
        campaign.world.site(SiteId(13)).unwrap().habitation,
        Habitation::Unsettled
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(13)),
            kind: ConstructionKind::Outpost,
            builder: ArmyId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign
            .world
            .sites
            .iter()
            .map(|site| site.id)
            .collect::<Vec<_>>(),
        fixed
    );
    assert_privacy_and_blockers(&data);
}

#[test]
fn lawless_ruination_spawns_once_and_trivial_clearance_cannot_farm_experience() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(7))
        .unwrap()
        .habitation = Habitation::Unsettled;
    campaign.world.population.insert(SiteId(7), 0);
    let state = campaign.world.development.get_mut(&SiteId(7)).unwrap();
    state.ruined = true;
    state.ruination = 1;
    state.ruined_round = Some(0);
    campaign.validate(&data).unwrap();
    for _ in 0..data.threats.lawless_rounds - 1 {
        finish(&mut campaign, &data);
        assert!(campaign.active_threat(SiteId(7)).is_none());
    }
    finish(&mut campaign, &data);
    let spawned = campaign.active_threat(SiteId(7)).unwrap().id;
    assert_eq!(campaign.threats[&spawned].ruination, Some(1));
    assert!(campaign.world.development[&SiteId(7)].threat_created);
    campaign.threats.get_mut(&spawned).unwrap().headcount = 1;
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
    let report = clear(&mut campaign, &data, spawned.0);
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    finish(&mut campaign, &data);
    assert!(campaign
        .formations
        .values()
        .filter(|formation| formation.faction == FactionId(1))
        .all(|formation| formation.service.xp == 0));
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(1);
    for _ in 0..data.threats.lawless_rounds + 1 {
        finish(&mut campaign, &data);
    }
    assert!(campaign.active_threat(SiteId(7)).is_none());
    assert_eq!(
        campaign
            .threats
            .values()
            .filter(|threat| threat.site == SiteId(7))
            .count(),
        1
    );
    let state = campaign.world.development.get_mut(&SiteId(7)).unwrap();
    state.ruination = 2;
    state.ruined_round = Some(campaign.completed_rounds);
    state.threat_created = false;
    state.lawless_rounds = 0;
    for _ in 0..data.threats.lawless_rounds {
        finish(&mut campaign, &data);
    }
    let next = campaign.active_threat(SiteId(7)).unwrap();
    assert!(next.id > spawned);
    assert_eq!(next.ruination, Some(2));
    assert!(
        !campaign.threats.contains_key(&spawned),
        "older cleared spawned records are bounded"
    );
    assert!(
        !campaign.battles.is_empty(),
        "retained reports use weak threat references"
    );
    assert_eq!(
        reload(&data, &campaign),
        Campaign::Strategic(Box::new(campaign))
    );
}
