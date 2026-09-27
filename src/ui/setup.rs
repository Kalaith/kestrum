//! Production campaign choices stay visible before the seed is instantiated.

use super::{components::*, Context, UiAction};
use kestrum::data::{generation::ProductionSetup, rules::Emblem};
use macroquad::prelude::*;
use macroquad_toolkit::ui::text_entry::{keyboard_keys, KeyboardPage};

#[derive(Debug)]
pub struct SetupView {
    pub kingdom_name: String,
    pub emblem: Emblem,
    pub factions: usize,
    pub seed: u64,
    pub editing_name: bool,
    pub keyboard_page: KeyboardPage,
    pub name_error: Option<String>,
}

impl Default for SetupView {
    fn default() -> Self {
        Self {
            kingdom_name: "Rose".into(),
            emblem: Emblem::Rose,
            factions: 4,
            seed: 260926,
            editing_name: false,
            keyboard_page: KeyboardPage::default(),
            name_error: None,
        }
    }
}

impl SetupView {
    pub fn campaign_setup(&self) -> ProductionSetup {
        ProductionSetup {
            kingdom_name: self.kingdom_name.clone(),
            emblem: self.emblem,
            factions: self.factions,
            seed: self.seed,
        }
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_sheet(ctx);
    if ctx.setup.editing_name {
        draw_name_keyboard(ctx)
    } else {
        draw_choices(ctx)
    }
}

fn draw_sheet(ctx: &Context<'_>) {
    draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.02, 0.05, 0.05, 0.78));
    draw_rectangle(64.0, 28.0, 1152.0, 664.0, INK);
    draw_rectangle_lines(64.0, 28.0, 1152.0, 664.0, 1.0, BRASS);
    text(
        ctx,
        &ctx.text("setup_title"),
        vec2(112.0, 76.0),
        32.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.text("setup_intro"),
        vec2(112.0, 112.0),
        18.0,
        MUTED,
    );
    draw_line(112.0, 128.0, 1168.0, 128.0, 1.0, BRASS);
}

fn draw_choices(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("setup_kingdom_name"),
        vec2(120.0, 158.0),
        19.0,
        CREAM,
    );
    draw_rectangle(120.0, 170.0, 540.0, 54.0, Color::new(0.13, 0.20, 0.19, 1.0));
    draw_rectangle_lines(120.0, 170.0, 540.0, 54.0, 1.0, BRASS);
    let name = macroquad_toolkit::ui::truncate_text_to_width_ex(
        &ctx.setup.kingdom_name,
        510.0,
        ctx.body_font(),
        24.0,
    );
    body(ctx, &name, vec2(138.0, 205.0), 24.0, CREAM);
    if button(
        ctx,
        Rect::new(704.0, 170.0, 464.0, 54.0),
        &ctx.text("setup_edit_name"),
        true,
        false,
    ) {
        return Some(UiAction::OpenSetupName);
    }

    body(
        ctx,
        &ctx.text("setup_emblem"),
        vec2(120.0, 268.0),
        19.0,
        CREAM,
    );
    let emblem_count = ctx.rules.emblems.len().max(1);
    let width = 126.0;
    let gap = 8.0;
    for (index, definition) in ctx.rules.emblems.iter().enumerate() {
        let rect = Rect::new(120.0 + index as f32 * (width + gap), 280.0, width, 48.0);
        if button(
            ctx,
            rect,
            &definition.name,
            true,
            definition.id == ctx.setup.emblem,
        ) {
            return Some(UiAction::SelectSetupEmblem(definition.id));
        }
    }
    let _ = emblem_count;

    body(
        ctx,
        &ctx.text("setup_factions"),
        vec2(120.0, 382.0),
        19.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.text("setup_factions_help"),
        vec2(120.0, 408.0),
        17.0,
        MUTED,
    );
    if button(
        ctx,
        Rect::new(620.0, 354.0, 56.0, 48.0),
        "−",
        ctx.setup.factions > ctx.rules.min_factions,
        false,
    ) {
        return Some(UiAction::ChangeSetupFactionCount(-1));
    }
    centered(
        ctx,
        &ctx.setup.factions.to_string(),
        vec2(712.0, 389.0),
        30.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(748.0, 354.0, 56.0, 48.0),
        "+",
        ctx.setup.factions < ctx.rules.max_factions,
        false,
    ) {
        return Some(UiAction::ChangeSetupFactionCount(1));
    }

    body(
        ctx,
        &ctx.text("setup_seed"),
        vec2(120.0, 474.0),
        19.0,
        CREAM,
    );
    draw_rectangle(120.0, 488.0, 640.0, 50.0, Color::new(0.13, 0.20, 0.19, 1.0));
    body(
        ctx,
        &ctx.setup.seed.to_string(),
        vec2(140.0, 521.0),
        22.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(788.0, 488.0, 380.0, 50.0),
        &ctx.text("setup_randomize_seed"),
        true,
        false,
    ) {
        return Some(UiAction::RandomizeSetupSeed);
    }

    if button(
        ctx,
        Rect::new(120.0, 604.0, 230.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    let valid_name = ctx.rules.valid_kingdom_name(&ctx.setup.kingdom_name);
    if button(
        ctx,
        Rect::new(848.0, 604.0, 320.0, 48.0),
        &ctx.text("setup_create"),
        valid_name,
        true,
    ) {
        return Some(UiAction::StartProductionCampaign);
    }
    None
}

fn draw_name_keyboard(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("setup_name_help"),
        vec2(120.0, 165.0),
        19.0,
        MUTED,
    );
    draw_rectangle(
        120.0,
        190.0,
        1048.0,
        54.0,
        Color::new(0.13, 0.20, 0.19, 1.0),
    );
    let name = macroquad_toolkit::ui::truncate_text_to_width_ex(
        &ctx.setup.kingdom_name,
        1010.0,
        ctx.body_font(),
        24.0,
    );
    body(ctx, &name, vec2(140.0, 225.0), 24.0, CREAM);
    match keyboard_keys(
        Rect::new(120.0, 264.0, 1048.0, 282.0),
        ctx.setup.keyboard_page,
    ) {
        Ok(keys) => {
            for key in keys {
                if input_key(ctx, key.rect, &key.label) {
                    return Some(UiAction::EditSetupName(key.action));
                }
            }
        }
        Err(error) => body(ctx, &error, vec2(140.0, 294.0), 18.0, CREAM),
    }
    let status = ctx
        .setup
        .name_error
        .as_deref()
        .unwrap_or("32 character maximum");
    body(ctx, status, vec2(140.0, 574.0), 17.0, CREAM);
    if button(
        ctx,
        Rect::new(120.0, 612.0, 220.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::SetupNameDone);
    }
    if button(
        ctx,
        Rect::new(948.0, 612.0, 220.0, 48.0),
        &ctx.text("setup_name_done"),
        true,
        true,
    ) {
        return Some(UiAction::SetupNameDone);
    }
    None
}
