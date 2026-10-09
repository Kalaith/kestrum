//! Explore and occupy a connected realm while leaving independent rival kingdoms.

use super::issue;
use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, PersonClass, SiteId},
        GameData,
    },
    engine::{self, Command},
    state::{military::ArmyId, people::PersonStatus, StrategicCampaign},
};

pub(super) fn develop(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    host: ArmyId,
    home: SiteId,
) -> Result<(), String> {
    // The first new army remains at headquarters for recruits and courses;
    // two later companies explore separate routes with normal movement/supply.
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player)
        .map(|army| army.id)
        .collect();
    if armies.len() < 4 && campaign.factions[&campaign.player].resources.gold >= 160 {
        let command = Command::Recruit {
            site: home,
            army: None,
            kind: TroopKind::Warriors,
        };
        if engine::preview(campaign, data, engine::Actor::Player, command.clone()).is_ok() {
            issue(campaign, data, command)?;
        }
    }
    relieve_deficit(campaign, data)?;
    reinforce_at_home(campaign, data, home)?;
    let garrison = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && army.id != host)
        .map(|army| army.id)
        .min();
    let learners: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && army.id != host)
        .filter(|army| {
            army.formation_ids().any(|id| {
                campaign.formation_person(id).is_some_and(|person| {
                    person.class == kestrum::data::world::PersonClass::Recruit
                })
            })
        })
        .map(|army| army.id)
        .collect();
    if !learners.is_empty() && campaign.completed_rounds >= 40 {
        for army in learners.iter().copied() {
            advance_patrol(campaign, data, army, home)?;
        }
    }
    let travelers: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| {
            army.faction == campaign.player
                && Some(army.id) != garrison
                && !learners.contains(&army.id)
        })
        .map(|army| army.id)
        .collect();
    let at_war = campaign.relations.iter().any(|relation| {
        relation.state == DiplomaticState::War && relation.factions.contains(&campaign.player)
    });
    for army in travelers {
        // Wartime travellers advance on the nearest enemy-held ground, else defend home.
        if at_war {
            if counterattack(campaign, data, army)? {
                continue;
            }
            if campaign.armies[&army].site != home {
                if let Ok(preview) =
                    engine::movement_preview(campaign, data, campaign.player, &[army], home)
                {
                    if preview.can_confirm() && preview.reachable_steps > 0 {
                        move_order(campaign, data, preview.order)?;
                    }
                }
            }
            continue;
        }
        // A bounded number of orders lets the host clear threats without
        // spending the entire season revisiting the same protected site.
        for _ in 0..6 {
            if army == host && super::stage_local_battle(campaign, data, army)? {
                issue(campaign, data, Command::StartPendingBattle)?;
                if !campaign.armies.contains_key(&army) {
                    break;
                }
            }
            if !march(campaign, data, army)? {
                break;
            }
        }
    }
    Ok(())
}

fn counterattack(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army: ArmyId,
) -> Result<bool, String> {
    let enemies: Vec<_> = campaign
        .relations
        .iter()
        .filter(|relation| {
            relation.state == DiplomaticState::War && relation.factions.contains(&campaign.player)
        })
        .flat_map(|relation| relation.factions)
        .filter(|faction| *faction != campaign.player)
        .collect();
    let known = engine::explored_sites(campaign, campaign.player);
    let target = campaign
        .world
        .sites
        .iter()
        .filter(|site| {
            known.contains(&site.id)
                && site
                    .controller
                    .is_some_and(|owner| enemies.contains(&owner))
        })
        .filter_map(|site| {
            engine::movement_preview(campaign, data, campaign.player, &[army], site.id)
                .ok()
                .filter(|preview| preview.can_confirm() && preview.reachable_steps > 0)
        })
        .min_by_key(|preview| (preview.total_cost, preview.order.path.last().copied()));
    let Some(preview) = target else {
        return Ok(false);
    };
    move_order(campaign, data, preview.order)?;
    Ok(true)
}

/// A shortfall blocks recovery, so release the newest unnamed formation until upkeep fits.
fn relieve_deficit(campaign: &mut StrategicCampaign, data: &GameData) -> Result<(), String> {
    let Some(statement) = campaign.factions[&campaign.player].last_economy.clone() else {
        return Ok(());
    };
    let mut excess = statement.upkeep_due - statement.income.gold;
    while excess > 0 {
        let Some(formation) = campaign
            .formations
            .values()
            .filter(|formation| {
                formation.faction == campaign.player
                    && formation.headcount > 0
                    && campaign.formation_person(formation.id).is_none()
            })
            .max_by_key(|formation| (formation.created_round, formation.id))
            .map(|formation| formation.id)
        else {
            return Ok(());
        };
        let upkeep = data.economy.formations[&campaign.formations[&formation].kind].upkeep_gold;
        let command = Command::Disband { formation };
        if engine::preview(campaign, data, engine::Actor::Player, command.clone()).is_err() {
            return Ok(());
        }
        issue(campaign, data, command)?;
        excess -= upkeep;
    }
    Ok(())
}

