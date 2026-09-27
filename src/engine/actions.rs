//! Validated transactional commands shared by player and NPC phase processing.

mod types;
mod validation;
use super::{movement, recruitment, round, transfer, MoveOrder};
use crate::{
    data::{world::FactionId, GameData},
    state::{
        campaign::{DomainFact, DomainFactKind, FactId},
        CampaignPhase, StrategicCampaign,
    },
};
pub use types::{ActionOutcome, ActionPreview, Actor, Command, RuleError};
pub(super) use validation::validate_command;

pub fn preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<ActionPreview, RuleError> {
    if command == Command::StepNpc {
        let mut candidate = campaign.clone();
        let outcome = step_npc(&mut candidate, data, actor)?;
        return Ok(ActionPreview {
            active_faction_after: outcome.active_faction,
            round_completed: outcome.round_completed,
        });
    }
    if let Command::ClearThreat { armies, threat } = &command {
        campaign.validate(data).map_err(RuleError::InvalidState)?;
        validate_command(campaign, actor, &command)?;
        let owner = match actor {
            Actor::Player => campaign.player,
            Actor::Npc(id) => id,
        };
        super::threats::validate_order(campaign, data, owner, armies, *threat)?;
        return Ok(ActionPreview {
            active_faction_after: campaign.active_faction(),
            round_completed: false,
        });
    }
    if let Command::Siege(order) = &command {
        campaign.validate(data).map_err(RuleError::InvalidState)?;
        validate_command(campaign, actor, &command)?;
        let owner = match actor {
            Actor::Player => campaign.player,
            Actor::Npc(id) => id,
        };
        super::siege::validate_order(campaign, data, owner, order)?;
        return Ok(ActionPreview {
            active_faction_after: campaign.active_faction(),
            round_completed: false,
        });
    }
    if let Command::Move(order) = &command {
        let observer = match actor {
            Actor::Player => campaign.player,
            Actor::Npc(id) => id,
        };
        validate_command(campaign, actor, &command)?;
        let result = movement::preview_order(campaign, data, observer, order)?;
        if result.reachable_steps == 0 {
            let stop = result.stop.ok_or(RuleError::InvalidRoute)?;
            return Err(RuleError::MovementBlocked {
                site: stop.site,
                reason: stop.reason,
            });
        }
        return Ok(ActionPreview {
            active_faction_after: campaign.active_faction(),
            round_completed: false,
        });
    }
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
    if command == Command::StepNpc {
        return step_npc(campaign, data, actor);
    }
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
    if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
        return Err(RuleError::NpcPaused);
    }
    let decision = super::ai::propose(campaign, data, faction)?;
    let mut candidate = campaign.clone();
    let outcome = match apply(
        &mut candidate,
        data,
        Actor::Npc(faction),
        decision.command.clone(),
    ) {
        Ok(outcome) => {
            super::ai::accepted(&mut candidate, campaign, data, faction, &decision)?;
            outcome
        }
        Err(_) => {
            super::ai::rejected(&mut candidate, campaign, data, faction, &decision)?;
            apply(&mut candidate, data, Actor::Npc(faction), Command::EndTurn)?
        }
    };
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    *campaign = candidate;
    Ok(outcome)
}

mod npc;
use npc::step_npc;

