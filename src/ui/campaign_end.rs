//! A saved kingdom ending keeps records and utilities available without advancing play.

use super::{components::*, kingdom::lines, Context, UiAction};
use kestrum::state::{diplomacy::EndingKind, Overlay};
use macroquad::prelude::*;

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let ending = ctx.kingdom.data.as_ref()?.ending.as_ref()?;
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(
        ctx,
        &ctx.text("kingdom_milestone"),
        vec2(112.0, 105.0),
        30.0,
        BRASS,
    );
    text(
        ctx,
        &ctx.text(match ending.kind {
            EndingKind::Victory => "kingdom_victory",
            EndingKind::Defeat => "kingdom_defeat",
        }),
        vec2(112.0, 205.0),
        30.0,
        CREAM,
    );
    let season = &ctx.data.seasons[(ending.completed_rounds % 4) as usize];
    body(
        ctx,
        &format!(
            "{} · {} {}",
            season,
            ctx.text("year"),
            ctx.data.start_year + ending.completed_rounds / 4
        ),
        vec2(112.0, 248.0),
        20.0,
        BRASS,
    );
    lines(
        ctx,
        &ctx.text(match ending.kind {
            EndingKind::Victory => "kingdom_victory_help",
            EndingKind::Defeat => "kingdom_defeat_end_help",
        }),
        vec2(112.0, 324.0),
        1056.0,
        3,
        CREAM,
    );
    lines(
        ctx,
        &ctx.text("kingdom_ending_readonly"),
        vec2(112.0, 452.0),
        1056.0,
        2,
        MUTED,
    );
    for (index, (key, action)) in [
        ("records", UiAction::OpenRecords),
        ("save", UiAction::Save),
        ("menu", UiAction::Open(Overlay::Menu)),
        ("new_game", UiAction::NewGame),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 272.0, 594.0, 240.0, 48.0),
            &ctx.text(key),
            true,
            index == 0,
        ) {
            return Some(action);
        }
    }
    None
}
