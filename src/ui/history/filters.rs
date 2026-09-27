//! Explicit filters and the shared touch keyboard keep discovery keyboard-optional.

use super::*;
use kestrum::state::history::HistoryKindFilter;
use macroquad_toolkit::ui::text_entry::keyboard_keys;

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("history_filter_help"),
        vec2(112.0, 130.0),
        18.0,
        MUTED,
    );
    let kinds = [
        None,
        Some(HistoryKindFilter::Battle),
        Some(HistoryKindFilter::Recruitment),
        Some(HistoryKindFilter::Disbanding),
        Some(HistoryKindFilter::Movement),
        Some(HistoryKindFilter::Transfer),
        Some(HistoryKindFilter::Veterancy),
        Some(HistoryKindFilter::Construction),
        Some(HistoryKindFilter::Focus),
        Some(HistoryKindFilter::Siege),
        Some(HistoryKindFilter::Development),
        Some(HistoryKindFilter::Diplomacy),
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        let rect = Rect::new(
            112.0 + (index % 3) as f32 * 360.0,
            166.0 + (index / 3) as f32 * 58.0,
            336.0,
            48.0,
        );
        if button(
            ctx,
            rect,
            &ctx.text(kind_key(kind)),
            true,
            ctx.history.filter.kind == kind,
        ) {
            return Some(UiAction::SetHistoryKind(kind));
        }
    }
    for (index, (key, round)) in [
        ("history_from", ctx.history.filter.from_round),
        ("history_to", ctx.history.filter.to_round),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 424.0 + index as f32 * 75.0;
        let value = round
            .map(|round| date(ctx, round))
            .unwrap_or_else(|| ctx.text("history_any_season"));
        body(
            ctx,
            &format!("{}: {value}", ctx.text(key)),
            vec2(112.0, y + 31.0),
            20.0,
            CREAM,
        );
        for (x, delta, label) in [(984.0, -1, "zoom_out"), (1110.0, 1, "zoom_in")] {
            if button(
                ctx,
                Rect::new(x, y, 58.0, 48.0),
                &ctx.text(label),
                true,
                false,
            ) {
                return Some(if index == 0 {
                    UiAction::ShiftHistoryFrom(delta)
                } else {
                    UiAction::ShiftHistoryTo(delta)
                });
            }
        }
    }
    if button(
        ctx,
        Rect::new(650.0, 626.0, 230.0, 48.0),
        &ctx.text("history_reset_filters"),
        true,
        false,
    ) {
        return Some(UiAction::ResetHistoryFilters);
    }
    if button(
        ctx,
        Rect::new(918.0, 626.0, 250.0, 48.0),
        &ctx.text("history_apply_filters"),
        true,
        true,
    ) {
        return Some(UiAction::ApplyHistoryFilters);
    }
    None
}

fn kind_key(kind: Option<HistoryKindFilter>) -> &'static str {
    match kind {
        Some(HistoryKindFilter::Development) => "history_development",
        Some(HistoryKindFilter::Diplomacy) => "history_diplomacy",
        None => "history_all_events",
        Some(HistoryKindFilter::Battle) => "history_battles",
        Some(HistoryKindFilter::Recruitment) => "recruit",
        Some(HistoryKindFilter::Disbanding) => "disband",
        Some(HistoryKindFilter::Movement) => "history_movement",
        Some(HistoryKindFilter::Transfer) => "transfer",
        Some(HistoryKindFilter::Veterancy) => "history_veterancy",
        Some(HistoryKindFilter::Construction) => "settlement_build",
        Some(HistoryKindFilter::Focus) => "settlement_focus",
        Some(HistoryKindFilter::Siege) => "siege",
    }
}

pub(super) fn search(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("history_search_help"),
        vec2(154.0, 163.0),
        18.0,
        MUTED,
    );
    draw_rectangle(154.0, 199.0, 972.0, 62.0, Color::new(0.13, 0.20, 0.19, 1.0));
    let search = truncate_text_to_width_ex(&ctx.history.search, 930.0, ctx.body_font(), 23.0);
    body(ctx, &search, vec2(174.0, 238.0), 23.0, CREAM);
    match keyboard_keys(
        Rect::new(154.0, 280.0, 972.0, 290.0),
        ctx.history.keyboard_page,
    ) {
        Ok(keys) => {
            for key in keys {
                if input_key(ctx, key.rect, &key.label) {
                    return Some(UiAction::EditHistorySearch(key.action));
                }
            }
        }
        Err(error) => paragraph(ctx, &error, vec2(174.0, 308.0), 930.0),
    }
    if button(
        ctx,
        Rect::new(918.0, 626.0, 250.0, 48.0),
        &ctx.text("history_search"),
        true,
        true,
    ) {
        return Some(UiAction::ApplyHistorySearch);
    }
    None
}
