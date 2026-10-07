//! Deterministic AI-only stepping and automatic observer decisions.

use super::{actions, ActionOutcome, Actor, Command, RuleError};
use crate::{
    data::GameData,
    state::{CampaignPhase, StrategicCampaign},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserverStepOutcome {
    pub action: String,
    pub diagnostics: Vec<String>,
    pub outcome: ActionOutcome,
}

pub fn advance_observer(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    Ok(advance_observer_without_diagnostics(campaign, data)?.outcome)
}

pub fn advance_observer_without_diagnostics(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ObserverStepOutcome, RuleError> {
    advance_observer_inner(campaign, data, false)
}

pub fn advance_observer_diagnosed(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ObserverStepOutcome, RuleError> {
    advance_observer_inner(campaign, data, true)
}

fn advance_observer_inner(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    include_diagnostics: bool,
) -> Result<ObserverStepOutcome, RuleError> {
    require_observer(campaign)?;
    if campaign.observer_finished() {
        return Err(RuleError::NotNpcPhase);
    }
    if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
        return Err(RuleError::NpcPaused);
    }
    if campaign.pending_battle.is_some() {
        let battle = campaign
            .pending_battle
            .as_ref()
            .map(|battle| battle.report.id);
        return diagnosed_action(
            campaign,
            data,
            Actor::Player,
            Command::StartPendingBattle,
            format!("Resolve pending battle {battle:?}"),
        );
    }
    if let Some(offer) = campaign.diplomacy.pending_offers.first() {
        let recipient = offer.recipient;
        let proposer = offer.proposer;
        let accept = super::diplomacy::peace_desired(campaign, data, recipient, proposer);
        return diagnosed_action(
            campaign,
            data,
            Actor::Npc(recipient),
            Command::RespondPeace { proposer, accept },
            format!(
                "{} peace offer from faction #{}",
                if accept { "Accept" } else { "Decline" },
                proposer.0
            ),
        );
    }
    if let Some(defeat) = campaign.diplomacy.pending_defeats.first() {
        let faction = defeat.faction;
        let victor = defeat.victor;
        return diagnosed_action(
            campaign,
            data,
            Actor::Npc(victor),
            Command::ResolveDefeat {
                faction,
                resolution: crate::state::diplomacy::DefeatResolution::Annex,
            },
            format!("Annex defeated faction #{}", faction.0),
        );
    }
    if include_diagnostics {
        actions::advance_npc_action_diagnosed(campaign, data)
    } else {
        actions::advance_npc_action_without_diagnostics(campaign, data)
    }
}

fn diagnosed_action(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
    label: String,
) -> Result<ObserverStepOutcome, RuleError> {
    let outcome = actions::apply(campaign, data, actor, command)?;
    Ok(ObserverStepOutcome {
        action: label,
        diagnostics: Vec::new(),
        outcome,
    })
}

pub fn step_observer(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    Ok(step_observer_inner(campaign, data, false)?.outcome)
}

pub fn step_observer_diagnosed(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ObserverStepOutcome, RuleError> {
    step_observer_inner(campaign, data, true)
}

fn step_observer_inner(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    include_diagnostics: bool,
) -> Result<ObserverStepOutcome, RuleError> {
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
    let outcome = advance_observer_inner(&mut candidate, data, include_diagnostics)?;
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
