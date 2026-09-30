//! Explore and occupy a connected realm while leaving independent rival kingdoms.

use super::issue;
use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, SiteId},
        GameData,
    },
    engine::{self, Command},
    state::{military::ArmyId, StrategicCampaign},
};

#[derive(Default)]
pub(super) struct Patrol {
    returning: bool,
    complete: bool,
}

pub(super) fn develop(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    host: ArmyId,
    home: SiteId,
    patrol: &mut Patrol,
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
    let garrison = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && army.id != host)
        .map(|army| army.id)
        .min();
    if let Some(army) = garrison.filter(|_| campaign.completed_rounds >= 40) {
        patrol.advance(campaign, data, army, home)?;
    }
    let travelers: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && Some(army.id) != garrison)
        .map(|army| army.id)
        .collect();
    for army in travelers {
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

impl Patrol {
    fn advance(
        &mut self,
        campaign: &mut StrategicCampaign,
        data: &GameData,
        army: ArmyId,
        home: SiteId,
    ) -> Result<(), String> {
        if self.complete {
            return Ok(());
        }
        for _ in 0..6 {
            let formation = campaign.armies[&army]
                .formation_ids()
                .next()
                .ok_or("Patrol has no formation")?;
            let traveled = &campaign.formations[&formation]
                .service
                .ledger
                .traversed_routes;
            self.returning |= traveled.len() >= data.progression.careers.scout_routes as usize;
            if self.returning && campaign.armies[&army].site == home {
                self.complete = true;
                break;
            }
            let target = if self.returning {
                Some(home)
            } else {
                campaign
                    .supplied_sites(campaign.player)
                    .into_iter()
                    .filter_map(|site| {
                        engine::movement_preview(campaign, data, campaign.player, &[army], site)
                            .ok()
                    })
                    .filter(|preview| {
                        preview.reachable_steps > 0
                            && preview
                                .steps
                                .iter()
                                .take(preview.reachable_steps)
                                .any(|step| !traveled.contains(&step.route))
                            && preview
                                .order
                                .path
                                .iter()
                                .all(|site| campaign.world.is_secure(*site, campaign.player))
                    })
                    .min_by_key(|preview| (preview.total_cost, preview.order.path.last().copied()))
                    .and_then(|preview| preview.order.path.last().copied())
            };
            let Some(destination) = target else {
                break;
            };
            let Ok(preview) =
                engine::movement_preview(campaign, data, campaign.player, &[army], destination)
            else {
                break;
            };
            if preview.reachable_steps == 0 {
                break;
            }
            move_order(campaign, data, preview.order)?;
        }
        Ok(())
    }
}

fn march(campaign: &mut StrategicCampaign, data: &GameData, army: ArmyId) -> Result<bool, String> {
    let known = engine::explored_sites(campaign, campaign.player);
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
        .min_by_key(|preview| (preview.total_cost, preview.order.path.last().copied()));
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
