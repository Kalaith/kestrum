//! Prepare changing text before any visible glyphs are batched for this frame.

use super::{Context, SaveMode};
use kestrum::state::Overlay;

pub fn prepare_dynamic_text(ctx: &Context<'_>, feedback: Option<&str>) {
    if ctx.state.overlay == Overlay::Battle {
        super::battle::prepare_text(ctx);
    }
    if ctx.state.overlay == Overlay::History {
        super::history::prepare_text(ctx);
    }
    let Some(font) = ctx.body_font() else {
        return;
    };
    let mut samples = Vec::with_capacity(16);
    let mut titles = Vec::new();
    if let Some(view) = ctx.campaign_view {
        samples.push((18, view.active_faction_name.as_str()));
        // Names come from saved campaigns, including Unicode and renamed sites.
        // Prepare both atlases before any visible labels are submitted.
        for name in view
            .world
            .markers
            .iter()
            .map(|marker| marker.name.as_str())
            .chain(view.world.sites.iter().map(|site| site.name.as_str()))
            .chain(view.factions.iter().map(|faction| faction.name.as_str()))
        {
            for size in [18, 19, 20] {
                samples.push((size, name));
            }
            titles.push((24, name));
            titles.push((28, name));
        }
        for name in view
            .armies
            .iter()
            .map(|army| army.name.as_str())
            .chain(view.people.iter().map(|person| person.name.as_str()))
        {
            for size in [16, 18, 20, 21] {
                samples.push((size, name));
            }
            titles.push((24, name));
            titles.push((28, name));
        }
    }
    if let Some(message) = feedback {
        samples.push((19, message));
    }
    for size in [18, 20] {
        samples.push((size, ctx.movement.status.as_str()));
    }
    overlay_samples(ctx, &mut samples);
    macroquad_toolkit::ui::prepare_font_text(font, &samples);
    if let Some(font) = ctx.font() {
        macroquad_toolkit::ui::prepare_font_text(font, &titles);
    }
}

fn overlay_samples<'a>(ctx: &'a Context<'_>, samples: &mut Vec<(u16, &'a str)>) {
    match ctx.state.overlay {
        Overlay::Armies => {
            samples.push((18, ctx.army.status.as_str()));
            if let Some(reason) = &ctx.army.transfer.blocked {
                samples.push((18, reason));
            }
            if let Some(reason) = &ctx.army.transfer.split_blocked {
                samples.push((18, reason));
            }
            if let Some(recovery) = &ctx.army.recovery {
                if let Some(reason) = &recovery.blocked {
                    samples.push((18, reason));
                }
            }
            for option in &ctx.army.options {
                if let Some(reason) = &option.blocked {
                    samples.push((18, reason.as_str()));
                }
            }
        }
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
}
