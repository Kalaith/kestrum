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

fn validate_observer_actor(
    campaign: &StrategicCampaign,
    actor: Actor,
    command: &Command,
) -> Result<(), RuleError> {
    if campaign.observer_mode
        && actor == Actor::Player
        && !matches!(
            command,
            Command::StartPendingBattle | Command::SetNpcPaused(_) | Command::StepNpc
        )
    {
        return Err(RuleError::ObserverControlOnly);
    }
    Ok(())
}

pub fn preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<ActionPreview, RuleError> {
    validate_observer_actor(campaign, actor, &command)?;
    if matches!(
        command,
        Command::SetFormationTactics { .. }
            | Command::SetBattleDoctrine { .. }
            | Command::SaveBattleTemplate { .. }
            | Command::ApplyBattleTemplate { .. }
            | Command::SwapFormationSlots { .. }
    ) || (campaign.pending_battle.is_some()
        && matches!(command, Command::SetBattleLeader { .. }))
    {
        super::battle_preparation::validate(campaign, data, actor, &command)?;
        return Ok(ActionPreview {
            active_faction_after: campaign.active_faction(),
            round_completed: false,
        });
    }
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
        if !result.can_confirm() {
            let stop = result
                .blocked
                .or(result.stop)
                .ok_or(RuleError::InvalidRoute)?;
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
    validate_observer_actor(campaign, actor, &command)?;
    if matches!(
        command,
        Command::SetFormationTactics { .. }
            | Command::SetBattleDoctrine { .. }
            | Command::SaveBattleTemplate { .. }
            | Command::ApplyBattleTemplate { .. }
            | Command::SwapFormationSlots { .. }
    ) || (campaign.pending_battle.is_some()
        && matches!(command, Command::SetBattleLeader { .. }))
    {
        return super::battle_preparation::apply(campaign, data, actor, command);
    }
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
    if campaign.observer_mode {
        return super::observer::advance_observer(campaign, data);
    }
    advance_npc_action(campaign, data)
}

pub(super) fn advance_npc_action(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    Ok(advance_npc_action_without_diagnostics(campaign, data)?.outcome)
}

pub(super) fn advance_npc_action_without_diagnostics(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<super::observer::ObserverStepOutcome, RuleError> {
    advance_npc_action_inner(campaign, data, false)
}

pub(super) fn advance_npc_action_diagnosed(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<super::observer::ObserverStepOutcome, RuleError> {
    advance_npc_action_inner(campaign, data, true)
}

fn advance_npc_action_inner(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    include_diagnostics: bool,
) -> Result<super::observer::ObserverStepOutcome, RuleError> {
    if campaign.pending_battle.is_some() {
        return Err(RuleError::BattlePending);
    }
    let CampaignPhase::NpcTurn { faction, .. } = campaign.phase else {
        return Err(RuleError::NotNpcPhase);
    };
    if matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }) {
        return Err(RuleError::NpcPaused);
    }
    let decision = if include_diagnostics {
        super::ai::propose_diagnosed(campaign, data, faction)?
    } else {
        super::ai::propose(campaign, data, faction)?
    };
    let mut candidate = campaign.clone();
    let actor = Actor::Npc(faction);
    let (outcome, resolution) = match apply(&mut candidate, data, actor, decision.command.clone()) {
        Ok(mut outcome) => {
            if candidate.pending_battle.as_ref().is_some_and(|pending| {
                candidate.observer_mode
                    || !pending
                        .report
                        .participant_factions()
                        .any(|participant| participant == campaign.player)
            }) {
                let accepted = apply(
                    &mut candidate,
                    data,
                    Actor::Player,
                    Command::StartPendingBattle,
                )?;
                merge_outcome(&mut outcome, accepted);
            }
            super::ai::accepted(&mut candidate, campaign, data, faction, &decision)?;
            let resolution = if matches!(decision.command, Command::EndTurn) {
                "No legal command was selected; faction passed.".to_owned()
            } else {
                "Command accepted.".to_owned()
            };
            (outcome, resolution)
        }
        Err(error) => {
            super::ai::rejected(&mut candidate, campaign, data, faction, &decision)?;
            (
                apply(&mut candidate, data, actor, Command::EndTurn)?,
                format!("Command rejected ({error}); faction passed."),
            )
        }
    };
    *campaign = candidate;
    let mut diagnostics = decision.diagnostics;
    diagnostics.push(resolution);
    Ok(super::observer::ObserverStepOutcome {
        action: format!("{:?}", decision.command),
        diagnostics,
        outcome,
    })
}

pub(super) fn merge_outcome(target: &mut ActionOutcome, mut later: ActionOutcome) {
    target
        .continued_movements
        .append(&mut later.continued_movements);
    target.life_events.append(&mut later.life_events);
    target
        .automatic_retirements
        .append(&mut later.automatic_retirements);
    target.battle = later.battle.or(target.battle);
    target.battle_pending = later.battle_pending;
    target.accepted_sequence = later.accepted_sequence;
    target.active_faction = later.active_faction;
    target.round_completed |= later.round_completed;
    target.facts.append(&mut later.facts);
    target.consumed_facts.append(&mut later.consumed_facts);
    target.recruited = later.recruited.or(target.recruited);
    target.disbanded = later.disbanded.or(target.disbanded);
    if later.movement.is_some() {
        target.movement = later.movement;
    }
    target.split_army = later.split_army.or(target.split_army);
    target.succession.append(&mut later.succession);
    target.new_people.append(&mut later.new_people);
    target
        .legacy_items_changed
        .append(&mut later.legacy_items_changed);
    target
        .anniversary_reminders
        .append(&mut later.anniversary_reminders);
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
    data.households
        .validate()
        .map_err(RuleError::InvalidState)?;
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
    validate_personnel_command(campaign, data, owner, &command)?;
    let mut candidate = campaign.clone();
    candidate.accepted_sequence =
        candidate
            .accepted_sequence
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "accepted action sequence",
            })?;
    if candidate.appearance_registry.catalog_revision != data.portraits.catalog_revision
        || candidate.appearance_registry.allocation_revision != data.portraits.allocation_revision
    {
        // Expansion affects future people only, and becomes durable with this transaction.
        candidate
            .appearance_registry
            .transition_to_catalog(&data.portraits)
            .map_err(RuleError::InvalidState)?;
    }
    let mut outcome = ActionOutcome {
        continued_movements: Vec::new(),
        life_events: Vec::new(),
        automatic_retirements: Vec::new(),
        battle: None,
        battle_pending: false,
        accepted_sequence: candidate.accepted_sequence,
        active_faction: candidate.active_faction(),
        round_completed: false,
        facts: Vec::new(),
        consumed_facts: Vec::new(),
        recruited: None,
        disbanded: None,
        movement: None,
        split_army: None,
        succession: Vec::new(),
        new_people: Vec::new(),
        legacy_items_changed: Vec::new(),
        anniversary_reminders: Vec::new(),
    };
    super::exploration::observe(&mut candidate);
    if let Command::SetBattleLeader { formation, leader } = &command {
        super::battle_preparation::validate_leader_assignment(
            campaign, data, owner, *formation, *leader,
        )?;
    }
    execute(&mut candidate, campaign, data, owner, command, &mut outcome)?;
    finish(&mut candidate, campaign, data, &mut outcome)?;
    Ok((candidate, outcome))
}

