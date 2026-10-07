//! AI recovery follows ordinary commands to free stranded builders and supply routes.

use kestrum::{
    data::{
        economy::{Habitation, OrderKind, Resources, TroopKind},
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{ai, apply, preview, route_cost, Actor, Command, MoveOrder},
    state::{
        construction::{
            ConstructionKind, ConstructionOrder, ConstructionPause, ConstructionStatus,
            ConstructionTarget,
        },
        military::ArmyId,
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

const OAK: FactionId = FactionId(2);
const ROSE: FactionId = FactionId(1);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), OAK);
    (data, campaign)
}

fn next_oak_turn(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    for _ in 0..5 {
        if campaign.completed_rounds > round && campaign.active_faction() == OAK {
            return;
        }
        let active = campaign.active_faction();
        let actor = if active == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(active)
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
    panic!("one cycle returns to Oak");
}

#[test]
fn old_unsupplied_work_cancels_and_releases_its_builder() {
    let (data, mut campaign) = recovery_turn();
    let target = unsupplied_owned_site(&data, &mut campaign);
    let builder = campaign
        .armies
        .values()
        .find(|army| army.faction == OAK)
        .expect("Oak has a builder army")
        .id;
    campaign.armies.get_mut(&builder).unwrap().site = target;
    let order = add_supply_lost_order(&data, &mut campaign, target, builder);
    campaign.validate(&data).unwrap();

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert_eq!(decision.command, Command::CancelConstruction { order });
    assert_eq!(
        decision.intent.kind,
        kestrum::state::ai::AiIntentKind::CancelConstruction
    );

    apply(&mut campaign, &data, Actor::Npc(OAK), decision.command).unwrap();
    assert!(!campaign.construction[&order].is_open());
    assert!(campaign
        .construction
        .values()
        .all(|work| !work.is_open() || work.builder != Some(builder)));
}

#[test]
fn a_supply_lost_label_does_not_cancel_work_at_a_supplied_site() {
    let (data, mut campaign) = recovery_turn();
    let headquarters = campaign.factions[&OAK].headquarters;
    let builder = campaign
        .armies
        .values()
        .find(|army| army.faction == OAK)
        .expect("Oak has a builder army")
        .id;
    let order = add_supply_lost_order(&data, &mut campaign, headquarters, builder);
    campaign.validate(&data).unwrap();

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert_ne!(decision.command, Command::CancelConstruction { order });
    assert!(campaign.construction[&order].is_open());
    assert!(campaign.supplied_sites(OAK).contains(&headquarters));
}

#[test]
fn a_corridor_war_waits_for_truce_then_reconnects_city_supply() {
    let (data, mut campaign, middle, enclave, army) = corridor_fixture();
    let truce = campaign.completed_rounds + 1;
    campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == [ROSE, OAK])
        .unwrap()
        .truce_until = Some(truce);
    campaign.validate(&data).unwrap();

    let during_truce = ai::propose(&campaign, &data, OAK).unwrap();
    assert_ne!(during_truce.command, Command::DeclareWar { faction: ROSE });

    campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == [ROSE, OAK])
        .unwrap()
        .truce_until = None;
    let declaration = ai::propose(&campaign, &data, OAK).unwrap();
    assert_eq!(declaration.command, Command::DeclareWar { faction: ROSE });
    apply(&mut campaign, &data, Actor::Npc(OAK), declaration.command).unwrap();
    assert_eq!(
        campaign
            .relations
            .iter()
            .find(|relation| relation.factions == [ROSE, OAK])
            .unwrap()
            .state,
        DiplomaticState::War
    );

    let movement = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        matches!(
            movement.command,
            Command::Move(MoveOrder { ref armies, ref path })
                if armies == &[army] && path == &[enclave, middle]
        ),
        "{movement:?}"
    );
    apply(&mut campaign, &data, Actor::Npc(OAK), movement.command).unwrap();

    assert!(campaign.supplied_sites(OAK).contains(&enclave));
    assert!(preview(
        &campaign,
        &data,
        Actor::Npc(OAK),
        Command::DevelopCity { site: enclave },
    )
    .is_ok());
}

