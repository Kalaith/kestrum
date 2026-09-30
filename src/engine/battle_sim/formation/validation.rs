//! Structural and rule-block validation before the pure resolver allocates runtime state.

use crate::{
    data::battle_tactics::{
        BattleTacticsRules, TacticAction, TacticCondition, TacticRule, TacticTrigger, TargetFilter,
    },
    state::battle::simulation::{BattleSide, FormationBattleInput},
};
use std::collections::BTreeSet;

pub(super) fn canonical_input(
    input: &FormationBattleInput,
    rules: &BattleTacticsRules,
) -> Result<FormationBattleInput, String> {
    if !(1..=10_000).contains(&input.terrain_permille) {
        return Err("Battle terrain must be between 1 and 10000 permille.".into());
    }
    let mut opening = input.clone();
    opening.armies.sort_by_key(|army| army.id);
    let mut army_ids = BTreeSet::new();
    let mut unit_ids = BTreeSet::new();
    let mut side_armies = [0_u32; 2];
    let mut side_units = [0_u32; 2];
    let mut total = 0_u32;
    let mut factions = [BTreeSet::new(), BTreeSet::new()];

    for army in &opening.armies {
        if !army_ids.insert(army.id) {
            return Err(format!("Duplicate battle army {:?}.", army.id));
        }
        let index = usize::from(army.side == BattleSide::Defender);
        side_armies[index] += 1;
        factions[index].insert(army.faction);
        for (slot, unit) in army.slots.iter().enumerate() {
            let Some(unit) = unit else { continue };
            if !unit_ids.insert(unit.id) {
                return Err(format!("Duplicate battle combatant {:?}.", unit.id));
            }
            if unit.headcount == 0
                || unit.capacity == 0
                || unit.headcount > unit.capacity
                || unit.attack == 0
                || unit.resistance == 0
                || unit.attack > 1_000_000
                || unit.resistance > 1_000_000
                || unit.activation_tactics.len() > rules.max_tactic_rows as usize
                || unit.reaction_tactics.len() > rules.max_tactic_rows as usize
            {
                return Err(format!("Invalid battle unit {:?} in slot {slot}.", unit.id));
            }
            validate_tactics(&unit.activation_tactics, TacticTrigger::Activation)?;
            validate_tactics(&unit.reaction_tactics, TacticTrigger::IncomingAttack)?;
            total += 1;
            side_units[index] += 1;
        }
    }
    if side_armies.contains(&0)
        || side_units.contains(&0)
        || side_armies
            .iter()
            .any(|count| *count > rules.max_armies_per_side)
        || total > rules.max_activations_per_round
        || factions[0].is_empty()
        || factions[1].is_empty()
        || !factions[0].is_disjoint(&factions[1])
    {
        return Err("Battle must have bounded, opposing army boards on both sides.".into());
    }
    Ok(opening)
}

fn validate_tactics(rows: &[TacticRule], trigger: TacticTrigger) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for rule in rows {
        let attack = matches!(
            rule.action,
            TacticAction::Attack
                | TacticAction::Volley
                | TacticAction::Charge
                | TacticAction::Breakthrough
        );
        let action_allowed = match trigger {
            TacticTrigger::Activation => rule.action != TacticAction::Brace,
            TacticTrigger::IncomingAttack => {
                matches!(rule.action, TacticAction::Brace | TacticAction::Guard)
            }
            TacticTrigger::EndOfRound => false,
        };
        let condition_allowed = match trigger {
            TacticTrigger::Activation => rule.condition != TacticCondition::IncomingCavalryCharge,
            TacticTrigger::IncomingAttack => matches!(
                rule.condition,
                TacticCondition::Always | TacticCondition::IncomingCavalryCharge
            ),
            TacticTrigger::EndOfRound => false,
        };
        if rule.trigger != trigger
            || rule.id.trim().is_empty()
            || !ids.insert(rule.id.as_str())
            || !action_allowed
            || !condition_allowed
            || (attack && rule.target_filter == TargetFilter::None)
            || (!attack && rule.target_filter != TargetFilter::None)
        {
            return Err(format!("Invalid {trigger:?} tactic {:?}.", rule.id));
        }
        if matches!(rule.action, TacticAction::Breakthrough)
            && rule.target_filter == TargetFilter::EnemyFront
        {
            return Err(format!(
                "Breakthrough tactic {:?} must target a rear unit.",
                rule.id
            ));
        }
    }
    Ok(())
}
