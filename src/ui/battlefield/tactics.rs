//! Compact preparation controls for the currently selected friendly formation.

use super::{BattlefieldAction, TacticEdit};
use crate::ui::{components, Context, UiAction};
use kestrum::{
    data::battle_tactics::{
        leader_capabilities, BattleCapability, BattleDoctrine, TacticAction, TacticCondition,
        TacticTrigger, TargetFilter,
    },
    state::{battle::simulation::BattleUnitId, military::FormationId, people::PersonAssignment},
};
use macroquad::prelude::*;

pub(super) fn prepare_text(ctx: &Context<'_>) {
    let labels = [
        (16, ctx.text("battle_tactic_activation")),
        (16, ctx.text("battle_tactic_reaction")),
        (14, ctx.text("battle_tactic_fallback")),
        (14, ctx.text("battle_tactic_add")),
        (12, ctx.text("battle_tactic_always")),
        (12, ctx.text("battle_tactic_enemy_rear")),
        (12, ctx.text("battle_tactic_enemy_cavalry")),
        (12, ctx.text("battle_tactic_first_activation")),
        (12, ctx.text("battle_tactic_self_hurt")),
        (12, ctx.text("battle_tactic_ally_hurt")),
        (12, ctx.text("battle_tactic_self_morale")),
        (12, ctx.text("battle_tactic_ally_morale")),
        (12, ctx.text("battle_tactic_cavalry_charge")),
        (12, ctx.text("battle_tactic_any_enemy")),
        (12, ctx.text("battle_tactic_front")),
        (12, ctx.text("battle_tactic_rear")),
        (12, ctx.text("battle_tactic_exposed_rear")),
        (12, ctx.text("battle_tactic_no_target")),
        (15, ctx.text("battle_leader")),
        (15, ctx.text("battle_leader_none")),
        (15, ctx.text("battle_leader_unavailable")),
        (11, ctx.text("battle_tactic_needs_officer")),
        (11, ctx.text("battle_tactic_needs_infantry")),
        (11, ctx.text("battle_tactic_leader_unavailable")),
        (12, ctx.text("battle_doctrine_defensive")),
        (12, ctx.text("battle_doctrine_ranged")),
        (12, ctx.text("battle_doctrine_breakthrough")),
        (12, ctx.text("battle_template_save")),
        (12, ctx.text("battle_template_update")),
        (12, ctx.text("battle_template_apply")),
        (12, ctx.text("battle_template_new")),
        (11, ctx.text("battle_doctrine_label")),
    ];
    let prepared: Vec<_> = labels
        .iter()
        .map(|(size, label)| (*size, label.as_str()))
        .collect();
    if let Some(font) = ctx.body_font() {
        macroquad_toolkit::ui::prepare_font_text(font, &prepared);
    }
}

