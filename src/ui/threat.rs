//! One adjacent expedition decision; threat strength remains unknown until contact.
use super::{components::*, Context, UiAction};
use kestrum::{
    engine::{ThreatPreview, ThreatView, VisibleThreat},
    state::{military::ArmyId, threat::ThreatId},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

pub const THREAT_PAGE_SIZE: usize = 5;
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThreatStage {
    Targets,
    #[default]
    Group,
    Review,
}
#[derive(Debug, Default)]
pub struct ThreatPanel {
    pub stage: ThreatStage,
    pub id: Option<ThreatId>,
    pub selected: Vec<ArmyId>,
    pub page: usize,
    pub targets: Vec<VisibleThreat>,
    pub view: Option<ThreatView>,
    pub preview: Option<ThreatPreview>,
    pub blocked: Option<String>,
    pub status: String,
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(
        ctx,
        &ctx.text("clear_threat"),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    if let Some(view) = &ctx.threat.view {
        let site = ctx
            .campaign_view
            .and_then(|c| c.world.site(view.threat.site))
            .map(|s| s.name.as_str())
            .unwrap_or_default();
        body(
            ctx,
            &truncate_text_to_width_ex(
                &format!("{} / {site}", view.threat.name),
                1056.0,
                ctx.body_font(),
                20.0,
            ),
            vec2(112.0, 127.0),
            20.0,
            CREAM,
        );
    }
    let action = match ctx.threat.stage {
        ThreatStage::Targets => targets(ctx),
        ThreatStage::Group => group(ctx),
        ThreatStage::Review => review(ctx),
    };
    lines(
        ctx,
        &ctx.threat.status,
        vec2(112.0, 598.0),
        1056.0,
        1,
        BRASS,
    );
    action.or_else(|| {
        button(
            ctx,
            Rect::new(112.0, 626.0, 166.0, 48.0),
            &ctx.text("back"),
            true,
            false,
        )
        .then_some(UiAction::ThreatBack)
    })
}

fn targets(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, threat) in ctx
        .threat
        .targets
        .iter()
        .skip(ctx.threat.page * THREAT_PAGE_SIZE)
        .take(THREAT_PAGE_SIZE)
        .enumerate()
    {
        let name = ctx
            .campaign_view
            .and_then(|c| c.world.site(threat.site))
            .map(|s| s.name.as_str())
            .unwrap_or_default();
        let y = 210.0 + index as f32 * 66.0;
        body(
            ctx,
            &truncate_text_to_width_ex(
                &format!("{} / {name}", threat.name),
                810.0,
                ctx.body_font(),
                20.0,
            ),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        if button(
            ctx,
            Rect::new(966.0, y - 22.0, 202.0, 48.0),
            &ctx.text("select"),
            true,
            false,
        ) {
            return Some(UiAction::OpenThreat(threat.id));
        }
    }
    if ctx.threat.targets.is_empty() {
        lines(
            ctx,
            &ctx.text("threat_none_adjacent"),
            vec2(112.0, 220.0),
            1056.0,
            3,
            MUTED,
        );
    }
    pages(ctx, ctx.threat.targets.len())
}

fn group(ctx: &Context<'_>) -> Option<UiAction> {
    let Some(view) = &ctx.threat.view else {
        lines(
            ctx,
            &ctx.text("threat_unavailable"),
            vec2(112.0, 210.0),
            1056.0,
            3,
            MUTED,
        );
        return None;
    };
    lines(
        ctx,
        &ctx.text("threat_choose_armies"),
        vec2(112.0, 174.0),
        1056.0,
        2,
        MUTED,
    );
    for (index, army) in view
        .armies
        .iter()
        .skip(ctx.threat.page * THREAT_PAGE_SIZE)
        .take(THREAT_PAGE_SIZE)
        .enumerate()
    {
        let y = 237.0 + index as f32 * 58.0;
        body(
            ctx,
            &truncate_text_to_width_ex(&army.name, 820.0, ctx.body_font(), 20.0),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        let note = army.blocked.clone().unwrap_or_else(|| {
            ctx.campaign_view
                .and_then(|c| c.world.site(army.origin))
                .map(|s| s.name.clone())
                .unwrap_or_default()
        });
        lines(
            ctx,
            &note,
            vec2(112.0, y + 23.0),
            820.0,
            1,
            if army.blocked.is_some() { BRASS } else { MUTED },
        );
        let selected = ctx.threat.selected.contains(&army.id);
        if button(
            ctx,
            Rect::new(966.0, y - 18.0, 202.0, 48.0),
            &ctx.text(if selected { "selected" } else { "select" }),
            army.blocked.is_none(),
            selected,
        ) {
            return Some(UiAction::ToggleThreatArmy(army.id));
        }
    }
    if view.armies.is_empty() {
        lines(
            ctx,
            &ctx.text("threat_no_armies"),
            vec2(112.0, 245.0),
            1056.0,
            3,
            BRASS,
        );
    }
    if let Some(action) = pages(ctx, view.armies.len()) {
        return Some(action);
    }
    if button(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        &ctx.text("review_clear_threat"),
        !ctx.threat.selected.is_empty(),
        true,
    ) {
        return Some(UiAction::ReviewThreat);
    }
    None
}

fn review(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.threat.view.as_ref()?;
    lines(
        ctx,
        &ctx.text("threat_review_risk"),
        vec2(112.0, 205.0),
        1056.0,
        3,
        CREAM,
    );
    body(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("siege_selected_forces"),
            ctx.threat.selected.len()
        ),
        vec2(112.0, 313.0),
        20.0,
        CREAM,
    );
    if let Some(preview) = &ctx.threat.preview {
        body(
            ctx,
            &format!(
                "{}: {} · {}: {}",
                ctx.text("cost"),
                preview.cost,
                ctx.text("movement_left"),
                preview.remaining
            ),
            vec2(112.0, 351.0),
            20.0,
            CREAM,
        );
    }
    lines(
        ctx,
        &format!(
            "{}: {} {} · {} {} · {} {}",
            ctx.text("threat_reward_offer"),
            view.reward.gold,
            ctx.text("gold"),
            view.reward.wood,
            ctx.text("wood"),
            view.reward.stone,
            ctx.text("stone")
        ),
        vec2(112.0, 407.0),
        1056.0,
        2,
        BRASS,
    );
    lines(
        ctx,
        &ctx.text("threat_reward_once"),
        vec2(112.0, 462.0),
        1056.0,
        2,
        MUTED,
    );
    if let Some(reason) = &ctx.threat.blocked {
        lines(ctx, reason, vec2(112.0, 525.0), 1056.0, 2, BRASS);
    }
    button(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        &ctx.text("confirm_clear_threat"),
        ctx.threat.preview.is_some(),
        true,
    )
    .then_some(UiAction::ConfirmThreat)
}

fn pages(ctx: &Context<'_>, count: usize) -> Option<UiAction> {
    let total = count.div_ceil(THREAT_PAGE_SIZE).max(1);
    let page = ctx.threat.page.min(total - 1);
    centered(
        ctx,
        &format!("{} / {total}", page + 1),
        vec2(640.0, 567.0),
        20.0,
        CREAM,
    );
    for (x, delta, key, enabled) in [
        (112.0, -1, "previous", page > 0),
        (1008.0, 1, "next", page + 1 < total),
    ] {
        if button(
            ctx,
            Rect::new(x, 536.0, 160.0, 48.0),
            &ctx.text(key),
            enabled,
            false,
        ) {
            return Some(UiAction::ThreatPage(delta));
        }
    }
    None
}

fn lines(ctx: &Context<'_>, value: &str, at: Vec2, width: f32, count: usize, color: Color) {
    for (index, line) in wrap_text_ex(value, width, ctx.body_font(), 18.0)
        .iter()
        .take(count)
        .enumerate()
    {
        body(ctx, line, at + vec2(0.0, index as f32 * 23.0), 18.0, color);
    }
}

pub fn prepare_text(ctx: &Context<'_>) {
    let mut text = vec![ctx.threat.status.clone()];
    text.extend(ctx.threat.blocked.iter().cloned());
    if let Some(view) = &ctx.threat.view {
        text.push(view.threat.name.clone());
        for army in view
            .armies
            .iter()
            .skip(ctx.threat.page * THREAT_PAGE_SIZE)
            .take(THREAT_PAGE_SIZE)
        {
            text.push(army.name.clone());
            text.extend(army.blocked.iter().cloned());
        }
    }
    for threat in ctx
        .threat
        .targets
        .iter()
        .skip(ctx.threat.page * THREAT_PAGE_SIZE)
        .take(THREAT_PAGE_SIZE)
    {
        text.push(threat.name.clone());
    }
    if let Some(font) = ctx.body_font() {
        let samples: Vec<_> = text
            .iter()
            .flat_map(|s| [(18, s.as_str()), (20, s.as_str())])
            .collect();
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
}
