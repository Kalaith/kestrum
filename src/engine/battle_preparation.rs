//! Deterministic player edits to a pending battle's formation board and tactics.

use super::{actions::validate_command, combat, ActionOutcome, Actor, Command, RuleError};
use crate::{
    data::{battle_tactics::leader_capabilities, world::PersonClass, GameData},
    state::{
        battle::BattleSideReport,
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        StrategicCampaign,
    },
};

pub(super) fn validate_leader_assignment(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    formation: FormationId,
    leader: Option<PersonId>,
) -> Result<(), RuleError> {
    let unit = campaign
        .formations
        .get(&formation)
        .filter(|unit| unit.faction == owner)
        .ok_or(RuleError::FormationNotOwned { formation })?;
    let Some(id) = leader else {
        return Ok(());
    };
    let person = campaign
        .people
        .get(&id)
        .filter(|person| person.faction == owner)
        .ok_or(RuleError::UnknownPerson { person: id })?;
    let army = campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|entry| entry == formation))
        .expect("validated formation army");
    let leader_grants = leader_capabilities(unit.kind, Some(person.class));
    let troop_grants = leader_capabilities(unit.kind, None);
    let class_can_lead = leader_grants.len() > troop_grants.len()
        && matches!(
            person.class,
            PersonClass::Officer | PersonClass::Infantry | PersonClass::Cavalry
        );
    if person.assignment != (PersonAssignment::Formation { formation })
        || person.career.retired
        || !person.is_fit_for_field(
            campaign.completed_rounds,
            data.rules.leadership.field_min_age_years,
        )
        || (person.age_years(campaign.completed_rounds) >= data.lifecycle.elder_age_years
            && army.commander != Some(id))
        || !class_can_lead
    {
        return Err(RuleError::Progression(
            "Choose a fit attached Officer, Infantry or Cavalry leader for this troop role.".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: &Command,
) -> Result<(), RuleError> {
    data.battle_tactics
        .validate()
        .map_err(RuleError::InvalidState)?;
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    validate_command(campaign, actor, command)?;
    let owner = match actor {
        Actor::Player => campaign.player,
        Actor::Npc(faction) => faction,
    };
    let pending = campaign
        .pending_battle
        .as_ref()
        .ok_or(RuleError::NoPendingBattle)?;
    match command {
        Command::SetFormationTactics { formation, tactics } => {
            data.battle_tactics
                .validate_configuration(
                    campaign
                        .formations
                        .get(formation)
                        .filter(|entry| entry.faction == owner)
                        .ok_or(RuleError::FormationNotOwned {
                            formation: *formation,
                        })?
                        .kind,
                    tactics,
                )
                .map_err(RuleError::InvalidState)?;
            if !pending
                .report
                .faction_sides()
                .filter(|side| side.faction == owner)
                .flat_map(|side| &side.armies)
                .any(|army| army.formations.iter().any(|entry| entry.id == *formation))
            {
                return Err(RuleError::BattlePending);
            }
        }
        Command::SetBattleLeader { formation, leader } => {
            validate_leader_assignment(campaign, data, owner, *formation, *leader)?;
            if !pending
                .report
                .faction_sides()
                .filter(|side| side.faction == owner)
                .flat_map(|side| &side.armies)
                .any(|army| army.formations.iter().any(|entry| entry.id == *formation))
            {
                return Err(RuleError::BattlePending);
            }
        }
        Command::SwapFormationSlots {
            army,
            first,
            second,
        } => {
            if *first >= 6 || *second >= 6 || first == second {
                return Err(RuleError::InvalidSlot);
            }
            let actual = campaign
                .armies
                .get(army)
                .filter(|entry| entry.faction == owner)
                .ok_or(RuleError::ArmyNotOwned { army: *army })?;
            if !pending
                .report
                .faction_sides()
                .filter(|side| side.faction == owner)
                .flat_map(|side| &side.armies)
                .any(|entry| entry.id == *army)
            {
                return Err(RuleError::BattlePending);
            }
            if actual.slots[usize::from(*first)].is_none()
                && actual.slots[usize::from(*second)].is_none()
            {
                return Err(RuleError::InvalidSlot);
            }
        }
        _ => return Err(RuleError::NoPendingBattle),
    }
    Ok(())
}

pub(super) fn apply(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) -> Result<ActionOutcome, RuleError> {
    validate(campaign, data, actor, &command)?;
    let mut candidate = campaign.clone();
    candidate.accepted_sequence =
        candidate
            .accepted_sequence
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "accepted action sequence",
            })?;
    match command {
        Command::SetFormationTactics { formation, tactics } => {
            candidate
                .formations
                .get_mut(&formation)
                .expect("validated")
                .tactics = Some(tactics);
        }
        Command::SetBattleLeader { formation, leader } => {
            candidate
                .formations
                .get_mut(&formation)
                .expect("validated")
                .battle_leader = leader;
        }
        Command::SwapFormationSlots {
            army,
            first,
            second,
        } => {
            candidate
                .armies
                .get_mut(&army)
                .expect("validated")
                .slots
                .swap(usize::from(first), usize::from(second));
            sync_report_slots(&mut candidate, army);
        }
        _ => return Err(RuleError::NoPendingBattle),
    }
    candidate
        .pending_battle
        .as_mut()
        .expect("validated pending battle")
        .report
        .sequence = candidate.accepted_sequence;
    combat::refresh_pending(&mut candidate, data)?;
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    let outcome = ActionOutcome {
        life_events: Vec::new(),
        automatic_retirements: Vec::new(),
        battle: None,
        battle_pending: true,
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
    *campaign = candidate;
    Ok(outcome)
}

fn sync_report_slots(campaign: &mut StrategicCampaign, id: ArmyId) {
    let slots = campaign.armies[&id].slots;
    let pending = campaign.pending_battle.as_mut().expect("pending");
    for side in sides_mut(&mut pending.report.attacker, &mut pending.report.defender) {
        let Some(army) = side.armies.iter_mut().find(|army| army.id == id) else {
            continue;
        };
        for formation in &mut army.formations {
            formation.slot = slots
                .iter()
                .position(|slot| *slot == Some(formation.id))
                .expect("every formation remains in one army slot");
        }
        army.formations.sort_by_key(|formation| formation.slot);
    }
}

fn sides_mut<'a>(
    attacker: &'a mut BattleSideReport,
    defender: &'a mut crate::state::battle::BattleDefender,
) -> impl Iterator<Item = &'a mut BattleSideReport> {
    std::iter::once(attacker).chain(defender.faction_side_mut())
}
