//! Deterministic review campaign; time, economy and rivals advance through real commands.

#[path = "campaign/expansion.rs"]
mod expansion;
#[path = "campaign/roster.rs"]
mod roster;

use kestrum::{
    data::{
        battle_tactics::BattleDoctrine, economy::TroopKind, generation::ProductionSetup,
        rules::Emblem, GameData,
    },
    engine::{self, apply, Actor, Command},
    state::{diplomacy::DefeatResolution, military::ArmyId, CampaignPhase, StrategicCampaign},
};

const MIDGAME_ROUNDS: u32 = 60;
/// The review realm turns from settlement to conquest after this many seasons.
const WAR_AFTER_ROUNDS: u32 = 30;

pub fn generate(data: &GameData) -> Result<StrategicCampaign, String> {
    play(data, MIDGAME_ROUNDS, |_| {})
}

/// Play the review kingdom through `rounds` seasons, observing each player turn.
pub fn play(
    data: &GameData,
    rounds: u32,
    mut observe: impl FnMut(&StrategicCampaign),
) -> Result<StrategicCampaign, String> {
    let mut campaign = StrategicCampaign::new_production(
        data,
        &ProductionSetup {
            kingdom_name: "Briarhold".into(),
            emblem: Emblem::Rose,
            factions: 4,
            seed: 88,
        },
    )?;
    campaign.tutorial.dismiss();
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == campaign.player)
        .ok_or("The review campaign has no founding army")?
        .id;
    let home = campaign.factions[&campaign.player].headquarters;
    // Follow the guided opening: invest in the capital before filling the host.
    issue(&mut campaign, data, Command::DevelopCity { site: home })?;
    for _ in 0..3 {
        issue(
            &mut campaign,
            data,
            Command::Recruit {
                site: home,
                army: Some(army),
                kind: TroopKind::Warriors,
            },
        )?;
    }
    let mut opening_complete = false;
    for _ in 0..20_000 {
        if campaign.diplomacy.ending.is_some() {
            return Err(format!(
                "Review campaign ended at round {}",
                campaign.completed_rounds
            ));
        }
        if campaign.pending_battle.is_some() {
            issue(&mut campaign, data, Command::StartPendingBattle)?;
            continue;
        }
        if let Some(defeat) = campaign
            .diplomacy
            .pending_defeats
            .iter()
            .find(|entry| entry.victor == campaign.player)
            .cloned()
        {
            issue(
                &mut campaign,
                data,
                Command::ResolveDefeat {
                    faction: defeat.faction,
                    resolution: DefeatResolution::Annex,
                },
            )?;
            continue;
        }
        if let Some(offer) = campaign
            .diplomacy
            .pending_offers
            .iter()
            .find(|offer| offer.recipient == campaign.player)
            .cloned()
        {
            issue(
                &mut campaign,
                data,
                Command::RespondPeace {
                    proposer: offer.proposer,
                    accept: true,
                },
            )?;
            continue;
        }
        match campaign.phase {
            CampaignPhase::PlayerTurn if campaign.completed_rounds >= rounds => {
                if !campaign.armies.contains_key(&army) {
                    return Err("The review army was lost before midgame.".into());
                }
                expansion::open_frontier(&mut campaign, data)?;
                campaign.validate(data)?;
                return Ok(campaign);
            }
            CampaignPhase::PlayerTurn => {
                observe(&campaign);
                if !opening_complete {
                    if stage_local_battle(&mut campaign, data, army)? {
                        issue(&mut campaign, data, Command::StartPendingBattle)?;
                        opening_complete = true;
                        continue;
                    }
                    if approach_threat(&mut campaign, data, army)? {
                        continue;
                    }
                }
                diplomacy(&mut campaign, data)?;
                roster::develop(&mut campaign, data, home)?;
                expansion::develop(&mut campaign, data, army, home)?;
                issue(&mut campaign, data, Command::EndTurn)?;
            }
            CampaignPhase::NpcTurn { .. } => {
                engine::advance_npc(&mut campaign, data).map_err(|error| error.to_string())?;
            }
        }
    }
    Err("The review campaign did not reach its bounded midgame target.".into())
}