pub(super) fn validate_personnel_command(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    super::progression::validate_command(campaign, data, owner, command)?;
    super::lifecycle::validate(campaign, data, owner, command)?;
    super::mentorship::validate(campaign, data, owner, command)?;
    super::succession::validate(campaign, data, owner, command)?;
    super::legacy::validate(campaign, owner, command)?;
    Ok(())
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
    let before_phase = (!candidate.diplomacy.is_blocked()
        && !candidate
            .round_order
            .iter()
            .any(|id| candidate.is_independent(*id) && !candidate.acted.contains(id)))
    .then(|| candidate.clone());
    round::reconcile_phase(candidate, data, outcome)?;
    if outcome.round_completed && !already_completed {
        super::diplomacy::reconcile(
            before_phase
                .as_ref()
                .expect("a completed round has a phase snapshot"),
            candidate,
            data,
            outcome,
        )?;
    }
    candidate.reconcile_movement_plans();
    candidate.reconcile_region_control();
    super::mentorship::reconcile(candidate, data);
    super::lifecycle::reconcile_roles(candidate);
    super::succession::reconcile(before, candidate);
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
    super::progression::refund_departures(candidate, before)?;
    super::history::record_facts(candidate, before, &outcome.facts)?;
    outcome.life_events = super::history::record_life_changes(candidate, before)?;
    outcome.legacy_items_changed =
        super::history::record_legacy_custody_changes(candidate, before)?
            .into_iter()
            .filter(|id| {
                candidate
                    .legacy_items
                    .get(id)
                    .is_some_and(|item| item.faction == candidate.player)
            })
            .collect();
    if outcome.round_completed {
        outcome.anniversary_reminders = super::history::record_anniversaries(candidate)?
            .into_iter()
            .filter(|subject| match subject {
                crate::state::history::AnniversarySubject::Person(id) => candidate
                    .people
                    .get(id)
                    .is_some_and(|person| person.faction == candidate.player),
                crate::state::history::AnniversarySubject::Site(id) => candidate
                    .world
                    .site(*id)
                    .is_some_and(|site| site.controller == Some(candidate.player)),
            })
            .collect();
    }
    outcome.succession = super::succession::notices(before, candidate);
    outcome.battle_pending = candidate.pending_battle.is_some();
    super::exploration::observe(candidate);
    super::notifications::collect(before, candidate, data, outcome)
        .map_err(RuleError::InvalidState)?;
    // Receipts retain event-time facts before narrative/person retention removes them.
    if outcome.round_completed {
        super::history::prune(candidate, data);
    }
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    if !candidate.movement_plans.is_empty()
        && (outcome.round_completed
            || candidate.active_faction() != before.active_faction()
            || (before.pending_battle.is_some() && candidate.pending_battle.is_none())
            || (matches!(before.phase, CampaignPhase::NpcTurn { paused: true, .. })
                && matches!(
                    candidate.phase,
                    CampaignPhase::NpcTurn { paused: false, .. }
                )))
    {
        movement::resume_plans(candidate, data, outcome)?;
    }
    if outcome.round_completed {
        candidate.round_checkpoint_sequence = Some(candidate.accepted_sequence);
    }
    outcome.battle_pending = candidate.pending_battle.is_some();
    outcome.active_faction = candidate.active_faction();
    outcome.accepted_sequence = candidate.accepted_sequence;
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
        Command::StartPendingBattle => {
            super::combat::commit_pending(candidate, data, outcome)?;
        }
        Command::SetFormationTactics { .. }
        | Command::SetBattleDoctrine { .. }
        | Command::SaveBattleTemplate { .. }
        | Command::ApplyBattleTemplate { .. }
        | Command::SwapFormationSlots { .. } => {
            return Err(RuleError::NoPendingBattle);
        }
        Command::SetBattleLeader { formation, leader } => {
            candidate
                .formations
                .get_mut(&formation)
                .expect("validated battle leader formation")
                .battle_leader = leader;
        }
        command @ (Command::Resettle { .. }
        | Command::DevelopCity { .. }
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
        Command::TransferLegacyItem { item, to } => {
            super::legacy::execute(candidate, owner, item, to)?;
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
        Command::CancelMovementPlan { army } => movement::cancel_plan(candidate, owner, army)?,
        Command::TransferFormation {
            formation,
            to_army,
            to_slot,
        } => {
            let fact = transfer::formation(candidate, data, owner, formation, to_army, to_slot)?;
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
            let (army, fact) = transfer::split(candidate, data, owner, formation)?;
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
        command @ (Command::FormHousehold { .. }
        | Command::EndHousehold { .. }
        | Command::SetHouseholdChildraising { .. }
        | Command::AdoptWard { .. }
        | Command::AssignTrainee { .. }
        | Command::EnterService { .. }
        | Command::InviteApprentice { .. }
        | Command::DesignateSuccessor { .. }) => {
            outcome.new_people = super::succession::execute(candidate, data, owner, &command)?;
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
    let fact = if let Some(pending) = candidate.pending_battle.as_mut() {
        pending.movement = Some(receipt);
        None
    } else if moved.spent > 0 {
        Some(DomainFactKind::ArmiesMoved {
            faction: owner,
            armies: moved.armies.clone(),
            path: moved.path.clone(),
            spent: moved.spent,
            movement: Some(receipt),
        })
    } else {
        None
    };
    if let Some(fact) = fact {
        record_fact(candidate, outcome, fact)?;
    }
    outcome.battle = moved.battle.or(outcome.battle);
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
