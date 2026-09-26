//! Validated transactional commands shared by player and NPC phase processing.

use super::round;
use crate::{
    data::{world::FactionId, GameData},
    state::{campaign::DomainFact, CampaignPhase, StrategicCampaign},
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Player,
    Npc(FactionId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    EndTurn,
    SetNpcPaused(bool),
    StepNpc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleError {
    NoCampaign,
    LegacyReadOnly,
    PlayObstructed,
    UnknownActor,
    WrongActor,
    NotYourTurn { active: FactionId },
    NotNpcPhase,
    NpcPaused,
    PauseRequired,
    PauseUnchanged,
    Overflow { field: &'static str },
    InvalidState(String),
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCampaign => formatter.write_str("Start or load a campaign first."),
            Self::LegacyReadOnly => formatter.write_str(
                "This empty-atlas save is read-only. Start a strategic campaign to play.",
            ),
            Self::PlayObstructed => {
                formatter.write_str("Close the open panel and return to the campaign first.")
            }
            Self::UnknownActor => formatter.write_str("That faction is unavailable."),
            Self::WrongActor => formatter.write_str("That faction cannot issue this command."),
            Self::NotYourTurn { .. } => formatter.write_str("Wait for your faction's turn."),
            Self::NotNpcPhase => formatter.write_str("NPC phases have already finished."),
            Self::NpcPaused => formatter.write_str("NPC phases are paused. Tap STEP or RESUME."),
            Self::PauseRequired => {
                formatter.write_str("Pause NPC phases before taking a single step.")
            }
            Self::PauseUnchanged => {
                formatter.write_str("NPC progression is already in that state.")
            }
            Self::Overflow { field } => write!(
                formatter,
                "The campaign cannot advance: {field} is exhausted."
            ),
            Self::InvalidState(reason) => {
                write!(formatter, "The campaign could not be validated: {reason}")
            }
        }
    }
}

impl std::error::Error for RuleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutcome {
    pub accepted_sequence: u64,
    pub active_faction: FactionId,
    pub round_completed: bool,
    pub facts: Vec<DomainFact>,
    pub consumed_facts: Vec<DomainFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPreview {
    pub active_faction_after: FactionId,
    pub round_completed: bool,
}

pub fn preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<ActionPreview, RuleError> {
    let (_, outcome) = prepare(campaign, data, actor, command)?;
    Ok(ActionPreview {
        active_faction_after: outcome.active_faction,
        round_completed: outcome.round_completed,
    })
}

pub fn apply(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<ActionOutcome, RuleError> {
    let (candidate, outcome) = prepare(campaign, data, actor, command)?;
    *campaign = candidate;
    Ok(outcome)
}

/// Runtime pacing requests exactly one legal NPC action; frame time is not simulation input.
pub fn advance_npc(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    let CampaignPhase::NpcTurn { faction, .. } = campaign.phase else {
        return Err(RuleError::NotNpcPhase);
    };
    apply(campaign, data, Actor::Npc(faction), Command::EndTurn)
}

fn prepare(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<(StrategicCampaign, ActionOutcome), RuleError> {
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    validate_command(campaign, actor, command)?;
    let mut candidate = campaign.clone();
    candidate.accepted_sequence =
        candidate
            .accepted_sequence
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "accepted action sequence",
            })?;
    let mut outcome = ActionOutcome {
        accepted_sequence: candidate.accepted_sequence,
        active_faction: candidate.active_faction(),
        round_completed: false,
        facts: Vec::new(),
        consumed_facts: Vec::new(),
    };
    match command {
        Command::EndTurn | Command::StepNpc => round::pass_faction(&mut candidate, &mut outcome)?,
        Command::SetNpcPaused(paused) => {
            candidate.phase = CampaignPhase::NpcTurn {
                faction: candidate.active_faction(),
                paused,
            };
        }
    }
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    outcome.active_faction = candidate.active_faction();
    Ok((candidate, outcome))
}

fn validate_command(
    campaign: &StrategicCampaign,
    actor: Actor,
    command: Command,
) -> Result<(), RuleError> {
    let faction = match actor {
        Actor::Player => campaign.player,
        Actor::Npc(faction) if faction != campaign.player => faction,
        Actor::Npc(_) => return Err(RuleError::WrongActor),
    };
    if !campaign.is_independent(faction) {
        return Err(RuleError::UnknownActor);
    }
    match command {
        Command::EndTurn => {
            if faction != campaign.active_faction() {
                return Err(RuleError::NotYourTurn {
                    active: campaign.active_faction(),
                });
            }
            if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
                return Err(RuleError::NpcPaused);
            }
        }
        Command::SetNpcPaused(requested) => {
            let paused = player_npc_control(campaign, actor)?;
            if requested == paused {
                return Err(RuleError::PauseUnchanged);
            }
        }
        Command::StepNpc => {
            if !player_npc_control(campaign, actor)? {
                return Err(RuleError::PauseRequired);
            }
        }
    }
    Ok(())
}

fn player_npc_control(campaign: &StrategicCampaign, actor: Actor) -> Result<bool, RuleError> {
    if actor != Actor::Player {
        return Err(RuleError::WrongActor);
    }
    match campaign.phase {
        CampaignPhase::NpcTurn { paused, .. } => Ok(paused),
        CampaignPhase::PlayerTurn => Err(RuleError::NotNpcPhase),
    }
}
