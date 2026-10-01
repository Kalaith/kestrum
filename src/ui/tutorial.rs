//! One contextual next action in the existing header, with visible touch controls.

use super::{components::*, ArmyMode, Context, MoveStage, UiAction};
use kestrum::state::{
    tutorial::{TutorialStep, TUTORIAL_STEPS},
    Campaign, GameState, Overlay, Screen,
};
use kestrum::{data::world::MarkerLocation, navigation::MapScope};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

pub fn bounds(state: &GameState) -> Option<Rect> {
    if state.screen != Screen::Campaign {
        return None;
    }
    let campaign = state.campaign.as_ref()?.strategic()?;
    campaign.tutorial.current()?;
    if campaign.diplomacy.ending.is_some() {
        return None;
    }
    match state.overlay {
        Overlay::None => Some(Rect::new(244.0, 8.0, 888.0, 80.0)),
        Overlay::Armies | Overlay::MoveGroup | Overlay::MoveReview => {
            Some(Rect::new(92.0, 40.0, 1096.0, 56.0))
        }
        _ => None,
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let rect = bounds(ctx.state)?;
    let campaign = ctx.state.campaign.as_ref()?.strategic()?;
    let step = campaign.tutorial.current()?;
    let show_hq = step == TutorialStep::Headquarters;
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, INK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    let close = Rect::new(
        rect.right() - 108.0,
        rect.y + (rect.h - 48.0) / 2.0,
        100.0,
        48.0,
    );
    let hq = Rect::new(close.x - 132.0, close.y, 124.0, 48.0);
    let width = if show_hq { hq.x } else { close.x } - rect.x - 26.0;
    let label = prompt(ctx, step);
    let lines = wrap_text_ex(&label, width, ctx.body_font(), 18.0);
    let y = rect.y + (rect.h - lines.len() as f32 * 22.0) / 2.0 + 17.0;
    for (index, line) in lines.iter().enumerate() {
        body(
            ctx,
            line,
            vec2(rect.x + 12.0, y + index as f32 * 22.0),
            18.0,
            CREAM,
        );
    }
    if button(ctx, close, &ctx.text("close"), true, false) {
        return Some(UiAction::DismissTutorial);
    }
    if show_hq && button(ctx, hq, &ctx.text("tutorial_show_hq"), true, true) {
        return Some(UiAction::TutorialHeadquarters);
    }
    None
}

pub(super) fn prompt(ctx: &Context<'_>, step: TutorialStep) -> String {
    let campaign = ctx
        .state
        .campaign
        .as_ref()
        .and_then(Campaign::strategic)
        .expect("guide requires a campaign");
    let in_army = ctx.state.overlay == Overlay::Armies;
    let key = if ctx.campaign_view.is_some_and(|view| !view.player_turn) {
        if ctx.state.overlay == Overlay::None {
            "tutorial_npc"
        } else {
            "tutorial_npc_back"
        }
    } else {
        match step {
            TutorialStep::Headquarters => "tutorial_headquarters",
            TutorialStep::Movement => match ctx.movement.stage {
                MoveStage::Group => "tutorial_move_group",
                MoveStage::Review if ctx.movement.reviewing_plan => "move_planned_notice",
                MoveStage::Review => "tutorial_move_confirm",
                MoveStage::Map if ctx.movement.preview.is_some() => "tutorial_move_review",
                MoveStage::Map if ctx.movement.destination.is_some() => "tutorial_move_blocked",
                MoveStage::Map => "tutorial_move_destination",
                MoveStage::Inactive if in_army && ctx.army.mode == ArmyMode::Roster => {
                    "tutorial_move_orders"
                }
                MoveStage::Inactive if in_army && ctx.army.mode == ArmyMode::Orders => {
                    "tutorial_move_begin"
                }
                MoveStage::Inactive if in_army => "tutorial_back_roster",
                MoveStage::Inactive => "tutorial_find_army",
            },
            TutorialStep::Region if in_army => "tutorial_back_map",
            TutorialStep::Region if matches!(ctx.navigation.scope(), MapScope::Region(_)) => {
                "tutorial_region_return"
            }
            TutorialStep::Region
                if !ctx.campaign_view.is_some_and(|view| {
                    view.world
                        .markers
                        .iter()
                        .any(|marker| matches!(marker.location, MarkerLocation::Region { .. }))
                }) =>
            {
                "tutorial_region_explore"
            }
            TutorialStep::Region => "tutorial_region",
            TutorialStep::WorldMap if in_army => "tutorial_back_map",
            TutorialStep::WorldMap => "tutorial_world",
            TutorialStep::Career if !in_army => "tutorial_find_army",
            TutorialStep::Career if ctx.army.mode == ArmyMode::People => "tutorial_career",
            TutorialStep::Career if ctx.army.mode == ArmyMode::Orders => "tutorial_people",
            TutorialStep::Career if ctx.army.mode == ArmyMode::Roster => "tutorial_career_orders",
            TutorialStep::Career => "tutorial_back_roster",
            TutorialStep::Household if !in_army => "tutorial_find_army",
            TutorialStep::Household if ctx.army.mode == ArmyMode::Households => {
                "tutorial_household_review"
            }
            TutorialStep::Household if ctx.army.mode == ArmyMode::People => {
                "tutorial_household_open"
            }
            TutorialStep::Household if ctx.army.mode == ArmyMode::Roster => {
                "tutorial_career_orders"
            }
            TutorialStep::Household if ctx.army.mode == ArmyMode::Orders => "tutorial_people",
            TutorialStep::Household => "tutorial_household_back",
            TutorialStep::FirstTurn if in_army => "tutorial_back_map",
            TutorialStep::FirstTurn => "tutorial_turn",
            TutorialStep::Records if in_army => "tutorial_back_map",
            TutorialStep::Records => "tutorial_records",
        }
    };
    let name = campaign
        .world
        .site(campaign.factions[&campaign.player].headquarters)
        .map(|site| site.name.as_str())
        .unwrap_or_default();
    let name = truncate_text_to_width_ex(name, 220.0, ctx.body_font(), 18.0);
    let number = TUTORIAL_STEPS
        .iter()
        .position(|candidate| *candidate == step)
        .unwrap()
        + 1;
    format!(
        "{number}/{} · {}",
        TUTORIAL_STEPS.len(),
        ctx.text(key).replace("{name}", &name)
    )
}
