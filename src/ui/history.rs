//! Contextual records read only observer-filtered history and knowledge.

mod filters;
mod overview;
mod records;
mod rows;

use super::{components::*, Context, UiAction};
use kestrum::{
    engine::{HistoryFilter, HistoryPage, KnownPeoplePage, PersonKnowledge},
    state::history::HistorySubject,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{text_entry::KeyboardPage, truncate_text_to_width_ex, wrap_text_ex};

pub const HISTORY_ROWS_PER_SCREEN: usize = 5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HistoryMode {
    #[default]
    Records,
    Overview,
    Events,
    Filters,
    Search,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecordCategory {
    #[default]
    People,
    Places,
    Armies,
    Items,
}

#[derive(Debug, Default)]
pub struct HistoryView {
    pub mode: HistoryMode,
    pub category: RecordCategory,
    pub subject: Option<HistorySubject>,
    pub screen_page: usize,
    pub overview_page: usize,
    pub records_page: usize,
    pub filter: HistoryFilter,
    pub result: Option<HistoryPage>,
    pub person: Option<PersonKnowledge>,
    pub known_people: Option<KnownPeoplePage>,
    pub search: String,
    pub keyboard_page: KeyboardPage,
    pub status: String,
}

struct HistoryRow {
    heading: String,
    detail: String,
    action: Option<UiAction>,
    action_key: &'static str,
}

impl HistoryRow {
    fn new(heading: String, detail: String) -> Self {
        Self {
            heading,
            detail,
            action: None,
            action_key: "history_open",
        }
    }

    fn link(mut self, action: UiAction, key: &'static str) -> Self {
        self.action = Some(action);
        self.action_key = key;
        self
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    let title = title(ctx);
    let title = truncate_text_to_width_ex(&title, 1056.0, ctx.font(), 28.0);
    text(ctx, &title, vec2(112.0, 83.0), 28.0, CREAM);
    let action = match ctx.history.mode {
        HistoryMode::Records => records::draw(ctx),
        HistoryMode::Overview | HistoryMode::Events => subject(ctx),
        HistoryMode::Filters => filters::draw(ctx),
        HistoryMode::Search => filters::search(ctx),
    };
    if action.is_some() {
        return action;
    }
    if !ctx.history.status.is_empty() {
        let status = truncate_text_to_width_ex(&ctx.history.status, 1040.0, ctx.body_font(), 16.0);
        body(ctx, &status, vec2(112.0, 608.0), 16.0, BRASS);
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::HistoryBack);
    }
    None
}

fn title(ctx: &Context<'_>) -> String {
    match ctx.history.mode {
        HistoryMode::Records => ctx.text("records"),
        HistoryMode::Search => ctx.text("history_search"),
        HistoryMode::Filters => ctx.text("history_filters"),
        _ => overview::title(ctx),
    }
}

fn subject(ctx: &Context<'_>) -> Option<UiAction> {
    let caption = match &ctx.history.person {
        Some(PersonKnowledge::LastEncountered { .. }) => "history_last_known_help",
        _ => "history_retention_help",
    };
    body(ctx, &ctx.text(caption), vec2(112.0, 125.0), 18.0, MUTED);
    for (index, (mode, key)) in [
        (HistoryMode::Overview, "history_overview"),
        (HistoryMode::Events, "history_events"),
        (HistoryMode::Filters, "history_filters"),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 360.0, 151.0, 336.0, 48.0),
            &ctx.text(key),
            true,
            ctx.history.mode == mode,
        ) {
            return Some(UiAction::SetHistoryMode(mode));
        }
    }
    let (all_rows, page, total) = if ctx.history.mode == HistoryMode::Overview {
        let rows = overview::build(ctx);
        let total = rows.len();
        (rows, ctx.history.overview_page, total)
    } else {
        (
            rows::events(ctx),
            ctx.history.screen_page,
            ctx.history
                .result
                .as_ref()
                .map_or(0, |result| result.total_entries),
        )
    };
    let offset = if ctx.history.mode == HistoryMode::Overview {
        page * 5
    } else {
        page % 10 * 5
    };
    if let Some(action) = draw_rows(ctx, &all_rows, offset) {
        return Some(action);
    }
    pagination(ctx, page, total, ctx.history.mode == HistoryMode::Overview)
}

