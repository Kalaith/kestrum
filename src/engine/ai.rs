//! Deterministic bounded planning over the same observer view and commands as play.

mod economy;
mod intent;
mod military;
mod politics;
mod travel;

use super::{preview, project, Actor, Command, RuleError, VisibleCampaign};
use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    state::{ai::*, CampaignPhase, StrategicCampaign},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiDecision {
    pub command: Command,
    pub objective: Option<AiObjective>,
    pub intent: AiIntent,
}

struct Planner<'a> {
    campaign: &'a StrategicCampaign,
    data: &'a GameData,
    owner: FactionId,
    view: VisibleCampaign,
    objective: Option<AiObjective>,
}

pub fn propose(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
) -> Result<AiDecision, RuleError> {
    data.validate().map_err(RuleError::InvalidState)?;
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    if !matches!(campaign.phase, CampaignPhase::NpcTurn{faction: active, paused:false} if active==faction)
    {
        return Err(RuleError::NotNpcPhase);
    }
    if !campaign.is_independent(faction) || campaign.diplomacy.is_blocked() {
        return Err(RuleError::Diplomacy(
            "Resolve the pending sovereign decision first.".into(),
        ));
    }
    data.ai.validate().map_err(RuleError::InvalidState)?;
    let view = project(campaign, faction)?;
    let mut planner = Planner {
        campaign,
        data,
        owner: faction,
        view,
        objective: None,
    };
    planner.objective = planner.retained_objective();
    if campaign.ai.factions.get(&faction).is_some_and(|state| {
        state.phase_round == campaign.completed_rounds
            && state.accepted_commands >= data.ai.max_commands_per_phase
    }) {
        return Ok(planner.pass());
    }
    let emergency = planner.threatened_headquarters();
    if emergency {
        planner.objective = Some(planner.objective(
            campaign.factions[&faction].headquarters,
            AiObjectiveKind::Defend,
        ));
    }
    let decision = planner
        .defend()
        .or_else(|| planner.retreat())
        .or_else(|| planner.recruit(emergency))
        .or_else(|| planner.construct())
        .or_else(|| planner.peace())
        .or_else(|| {
            planner
                .objective
                .as_ref()
                .map(|_| planner.pursue().unwrap_or_else(|| planner.pass()))
        })
        .or_else(|| planner.clear_threat())
        .or_else(|| planner.expand())
        .or_else(|| planner.attack())
        .or_else(|| planner.declare_war())
        .or_else(|| planner.border());
    Ok(decision.unwrap_or_else(|| planner.pass()))
}

/// Called only after the ordinary command succeeds in the parent's atomic candidate.
pub fn accepted(
    candidate: &mut StrategicCampaign,
    before: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    decision: &AiDecision,
) -> Result<(), RuleError> {
    let state = candidate.ai.factions.entry(faction).or_default();
    if state.phase_round != before.completed_rounds {
        state.phase_round = before.completed_rounds;
        state.accepted_commands = 0;
    }
    if candidate.completed_rounds == before.completed_rounds
        && candidate.phase == before.phase
        && !matches!(decision.command, Command::EndTurn)
    {
        state.accepted_commands =
            state
                .accepted_commands
                .checked_add(1)
                .ok_or(RuleError::Overflow {
                    field: "AI command budget",
                })?;
    }
    state.objective = decision.objective.clone();
    state.rejected.clear();
    state.rejected_at_sequence = candidate.accepted_sequence;
    candidate.validate_ai(data).map_err(RuleError::InvalidState)
}

/// A failed intent is not attempted again against the same accepted state.
pub fn rejected(
    candidate: &mut StrategicCampaign,
    before: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    decision: &AiDecision,
) -> Result<(), RuleError> {
    let state = candidate.ai.factions.entry(faction).or_default();
    if state.rejected_at_sequence != before.accepted_sequence {
        state.rejected.clear();
    }
    state.phase_round = before.completed_rounds;
    state.rejected_at_sequence = before.accepted_sequence;
    if state.rejected.len() < data.ai.max_commands_per_phase as usize {
        state.rejected.insert(decision.intent.clone());
    }
    candidate.validate_ai(data).map_err(RuleError::InvalidState)
}

impl Planner<'_> {
    fn choose(&self, command: Command, objective: Option<AiObjective>) -> Option<AiDecision> {
        let intent = intent::of(&command)?;
        if self
            .campaign
            .ai
            .factions
            .get(&self.owner)
            .is_some_and(|state| {
                state.rejected_at_sequence == self.campaign.accepted_sequence
                    && state.rejected.contains(&intent)
            })
            || preview(
                self.campaign,
                self.data,
                Actor::Npc(self.owner),
                command.clone(),
            )
            .is_err()
        {
            return None;
        }
        Some(AiDecision {
            command,
            objective,
            intent,
        })
    }

    fn pass(&self) -> AiDecision {
        AiDecision {
            command: Command::EndTurn,
            objective: self.objective.clone(),
            intent: AiIntent {
                kind: AiIntentKind::Pass,
                targets: Vec::new(),
            },
        }
    }

    fn objective(&self, site: SiteId, kind: AiObjectiveKind) -> AiObjective {
        self.objective
            .as_ref()
            .filter(|old| old.site == site && old.kind == kind)
            .cloned()
            .unwrap_or(AiObjective {
                site,
                kind,
                chosen_round: self.campaign.completed_rounds,
            })
    }

    fn retained_objective(&self) -> Option<AiObjective> {
        let objective = self
            .campaign
            .ai
            .factions
            .get(&self.owner)?
            .objective
            .as_ref()?;
        if self
            .campaign
            .completed_rounds
            .saturating_sub(objective.chosen_round)
            >= self.data.ai.objective_rounds
        {
            return None;
        }
        let site = self.view.world.site(objective.site)?;
        let valid = match objective.kind {
            AiObjectiveKind::Expand => {
                site.controller.is_none()
                    && !self.view.hostile_presence.contains(&site.id)
                    && !self
                        .view
                        .threats
                        .iter()
                        .any(|threat| threat.site == site.id)
            }
            AiObjectiveKind::Defend => {
                site.controller == Some(self.owner) && self.threatened(site.id)
            }
            AiObjectiveKind::Threat => self
                .view
                .threats
                .iter()
                .any(|threat| threat.site == site.id),
            AiObjectiveKind::Attack => site.controller.is_some_and(|owner| self.at_war(owner)),
            AiObjectiveKind::Border => site.controller == Some(self.owner),
        };
        valid.then(|| objective.clone())
    }
}
