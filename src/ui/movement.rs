//! Army group choice and physical-route review; the atlas remains the destination picker.

mod group;
mod map;
pub use map::{draw_map_overlay, panel_bounds};
mod review;

use super::{components::*, Context, UiAction};
use kestrum::{data::world::SiteId, engine::MovementPreview, state::military::ArmyId};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};
use std::collections::BTreeMap;

pub const MOVE_GROUP_PAGE_SIZE: usize = 6;
pub const ROUTE_PAGE_SIZE: usize = 6;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MoveStage {
    #[default]
    Inactive,
    Group,
    Map,
    Review,
}

#[derive(Debug, Default)]
pub struct MoveView {
    pub stage: MoveStage,
    pub site: Option<SiteId>,
    pub armies: Vec<ArmyId>,
    pub page: usize,
    pub route_page: usize,
    pub destination: Option<SiteId>,
    pub planned_destination: Option<SiteId>,
    pub preview: Option<MovementPreview>,
    pub remaining: BTreeMap<ArmyId, u32>,
    pub nearby: BTreeMap<SiteId, u32>,
    pub status: String,
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.02, 0.05, 0.05, 0.78));
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(
        ctx,
        &ctx.text(if ctx.movement.stage == MoveStage::Group {
            "move_group"
        } else {
            "route_review"
        }),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    match ctx.movement.stage {
        MoveStage::Group => group::draw(ctx),
        MoveStage::Review => review::draw(ctx),
        _ => None,
    }
}

fn cost_summary(ctx: &Context<'_>, preview: &MovementPreview) -> String {
    format!(
        "{}: {}  ·  {}: {}",
        ctx.text("route_cost"),
        preview.total_cost,
        ctx.text("movement_left"),
        preview.remaining
    )
}

fn route_consequence(ctx: &Context<'_>, preview: &MovementPreview) -> String {
    if let Some(blocked) = &preview.blocked {
        blocked.reason.to_string()
    } else if let Some(stop) = &preview.stop {
        let end = ctx
            .campaign_view
            .and_then(|campaign| campaign.world.site(preview.reachable_site))
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        if matches!(
            stop.reason,
            kestrum::engine::MovementBlock::InsufficientMovement { .. }
        ) {
            ctx.text("move_planned_preview").replace("{site}", end)
        } else {
            format!("{}: {end}. {}", ctx.text("move_stops_at"), stop.reason)
        }
    } else if let Some(encounter) = &preview.encounter {
        ctx.text(match encounter {
            kestrum::engine::MovementEncounter::EstablishSiege => "move_possible_siege",
            kestrum::engine::MovementEncounter::JoinBesiegers => "move_join_siege",
            kestrum::engine::MovementEncounter::Relief => "move_relief",
        })
    } else if !preview.observed_hostile_sites.is_empty() {
        ctx.text("move_observed_hostile")
    } else if preview.uncertain_contact {
        ctx.text("move_uncertain_contact")
    } else {
        ctx.text("move_route_clear")
    }
}

fn lines(ctx: &Context<'_>, label: &str, at: Vec2, width: f32, limit: usize, color: Color) {
    let wrapped = wrap_text_ex(label, width, ctx.body_font(), 18.0);
    for (index, line) in wrapped.iter().take(limit).enumerate() {
        let label = if index + 1 == limit && wrapped.len() > limit {
            truncate_text_to_width_ex(&format!("{line}…"), width, ctx.body_font(), 18.0)
        } else {
            line.clone()
        };
        body(
            ctx,
            &label,
            at + vec2(0.0, index as f32 * 23.0),
            18.0,
            color,
        );
    }
}

fn tapped(ctx: &Context<'_>, rect: Rect) -> bool {
    ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin))
}
