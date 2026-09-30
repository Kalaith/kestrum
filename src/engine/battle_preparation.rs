//! Deterministic player edits to a pending battle's formation board and tactics.

use super::{actions::validate_command, combat, ActionOutcome, Actor, Command, RuleError};
use crate::{
    data::{
        battle_tactics::{leader_capabilities, BattleDoctrine, TroopTactics},
        world::PersonClass,
        GameData,
    },
    state::{
        battle::{BattleReport, BattleSideReport},
        battle_plans::{BattlePlanSlot, BattlePlanTemplate},
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        StrategicCampaign,
    },
};

#[path = "battle_preparation/rivals.rs"]
mod rivals;

pub(super) fn prepare_rivals(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) -> Result<(), RuleError> {
    rivals::prepare_rivals(campaign, data, report)
}

pub(super) fn apply_doctrine_snapshot(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army_id: ArmyId,
    doctrine: BattleDoctrine,
) {
    let formations = campaign.armies[&army_id]
        .formation_ids()
        .collect::<Vec<_>>();
    campaign
        .armies
        .get_mut(&army_id)
        .expect("validated army")
        .battle_doctrine = Some(doctrine);
    for id in formations {
        let formation = campaign.formations.get_mut(&id).expect("army member");
        if !formation.has_explicit_tactics() {
            formation.tactics = Some(
                data.battle_tactics
                    .doctrine_for(doctrine, formation.kind)
                    .expect("validated doctrine covers every troop kind")
                    .clone(),
            );
            formation.tactics_override = Some(false);
        }
    }
}

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
    if actor != Actor::Player
        && matches!(
            command,
            Command::SaveBattleTemplate { .. } | Command::ApplyBattleTemplate { .. }
        )
    {
        return Err(RuleError::WrongActor);
    }
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
        Command::SetBattleDoctrine { army, doctrine } => {
            if !data.battle_tactics.doctrines.contains_key(doctrine) {
                return Err(RuleError::InvalidState("Unknown battle doctrine.".into()));
            }
            validate_preparation_army(campaign, pending, owner, *army)?;
        }
        Command::SaveBattleTemplate { army, name } => {
            validate_preparation_army(campaign, pending, owner, *army)?;
            if !valid_template_name(name) {
                return Err(RuleError::InvalidState(
                    "Battle template names must contain 1 to 32 visible characters.".into(),
                ));
            }
            let exists = campaign
                .battle_templates
                .iter()
                .any(|template| template.name.eq_ignore_ascii_case(name));
            if !exists
                && campaign.battle_templates.len()
                    >= crate::state::battle_plans::MAX_BATTLE_TEMPLATES
            {
                return Err(RuleError::InvalidState(
                    "The campaign has reached its saved battle template limit.".into(),
                ));
            }
        }
        Command::ApplyBattleTemplate { army, name } => {
            validate_preparation_army(campaign, pending, owner, *army)?;
            if !campaign
                .battle_templates
                .iter()
                .any(|template| template.name == *name)
            {
                return Err(RuleError::InvalidState(
                    "Battle template is unavailable.".into(),
                ));
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

fn validate_preparation_army(
    campaign: &StrategicCampaign,
    pending: &crate::state::battle::PendingBattle,
    owner: crate::data::world::FactionId,
    army: ArmyId,
) -> Result<(), RuleError> {
    campaign
        .armies
        .get(&army)
        .filter(|entry| entry.faction == owner)
        .ok_or(RuleError::ArmyNotOwned { army })?;
    if !pending
        .report
        .faction_sides()
        .filter(|side| side.faction == owner)
        .flat_map(|side| &side.armies)
        .any(|entry| entry.id == army)
    {
        return Err(RuleError::BattlePending);
    }
    Ok(())
}

fn valid_template_name(name: &str) -> bool {
    !name.is_empty()
        && name.trim() == name
        && name.chars().count() <= crate::state::battle_plans::MAX_BATTLE_TEMPLATE_NAME_CHARS
        && !name.chars().any(char::is_control)
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
            let entry = candidate.formations.get_mut(&formation).expect("validated");
            entry.tactics = Some(tactics);
            entry.tactics_override = Some(true);
        }
        Command::SetBattleDoctrine { army, doctrine } => {
            apply_doctrine_snapshot(&mut candidate, data, army, doctrine);
        }
        Command::SaveBattleTemplate { army, name } => {
            save_template(&mut candidate, data, army, name);
        }
        Command::ApplyBattleTemplate { army, name } => {
            apply_template(&mut candidate, data, army, &name);
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
    sync_report_preparation(&mut candidate);
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

fn save_template(campaign: &mut StrategicCampaign, data: &GameData, army_id: ArmyId, name: String) {
    let army = &campaign.armies[&army_id];
    let slots = army
        .slots
        .iter()
        .enumerate()
        .filter_map(|(slot, formation)| formation.map(|id| (slot, id)))
        .map(|(slot, id)| {
            let formation = &campaign.formations[&id];
            let tactics = formation
                .tactics
                .clone()
                .or_else(|| {
                    army.battle_doctrine
                        .and_then(|doctrine| {
                            data.battle_tactics.doctrine_for(doctrine, formation.kind)
                        })
                        .cloned()
                })
                .or_else(|| data.battle_tactics.defaults_for(formation.kind).cloned())
                .expect("validated troop defaults");
            BattlePlanSlot {
                slot: slot as u8,
                kind: formation.kind,
                tactics,
                explicit_override: formation.has_explicit_tactics(),
            }
        })
        .collect();
    let template = BattlePlanTemplate {
        name,
        doctrine: army.battle_doctrine,
        slots,
    };
    if let Some(existing) = campaign
        .battle_templates
        .iter_mut()
        .find(|existing| existing.name.eq_ignore_ascii_case(&template.name))
    {
        *existing = template;
    } else {
        campaign.battle_templates.push(template);
    }
}

fn apply_template(campaign: &mut StrategicCampaign, data: &GameData, army_id: ArmyId, name: &str) {
    let template = campaign
        .battle_templates
        .iter()
        .find(|template| template.name == name)
        .expect("validated template")
        .clone();
    let slots = campaign.armies[&army_id].slots;
    campaign
        .armies
        .get_mut(&army_id)
        .expect("validated army")
        .battle_doctrine = template.doctrine;
    for (slot, formation_id) in slots
        .iter()
        .enumerate()
        .filter_map(|(slot, formation)| formation.map(|id| (slot, id)))
    {
        let formation = campaign
            .formations
            .get_mut(&formation_id)
            .expect("army member");
        if let Some(saved) = template
            .slots
            .iter()
            .find(|entry| entry.slot == slot as u8 && entry.kind == formation.kind)
        {
            formation.tactics = Some(saved.tactics.clone());
            formation.tactics_override = Some(saved.explicit_override);
        } else {
            formation.tactics = Some(base_tactics(data, template.doctrine, formation.kind));
            formation.tactics_override = Some(false);
        }
    }
}

fn base_tactics(
    data: &GameData,
    doctrine: Option<BattleDoctrine>,
    kind: crate::data::economy::TroopKind,
) -> TroopTactics {
    doctrine
        .and_then(|doctrine| data.battle_tactics.doctrine_for(doctrine, kind))
        .or_else(|| data.battle_tactics.defaults_for(kind))
        .expect("validated troop defaults")
        .clone()
}

fn sync_report_preparation(campaign: &mut StrategicCampaign) {
    let Some(pending) = campaign.pending_battle.as_mut() else {
        return;
    };
    // Keep persisted doctrine labels aligned with campaign snapshots.
    let sides = sides_mut(&mut pending.report.attacker, &mut pending.report.defender);
    for army in sides.flat_map(|side| &mut side.armies) {
        army.battle_doctrine = campaign
            .armies
            .get(&army.id)
            .and_then(|actual| actual.battle_doctrine);
        for formation in &mut army.formations {
            formation.battle_leader = campaign.formations[&formation.id].battle_leader;
        }
    }
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
