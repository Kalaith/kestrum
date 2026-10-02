//! A paused step uses the same single accepted policy action as automatic play.
use super::*;
pub(super) fn step_npc(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
) -> Result<ActionOutcome, RuleError> {
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    validate_command(campaign, actor, &Command::StepNpc)?;
    if campaign.observer_mode {
        return super::super::observer::step_observer(campaign, data);
    }
    let mut candidate = campaign.clone();
    candidate.phase = CampaignPhase::NpcTurn {
        faction: candidate.active_faction(),
        paused: false,
    };
    let outcome = advance_npc(&mut candidate, data)?;
    if let CampaignPhase::NpcTurn { paused, .. } = &mut candidate.phase {
        *paused = true;
    }
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    *campaign = candidate;
    Ok(outcome)
}
