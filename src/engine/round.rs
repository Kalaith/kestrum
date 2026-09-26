//! Stable-ID faction phases and one atomic seasonal boundary.

use super::{ActionOutcome, RuleError};
use crate::state::{
    campaign::{DomainFact, DomainFactKind, FactId},
    CampaignPhase, StrategicCampaign,
};

pub(super) fn pass_faction(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let faction = campaign.active_faction();
    let fact = DomainFact {
        id: campaign.next_ids.fact,
        sequence: campaign.accepted_sequence,
        completed_rounds: campaign.completed_rounds,
        kind: DomainFactKind::FactionPassed { faction },
    };
    campaign.next_ids.fact = FactId(campaign.next_ids.fact.0.checked_add(1).ok_or(
        RuleError::Overflow {
            field: "fact identifiers",
        },
    )?);
    campaign.acted.insert(faction);
    campaign.pending_facts.push(fact.clone());
    outcome.facts.push(fact);
    let next = campaign
        .round_order
        .iter()
        .copied()
        .find(|id| campaign.is_independent(*id) && !campaign.acted.contains(id));
    if let Some(next_faction) = next {
        let paused = matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. });
        campaign.phase = CampaignPhase::NpcTurn {
            faction: next_faction,
            paused,
        };
    } else {
        complete_round(campaign, outcome)?;
    }
    Ok(())
}

fn complete_round(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    // Only the calendar and phase facts exist in K02. Later packages insert
    // their real systems in P02 order; passing never invents income or service.
    campaign.completed_rounds =
        campaign
            .completed_rounds
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "completed rounds",
            })?;
    outcome.consumed_facts = std::mem::take(&mut campaign.pending_facts);
    campaign.consumed_sequence = campaign.accepted_sequence;
    campaign.acted.clear();
    campaign.round_order = campaign.independent_order();
    campaign.phase = CampaignPhase::PlayerTurn;
    outcome.round_completed = true;
    Ok(())
}