pub(super) fn draw_pending_editor(ctx: &Context<'_>, id: BattleUnitId) -> Option<Option<UiAction>> {
    let BattleUnitId::Formation(formation_id) = id else {
        return None;
    };
    let campaign = ctx.state.campaign.as_ref()?.strategic()?;
    let formation = campaign.formations.get(&formation_id)?;
    if formation.faction != campaign.player {
        return None;
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|id| id == formation_id))?;
    let pending = campaign.pending_battle.as_ref()?;
    if !pending
        .report
        .faction_sides()
        .filter(|side| side.faction == campaign.player)
        .flat_map(|side| &side.armies)
        .any(|entry| entry.id == army.id)
    {
        return None;
    }
    let rules = formation
        .tactics
        .as_ref()
        .or_else(|| {
            army.battle_doctrine
                .and_then(|doctrine| ctx.battle_tactics.doctrine_for(doctrine, formation.kind))
        })
        .or_else(|| ctx.battle_tactics.defaults_for(formation.kind))?;
    let (trigger, rows) = if ctx.battlefield.tactic_trigger == TacticTrigger::Activation {
        (TacticTrigger::Activation, &rules.activation)
    } else {
        (TacticTrigger::IncomingAttack, &rules.reaction)
    };
    let visible_rows = rows.len().min(5);
    let rect = Rect::new(310.0, 86.0, 660.0, 126.0 + visible_rows as f32 * 18.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.04, 0.08, 0.08, 0.94),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, components::BRASS);
    let slot = army
        .slots
        .iter()
        .position(|entry| *entry == Some(formation_id))?;
    components::text(
        ctx,
        &format!("{} · {}", troop_label(ctx, formation.kind), slot + 1),
        vec2(rect.x + 12.0, rect.y + 17.0),
        15.0,
        components::CREAM,
    );
    let mut action = None;
    for (left, delta) in [(true, -1_i32), (false, 1_i32)] {
        let x = if left { rect.x + 520.0 } else { rect.x + 556.0 };
        if components::button(
            ctx,
            Rect::new(x, rect.y + 1.0, 32.0, 22.0),
            if left { "<" } else { ">" },
            (0..6).contains(&(slot as i32 + delta)),
            false,
        ) {
            action = Some(UiAction::Battlefield(
                BattlefieldAction::SwapFormationSlots {
                    army: army.id,
                    first: slot as u8,
                    second: (slot as i32 + delta) as u8,
                },
            ));
        }
    }
    for (trigger_tab, x, label) in [
        (
            TacticTrigger::Activation,
            rect.x + 160.0,
            ctx.text("battle_tactic_activation"),
        ),
        (
            TacticTrigger::IncomingAttack,
            rect.x + 320.0,
            ctx.text("battle_tactic_reaction"),
        ),
    ] {
        if components::button(
            ctx,
            Rect::new(x, rect.y + 1.0, 150.0, 22.0),
            &label,
            true,
            trigger == trigger_tab,
        ) {
            action = Some(UiAction::Battlefield(BattlefieldAction::SetTacticTrigger(
                trigger_tab,
            )));
        }
    }
    components::body(
        ctx,
        &ctx.text("battle_doctrine_label"),
        vec2(rect.x + 10.0, rect.y + 42.0),
        11.0,
        components::MUTED,
    );
    for (doctrine, x, label) in [
        (
            BattleDoctrine::DefensiveLine,
            rect.x + 88.0,
            ctx.text("battle_doctrine_defensive"),
        ),
        (
            BattleDoctrine::RangedSupport,
            rect.x + 278.0,
            ctx.text("battle_doctrine_ranged"),
        ),
        (
            BattleDoctrine::Breakthrough,
            rect.x + 468.0,
            ctx.text("battle_doctrine_breakthrough"),
        ),
    ] {
        if components::button(
            ctx,
            Rect::new(x, rect.y + 26.0, 174.0, 19.0),
            &label,
            true,
            army.battle_doctrine == Some(doctrine),
        ) {
            action = Some(UiAction::Battlefield(
                BattlefieldAction::SetBattleDoctrine {
                    army: army.id,
                    doctrine,
                },
            ));
        }
    }
    let leaders = eligible_leaders(campaign, formation_id, formation.kind, army.commander, ctx);
    let mut choices = vec![None];
    choices.extend(leaders.into_iter().map(Some));
    let current = formation.battle_leader;
    let current_available = current.is_some_and(|id| choices.contains(&Some(id)));
    let current_index = choices.iter().position(|choice| *choice == current);
    let leader_name = current
        .filter(|_| current_available)
        .and_then(|id| campaign.people.get(&id))
        .map(|person| person.name.as_str())
        .map(|name| fit_label(ctx, name, 172.0))
        .unwrap_or_else(|| {
            if current.is_some() {
                ctx.text("battle_leader_unavailable")
            } else {
                ctx.text("battle_leader_none")
            }
        });
    components::body(
        ctx,
        &ctx.text("battle_leader"),
        vec2(rect.x + 12.0, rect.y + 61.0),
        14.0,
        components::MUTED,
    );
    components::body(
        ctx,
        &leader_name,
        vec2(rect.x + 72.0, rect.y + 61.0),
        14.0,
        components::CREAM,
    );
    let selected_leader_class = current
        .filter(|_| current_available)
        .and_then(|id| campaign.people.get(&id))
        .map(|person| person.class);
    let capabilities = leader_capabilities(formation.kind, selected_leader_class);
    for (left, x, step) in [(true, rect.x + 254.0, -1_i32), (false, rect.x + 282.0, 1)] {
        if components::button(
            ctx,
            Rect::new(x, rect.y + 48.0, 24.0, 18.0),
            if left { "<" } else { ">" },
            choices.len() > 1,
            false,
        ) {
            let next = current_index
                .map(|index| (index as i32 + step).rem_euclid(choices.len() as i32) as usize)
                .unwrap_or(usize::from(!left));
            action = Some(UiAction::Battlefield(BattlefieldAction::SetBattleLeader {
                formation: formation_id,
                leader: choices[next],
            }));
        }
    }
    if trigger == TacticTrigger::Activation {
        components::body(
            ctx,
            &ctx.text("battle_tactic_fallback"),
            vec2(rect.x + 330.0, rect.y + 61.0),
            12.0,
            components::MUTED,
        );
    }
    for (index, rule) in rows.iter().take(5).enumerate() {
        let y = rect.y + 72.0 + index as f32 * 18.0;
        let move_up = components::button(
            ctx,
            Rect::new(rect.x + 10.0, y, 22.0, 18.0),
            "U",
            index > 0,
            false,
        );
        let move_down = components::button(
            ctx,
            Rect::new(rect.x + 34.0, y, 22.0, 18.0),
            "D",
            index + 1 < rows.len(),
            false,
        );
        if move_up || move_down {
            action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
                formation: formation_id,
                edit: TacticEdit::Move(trigger, index, move_up),
            }));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 60.0, y, 116.0, 18.0),
            &action_label(ctx, rule.action),
            true,
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Action,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 180.0, y, 122.0, 18.0),
            &condition_label(ctx, rule.condition),
            true,
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Condition,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 306.0, y, 122.0, 18.0),
            &target_label(ctx, rule.target_filter),
            needs_target(rule.action),
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Target,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 432.0, y, 24.0, 18.0),
            "X",
            true,
            false,
        ) {
            action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
                formation: formation_id,
                edit: TacticEdit::Remove(trigger, index),
            }));
        }
        components::body(
            ctx,
            &(index + 1).to_string(),
            vec2(rect.x + 472.0, y + 13.0),
            12.0,
            components::MUTED,
        );
        let unavailable_reason = match rule.action {
            TacticAction::Rally if !capabilities.contains(&BattleCapability::OfficerRally) => {
                Some(if current.is_some() && !current_available {
                    "battle_tactic_leader_unavailable"
                } else {
                    "battle_tactic_needs_officer"
                })
            }
            TacticAction::HoldTheLine
                if !capabilities.contains(&BattleCapability::InfantryHoldTheLine) =>
            {
                Some(if current.is_some() && !current_available {
                    "battle_tactic_leader_unavailable"
                } else {
                    "battle_tactic_needs_infantry"
                })
            }
            _ => None,
        };
        if let Some(reason) = unavailable_reason {
            components::body(
                ctx,
                &ctx.text(reason),
                vec2(rect.x + 500.0, y + 13.0),
                11.0,
                components::MUTED,
            );
        }
    }
    if components::button(
        ctx,
        Rect::new(rect.x + 10.0, rect.y + rect.h - 40.0, 110.0, 18.0),
        &ctx.text("battle_tactic_add"),
        rows.len() < ctx.battle_tactics.max_tactic_rows as usize,
        false,
    ) {
        action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
            formation: formation_id,
            edit: TacticEdit::Add(trigger),
        }));
    }
    let template_count = campaign.battle_templates.len();
    let selected_template = usize::from(ctx.battlefield.template_index).min(template_count);
    let template_name = campaign
        .battle_templates
        .get(selected_template)
        .map(|template| template.name.as_str())
        .unwrap_or_else(|| "");
    let template_label = if selected_template == template_count {
        ctx.text("battle_template_new")
    } else {
        fit_label(ctx, template_name, 210.0)
    };
    let total_choices = template_count + 1;
    let save_label = if selected_template < template_count {
        ctx.text("battle_template_update")
    } else {
        ctx.text("battle_template_save")
    };
    if components::button(
        ctx,
        Rect::new(rect.x + 10.0, rect.y + rect.h - 19.0, 90.0, 18.0),
        &save_label,
        selected_template < template_count || template_count < 12,
        false,
    ) {
        action = Some(UiAction::Battlefield(
            BattlefieldAction::SaveBattleTemplate { army: army.id },
        ));
    }
    if components::button(
        ctx,
        Rect::new(rect.x + 106.0, rect.y + rect.h - 19.0, 24.0, 18.0),
        "<",
        total_choices > 1,
        false,
    ) {
        action = Some(UiAction::Battlefield(BattlefieldAction::SetTemplateIndex(
            ((selected_template + total_choices - 1) % total_choices) as u8,
        )));
    }
    components::body(
        ctx,
        &template_label,
        vec2(rect.x + 140.0, rect.y + rect.h - 6.0),
        12.0,
        components::CREAM,
    );
    if components::button(
        ctx,
        Rect::new(rect.x + 366.0, rect.y + rect.h - 19.0, 24.0, 18.0),
        ">",
        total_choices > 1,
        false,
    ) {
        action = Some(UiAction::Battlefield(BattlefieldAction::SetTemplateIndex(
            ((selected_template + 1) % total_choices) as u8,
        )));
    }
    if components::button(
        ctx,
        Rect::new(rect.x + 402.0, rect.y + rect.h - 19.0, 100.0, 18.0),
        &ctx.text("battle_template_apply"),
        selected_template < template_count,
        false,
    ) {
        action = Some(UiAction::Battlefield(
            BattlefieldAction::ApplyBattleTemplate {
                army: army.id,
                index: selected_template as u8,
            },
        ));
    }
    Some(action)
}

