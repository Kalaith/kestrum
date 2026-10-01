//! Read-only, paginated views of participant-filtered battle snapshots.

mod rows;
mod siege;
mod threat;

use super::{components::*, Context, UiAction};
use kestrum::state::battle::BattleReport;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

pub const BATTLE_ROWS_PER_PAGE: usize = 3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BattleTab {
    #[default]
    Outcome,
    Forces,
    People,
    Factors,
}

#[derive(Debug, Default)]
pub struct BattleView {
    pub index: usize,
    pub page: usize,
    pub tab: BattleTab,
}

impl BattleView {
    pub fn clamp(&mut self, reports: &[BattleReport]) {
        self.index = self.index.min(reports.len().saturating_sub(1));
        self.page = reports
            .get(self.index)
            .map(|report| self.page.min(self.page_count(report) - 1))
            .unwrap_or(0);
    }

    pub fn page_count(&self, report: &BattleReport) -> usize {
        let armies = report
            .attacker
            .armies
            .iter()
            .chain(report.defender.armies());
        let threat = usize::from(matches!(
            report.defender,
            kestrum::state::battle::BattleDefender::Threat(_)
        ));
        let count = match self.tab {
            BattleTab::Outcome => 2 + armies.count() + siege::extra_rows(report) + threat,
            BattleTab::Forces => armies.map(|army| army.formations.len()).sum::<usize>() + threat,
            BattleTab::People => {
                armies.map(|army| army.people.len()).sum::<usize>()
                    + report.person_events.len()
                    + report
                        .person_events
                        .iter()
                        .filter(|event| {
                            matches!(
                                event.outcome,
                                kestrum::state::people::PersonCombatOutcome::AssumedCommand { .. }
                            )
                        })
                        .count()
            }
            BattleTab::Factors => {
                2 + armies.count()
                    + threat
                    + usize::from(report.context != kestrum::state::battle::BattleContext::Field)
                    + report.counters.len()
                    + report.exchanges.len()
                    + rows::leadership_changes(report).len()
            }
        };
        count.div_ceil(BATTLE_ROWS_PER_PAGE).max(1)
    }
}

struct ReportRow {
    heading: String,
    detail: String,
    person: Option<kestrum::state::people::PersonId>,
}

fn current_report<'a>(ctx: &'a Context<'_>) -> Option<&'a BattleReport> {
    let reports = &ctx.campaign_view?.battles;
    reports.get(ctx.battle.index.min(reports.len().saturating_sub(1)))
}

fn report_heading(ctx: &Context<'_>, report: &BattleReport) -> String {
    let season = &ctx.data.seasons[report.completed_rounds as usize % 4];
    let year = ctx
        .data
        .start_year
        .saturating_add(report.completed_rounds / 4);
    format!(
        "{} · {season} / {} {year}",
        report.site_name,
        ctx.text("year")
    )
}

fn visible_rows(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let page = ctx.battle.page.min(ctx.battle.page_count(report) - 1);
    rows::build(ctx, report)
        .into_iter()
        .skip(page * BATTLE_ROWS_PER_PAGE)
        .take(BATTLE_ROWS_PER_PAGE)
        .collect()
}