/// Fill free slots at headquarters while seasonal income still covers the upkeep.
fn reinforce_at_home(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    home: SiteId,
) -> Result<(), String> {
    const ROTATION: [TroopKind; 3] = [TroopKind::Spearmen, TroopKind::Warriors, TroopKind::Archers];
    for _ in 0..6 {
        let faction = &campaign.factions[&campaign.player];
        let Some(statement) = faction.last_economy.as_ref() else {
            return Ok(());
        };
        let spare = statement.income.gold - statement.upkeep_due;
        if spare < 2 * data.economy.formations[&TroopKind::Spearmen].upkeep_gold
            || faction.resources.gold < 300
        {
            return Ok(());
        }
        let Some((army, slots)) = campaign
            .armies
            .values()
            .filter(|army| army.faction == campaign.player && army.site == home)
            .filter(|army| army.first_empty_slot().is_some())
            .map(|army| (army.id, army.formation_ids().count()))
            .min_by_key(|(id, count)| (*count, *id))
        else {
            return Ok(());
        };
        let command = Command::Recruit {
            site: home,
            army: Some(army),
            kind: ROTATION[slots % ROTATION.len()],
        };
        if engine::preview(campaign, data, engine::Actor::Player, command.clone()).is_err() {
            return Ok(());
        }
        issue(campaign, data, command)?;
        // Upkeep is assessed at the boundary; keep this season's additions modest.
        if slots % 2 == 1 {
            return Ok(());
        }
    }
    Ok(())
}

fn advance_patrol(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army: ArmyId,
    home: SiteId,
) -> Result<(), String> {
    let route_goal = data.progression.careers.scout_routes as usize;
    for _ in 0..6 {
        let route_history: Vec<_> = campaign.armies[&army]
            .formation_ids()
            .filter_map(|id| campaign.formation_person(id))
            .filter(|person| {
                person.faction == campaign.player
                    && person.class == PersonClass::Recruit
                    && person.is_alive()
                    && person.status == PersonStatus::Fit
                    && !person.career.retired
                    && person.career.course.is_none()
                    && person.age_years(campaign.completed_rounds) >= 17
            })
            .map(|person| person.evidence.traversed_routes.clone())
            .collect();
        if route_history.is_empty() {
            break;
        }
        let returning = route_history
            .iter()
            .all(|routes| routes.len() >= route_goal);
        let preview = if returning {
            engine::movement_preview(campaign, data, campaign.player, &[army], home)
                .ok()
                .filter(|preview| {
                    preview
                        .order
                        .path
                        .iter()
                        .all(|site| campaign.world.is_secure(*site, campaign.player))
                })
        } else {
            campaign
                .supplied_sites(campaign.player)
                .into_iter()
                .filter_map(|site| {
                    engine::movement_preview(campaign, data, campaign.player, &[army], site).ok()
                })
                .filter(|preview| {
                    preview.reachable_steps > 0
                        && preview
                            .steps
                            .iter()
                            .take(preview.reachable_steps)
                            .any(|step| {
                                route_history
                                    .iter()
                                    .any(|routes| !routes.contains(&step.route))
                            })
                        && preview
                            .order
                            .path
                            .iter()
                            .all(|site| campaign.world.is_secure(*site, campaign.player))
                })
                .min_by_key(|preview| (preview.total_cost, preview.order.path.last().copied()))
        };
        let Some(preview) = preview.filter(|preview| preview.reachable_steps > 0) else {
            break;
        };
        move_order(campaign, data, preview.order)?;
    }
    Ok(())
}

fn march(campaign: &mut StrategicCampaign, data: &GameData, army: ArmyId) -> Result<bool, String> {
    let known = engine::explored_sites(campaign, campaign.player);
    let supplied = campaign.supplied_sites(campaign.player);
    let origin = campaign.armies[&army].site;
    let destination = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none() && known.contains(&site.id))
        .filter(|site| campaign.active_threat(site.id).is_none())
        .filter_map(|site| {
            engine::movement_preview(campaign, data, campaign.player, &[army], site.id)
                .ok()
                .filter(|preview| preview.reachable_steps > 0 && preview.encounter.is_none())
                .filter(|preview| {
                    preview.order.path.iter().all(|id| {
                        campaign.world.site(*id).is_some_and(|site| {
                            site.controller.is_none() || site.controller == Some(campaign.player)
                        }) && campaign.active_threat(*id).is_none()
                    })
                })
        })
        // Contiguous claims keep supply and full income; distant ones only as a fallback.
        .min_by_key(|preview| {
            let destination = preview.order.path.last().copied();
            let contiguous = destination.is_some_and(|site| {
                campaign
                    .world
                    .adjacent_sites(site)
                    .iter()
                    .any(|near| supplied.contains(near))
            });
            (!contiguous, preview.total_cost, destination)
        });
    let Some(preview) = destination else {
        return Ok(false);
    };
    move_order(campaign, data, preview.order)?;
    Ok(campaign
        .armies
        .get(&army)
        .is_some_and(|army| army.site != origin))
}

pub(super) fn open_frontier(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let rivals: std::collections::BTreeSet<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(campaign.player))
        .flat_map(|site| campaign.world.adjacent_sites(site.id))
        .filter_map(|site| campaign.world.site(site)?.controller)
        .filter(|owner| *owner != campaign.player && campaign.is_independent(*owner))
        .collect();
    for faction in rivals {
        if campaign.relations.iter().any(|relation| {
            relation.factions.contains(&campaign.player)
                && relation.factions.contains(&faction)
                && relation.state == DiplomaticState::Peace
        }) {
            let command = Command::DeclareWar { faction };
            if engine::preview(campaign, data, engine::Actor::Player, command.clone()).is_ok() {
                issue(campaign, data, command)?;
            }
        }
    }
    Ok(())
}

fn move_order(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    order: engine::MoveOrder,
) -> Result<(), String> {
    issue(campaign, data, Command::Move(order))?;
    if campaign.pending_battle.is_some() {
        issue(campaign, data, Command::StartPendingBattle)?;
    }
    Ok(())
}