fn eligible_leaders(
    campaign: &kestrum::state::StrategicCampaign,
    formation: FormationId,
    kind: kestrum::data::economy::TroopKind,
    commander: Option<kestrum::state::people::PersonId>,
    ctx: &Context<'_>,
) -> Vec<kestrum::state::people::PersonId> {
    let baseline = leader_capabilities(kind, None);
    campaign
        .people
        .values()
        .filter(|person| {
            person.faction == campaign.player
                && person.assignment == (PersonAssignment::Formation { formation })
                && !person.career.retired
                && person.is_fit_for_field(
                    campaign.completed_rounds,
                    ctx.rules.leadership.field_min_age_years,
                )
                && (person.age_years(campaign.completed_rounds) < ctx.lifecycle.elder_age_years
                    || commander == Some(person.id))
                && leader_capabilities(kind, Some(person.class)).len() > baseline.len()
        })
        .map(|person| person.id)
        .collect()
}

fn fit_label(ctx: &Context<'_>, label: &str, max_width: f32) -> String {
    if measure_text(label, ctx.body_font(), 14, 1.0).width <= max_width {
        return label.to_owned();
    }
    let mut fitted = String::new();
    for character in label.chars() {
        let mut candidate = fitted.clone();
        candidate.push(character);
        candidate.push('…');
        if measure_text(&candidate, ctx.body_font(), 14, 1.0).width > max_width {
            break;
        }
        fitted.push(character);
    }
    format!("{fitted}…")
}

