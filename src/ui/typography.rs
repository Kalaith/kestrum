//! Prepare changing text before any visible glyphs are batched for this frame.

use super::{Context, SaveMode};
use kestrum::state::Overlay;

pub fn prepare_dynamic_text(ctx: &Context<'_>, feedback: Option<&str>) {
    let Some(font) = ctx.body_font() else {
        return;
    };
    let mut samples = Vec::with_capacity(16);
    if let Some(view) = ctx.campaign_view {
        samples.push((18, view.active_faction_name.as_str()));
    }
    if let Some(message) = feedback {
        samples.push((19, message));
    }
    match ctx.state.overlay {
        Overlay::Saves => match ctx.saves.mode {
            SaveMode::Browse => {
                for row in ctx
                    .saves
                    .rows
                    .iter()
                    .skip(ctx.saves.page * super::saves::PAGE_SIZE)
                    .take(super::saves::PAGE_SIZE)
                {
                    samples.push((21, row.name.as_str()));
                    samples.push((16, row.detail.as_str()));
                }
                samples.push((18, ctx.saves.status.as_str()));
            }
            SaveMode::Name { .. } => {
                samples.push((23, ctx.saves.name.as_str()));
                if let Some(error) = &ctx.saves.name_error {
                    samples.push((18, error.as_str()));
                }
            }
            SaveMode::ConfirmDelete(id) => {
                if let Some(row) = ctx.saves.rows.iter().find(|row| row.id == id) {
                    samples.push((25, row.name.as_str()));
                    samples.push((19, row.detail.as_str()));
                }
            }
        },
        Overlay::SaveRecovery => samples.push((19, ctx.save_error)),
        _ => {}
    }
    macroquad_toolkit::ui::prepare_font_text(font, &samples);
}
