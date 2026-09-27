//! Stable-ID faction phases and one atomic seasonal boundary.

use super::{actions::record_fact, economy, person_combat, recovery, ActionOutcome, RuleError};
use crate::{
    data::GameData,
    state::{campaign::DomainFactKind, CampaignPhase, StrategicCampaign},
};

pub(super) fn pass_faction(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let faction = campaign.active_faction();
    record_fact(campaign, outcome, DomainFactKind::FactionPassed { faction })?;
    campaign.acted.insert(faction);
    reconcile_phase(campaign, data, outcome)
}

pub(super) fn reconcile_phase(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if campaign.diplomacy.is_blocked() {
        return Ok(());
    }
    let next = campaign
        .round_order
        .iter()
        .copied()
        .find(|id| campaign.is_independent(*id) && !campaign.acted.contains(id));
    if let Some(next_faction) = next {
        let paused = matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. });
        campaign.phase = if next_faction == campaign.player {
            CampaignPhase::PlayerTurn
        } else {
            CampaignPhase::NpcTurn {
                faction: next_faction,
                paused,
            }
        };
    } else {
        complete_round(campaign, data, outcome)?;
    }
    Ok(())
}

fn complete_round(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    // Income and upkeep precede recovery and the new season's calendar.
    super::siege::reconcile(campaign, data, outcome)?;
    let supply = recovery::snapshot(campaign);
    let development = super::development::snapshot(campaign, data);
    let medics = super::evidence::recovery_medics(campaign);
    economy::resolve(campaign, data)?;
    super::construction::resolve(campaign, data, &supply, outcome)?;
    super::siege::progress(campaign, data, outcome)?;
    recovery::resolve(campaign, data, &supply)?;
    person_combat::heal_wounds(campaign, &supply);
    super::development::resolve(campaign, data, &development, outcome)?;
    campaign.completed_rounds =
        campaign
            .completed_rounds
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "completed rounds",
            })?;
    super::construction::reconcile(campaign, data, outcome)?;
    super::evidence::record_recovery(campaign, &medics)?;
    super::knowledge::prune_knowledge(campaign, data);
    campaign.acted.clear();
    campaign.round_order = campaign.independent_order();
    campaign.phase = CampaignPhase::PlayerTurn;
    for formation in campaign.formations.values_mut() {
        formation.movement_spent = 0;
    }
    for person in campaign.people.values_mut() {
        person.movement_spent = 0;
    }
    outcome.round_completed = true;
    Ok(())
}
