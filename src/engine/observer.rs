//! Deterministic AI-only stepping and automatic observer decisions.

use super::{actions, ActionOutcome, Actor, Command, RuleError};
use crate::{
    data::GameData,
    state::{CampaignPhase, StrategicCampaign},
};

pub fn advance_observer(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    require_observer(campaign)?;
    if campaign.observer_finished() {
        return Err(RuleError::NotNpcPhase);
    }
    if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
        return Err(RuleError::NpcPaused);
    }
    if campaign.pending_battle.is_some() {
        return actions::apply(campaign, data, Actor::Player, Command::StartPendingBattle);
    }
    if let Some(offer) = campaign.diplomacy.pending_offers.first() {
        let recipient = offer.recipient;
        let proposer = offer.proposer;
        let accept = super::diplomacy::peace_desired(campaign, data, recipient, proposer);
        return actions::apply(
            campaign,
            data,
            Actor::Npc(recipient),
            Command::RespondPeace { proposer, accept },
        );
    }
    if let Some(defeat) = campaign.diplomacy.pending_defeats.first() {
        let faction = defeat.faction;
        let victor = defeat.victor;
        return actions::apply(
            campaign,
            data,
            Actor::Npc(victor),
            Command::ResolveDefeat {
                faction,
                resolution: crate::state::diplomacy::DefeatResolution::Annex,
            },
        );
    }
    actions::advance_npc_action(campaign, data)
}

pub fn step_observer(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    require_observer(campaign)?;
    let CampaignPhase::NpcTurn { faction, paused } = campaign.phase else {
        return Err(RuleError::PauseRequired);
    };
    if !paused {
        return Err(RuleError::PauseRequired);
    }
    let mut candidate = campaign.clone();
    candidate.phase = CampaignPhase::NpcTurn {
        faction,
        paused: false,
    };
    let outcome = advance_observer(&mut candidate, data)?;
    if let CampaignPhase::NpcTurn { paused, .. } = &mut candidate.phase {
        *paused = true;
    }
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    *campaign = candidate;
    Ok(outcome)
}

fn require_observer(campaign: &StrategicCampaign) -> Result<(), RuleError> {
    if campaign.observer_mode {
        Ok(())
    } else {
        Err(RuleError::NotObserverCampaign)
    }
}