fn prepare(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<(StrategicCampaign, ActionOutcome), RuleError> {
    data.ai.validate().map_err(RuleError::InvalidState)?;
    data.diplomacy.validate().map_err(RuleError::InvalidState)?;
    data.development
        .validate()
        .map_err(RuleError::InvalidState)?;
    data.threats
        .validate(&data.scenario)
        .map_err(RuleError::InvalidState)?;
    data.siege.validate().map_err(RuleError::InvalidState)?;
    data.economy.validate().map_err(RuleError::InvalidState)?;
    data.rules.validate().map_err(RuleError::InvalidState)?;
    data.troops.validate().map_err(RuleError::InvalidState)?;
    data.combat.validate().map_err(RuleError::InvalidState)?;
    data.progression
        .validate()
        .map_err(RuleError::InvalidState)?;
    data.lifecycle.validate().map_err(RuleError::InvalidState)?;
    data.history.validate().map_err(RuleError::InvalidState)?;
    data.construction
        .validate()
        .map_err(RuleError::InvalidState)?;
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    validate_command(campaign, actor, &command)?;
    let owner = match actor {
        Actor::Player => campaign.player,
        Actor::Npc(id) => id,
    };
    super::progression::validate_command(campaign, data, owner, &command)?;
    super::lifecycle::validate(campaign, data, owner, &command)?;
    super::mentorship::validate(campaign, data, owner, &command)?;
    let mut candidate = campaign.clone();
    candidate.accepted_sequence =
        candidate
            .accepted_sequence
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "accepted action sequence",
            })?;
    let mut outcome = ActionOutcome {
        automatic_retirements: Vec::new(),
        battle: None,
        accepted_sequence: candidate.accepted_sequence,
        active_faction: candidate.active_faction(),
        round_completed: false,
        facts: Vec::new(),
        consumed_facts: Vec::new(),
        recruited: None,
        disbanded: None,
        movement: None,
        split_army: None,
    };
    execute(&mut candidate, campaign, data, owner, command, &mut outcome)?;
    finish(&mut candidate, campaign, data, &mut outcome)?;
    Ok((candidate, outcome))
}

fn finish(
    candidate: &mut StrategicCampaign,
    before: &StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if !outcome.round_completed {
        super::siege::reconcile(candidate, data, outcome)?;
        super::construction::reconcile(candidate, data, outcome)?;
    }
    super::diplomacy::reconcile(before, candidate, data, outcome)?;
    let already_completed = outcome.round_completed;
    let before_phase = candidate.clone();
    round::reconcile_phase(candidate, data, outcome)?;
    if outcome.round_completed && !already_completed {
        super::diplomacy::reconcile(&before_phase, candidate, data, outcome)?;
    }
    candidate.reconcile_region_control();
    super::mentorship::reconcile(candidate, data);
    super::lifecycle::reconcile_roles(candidate);
    if outcome.round_completed {
        super::mentorship::resolve_season(candidate, data);
    }
    if outcome.round_completed || candidate.diplomacy.ending.is_some() {
        outcome.consumed_facts = std::mem::take(&mut candidate.pending_facts);
        super::evidence::consume(candidate, data, &outcome.consumed_facts)?;
        super::progression::resolve(
            candidate,
            data,
            &outcome.consumed_facts,
            outcome.round_completed,
        )?;
        candidate.consumed_sequence = candidate.accepted_sequence;
        candidate.acted.clear();
    }
    super::history::record_facts(candidate, before, &outcome.facts)?;
    if outcome.round_completed {
        super::history::prune(candidate, data);
    }
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    outcome.active_faction = candidate.active_faction();
    Ok(())
}

