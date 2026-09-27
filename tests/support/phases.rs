//! Seasonal subsystem fixtures pass ordinary NPC turns explicitly; K12 tests exercise policy.
use kestrum::{
    data::GameData,
    engine::{apply, ActionOutcome, Actor, Command, RuleError},
    state::{CampaignPhase, StrategicCampaign},
};
pub fn pass_npc(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    let CampaignPhase::NpcTurn { faction, .. } = campaign.phase else {
        return Err(RuleError::NotNpcPhase);
    };
    apply(campaign, data, Actor::Npc(faction), Command::EndTurn)
}
