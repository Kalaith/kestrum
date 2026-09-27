//! Capture and military destruction precede symmetric, persistent defeat resolution.

use super::*;
use crate::state::{
    battle::BattleOutcome,
    construction::{CancellationReason, ConstructionStatus},
    people::{PersonAssignment, PersonStatus},
};

pub(crate) fn reconcile(
    before: &StrategicCampaign,
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if campaign.diplomacy.ending.is_some() {
        return Ok(());
    }
    remember_losses(before, campaign, data);
    let defeated: Vec<_> = campaign
        .factions
        .keys()
        .copied()
        .filter(|id| campaign.is_independent(*id) && campaign.defeat_eligible(*id))
        .collect();
    let player_defeated = defeated.contains(&campaign.player);
    for faction in defeated {
        let victor = campaign
            .diplomacy
            .last_attackers
            .get(&faction)
            .copied()
            .filter(|id| {
                *id != faction && campaign.is_independent(*id) && !campaign.defeat_eligible(*id)
            });
        if faction != campaign.player && !player_defeated && victor == Some(campaign.player) {
            let pending = PendingDefeat {
                faction,
                victor: campaign.player,
                completed_rounds: campaign.completed_rounds,
            };
            campaign.diplomacy.pending_defeats.push(pending.clone());
            retire_survivors(campaign, faction);
            cancel_orders(campaign, faction, outcome)?;
            inherit(campaign, faction, victor, outcome)?;
            record(
                campaign,
                outcome,
                DiplomacyReceipt::DefeatPending {
                    faction,
                    victor: pending.victor,
                },
            )?;
        } else {
            resolve(campaign, faction, victor, DefeatResolution::Annex, outcome)?;
        }
    }
    campaign
        .diplomacy
        .pending_defeats
        .sort_by_key(|entry| entry.faction);
    campaign.diplomacy.pending_offers.retain(|offer| {
        campaign.factions[&offer.proposer].status == FactionStatus::Independent
            && campaign.factions[&offer.recipient].status == FactionStatus::Independent
            && !campaign
                .diplomacy
                .pending_defeats
                .iter()
                .any(|defeat| [offer.proposer, offer.recipient].contains(&defeat.faction))
    });
    if player_defeated {
        end(campaign, EndingKind::Defeat, outcome)?;
    } else {
        check_victory(campaign, outcome)?;
    }
    Ok(())
}

fn remember_losses(before: &StrategicCampaign, campaign: &mut StrategicCampaign, data: &GameData) {
    for site in &before.world.sites {
        let Some(loser) = site.controller else {
            continue;
        };
        let Some(victor) = campaign
            .world
            .site(site.id)
            .and_then(|site| site.controller)
            .filter(|id| *id != loser)
        else {
            continue;
        };
        campaign.diplomacy.last_attackers.insert(loser, victor);
        if site.habitation != crate::data::economy::Habitation::Unsettled
            && !before.site_is_ruined(site.id)
        {
            campaign.diplomacy.losses.push(SiteLoss {
                faction: loser,
                victor,
                site: site.id,
                completed_rounds: campaign.completed_rounds,
            });
        }
    }
    for report in campaign
        .battles
        .values()
        .filter(|report| report.id >= before.next_ids.battle)
    {
        let Some(defender) = report.defender.faction() else {
            continue;
        };
        match report.outcome {
            BattleOutcome::AttackerVictory => {
                campaign
                    .diplomacy
                    .last_attackers
                    .insert(defender, report.attacker.faction);
            }
            BattleOutcome::DefenderVictory => {
                campaign
                    .diplomacy
                    .last_attackers
                    .insert(report.attacker.faction, defender);
            }
            BattleOutcome::MutualDestruction => {
                campaign
                    .diplomacy
                    .last_attackers
                    .insert(defender, report.attacker.faction);
                campaign
                    .diplomacy
                    .last_attackers
                    .insert(report.attacker.faction, defender);
            }
            BattleOutcome::Stalemate => {}
        }
    }
    campaign.diplomacy.losses.retain(|loss| {
        campaign
            .completed_rounds
            .saturating_sub(loss.completed_rounds)
            < data.diplomacy.recent_loss_rounds
    });
    campaign
        .diplomacy
        .losses
        .sort_by_key(|loss| (loss.completed_rounds, loss.faction, loss.victor, loss.site));
    campaign.diplomacy.losses.dedup();
}