/// Called before the frame's first visible draw; only this report page is warmed.
pub fn prepare_text(ctx: &Context<'_>) {
    let Some(report) = current_report(ctx) else {
        return;
    };
    let Some(font) = ctx.body_font() else {
        return;
    };
    let heading = report_heading(ctx, report);
    let rows = visible_rows(ctx, report);
    let mut samples = vec![(20, heading.as_str())];
    for row in &rows {
        samples.push((20, row.heading.as_str()));
        samples.push((18, row.detail.as_str()));
    }
    macroquad_toolkit::ui::prepare_font_text(font, &samples);
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(
        ctx,
        &ctx.text("battle_reports"),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    let report = current_report(ctx);
    if let Some(report) = report {
        if let Some(action) = header(ctx, report) {
            return Some(action);
        }
        if let Some(action) = draw_rows(ctx, report) {
            return Some(action);
        }
        body(
            ctx,
            &ctx.text("battle_recorded"),
            vec2(112.0, 595.0),
            16.0,
            MUTED,
        );
        if let Some(action) = page_controls(ctx, report) {
            return Some(action);
        }
    } else {
        body(
            ctx,
            &ctx.text("no_battle_reports"),
            vec2(112.0, 225.0),
            20.0,
            CREAM,
        );
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    None
}

fn draw_rows(ctx: &Context<'_>, report: &BattleReport) -> Option<UiAction> {
    for (index, row) in visible_rows(ctx, report).iter().enumerate() {
        let y = 244.0 + index as f32 * 108.0;
        let width = if row.person.is_some() { 850.0 } else { 1056.0 };
        for (line, label) in wrap_text_ex(&row.heading, width, ctx.body_font(), 20.0)
            .iter()
            .take(2)
            .enumerate()
        {
            body(ctx, label, vec2(112.0, y + line as f32 * 24.0), 20.0, CREAM);
        }
        for (line, label) in wrap_text_ex(&row.detail, width, ctx.body_font(), 18.0)
            .iter()
            .take(2)
            .enumerate()
        {
            body(
                ctx,
                label,
                vec2(112.0, y + 51.0 + line as f32 * 23.0),
                18.0,
                MUTED,
            );
        }
        if let Some(person) = row.person {
            if button(
                ctx,
                Rect::new(984.0, y - 19.0, 184.0, 48.0),
                &ctx.text("history_open"),
                true,
                false,
            ) {
                return Some(UiAction::OpenHistory(
                    kestrum::state::history::HistorySubject::Person(person),
                ));
            }
        }
        draw_line(
            112.0,
            y + 91.0,
            1168.0,
            y + 91.0,
            1.0,
            Color::new(0.21, 0.29, 0.25, 1.0),
        );
    }
    None
}

fn page_controls(ctx: &Context<'_>, report: &BattleReport) -> Option<UiAction> {
    let pages = ctx.battle.page_count(report);
    let page = ctx.battle.page.min(pages - 1);
    if button(
        ctx,
        Rect::new(460.0, 626.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::BattlePage(-1));
    }
    centered(
        ctx,
        &format!("{} / {pages}", page + 1),
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
        return Some(UiAction::BattlePage(1));
    }
    None
}

fn header(ctx: &Context<'_>, report: &BattleReport) -> Option<UiAction> {
    let reports = &ctx.campaign_view?.battles;
    let index = ctx.battle.index.min(reports.len().saturating_sub(1));
    if button(
        ctx,
        Rect::new(704.0, 60.0, 156.0, 48.0),
        &ctx.text("older_report"),
        index > 0,
        false,
    ) {
        return Some(UiAction::BattleReport(-1));
    }
    centered(
        ctx,
        &format!("{} / {}", index + 1, reports.len()),
        vec2(932.0, 91.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 60.0, 160.0, 48.0),
        &ctx.text("newer_report"),
        index + 1 < reports.len(),
        false,
    ) {
        return Some(UiAction::BattleReport(1));
    }
    let heading =
        truncate_text_to_width_ex(&report_heading(ctx, report), 1056.0, ctx.body_font(), 20.0);
    body(ctx, &heading, vec2(112.0, 132.0), 20.0, BRASS);
    for (index, (tab, key)) in [
        (BattleTab::Outcome, "battle_outcome"),
        (BattleTab::Forces, "battle_forces"),
        (BattleTab::People, "army_people"),
        (BattleTab::Factors, "battle_factors"),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 268.0, 156.0, 252.0, 48.0),
            &ctx.text(key),
            true,
            ctx.battle.tab == tab,
        ) {
            return Some(UiAction::SetBattleTab(tab));
        }
    }
    None
}