fn execute(
    candidate: &mut StrategicCampaign,
    before: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: Command,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    match command {
        command @ (Command::DeclareWar { .. }
        | Command::OfferPeace { .. }
        | Command::RespondPeace { .. }
        | Command::ResolveDefeat { .. }) => {
            super::diplomacy::execute(candidate, data, owner, command, outcome)?;
        }
        Command::ClearThreat { armies, threat } => {
            super::threats::execute(candidate, data, owner, &armies, threat, outcome)?;
        }
        command @ (Command::Resettle { .. }
        | Command::RenameSite { .. }
        | Command::MoveCapital { .. }
        | Command::RelocateHeadquarters { .. }) => {
            super::development::execute(candidate, data, owner, command, outcome)?;
        }
        Command::Siege(order) => super::siege::execute(candidate, data, owner, order, outcome)?,
        command @ (Command::StartConstruction { .. }
        | Command::CancelConstruction { .. }
        | Command::ReassignBuilder { .. }
        | Command::SetFocus { .. }) => {
            execute_construction(candidate, data, owner, command, outcome)?;
        }
        Command::EndTurn | Command::StepNpc => {
            round::pass_faction(candidate, data, outcome)?;
        }
        Command::SetNpcPaused(paused) => {
            candidate.phase = CampaignPhase::NpcTurn {
                faction: candidate.active_faction(),
                paused,
            };
        }
        Command::Recruit { site, army, kind } => {
            let recruited = recruitment::recruit(candidate, data, site, army, kind)?;
            let fact = DomainFactKind::FormationRecruited {
                faction: candidate.active_faction(),
                army: recruited.army,
                formation: recruited.formation,
                site,
                troop: kind,
            };
            record_fact(candidate, outcome, fact)?;
            outcome.recruited = Some(recruited);
        }
        Command::Disband { formation } => {
            let fact = recruitment::disband(candidate, formation)?;
            record_fact(candidate, outcome, fact)?;
            outcome.disbanded = Some(formation);
        }
        Command::Move(order) => {
            execute_move(candidate, before, data, owner, order, outcome)?;
        }
        Command::TransferFormation {
            formation,
            to_army,
            to_slot,
        } => {
            let fact = transfer::formation(candidate, owner, formation, to_army, to_slot)?;
            record_fact(candidate, outcome, fact)?;
        }
        Command::TransferPerson {
            person,
            to_formation,
        } => {
            let fact = transfer::person(candidate, data, owner, person, to_formation)?;
            record_fact(candidate, outcome, fact)?;
        }
        Command::SplitArmy { formation } => {
            let (army, fact) = transfer::split(candidate, owner, formation)?;
            record_fact(candidate, outcome, fact)?;
            outcome.split_army = Some(army);
        }
        command @ (Command::SetCommander { .. }
        | Command::TrainPerson { .. }
        | Command::PracticeRiding { .. }
        | Command::CancelPersonCourse { .. }
        | Command::SpecializeFormation { .. }
        | Command::CancelFormationCourse { .. }) => {
            super::progression::execute(candidate, data, owner, &command)?
        }
        command @ (Command::RecoverPersonAtSite { .. }
        | Command::RetirePerson { .. }
        | Command::AppointGovernor { .. }) => {
            super::lifecycle::execute(candidate, data, owner, &command)?;
        }
        command @ (Command::StartMentorship { .. } | Command::EndMentorship { .. }) => {
            super::mentorship::execute(candidate, data, owner, &command)?;
        }
    }
    Ok(())
}

fn execute_move(
    candidate: &mut StrategicCampaign,
    before: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    order: MoveOrder,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let moved = movement::execute(candidate, data, &order, outcome)?;
    let receipt = movement::service_snapshot(before, &moved);
    let fact = if let Some(battle) = moved.battle {
        DomainFactKind::BattleResolved {
            battle,
            movement: Some(receipt),
        }
    } else {
        DomainFactKind::ArmiesMoved {
            faction: owner,
            armies: moved.armies.clone(),
            path: moved.path.clone(),
            spent: moved.spent,
            movement: Some(receipt),
        }
    };
    record_fact(candidate, outcome, fact)?;
    outcome.battle = moved.battle;
    outcome.movement = Some(moved);
    Ok(())
}

fn execute_construction(
    candidate: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: Command,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let fact = match command {
        Command::StartConstruction {
            target,
            kind,
            builder,
        } => super::construction::start(candidate, data, owner, target, kind, builder)?,
        Command::CancelConstruction { order } => {
            super::construction::cancel(candidate, data, owner, order)?
        }
        Command::ReassignBuilder { order, builder } => {
            super::construction::reassign(candidate, owner, order, builder)?
        }
        Command::SetFocus { site, focus } => {
            super::construction::set_focus(candidate, data, owner, site, focus)?
        }
        _ => {
            return Err(RuleError::InvalidState(
                "Unexpected construction dispatch.".into(),
            ))
        }
    };
    record_fact(candidate, outcome, fact)
}

pub(super) fn record_fact(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
    kind: DomainFactKind,
) -> Result<(), RuleError> {
    let fact = DomainFact {
        id: campaign.next_ids.fact,
        sequence: campaign.accepted_sequence,
        completed_rounds: campaign.completed_rounds,
        kind,
    };
    campaign.next_ids.fact = FactId(campaign.next_ids.fact.0.checked_add(1).ok_or(
        RuleError::Overflow {
            field: "fact identifiers",
        },
    )?);
    campaign.pending_facts.push(fact.clone());
    outcome.facts.push(fact);
    Ok(())
}