fn draw_rows(ctx: &Context<'_>, rows: &[HistoryRow], offset: usize) -> Option<UiAction> {
    if rows.is_empty() {
        body(
            ctx,
            &ctx.text("history_empty"),
            vec2(112.0, 254.0),
            20.0,
            CREAM,
        );
    }
    for (index, row) in rows
        .iter()
        .skip(offset)
        .take(HISTORY_ROWS_PER_SCREEN)
        .enumerate()
    {
        let y = 231.0 + index as f32 * 74.0;
        let width = if row.action.is_some() { 850.0 } else { 1056.0 };
        let heading = truncate_text_to_width_ex(&row.heading, width, ctx.body_font(), 20.0);
        body(ctx, &heading, vec2(112.0, y), 20.0, CREAM);
        for (line, label) in wrap_text_ex(&row.detail, width, ctx.body_font(), 18.0)
            .iter()
            .take(2)
            .enumerate()
        {
            body(
                ctx,
                label,
                vec2(112.0, y + 24.0 + line as f32 * 22.0),
                18.0,
                MUTED,
            );
        }
        if let Some(action) = row.action {
            if button(
                ctx,
                Rect::new(984.0, y - 20.0, 184.0, 48.0),
                &ctx.text(row.action_key),
                true,
                false,
            ) {
                return Some(action);
            }
        }
        draw_line(
            112.0,
            y + 56.0,
            1168.0,
            y + 56.0,
            1.0,
            Color::new(0.21, 0.29, 0.25, 1.0),
        );
    }
    None
}

fn pagination(ctx: &Context<'_>, page: usize, total: usize, overview: bool) -> Option<UiAction> {
    let pages = total.div_ceil(HISTORY_ROWS_PER_SCREEN).max(1);
    let action = |delta| {
        if ctx.history.mode == HistoryMode::Records {
            UiAction::RecordsPage(delta)
        } else if overview {
            UiAction::HistoryOverviewPage(delta)
        } else {
            UiAction::HistoryPage(delta)
        }
    };
    if button(
        ctx,
        Rect::new(460.0, 626.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(action(-1));
    }
    centered(
        ctx,
        &format!("{} / {pages}", page.min(pages - 1) + 1),
        vec2(816.0, 657.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 626.0, 160.0, 48.0),
        &ctx.text("next"),
        page + 1 < pages,
        false,
    ) {
        return Some(action(1));
    }
    None
}

pub fn prepare_text(ctx: &Context<'_>) {
    let rows = match ctx.history.mode {
        HistoryMode::Overview => overview::build(ctx),
        HistoryMode::Events => rows::events(ctx),
        HistoryMode::Records => records::rows(ctx),
        _ => Vec::new(),
    };
    let offset = match ctx.history.mode {
        HistoryMode::Overview => ctx.history.overview_page * 5,
        HistoryMode::Events => ctx.history.screen_page % 10 * 5,
        HistoryMode::Records if ctx.history.category == RecordCategory::People => {
            ctx.history.records_page % 10 * 5
        }
        HistoryMode::Records => ctx.history.records_page * 5,
        _ => 0,
    };
    let title = title(ctx);
    if let Some(font) = ctx.font() {
        macroquad_toolkit::ui::prepare_font_text(font, &[(28, &title)]);
    }
    if let Some(font) = ctx.body_font() {
        let mut samples = vec![
            (23, ctx.history.search.as_str()),
            (16, ctx.history.status.as_str()),
        ];
        for row in rows.iter().skip(offset).take(5) {
            samples.push((20, row.heading.as_str()));
            samples.push((18, row.detail.as_str()));
        }
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
}

fn date(ctx: &Context<'_>, round: u32) -> String {
    format!(
        "{} / {} {}",
        ctx.game_text.seasons[round as usize % 4],
        ctx.text("year"),
        ctx.data.start_year.saturating_add(round / 4)
    )
}
