//! Deterministic bounded planning over the same observer view and commands as play.

mod economy;
mod eligibility;
mod expansion;
mod intent;
mod military;
mod objectives;
pub use objectives::{rank_targets, AiTarget};
mod politics;
mod progression;
mod recovery;
mod routes;
mod travel;
pub use routes::ObservedRoutes;

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
    pub diagnostics: Vec<String>,
}

#[derive(Default)]
struct CandidateTrace {
    evaluated: u32,
    rejection_counts: std::collections::BTreeMap<String, u32>,
    examples: Vec<String>,
}

struct Planner<'a> {
    campaign: &'a StrategicCampaign,
    data: &'a GameData,
    owner: FactionId,
    view: VisibleCampaign,
    objective: Option<AiObjective>,
    routes: ObservedRoutes,
    rejected_candidates: std::cell::RefCell<Vec<Command>>,
    include_diagnostics: bool,
    candidate_trace: std::cell::RefCell<CandidateTrace>,
}

pub fn propose(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
) -> Result<AiDecision, RuleError> {
    propose_with_diagnostics(campaign, data, faction, false)
}

pub fn propose_diagnosed(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
) -> Result<AiDecision, RuleError> {
    propose_with_diagnostics(campaign, data, faction, true)
}

fn propose_with_diagnostics(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    include_diagnostics: bool,
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
    let enemies = campaign
        .relations
        .iter()
        .filter(|relation| {
            relation.state == crate::data::world::DiplomaticState::War
                && relation.factions.contains(&faction)
        })
        .flat_map(|relation| relation.factions)
        .filter(|other| *other != faction)
        .collect();
    let routes = ObservedRoutes::new(&view, data, &enemies);
    let mut planner = Planner {
        campaign,
        data,
        owner: faction,
        view,
        objective: None,
        routes,
        rejected_candidates: Default::default(),
        include_diagnostics,
        candidate_trace: Default::default(),
    };
    planner.objective = planner.retained_objective();
    if campaign.ai.factions.get(&faction).is_some_and(|state| {
        state.phase_round == campaign.completed_rounds
            && state.accepted_commands >= data.ai.max_commands_per_phase
    }) {
        let decision = planner.pass();
        return Ok(if include_diagnostics {
            planner.finish(decision)
        } else {
            decision
        });
    }
    let emergency = planner.threatened_headquarters();
    if emergency {
        planner.objective = Some(planner.objective(
            campaign.factions[&faction].headquarters,
            AiObjectiveKind::Defend,
        ));
    }
    let decision = planner
        .release_stranded_builders()
        .or_else(|| planner.defend())
        // Keep useful wartime operations ahead of regrouping: an isolated
        // army may still be the force needed to finish an attack or retake land.
        .or_else(|| {
            planner
                .objective
                .as_ref()
                .filter(|objective| matches!(objective.kind, AiObjectiveKind::Attack))
                .and_then(|_| planner.pursue_attack())
        })
        .or_else(|| planner.recover_lost_territory())
        .or_else(|| planner.attack())
        .or_else(|| planner.reconnect_supply())
        .or_else(|| planner.retreat())
        .or_else(|| planner.recruit(emergency))
        .or_else(|| planner.progression())
        .or_else(|| planner.construct())
        .or_else(|| planner.develop_city())
        .or_else(|| planner.peace())
        .or_else(|| {
            planner
                .temperament()
                .guards_borders
                .then(|| planner.border())
                .flatten()
        })
        .or_else(|| {
            planner
                .objective
                .as_ref()
                .filter(|objective| {
                    objective.kind != AiObjectiveKind::Border
                        || !planner.posted_border_post(objective.site)
                })
                .map(|_| planner.pursue().unwrap_or_else(|| planner.pass()))
        })
        .or_else(|| planner.clear_threat())
        .or_else(|| planner.expand())
        .or_else(|| planner.declare_war())
        .or_else(|| planner.border());
    let decision = decision.unwrap_or_else(|| planner.pass());
    Ok(if include_diagnostics {
        planner.finish(decision)
    } else {
        decision
    })
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
    /// This sovereign's temperament, which tunes war, expansion and garrisons.
    fn temperament(&self) -> &crate::data::ai::AiPersonalityProfile {
        self.data
            .ai
            .profile(self.campaign.factions[&self.owner].personality)
    }

    fn choose(&self, command: Command, objective: Option<AiObjective>) -> Option<AiDecision> {
        let intent = intent::of(&command)?;
        if self.include_diagnostics {
            self.candidate_trace.borrow_mut().evaluated += 1;
        }
        if self
            .campaign
            .ai
            .factions
            .get(&self.owner)
            .is_some_and(|state| {
                state.rejected_at_sequence == self.campaign.accepted_sequence
                    && state.rejected.contains(&intent)
            })
            || self.rejected_candidates.borrow().contains(&command)
        {
            self.record_rejection(
                &command,
                "the same intent was already rejected at this campaign state",
            );
            return None;
        }
        // The planner issues one edge at a time. Saving an unaffordable edge
        // repeatedly would spend its entire command budget without travelling.
        if let Err(reason) = self.check_candidate(&command) {
            if let Some(reason) = reason {
                self.record_rejection(&command, &reason);
            }
            self.rejected_candidates.borrow_mut().push(command);
            return None;
        }
        Some(AiDecision {
            command,
            objective,
            intent,
            diagnostics: Vec::new(),
        })
    }

    fn check_candidate(&self, command: &Command) -> Result<(), Option<String>> {
        self.candidate_allowed(command)
            .map_err(|error| self.include_diagnostics.then(|| error.to_string()))?;
        if let Command::Move(order) = command {
            let route = super::movement::preview_order(self.campaign, self.data, self.owner, order)
                .map_err(|error| self.include_diagnostics.then(|| error.to_string()))?;
            if route.can_confirm() && route.reachable_steps > 0 {
                return Ok(());
            }
            if let Some(stop) = route.blocked.as_ref().or(route.stop.as_ref()) {
                return Err(self
                    .include_diagnostics
                    .then(|| format!("route stops at site {}: {:?}", stop.site.0, stop.reason)));
            }
            return Err(self.include_diagnostics.then(|| {
                format!(
                    "route has no reachable step (remaining movement {})",
                    route.remaining
                )
            }));
        }
        preview(
            self.campaign,
            self.data,
            Actor::Npc(self.owner),
            command.clone(),
        )
        .map(|_| ())
        .map_err(|error| self.include_diagnostics.then(|| error.to_string()))
    }

    fn record_rejection(&self, command: &Command, reason: &str) {
        if !self.include_diagnostics {
            return;
        }
        let mut trace = self.candidate_trace.borrow_mut();
        *trace.rejection_counts.entry(reason.to_owned()).or_default() += 1;
        if trace.examples.len() < 12 {
            trace.examples.push(format!("{command:?}: {reason}"));
        }
    }

    fn finish(&self, mut decision: AiDecision) -> AiDecision {
        decision.diagnostics = self.diagnostic_lines(&decision);
        decision
    }

    fn diagnostic_lines(&self, decision: &AiDecision) -> Vec<String> {
        let faction = &self.campaign.factions[&self.owner];
        let owned = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(self.owner))
            .count();
        let neutral = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| site.controller.is_none())
            .count();
        let wars: Vec<_> = self
            .view
            .factions
            .iter()
            .filter(|other| self.at_war(other.id))
            .map(|other| other.name.as_str())
            .collect();
        let budget = self.campaign.ai.factions.get(&self.owner);
        let budget_reached = budget.is_some_and(|state| {
            state.phase_round == self.campaign.completed_rounds
                && state.accepted_commands >= self.data.ai.max_commands_per_phase
        });
        let mut lines = vec![format!(
            "Faction {} (#{}, {:?}): resources {:?}, territory {owned}, neutral sites {neutral}, wars {:?}, action budget {}/{}",
            faction.name,
            self.owner.0,
            faction.personality,
            faction.resources,
            wars,
            budget.map_or(0, |state| state.accepted_commands),
            self.data.ai.max_commands_per_phase
        )];
        lines.push(format!(
            "Military plan: {} armies, {} supplied, target {}; last seasonal gold income {}, upkeep {}, deficit {}",
            self.view.armies.len(),
            self.view.supplied_armies.len(),
            self.recruitment_target(),
            faction.last_economy.as_ref().map_or(0, |statement| statement.income.gold),
            faction.last_economy.as_ref().map_or(0, |statement| statement.upkeep_due),
            faction.deficit,
        ));
        lines.push(format!(
            "Decision {:?}; objective {:?}; evaluated {} candidate commands",
            decision.command,
            decision.objective,
            self.candidate_trace.borrow().evaluated
        ));
        for army in &self.view.armies {
            let remaining = self
                .campaign
                .army_movement_remaining(army.id, self.data)
                .unwrap_or(0);
            let formation_summary: Vec<_> = army
                .formation_ids()
                .filter_map(|id| {
                    self.view
                        .formations
                        .iter()
                        .find(|formation| formation.id == id)
                })
                .map(|formation| {
                    format!(
                        "{:?} {}/{}",
                        formation.kind, formation.headcount, formation.capacity
                    )
                })
                .collect();
            let location = self
                .view
                .world
                .site(army.site)
                .map_or("unknown", |site| site.name.as_str());
            let active_siege = self
                .view
                .sieges
                .iter()
                .any(|siege| siege.own_armies.contains(&army.id));
            let building = self
                .view
                .construction
                .iter()
                .any(|order| order.is_open() && order.builder == Some(army.id));
            let reachable_targets =
                if matches!(&decision.command, Command::EndTurn) && self.moving(army, false) {
                    self.view
                        .world
                        .sites
                        .iter()
                        .filter(|site| {
                            site.controller.is_none()
                                || site
                                    .controller
                                    .is_some_and(|owner| owner != self.owner && self.at_war(owner))
                        })
                        .filter(|site| {
                            self.path(
                                army.site,
                                site.id,
                                site.controller.is_some_and(|owner| self.at_war(owner)),
                            )
                            .is_some_and(|(_, path)| path.len() > 1)
                        })
                        .count()
                } else {
                    0
                };
            lines.push(format!(
                "Army {} {} at {} (#{}): movement {remaining}, supplied {}, weak {}, siege {}, builder {}, formations {:?}{}",
                army.id.0,
                army.name,
                location,
                army.site.0,
                self.campaign.army_is_supplied(army.id),
                self.weak(army),
                active_siege,
                building,
                formation_summary,
                if matches!(&decision.command, Command::EndTurn) {
                    format!(", reachable neutral/war targets {reachable_targets}")
                } else {
                    String::new()
                }
            ));
        }
        let trace = self.candidate_trace.borrow();
        for (reason, count) in &trace.rejection_counts {
            lines.push(format!("Rejected {count} candidate(s): {reason}"));
        }
        lines.extend(
            trace
                .examples
                .iter()
                .map(|example| format!("Candidate: {example}")),
        );
        if matches!(&decision.command, Command::EndTurn) {
            lines.push(if budget_reached {
                "Pass cause: the faction reached its accepted-command limit for this phase.".into()
            } else {
                "Pass cause: no eligible planner priority produced a legal command; inspect army movement, supply, target reachability and candidate rejection details above.".into()
            });
        }
        lines
    }

    fn pass(&self) -> AiDecision {
        AiDecision {
            command: Command::EndTurn,
            objective: self.objective.clone(),
            intent: AiIntent {
                kind: AiIntentKind::Pass,
                targets: Vec::new(),
            },
            diagnostics: Vec::new(),
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
        let expired = self
            .campaign
            .completed_rounds
            .saturating_sub(objective.chosen_round)
            >= self.data.ai.objective_rounds;
        if expired
            && !(objective.kind == AiObjectiveKind::Border && self.border_post(objective.site))
        {
            return None;
        }
        let site = self.view.world.site(objective.site)?;
        let valid = match objective.kind {
            AiObjectiveKind::Expand => {
                !self
                    .view
                    .threats
                    .iter()
                    .any(|threat| threat.site == site.id)
                    && self.claims().iter().any(|claim| claim.site == site.id)
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
            AiObjectiveKind::Border => self.border_post(site.id),
        };
        valid.then(|| objective.clone())
    }
}