#[test]
fn corridor_opening_requires_funded_nonweak_forces_without_a_deficit() {
    let (data, campaign, _, _, _) = corridor_fixture();
    let mut variants = Vec::new();

    let mut deficit = campaign.clone();
    deficit.factions.get_mut(&OAK).unwrap().deficit = true;
    variants.push(deficit);

    let mut underfunded = campaign.clone();
    let reserve = upkeep_reserve(&underfunded, &data);
    underfunded.factions.get_mut(&OAK).unwrap().resources.gold = reserve - 1;
    variants.push(underfunded);

    let mut weak = campaign;
    for formation in weak
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.headcount = 1;
    }
    variants.push(weak);

    for variant in variants {
        let decision = ai::propose(&variant, &data, OAK).unwrap();
        assert_ne!(
            decision.command,
            Command::DeclareWar { faction: ROSE },
            "{decision:?}"
        );
    }
}

fn recovery_turn() -> (GameData, StrategicCampaign) {
    let (data, mut campaign) = fixture();
    for _ in 0..data.ai.objective_rounds {
        next_oak_turn(&mut campaign, &data);
    }
    (data, campaign)
}

fn add_supply_lost_order(
    data: &GameData,
    campaign: &mut StrategicCampaign,
    target: SiteId,
    builder: ArmyId,
) -> kestrum::state::construction::OrderId {
    let id = campaign.next_ids.order;
    campaign.next_ids.order.0 += 1;
    let kind = ConstructionKind::Outpost;
    let order = ConstructionOrder {
        id,
        owner: OAK,
        kind,
        target: ConstructionTarget::Site(target),
        builder: Some(builder),
        created_round: campaign.completed_rounds - data.ai.objective_rounds,
        progress: 0,
        required_steps: data.construction.outpost_steps,
        paid: data.economy.orders[&OrderKind::EstablishOutpost].cost,
        status: ConstructionStatus::Paused {
            reason: ConstructionPause::SupplyLost,
        },
        last_progress_round: None,
    };
    campaign.construction.insert(id, order);
    id
}

fn unsupplied_owned_site(data: &GameData, campaign: &mut StrategicCampaign) -> SiteId {
    let owned = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(OAK))
        .map(|site| site.id)
        .collect::<BTreeSet<_>>();
    let target = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none())
        .filter(|site| {
            campaign
                .world
                .adjacent_sites(site.id)
                .iter()
                .all(|neighbor| !owned.contains(neighbor))
        })
        .map(|site| site.id)
        .next()
        .expect("an unconnected neutral site exists");
    campaign
        .set_site_control(data, target, Some(OAK), false)
        .unwrap();
    assert!(!campaign.supplied_sites(OAK).contains(&target));
    target
}