#[derive(Clone, Copy)]
enum EditField {
    Action,
    Condition,
    Target,
}

fn edit_tactics(
    formation: FormationId,
    trigger: TacticTrigger,
    index: usize,
    field: EditField,
) -> UiAction {
    let edit = match field {
        EditField::Action => TacticEdit::CycleAction(trigger, index),
        EditField::Condition => TacticEdit::CycleCondition(trigger, index),
        EditField::Target => TacticEdit::CycleTarget(trigger, index),
    };
    UiAction::Battlefield(BattlefieldAction::EditTactics { formation, edit })
}

fn action_label(ctx: &Context<'_>, action: TacticAction) -> String {
    match action {
        TacticAction::Attack => ctx.text("battle_play_attack"),
        TacticAction::Volley => ctx.text("battle_play_volley"),
        TacticAction::Charge => ctx.text("battle_play_charge"),
        TacticAction::Breakthrough => ctx.text("battle_play_breakthrough"),
        TacticAction::Guard => ctx.text("battle_play_guard"),
        TacticAction::Brace => ctx.text("battle_play_brace"),
        TacticAction::Wait => ctx.text("battle_play_wait"),
        TacticAction::Advance => ctx.text("battle_play_advance"),
        TacticAction::Rally => ctx.text("battle_play_rally"),
        TacticAction::HoldTheLine => ctx.text("battle_play_hold_line"),
        TacticAction::Stabilize => ctx.text("battle_play_stabilize"),
    }
}

