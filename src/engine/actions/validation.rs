//! Phase, ownership and pending-decision guards shared by all commands.
use super::*;

pub(in crate::engine) fn validate_command(
    campaign: &StrategicCampaign,
    actor: Actor,
    command: &Command,
) -> Result<(), RuleError> {
    let faction = match actor {
        Actor::Player => campaign.player,
        Actor::Npc(faction) if faction != campaign.player => faction,
        Actor::Npc(_) => return Err(RuleError::WrongActor),
    };
    if campaign.diplomacy.ending.is_some() {
        return Err(RuleError::Diplomacy(
            "This campaign has ended. Its records and saves remain available.".into(),
        ));
    }
    let decision = matches!(
        command,
        Command::RespondPeace { .. } | Command::ResolveDefeat { .. }
    );
    if campaign.diplomacy.has_pending_decision() && !decision {
        return Err(RuleError::Diplomacy(
            "Resolve the pending kingdom decision before issuing orders.".into(),
        ));
    }
    if !campaign.is_independent(faction) {
        return Err(RuleError::UnknownActor);
    }
    match command {
        Command::RespondPeace { .. } | Command::ResolveDefeat { .. } => {
            if actor != Actor::Player {
                return Err(RuleError::WrongActor);
            }
        }
        Command::EndTurn
        | Command::DeclareWar { .. }
        | Command::OfferPeace { .. }
        | Command::ClearThreat { .. }
        | Command::Resettle { .. }
        | Command::RenameSite { .. }
        | Command::MoveCapital { .. }
        | Command::RelocateHeadquarters { .. }
        | Command::Siege(_)
        | Command::Recruit { .. }
        | Command::Disband { .. }
        | Command::Move(_)
        | Command::StartConstruction { .. }
        | Command::CancelConstruction { .. }
        | Command::ReassignBuilder { .. }
        | Command::SetFocus { .. } => {
            if faction != campaign.active_faction() {
                return Err(RuleError::NotYourTurn {
                    active: campaign.active_faction(),
                });
            }
            if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
                return Err(RuleError::NpcPaused);
            }
        }
        Command::TransferFormation { .. }
        | Command::TransferPerson { .. }
        | Command::SplitArmy { .. } => {
            if faction != campaign.active_faction()
                && !matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. })
            {
                return Err(RuleError::TransferPauseRequired);
            }
        }
        Command::SetNpcPaused(requested) => {
            let paused = player_npc_control(campaign, actor)?;
            if *requested == paused {
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