pub(super) fn resolve_choice(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    faction: FactionId,
    resolution: DefeatResolution,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if owner != campaign.player
        || !campaign
            .diplomacy
            .pending_defeats
            .iter()
            .any(|pending| pending.faction == faction && pending.victor == owner)
    {
        return Err(error("This kingdom is not awaiting your defeat decision."));
    }
    if !campaign.defeat_eligible(faction) {
        return Err(error(
            "The defeated kingdom still has an independent military base.",
        ));
    }
    campaign
        .diplomacy
        .pending_defeats
        .retain(|pending| pending.faction != faction);
    resolve(campaign, faction, Some(owner), resolution, outcome)?;
    check_victory(campaign, outcome)
}

fn resolve(
    campaign: &mut StrategicCampaign,
    faction: FactionId,
    victor: Option<FactionId>,
    resolution: DefeatResolution,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    campaign.factions.get_mut(&faction).expect("faction").status = match resolution {
        DefeatResolution::Annex => FactionStatus::Eliminated,
        DefeatResolution::Submission => FactionStatus::Vassal {
            sovereign: victor.ok_or_else(|| error("Submission needs a surviving sovereign."))?,
        },
    };
    retire_survivors(campaign, faction);
    cancel_orders(campaign, faction, outcome)?;
    inherit(campaign, faction, victor, outcome)?;
    record(
        campaign,
        outcome,
        DiplomacyReceipt::FactionResolved {
            faction,
            victor,
            resolution,
        },
    )
}

fn retire_survivors(campaign: &mut StrategicCampaign, faction: FactionId) {
    for person in campaign
        .people
        .values_mut()
        .filter(|person| person.faction == faction && person.is_alive())
    {
        let site = match person.assignment {
            PersonAssignment::Site { site } => site,
            PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => site,
            PersonAssignment::Formation { formation } => campaign
                .armies
                .values()
                .find(|army| army.formation_ids().any(|id| id == formation))
                .map_or(campaign.factions[&faction].headquarters, |army| army.site),
            PersonAssignment::Dead => continue,
        };
        person.assignment = PersonAssignment::Site { site };
        person.status = PersonStatus::Displaced {
            completed_rounds: campaign.completed_rounds,
            site,
        };
        person.movement_spent = 0;
    }
}

fn cancel_orders(
    campaign: &mut StrategicCampaign,
    faction: FactionId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let ids: Vec<_> = campaign
        .construction
        .values()
        .filter(|order| order.owner == faction && order.is_open())
        .map(|order| order.id)
        .collect();
    for id in ids {
        let order = campaign.construction.get_mut(&id).expect("order");
        order.status = ConstructionStatus::Cancelled {
            completed_rounds: campaign.completed_rounds,
            reason: CancellationReason::FactionDefeated,
        };
        let receipt = order.clone();
        record_fact(
            campaign,
            outcome,
            DomainFactKind::ConstructionChanged { order: receipt },
        )?;
    }
    campaign.trim_terminal_orders();
    Ok(())
}

fn inherit(
    campaign: &mut StrategicCampaign,
    former: FactionId,
    victor: Option<FactionId>,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let heirs: Vec<_> = campaign
        .factions
        .values()
        .filter(|faction| faction.status == FactionStatus::Vassal { sovereign: former })
        .map(|faction| faction.id)
        .collect();
    for faction in heirs {
        campaign.factions.get_mut(&faction).expect("vassal").status = match victor {
            Some(sovereign) => FactionStatus::Vassal { sovereign },
            None => FactionStatus::Eliminated,
        };
        record(
            campaign,
            outcome,
            DiplomacyReceipt::FactionResolved {
                faction,
                victor,
                resolution: if victor.is_some() {
                    DefeatResolution::Submission
                } else {
                    DefeatResolution::Annex
                },
            },
        )?;
    }
    Ok(())
}
fn check_victory(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if campaign.diplomacy.pending_defeats.is_empty()
        && campaign
            .factions
            .values()
            .filter(|faction| faction.id != campaign.player)
            .all(|faction| {
                faction.status == FactionStatus::Eliminated
                    || faction.status
                        == FactionStatus::Vassal {
                            sovereign: campaign.player,
                        }
            })
    {
        end(campaign, EndingKind::Victory, outcome)?;
    }
    Ok(())
}
fn end(
    campaign: &mut StrategicCampaign,
    kind: EndingKind,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let ending = CampaignEnding {
        kind,
        completed_rounds: campaign.completed_rounds,
    };
    campaign.diplomacy.ending = Some(ending.clone());
    campaign.diplomacy.pending_defeats.clear();
    campaign.diplomacy.pending_offers.clear();
    record(
        campaign,
        outcome,
        DiplomacyReceipt::CampaignEnded { ending },
    )
}
