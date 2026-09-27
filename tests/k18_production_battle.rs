//! A deterministic player campaign through its first production-world battle.

use kestrum::{
    data::{
        economy::{Habitation, TroopKind},
        generation::ProductionSetup,
        rules::Emblem,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, apply, Actor, Command, MoveOrder, MovementBlock},
    state::{
        diplomacy::DefeatResolution,
        siege::{SiegeAction, SiegeOrder},
        Campaign, CampaignPhase, FactionStatus, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;

const SEED: u64 = 1_809_281;
const ROUND_CAP: u32 = 20;
const OBJECTIVES: [(FactionId, SiteId); 3] = [
    (FactionId(3), SiteId(14)),
    (FactionId(2), SiteId(32)),
    (FactionId(4), SiteId(50)),
];

#[test]
fn four_faction_production_campaign_validates_a_real_battle_and_save_reload() {
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
    for (faction, site) in OBJECTIVES {
        assert_eq!(campaign.factions[&faction].headquarters, site);
    }

    let mut npc_actions = 0_u64;
    let mut boundaries = 0_u32;
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
    while campaign.diplomacy.ending.is_none()
        && campaign.completed_rounds < ROUND_CAP
        && !has_player_battle_cleanup(&campaign, player)
    {
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
                engine::advance_npc(&mut campaign, &data).unwrap();
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
                let Some((target, headquarters)) =
                    OBJECTIVES.iter().copied().find(|(faction, _)| {
                        visible.factions.iter().any(|entry| {
                            entry.id == *faction && entry.status == FactionStatus::Independent
                        })
                    })
                else {
                    issue(&mut campaign, &data, Command::EndTurn);
                    continue;
                };

                if let Some(relation) = engine::diplomacy_view(&campaign, &data, player)
                    .factions
                    .into_iter()
                    .find(|entry| entry.id == target)
                {
                    if relation.relation == kestrum::data::world::DiplomaticState::Peace {
                        issue(
                            &mut campaign,
                            &data,
                            Command::DeclareWar { faction: target },
                        );
                        player_orders += 1;
                        continue;
                    }
                }

                let mut objectives = if visible
                    .world
                    .site(headquarters)
                    .is_some_and(|site| site.controller == Some(target))
                {
                    vec![headquarters]
                } else {
                    visible
                        .world
                        .sites
                        .iter()
                        .filter(|site| {
                            site.controller == Some(target)
                                && site.habitation >= Habitation::Outpost
                        })
                        .map(|site| site.id)
                        .collect::<Vec<_>>()
                };
                objectives.extend(visible.hostile_presence.iter().copied().filter(|site| {
                    visible
                        .world
                        .site(*site)
                        .is_some_and(|entry| entry.controller == Some(target))
                }));
                objectives.sort();
                objectives.dedup();
                if objectives.is_empty() {
                    objectives.push(headquarters);
                }
                let objective = objectives
                    .into_iter()
                    .filter_map(|site| {
                        visible
                            .armies
                            .iter()
                            .filter(|army| !army.is_empty())
                            .filter(|army| army.site != site)
                            .filter_map(|army| {
                                engine::movement_preview(&campaign, &data, player, &[army.id], site)
                                    .ok()
                                    .map(|preview| preview.total_cost)
                            })
                            .min()
                            .map(|cost| (cost, site))
                    })
                    .min_by_key(|(cost, site)| (*cost, *site))
                    .map_or(headquarters, |(_, site)| site);

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

                let candidates = visible
                    .armies
                    .iter()
                    .filter(|entry| !entry.is_empty())
                    .filter(|entry| entry.site != objective)
                    .map(|entry| {
                        let preview = engine::movement_preview(
                            &campaign,
                            &data,
                            player,
                            &[entry.id],
                            objective,
                        )
                        .unwrap_or_else(|error| {
                            panic!(
                                "production route from {:?} to {objective:?} for {:?} at round {}: {error}; sieges {:?}",
                                entry.site,
                                entry.id,
                                campaign.completed_rounds,
                                campaign.sieges.keys(),
                            )
                        });
                        (preview.total_cost, entry.id, preview)
                    })
                    .collect::<Vec<_>>();
                let Some((_, army, preview)) = candidates
                    .into_iter()
                    .min_by_key(|(cost, army, _)| (*cost, *army))
                else {
                    issue(&mut campaign, &data, Command::EndTurn);
                    continue;
                };
                let group = vec![army];
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

    let player_battle = campaign
        .battles
        .values()
        .find(|battle| {
            battle.defender.faction().is_some_and(|defender| {
                ((battle.attacker.faction == player && defender != player)
                    || (defender == player && battle.attacker.faction != player))
                    && battle_has_destroyed_formation(battle)
            })
        })
        .expect("no production battle destroyed a formation before the round cap");
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
    let restored = kestrum::state::persistence::load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(restored, campaign);
    println!(
        "K18_PRODUCTION_BATTLE seed={SEED} rounds={} boundaries={boundaries} npc_actions={npc_actions} player_orders={player_orders} battle_site={:?} save_bytes={}",
        campaign.completed_rounds,
        player_battle.site,
        encoded.len(),
    );
}

fn has_player_battle_cleanup(campaign: &StrategicCampaign, player: FactionId) -> bool {
    campaign.battles.values().any(|battle| {
        battle.defender.faction().is_some_and(|defender| {
            ((battle.attacker.faction == player && defender != player)
                || (defender == player && battle.attacker.faction != player))
                && battle_has_destroyed_formation(battle)
        })
    })
}

fn battle_has_destroyed_formation(battle: &kestrum::state::battle::BattleReport) -> bool {
    battle
        .faction_sides()
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.formations)
        .any(|formation| formation.start > 0 && formation.end == 0)
}

fn issue(campaign: &mut StrategicCampaign, data: &GameData, command: Command) {
    let label = format!("{command:?}");
    apply(campaign, data, Actor::Player, command)
        .unwrap_or_else(|error| panic!("production campaign order {label}: {error}"));
}
