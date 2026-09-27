//! A deterministic player campaign from production setup through AI victory.

use kestrum::{
    data::{
        economy::{Habitation, TroopKind},
        generation::ProductionSetup,
        rules::Emblem,
        world::SiteId,
        GameData,
    },
    engine::{self, apply, Actor, Command, MoveOrder, MovementBlock},
    state::{
        diplomacy::{DefeatResolution, EndingKind},
        siege::{SiegeAction, SiegeOrder},
        Campaign, CampaignPhase, FactionStatus, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;
use std::collections::BTreeMap;

const SEED: u64 = 88;
const ROUND_CAP: u32 = 120;

#[test]
fn four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save() {
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
    assert_eq!(campaign.world.markers.len(), 80);
    assert_eq!(campaign.world.sites.len(), 152);
    let mut npc_actions = 0_u64;
    let mut boundaries = 0_u32;
    let mut player_battle = None;
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == player)
        .expect("production founder army")
        .id;
    let home = campaign.factions[&player].headquarters;
    for _ in 0..3 {
        issue(
            &mut campaign,
            &data,
            Command::Recruit {
                site: home,
                army: Some(army),
                kind: TroopKind::Warriors,
            },
        );
    }
    let mut player_orders = 3_u32;
    while campaign.diplomacy.ending.is_none() && campaign.completed_rounds < ROUND_CAP {
        if player_battle.is_none() {
            player_battle = campaign
                .battles
                .values()
                .find(|battle| is_player_battle(battle, player))
                .cloned();
        }
        if let Some(defeat) = campaign
            .diplomacy
            .pending_defeats
            .iter()
            .find(|entry| entry.victor == player)
            .cloned()
        {
            issue(
                &mut campaign,
                &data,
                Command::ResolveDefeat {
                    faction: defeat.faction,
                    resolution: DefeatResolution::Annex,
                },
            );
            player_orders += 1;
            continue;
        }
        if let Some(offer) = campaign
            .diplomacy
            .pending_offers
            .iter()
            .find(|offer| offer.recipient == player)
            .cloned()
        {
            issue(
                &mut campaign,
                &data,
                Command::RespondPeace {
                    proposer: offer.proposer,
                    accept: false,
                },
            );
            player_orders += 1;
            continue;
        }

        match campaign.phase {
            CampaignPhase::NpcTurn { .. } => {
                let before = campaign.completed_rounds;
                engine::advance_npc(&mut campaign, &data).unwrap_or_else(|error| {
                    panic!(
                        "NPC action failed in {:?} at round {before}, sequence {}: {error}",
                        campaign.phase, campaign.accepted_sequence
                    )
                });
                npc_actions += 1;
                campaign.validate(&data).unwrap_or_else(|error| {
                    panic!(
                        "invalid production state after NPC action {npc_actions} at round {before}, phase {:?}: {error}",
                        campaign.phase
                    )
                });
                boundaries += campaign.completed_rounds.saturating_sub(before);
            }
            CampaignPhase::PlayerTurn => {
                let visible = engine::project(&campaign, player).unwrap();
                let mut own_armies = visible
                    .armies
                    .iter()
                    .filter(|army| army.faction == player && !army.is_empty())
                    .collect::<Vec<_>>();
                own_armies.sort_by_key(|army| army.id);
                let siege_order = visible.sieges.iter().find_map(|siege| {
                    let view = engine::siege_view(&campaign, &data, player, siege.site)?;
                    let preferred = match view.role {
                        kestrum::engine::SiegeRole::Besieger => {
                            [SiegeAction::Assault, SiegeAction::Maintain]
                        }
                        kestrum::engine::SiegeRole::Defender => {
                            [SiegeAction::Sortie, SiegeAction::Escape]
                        }
                    };
                    preferred.into_iter().find_map(|action| {
                        let option = view
                            .actions
                            .iter()
                            .find(|option| option.action == action && option.blocked.is_none())?;
                        Some(SiegeOrder {
                            site: view.site,
                            action,
                            armies: view.own_armies.clone(),
                            destination: matches!(
                                action,
                                SiegeAction::Escape | SiegeAction::Withdraw
                            )
                            .then(|| option.destinations.first().copied())
                            .flatten(),
                        })
                    })
                });
                if let Some(order) = siege_order {
                    issue(&mut campaign, &data, Command::Siege(order));
                    player_orders += 1;
                    continue;
                }
                let co_located_hostiles = visible
                    .hostile_presence
                    .iter()
                    .copied()
                    .filter(|site| own_armies.iter().any(|army| army.site == *site))
                    .collect::<Vec<_>>();
                let mut disengage = co_located_hostiles.into_iter().find_map(|site| {
                    let group = own_armies
                        .iter()
                        .filter(|army| army.site == site)
                        .map(|army| army.id)
                        .collect::<Vec<_>>();
                    visible
                        .world
                        .adjacent_sites(site)
                        .into_iter()
                        .filter(|destination| visible.world.site(*destination).is_some())
                        .filter_map(|destination| {
                            engine::movement_preview(&campaign, &data, player, &group, destination)
                                .ok()
                                .filter(|preview| preview.reachable_steps > 0)
                                .map(|preview| {
                                    (preview.total_cost, destination, group.clone(), preview)
                                })
                        })
                        .min_by_key(|(cost, destination, _, _)| (*cost, *destination))
                });
                if let Some((_, _, group, preview)) = disengage.take() {
                    issue(
                        &mut campaign,
                        &data,
                        Command::Move(MoveOrder {
                            armies: group,
                            path: preview.order.path,
                        }),
                    );
                    player_orders += 1;
                    continue;
                }
                let recruit_cost = data.economy.formations[&TroopKind::Warriors].recruit_cost;
                let resources = campaign.factions[&player].resources;
                if resources.gold >= recruit_cost.gold
                    && resources.wood >= recruit_cost.wood
                    && own_armies
                        .iter()
                        .map(|army| army.slots.iter().flatten().count())
                        .sum::<usize>()
                        < 6
                {
                    let site = visible
                        .world
                        .sites
                        .iter()
                        .filter(|site| {
                            site.controller == Some(player)
                                && site.habitation >= Habitation::Outpost
                                && campaign.supplied_sites(player).contains(&site.id)
                        })
                        .max_by_key(|site| {
                            let formations_at_site = own_armies
                                .iter()
                                .filter(|army| army.site == site.id)
                                .map(|army| army.slots.iter().flatten().count())
                                .sum::<usize>();
                            (
                                formations_at_site,
                                site.habitation,
                                std::cmp::Reverse(site.id),
                            )
                        })
                        .map(|site| site.id);
                    if let Some(site) = site {
                        let army = own_armies
                            .iter()
                            .find(|army| {
                                army.site == site && army.slots.iter().flatten().count() < 6
                            })
                            .map(|army| army.id);
                        issue(
                            &mut campaign,
                            &data,
                            Command::Recruit {
                                site,
                                army,
                                kind: TroopKind::Warriors,
                            },
                        );
                        player_orders += 1;
                        continue;
                    }
                }
                let relations = engine::diplomacy_view(&campaign, &data, player).factions;
                let mut army_groups: BTreeMap<SiteId, Vec<_>> = BTreeMap::new();
                for army in own_armies {
                    army_groups.entry(army.site).or_default().push(army.id);
                }
                let mut targets = Vec::new();
                for target in visible
                    .factions
                    .iter()
                    .filter(|faction| {
                        faction.id != player && faction.status == FactionStatus::Independent
                    })
                    .map(|faction| faction.id)
                {
                    let Some(relation) = relations.iter().find(|relation| relation.id == target)
                    else {
                        continue;
                    };
                    if relation.relation != kestrum::data::world::DiplomaticState::War
                        && (relation.relation != kestrum::data::world::DiplomaticState::Peace
                            || relation
                                .truce_until
                                .is_some_and(|until| campaign.completed_rounds < until))
                    {
                        continue;
                    }
                    let target_holding_count = visible
                        .world
                        .sites
                        .iter()
                        .filter(|site| {
                            site.controller == Some(target)
                                && site.habitation >= Habitation::Outpost
                        })
                        .count() as u32;
                    for site in visible.world.sites.iter().filter(|site| {
                        site.controller == Some(target)
                            && (site.habitation >= Habitation::Outpost
                                || visible.hostile_presence.contains(&site.id))
                    }) {
                        for (origin, armies) in &army_groups {
                            if *origin == site.id {
                                continue;
                            }
                            if let Ok(preview) =
                                engine::movement_preview(&campaign, &data, player, armies, site.id)
                            {
                                let can_declare_war = relation.relation
                                    == kestrum::data::world::DiplomaticState::Peace;
                                let can_clear_threat = preview.stop.as_ref().is_some_and(|stop| {
                                    stop.reason == MovementBlock::ThreatRequiresClear
                                        && preview.reachable_steps == 0
                                        && preview
                                            .steps
                                            .first()
                                            .is_some_and(|step| step.cost <= preview.remaining)
                                });
                                if preview.reachable_steps > 0
                                    || can_clear_threat
                                    || can_declare_war
                                {
                                    targets.push((
                                        target_holding_count,
                                        preview.total_cost,
                                        Some(target),
                                        site.id,
                                        armies.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
                if relations
                    .iter()
                    .any(|relation| relation.relation == kestrum::data::world::DiplomaticState::War)
                {
                    for site in &visible.hostile_presence {
                        for (origin, armies) in &army_groups {
                            if *origin == *site {
                                continue;
                            }
                            if let Ok(preview) =
                                engine::movement_preview(&campaign, &data, player, armies, *site)
                            {
                                if preview.reachable_steps > 0 {
                                    targets.push((
                                        u32::MAX,
                                        preview.total_cost,
                                        None,
                                        *site,
                                        armies.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
                let Some((_, _, target, objective, group)) =
                    targets
                        .into_iter()
                        .min_by_key(|(rank, cost, faction, site, armies)| {
                            (*rank, *cost, *faction, *site, armies.clone())
                        })
                else {
                    issue(&mut campaign, &data, Command::EndTurn);
                    continue;
                };

                if let Some(target) = target {
                    if relations.iter().any(|relation| {
                        relation.id == target
                            && relation.relation == kestrum::data::world::DiplomaticState::Peace
                    }) {
                        issue(
                            &mut campaign,
                            &data,
                            Command::DeclareWar { faction: target },
                        );
                        player_orders += 1;
                        continue;
                    }
                }

                if let Some(view) = engine::siege_view(&campaign, &data, player, objective) {
                    if view.actions.iter().any(|option| {
                        option.action == SiegeAction::Assault && option.blocked.is_none()
                    }) {
                        issue(
                            &mut campaign,
                            &data,
                            Command::Siege(SiegeOrder {
                                site: objective,
                                action: SiegeAction::Assault,
                                armies: view.own_armies,
                                destination: None,
                            }),
                        );
                        player_orders += 1;
                    } else {
                        issue(&mut campaign, &data, Command::EndTurn);
                    }
                    continue;
                }

                let preview = engine::movement_preview(
                    &campaign,
                    &data,
                    player,
                    &group,
                    objective,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "production route to {objective:?} for {group:?} at round {}: {error}; sieges {:?}",
                        campaign.completed_rounds,
                        campaign.sieges.keys(),
                    )
                });
                if let Some(stop) = &preview.stop {
                    if stop.reason == MovementBlock::ThreatRequiresClear
                        && preview.reachable_steps == 0
                        && preview
                            .steps
                            .first()
                            .is_some_and(|step| step.cost <= preview.remaining)
                    {
                        if let Some(threat) = visible
                            .threats
                            .iter()
                            .find(|threat| threat.site == stop.site)
                        {
                            issue(
                                &mut campaign,
                                &data,
                                Command::ClearThreat {
                                    armies: group.clone(),
                                    threat: threat.id,
                                },
                            );
                            player_orders += 1;
                            continue;
                        }
                    }
                }
                if preview.reachable_steps > 0 {
                    issue(
                        &mut campaign,
                        &data,
                        Command::Move(MoveOrder {
                            armies: group,
                            path: preview.order.path,
                        }),
                    );
                    player_orders += 1;
                } else {
                    issue(&mut campaign, &data, Command::EndTurn);
                }
            }
        }
    }

    let player_battle = player_battle
        .expect("the production campaign did not destroy a formation in player-vs-rival combat");
    let victory = campaign
        .diplomacy
        .ending
        .as_ref()
        .unwrap_or_else(|| panic!("no production victory by round {ROUND_CAP}"));
    assert_eq!(victory.kind, EndingKind::Victory);
    assert!(campaign.diplomacy.pending_defeats.is_empty());
    assert!(campaign
        .factions
        .values()
        .filter(|faction| faction.id != player)
        .all(|faction| faction.status == FactionStatus::Eliminated
            || faction.status == FactionStatus::Vassal { sovereign: player }));
    assert!(campaign.completed_rounds > 0);
    assert!(npc_actions > 0 && boundaries > 0);
    assert!(player_orders > 0);
    assert_ne!(
        player_battle.attacker.faction,
        player_battle
            .defender
            .faction()
            .expect("production battle defender")
    );
    let destroyed_formations = player_battle
        .faction_sides()
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.formations)
        .filter(|formation| formation.start > 0 && formation.end == 0)
        .map(|formation| formation.id)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(player_battle
        .faction_sides()
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.people)
        .all(|person| !destroyed_formations.contains(&person.starting_formation)
            || !matches!(person.assignment, kestrum::state::people::PersonAssignment::Formation { formation }
                if destroyed_formations.contains(&formation))));
    campaign.validate(&data).unwrap();

    let encoded = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    let mut restored = kestrum::state::persistence::load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(restored, campaign);
    let terminal = restored.clone();
    assert!(apply(&mut restored, &data, Actor::Player, Command::EndTurn).is_err());
    assert!(engine::advance_npc(&mut restored, &data).is_err());
    assert_eq!(
        restored, terminal,
        "terminal victory replayed campaign effects"
    );
    println!(
        "K18_PRODUCTION_VICTORY ending={:?} seed={SEED} rounds={} boundaries={boundaries} npc_actions={npc_actions} player_orders={player_orders} battle_site={:?} save_bytes={}",
        victory.kind,
        campaign.completed_rounds,
        player_battle.site,
        encoded.len(),
    );
}

fn battle_has_destroyed_formation(battle: &kestrum::state::battle::BattleReport) -> bool {
    battle
        .faction_sides()
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.formations)
        .any(|formation| formation.start > 0 && formation.end == 0)
}

fn is_player_battle(
    battle: &kestrum::state::battle::BattleReport,
    player: kestrum::data::world::FactionId,
) -> bool {
    battle
        .defender
        .faction()
        .is_some_and(|defender| battle.attacker.faction == player || defender == player)
        && battle
            .defender
            .faction()
            .is_some_and(|defender| defender != player)
        && battle_has_destroyed_formation(battle)
}

fn issue(campaign: &mut StrategicCampaign, data: &GameData, command: Command) {
    let label = format!("{command:?}");
    apply(campaign, data, Actor::Player, command)
        .unwrap_or_else(|error| panic!("production campaign order {label}: {error}"));
}