fn condition_label(ctx: &Context<'_>, condition: TacticCondition) -> String {
    let key = match condition {
        TacticCondition::Always => "battle_tactic_always",
        TacticCondition::EnemyRearExposed => "battle_tactic_enemy_rear",
        TacticCondition::EnemyCavalryPresent => "battle_tactic_enemy_cavalry",
        TacticCondition::FirstActivation => "battle_tactic_first_activation",
        TacticCondition::SelfBelowHalf => "battle_tactic_self_hurt",
        TacticCondition::AllyInSameRowBelowHalf => "battle_tactic_ally_hurt",
        TacticCondition::IncomingCavalryCharge => "battle_tactic_cavalry_charge",
        TacticCondition::SelfBelowHalfMorale => "battle_tactic_self_morale",
        TacticCondition::AllyBelowHalfMorale => "battle_tactic_ally_morale",
    };
    ctx.text(key)
}

fn target_label(ctx: &Context<'_>, target: TargetFilter) -> String {
    let key = match target {
        TargetFilter::None => "battle_tactic_no_target",
        TargetFilter::AnyEnemy => "battle_tactic_any_enemy",
        TargetFilter::EnemyFront => "battle_tactic_front",
        TargetFilter::EnemyRear => "battle_tactic_rear",
        TargetFilter::EnemyCavalry => "battle_tactic_enemy_cavalry",
        TargetFilter::ExposedEnemyRear => "battle_tactic_exposed_rear",
        TargetFilter::AllyLowestMorale => "battle_tactic_ally_lowest_morale",
    };
    ctx.text(key)
}

fn troop_label(ctx: &Context<'_>, kind: kestrum::data::economy::TroopKind) -> String {
    let key = match kind {
        kestrum::data::economy::TroopKind::Warriors => "troop_warriors",
        kestrum::data::economy::TroopKind::Spearmen => "troop_spearmen",
        kestrum::data::economy::TroopKind::Archers => "troop_archers",
        kestrum::data::economy::TroopKind::Riders => "troop_riders",
        kestrum::data::economy::TroopKind::Medics => "troop_medics",
        kestrum::data::economy::TroopKind::SiegeEngines => "troop_siege_engines",
    };
    ctx.text(key)
}

fn is_attack(action: TacticAction) -> bool {
    matches!(
        action,
        TacticAction::Attack
            | TacticAction::Volley
            | TacticAction::Charge
            | TacticAction::Breakthrough
    )
}

fn needs_target(action: TacticAction) -> bool {
    is_attack(action) || action == TacticAction::Stabilize
}