fn stage_local_battle(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army: ArmyId,
) -> Result<bool, String> {
    let visible = engine::project(campaign, campaign.player).map_err(|error| error.to_string())?;
    let threat = visible
        .threats
        .iter()
        .find(|threat| {
            engine::threat_preview(campaign, data, campaign.player, &[army], threat.id).is_ok()
        })
        .map(|threat| threat.id);
    let Some(threat) = threat else {
        return Ok(false);
    };
    issue(
        campaign,
        data,
        Command::ClearThreat {
            armies: vec![army],
            threat,
        },
    )?;
    issue(
        campaign,
        data,
        Command::SetBattleDoctrine {
            army,
            doctrine: BattleDoctrine::RangedSupport,
        },
    )?;
    let archer = campaign.armies[&army]
        .slots
        .iter()
        .position(|id| id.is_some_and(|id| campaign.formations[&id].kind == TroopKind::Archers));
    if let Some(slot) = archer.filter(|slot| *slot < 3) {
        issue(
            campaign,
            data,
            Command::SwapFormationSlots {
                army,
                first: slot as u8,
                second: 5,
            },
        )?;
    }
    if let Some(leader) = campaign.armies[&army].commander {
        if let Some(formation) = campaign
            .formations
            .values()
            .find(|formation| {
                campaign.people[&leader].assignment
                    == kestrum::state::people::PersonAssignment::Formation {
                        formation: formation.id,
                    }
            })
            .map(|formation| formation.id)
        {
            let command = Command::SetBattleLeader {
                formation,
                leader: Some(leader),
            };
            if engine::preview(campaign, data, Actor::Player, command.clone()).is_ok() {
                issue(campaign, data, command)?;
            }
        }
    }
    issue(
        campaign,
        data,
        Command::SaveBattleTemplate {
            army,
            name: "Homeward Line".into(),
        },
    )?;
    Ok(true)
}

fn approach_threat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army: ArmyId,
) -> Result<bool, String> {
    let origin = campaign.armies[&army].site;
    // The review route chooses a safe staging site; all travel still spends normal
    // movement, updates knowledge and passes the same command validation as play.
    let destination = campaign
        .threats
        .values()
        .filter(|threat| threat.status == kestrum::state::threat::ThreatStatus::Active)
        .flat_map(|threat| campaign.world.adjacent_sites(threat.site))
        .filter(|site| *site != origin)
        .filter_map(|site| {
            engine::movement_preview(campaign, data, campaign.player, &[army], site)
                .ok()
                .filter(|preview| preview.reachable_steps > 0 && preview.encounter.is_none())
        })
        .min_by_key(|preview| (preview.total_cost, preview.order.path.last().copied()));
    if let Some(preview) = destination {
        issue(campaign, data, Command::Move(preview.order))?;
        return Ok(true);
    }
    Ok(false)
}

/// Surviving troops; the review script may read every faction's totals.
fn troops(campaign: &StrategicCampaign, faction: kestrum::data::world::FactionId) -> u32 {
    campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction)
        .map(|formation| formation.headcount)
        .sum()
}

/// Make war on the weakest neighbor once the realm has room to grow, and offer
/// peace to a stronger enemy; a rival accepts peace only on its own terms.
fn diplomacy(campaign: &mut StrategicCampaign, data: &GameData) -> Result<(), String> {
    use kestrum::data::world::DiplomaticState;
    let player = campaign.player;
    let enemies: Vec<_> = campaign
        .relations
        .iter()
        .filter(|relation| {
            relation.state == DiplomaticState::War && relation.factions.contains(&player)
        })
        .flat_map(|relation| relation.factions)
        .filter(|faction| *faction != player)
        .collect();
    for faction in enemies.iter().copied() {
        if troops(campaign, faction) > troops(campaign, player) {
            let command = Command::OfferPeace { faction };
            if engine::preview(campaign, data, Actor::Player, command.clone()).is_ok() {
                issue(campaign, data, command)?;
            }
        }
    }
    if !enemies.is_empty() || campaign.completed_rounds < WAR_AFTER_ROUNDS {
        return Ok(());
    }
    let neighbors: std::collections::BTreeSet<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(player))
        .flat_map(|site| campaign.world.adjacent_sites(site.id))
        .filter_map(|site| campaign.world.site(site)?.controller)
        .filter(|owner| *owner != player && campaign.is_independent(*owner))
        .collect();
    let target = neighbors
        .into_iter()
        .filter(|faction| troops(campaign, *faction) < troops(campaign, player))
        .min_by_key(|faction| (troops(campaign, *faction), *faction));
    if let Some(faction) = target {
        let command = Command::DeclareWar { faction };
        if engine::preview(campaign, data, Actor::Player, command.clone()).is_ok() {
            issue(campaign, data, command)?;
        }
    }
    Ok(())
}

fn issue(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    command: Command,
) -> Result<(), String> {
    apply(campaign, data, Actor::Player, command)
        .map(|_| ())
        .map_err(|error| {
            format!(
                "Review command failed at round {}: {error}",
                campaign.completed_rounds
            )
        })
}