fn corridor_fixture() -> (GameData, StrategicCampaign, SiteId, SiteId, ArmyId) {
    let (mut data, mut campaign) = recovery_turn();
    establish_oak_forces(&mut data, &mut campaign);

    let headquarters = campaign.factions[&OAK].headquarters;
    let protected: BTreeMap<_, _> = campaign
        .factions
        .values()
        .flat_map(|faction| {
            [faction.headquarters, faction.capital]
                .into_iter()
                .map(move |site| (site, faction.id))
        })
        .collect();
    let minimum = data.development.city_development.minimum_habitation;
    let candidates: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| {
            site.habitation >= minimum
                && site.habitation < Habitation::City
                && !protected.contains_key(&site.id)
                && !campaign.site_is_ruined(site.id)
                && !campaign.world.contested_sites.contains(&site.id)
                && !campaign
                    .world
                    .adjacent_sites(site.id)
                    .iter()
                    .any(|neighbor| {
                        campaign.world.site(*neighbor).is_some_and(|nearby| {
                            matches!(nearby.habitation, Habitation::City | Habitation::MajorCity)
                        })
                    })
        })
        .map(|site| site.id)
        .collect();
    let pairs = candidates.iter().find_map(|enclave| {
        if campaign
            .world
            .adjacent_sites(headquarters)
            .contains(enclave)
        {
            return None;
        }
        campaign
            .world
            .adjacent_sites(*enclave)
            .into_iter()
            .filter(|middle| *middle != headquarters && !protected.contains_key(middle))
            .find(|middle| {
                campaign.world.adjacent_sites(headquarters).contains(middle)
                    && !campaign.world.contested_sites.contains(middle)
                    && !campaign.threats.values().any(|threat| {
                        threat.site == *middle
                            && threat.status == kestrum::state::threat::ThreatStatus::Active
                    })
                    && campaign
                        .world
                        .connected_route(*middle, *enclave)
                        .is_some_and(|route| {
                            route_cost(route, &data)
                                <= army_step_allowance(&campaign, OAK, *enclave, &data)
                        })
            })
            .map(|middle| (middle, *enclave))
    });
    let (middle, enclave) = pairs.expect("a city-ready site lies two routes from Oak HQ");

    let original_controllers: BTreeMap<_, _> = campaign
        .world
        .sites
        .iter()
        .map(|site| (site.id, site.controller))
        .collect();
    let all_sites: Vec<_> = campaign.world.sites.iter().map(|site| site.id).collect();
    for site in all_sites {
        let owner = if site == enclave || site == headquarters {
            Some(OAK)
        } else if site == middle {
            Some(ROSE)
        } else {
            protected
                .get(&site)
                .copied()
                .or_else(|| {
                    original_controllers
                        .get(&site)
                        .copied()
                        .flatten()
                        .filter(|owner| *owner != OAK)
                })
                .or(Some(ROSE))
        };
        campaign
            .set_site_control(&data, site, owner, false)
            .unwrap();
    }
    for army in campaign.armies.values_mut() {
        army.site = campaign.factions[&army.faction].headquarters;
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == OAK)
        .expect("Oak has an army")
        .id;
    campaign.armies.get_mut(&army).unwrap().site = enclave;
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.headcount = formation.capacity;
        formation.movement_spent = 0;
    }
    let resources = campaign.factions[&OAK].resources;
    campaign.factions.get_mut(&OAK).unwrap().resources = Resources {
        gold: upkeep_reserve(&campaign, &data).max(resources.gold),
        wood: 0,
        stone: 0,
    };
    campaign.factions.get_mut(&OAK).unwrap().deficit = false;

    assert!(!campaign.supplied_sites(OAK).contains(&enclave));
    campaign.validate(&data).unwrap();
    (data, campaign, middle, enclave, army)
}

fn establish_oak_forces(data: &mut GameData, campaign: &mut StrategicCampaign) {
    let headquarters = campaign.factions[&OAK].headquarters;
    campaign.factions.get_mut(&OAK).unwrap().resources = Resources {
        gold: 100_000,
        wood: 100_000,
        stone: 100_000,
    };
    campaign.factions.get_mut(&OAK).unwrap().deficit = false;
    while campaign
        .armies
        .values()
        .filter(|army| army.faction == OAK)
        .count()
        < data.ai.target_armies
    {
        apply(
            campaign,
            data,
            Actor::Npc(OAK),
            Command::Recruit {
                site: headquarters,
                army: None,
                kind: TroopKind::Warriors,
            },
        )
        .unwrap();
    }
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == OAK)
        .map(|army| army.id)
        .collect();
    for army in armies {
        while campaign.armies[&army].formation_ids().count() < data.ai.minimum_formations {
            apply(
                campaign,
                data,
                Actor::Npc(OAK),
                Command::Recruit {
                    site: headquarters,
                    army: Some(army),
                    kind: TroopKind::Warriors,
                },
            )
            .unwrap();
        }
    }
}

fn army_step_allowance(
    campaign: &StrategicCampaign,
    faction: FactionId,
    site: SiteId,
    data: &GameData,
) -> u32 {
    campaign
        .armies
        .values()
        .find(|army| army.faction == faction)
        .into_iter()
        .flat_map(|army| army.formation_ids())
        .filter_map(|id| campaign.formations.get(&id))
        .map(|formation| formation.movement_allowance(data))
        .min()
        .unwrap_or(0)
        .min(
            campaign
                .world
                .connected_route(site, campaign.factions[&faction].headquarters)
                .map(|route| route_cost(route, data))
                .unwrap_or(u32::MAX),
        )
}

fn upkeep_reserve(campaign: &StrategicCampaign, data: &GameData) -> i64 {
    let upkeep = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == OAK)
        .map(|formation| data.economy.formations[&formation.kind].upkeep_gold)
        .fold(0_i64, i64::saturating_add);
    upkeep.saturating_mul(i64::from(data.ai.reserve_upkeep_rounds))
}
