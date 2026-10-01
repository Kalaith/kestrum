//! On-demand save selection and explicit recovery decisions.

use super::{components::*, Context, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::text_entry::{keyboard_keys, KeyboardPage};

pub const PAGE_SIZE: usize = 5;

#[derive(Debug, Clone)]
pub struct SaveRow {
    pub id: u64,
    pub name: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveMode {
    #[default]
    Browse,
    Name {
        target: Option<u64>,
    },
    ConfirmDelete(u64),
}

#[derive(Debug, Default)]
pub struct SaveView {
    pub rows: Vec<SaveRow>,
    pub page: usize,
    pub selected: Option<u64>,
    pub mode: SaveMode,
    pub name: String,
    pub keyboard_page: KeyboardPage,
    pub name_error: Option<String>,
    pub status: String,
    pub ready: bool,
    pub can_save: bool,
}

impl SaveView {
    pub fn page_count(&self) -> usize {
        self.rows.len().div_ceil(PAGE_SIZE).max(1)
    }

    pub fn selected_row(&self) -> Option<&SaveRow> {
        self.rows
            .iter()
            .skip(self.page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .find(|row| Some(row.id) == self.selected)
    }

    pub fn select_visible(&mut self) {
        if self.selected_row().is_none() {
            self.selected = self.rows.get(self.page * PAGE_SIZE).map(|row| row.id);
        }
    }

    pub fn change_page(&mut self, delta: i32) {
        self.page = self
            .page
            .saturating_add_signed(delta as isize)
            .min(self.page_count() - 1);
        self.select_visible();
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.saves;
    sheet(ctx, "save_catalogue");
    match view.mode {
        SaveMode::Browse => browse(ctx),
        SaveMode::Name { target } => name(ctx, target),
        SaveMode::ConfirmDelete(id) => confirm_delete(ctx, id),
    }
}

fn sheet(ctx: &Context<'_>, title: &str) {
    draw_rectangle(108.0, 44.0, 1064.0, 632.0, INK);
    text(ctx, &ctx.text(title), vec2(144.0, 100.0), 30.0, CREAM);
    draw_line(144.0, 125.0, 1136.0, 125.0, 1.0, BRASS);
}

fn browse(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.saves;
    if button(
        ctx,
        Rect::new(936.0, 64.0, 200.0, 48.0),
        &ctx.text("new_save"),
        view.ready && view.can_save,
        true,
    ) {
        return Some(UiAction::NameSave(None));
    }
    if view.rows.is_empty() {
        paragraph(
            ctx,
            &ctx.text("no_catalogue_saves"),
            vec2(154.0, 190.0),
            830.0,
        );
    }
    for (index, row) in view
        .rows
        .iter()
        .skip(view.page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
    {
        let rect = Rect::new(144.0, 143.0 + index as f32 * 69.0, 992.0, 62.0);
        let selected = Some(row.id) == view.selected;
        if selected || ctx.pointer.hovering_over(rect) {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.17, 0.24, 0.22, 1.0),
            );
        }
        body(
            ctx,
            &macroquad_toolkit::ui::truncate_text_to_width_ex(
                &row.name,
                948.0,
                ctx.body_font(),
                21.0,
            ),
            vec2(rect.x + 14.0, rect.y + 25.0),
            21.0,
            CREAM,
        );
        body(
            ctx,
            &row.detail,
            vec2(rect.x + 14.0, rect.y + 50.0),
            16.0,
            MUTED,
        );
        if ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin)) {
            return Some(UiAction::SelectSave(row.id));
        }
    }
    browse_controls(ctx)
}

fn browse_controls(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.saves;
    if button(
        ctx,
        Rect::new(144.0, 505.0, 132.0, 48.0),
        &ctx.text("previous"),
        view.page > 0,
        false,
    ) {
        return Some(UiAction::SavePage(-1));
    }
    centered(
        ctx,
        &format!("{} / {}", view.page + 1, view.page_count()),
        vec2(336.0, 536.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(399.0, 505.0, 120.0, 48.0),
        &ctx.text("next"),
        view.page + 1 < view.page_count(),
        false,
    ) {
        return Some(UiAction::SavePage(1));
    }
    let selected = view.selected_row();
    for (x, key, action, enabled) in [
        (
            572.0,
            "load",
            UiAction::LoadSelectedSave,
            selected.is_some(),
        ),
        (
            760.0,
            "overwrite",
            UiAction::OverwriteSave,
            selected.is_some() && view.can_save && view.ready,
        ),
        (
            948.0,
            "delete",
            UiAction::AskDeleteSave,
            selected.is_some() && view.ready,
        ),
    ] {
        if button(
            ctx,
            Rect::new(x, 505.0, 188.0, 48.0),
            &ctx.text(key),
            enabled,
            key == "load",
        ) {
            return Some(action);
        }
    }
    let mut style = macroquad_toolkit::ui::TextStyle::new(18.0, CREAM).with_line_gap(5.0);
    if let Some(font) = ctx.body_font() {
        style = style.with_font(font);
    }
    let status = macroquad_toolkit::ui::fit_text_to_box_ex(&view.status, 720.0, 44.0, style, 18.0);
    for (index, line) in status.lines.iter().enumerate() {
        body(
            ctx,
            line,
            vec2(154.0, 578.0 + index as f32 * 23.0),
            18.0,
            CREAM,
        );
    }
    if !view.ready
        && button(
            ctx,
            Rect::new(876.0, 567.0, 260.0, 48.0),
            &ctx.text("retry_storage"),
            true,
            false,
        )
    {
        return Some(UiAction::RetryStorage);
    }
    if button(
        ctx,
        Rect::new(530.0, 616.0, 220.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    None
}

fn name(ctx: &Context<'_>, target: Option<u64>) -> Option<UiAction> {
    let key = if target.is_some() {
        "overwrite_warning"
    } else {
        "save_name_help"
    };
    paragraph(ctx, &ctx.text(key), vec2(154.0, 163.0), 950.0);
    draw_rectangle(154.0, 199.0, 972.0, 62.0, Color::new(0.13, 0.20, 0.19, 1.0));
    body(ctx, &ctx.saves.name, vec2(174.0, 238.0), 23.0, CREAM);
    match keyboard_keys(
        Rect::new(154.0, 280.0, 972.0, 290.0),
        ctx.saves.keyboard_page,
    ) {
        Ok(keys) => {
            for key in keys {
                if input_key(ctx, key.rect, &key.label) {
                    return Some(UiAction::EditSaveName(key.action));
                }
            }
        }
        Err(error) => paragraph(ctx, &error, vec2(174.0, 308.0), 930.0),
    }
    let limit = kestrum::state::persistence::MAX_SAVE_NAME_CHARS;
    let name_status = ctx
        .saves
        .name_error
        .clone()
        .unwrap_or_else(|| format!("{} / {limit}", ctx.saves.name.chars().count()));
    body(ctx, &name_status, vec2(174.0, 578.0), 18.0, CREAM);
    if button(
        ctx,
        Rect::new(820.0, 600.0, 306.0, 48.0),
        &ctx.text(if target.is_some() {
            "confirm_overwrite"
        } else {
            "create_save"
        }),
        ctx.saves.ready && !ctx.saves.name.trim().is_empty(),
        true,
    ) {
        return Some(UiAction::CommitNamedSave);
    }
    if button(
        ctx,
        Rect::new(154.0, 600.0, 220.0, 48.0),
        &ctx.text("cancel"),
        true,
        false,
    ) {
        return Some(UiAction::CancelSaveEdit);
    }
    None
}

fn confirm_delete(ctx: &Context<'_>, id: u64) -> Option<UiAction> {
    paragraph(ctx, &ctx.text("delete_warning"), vec2(190.0, 206.0), 900.0);
    if let Some(row) = ctx.saves.rows.iter().find(|row| row.id == id) {
        body(
            ctx,
            &macroquad_toolkit::ui::truncate_text_to_width_ex(
                &row.name,
                890.0,
                ctx.body_font(),
                25.0,
            ),
            vec2(190.0, 296.0),
            25.0,
            CREAM,
        );
        body(ctx, &row.detail, vec2(190.0, 334.0), 19.0, MUTED);
    }
    if button(
        ctx,
        Rect::new(720.0, 448.0, 330.0, 48.0),
        &ctx.text("confirm_delete"),
        ctx.saves.ready,
        true,
    ) {
        return Some(UiAction::ConfirmDeleteSave(id));
    }
    if button(
        ctx,
        Rect::new(230.0, 448.0, 260.0, 48.0),
        &ctx.text("cancel"),
        true,
        false,
    ) {
        return Some(UiAction::CancelSaveEdit);
    }
    None
}

pub fn recovery(ctx: &Context<'_>, message: &str) -> Option<UiAction> {
    sheet(ctx, "round_unsaved");
    paragraph(
        ctx,
        &ctx.text("round_unsaved_help"),
        vec2(180.0, 203.0),
        920.0,
    );
    paragraph(ctx, message, vec2(180.0, 326.0), 920.0);
    for (x, key, action) in [
        (228.0, "retry_save", UiAction::RetrySave),
        (712.0, "continue_unsaved", UiAction::ContinueUnsaved),
    ] {
        if button(
            ctx,
            Rect::new(x, 506.0, 340.0, 48.0),
            &ctx.text(key),
            true,
            key == "retry_save",
        ) {
            return Some(action);
        }
    }
    None
}
