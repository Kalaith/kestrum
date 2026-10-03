//! Local construction decisions use owned order previews and public place facts.

mod choices;
mod details;
mod development;
mod labels;

use super::{components::*, Context, UiAction};
use kestrum::{
    data::{economy::Resources, world::SiteId},
    engine::{CityDevelopmentOption, ConstructionOption},
    state::{
        construction::{ConstructionKind, ConstructionOrder, ConstructionTarget, Focus, OrderId},
        military::ArmyId,
    },
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

pub use labels::{focus_key, habitation, kind_name, order_status};
pub const SETTLEMENT_PAGE_SIZE: usize = 4;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettlementMode {
    #[default]
    Overview,
    LocalActions,
    Rename,
    Resettle,
    LocalReview,
    Build,
    Roads,
    Review,
    Builders,
    Order,
    Cancel,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalAction {
    DevelopCity,
    Rename,
    Resettle,
    MoveCapital,
    RelocateHeadquarters,
}

#[derive(Debug)]
pub struct LocalDestination {
    pub site: SiteId,
    pub name: String,
    pub blocked: Option<String>,
}

#[derive(Debug)]
pub struct BuildChoice {
    pub target: ConstructionTarget,
    pub option: ConstructionOption,
}

#[derive(Debug)]
pub struct BuilderChoice {
    pub id: ArmyId,
    pub name: String,
    pub location: String,
    pub blocked: Option<String>,
}

#[derive(Debug)]
pub struct FocusChoice {
    pub focus: Focus,
    pub blocked: Option<String>,
}

#[derive(Debug, Default)]
pub struct SettlementView {
    pub site: Option<SiteId>,
    pub development: Option<kestrum::engine::DevelopmentView>,
    pub city_development: Option<CityDevelopmentOption>,
    pub local_action: Option<LocalAction>,
    pub destination: Option<SiteId>,
    pub destinations: Vec<LocalDestination>,
    pub name: String,
    pub keyboard_page: macroquad_toolkit::ui::text_entry::KeyboardPage,
    pub mode: SettlementMode,
    pub page: usize,
    pub choice: Option<(ConstructionTarget, ConstructionKind)>,
    pub builder: Option<ArmyId>,
    pub order: Option<OrderId>,
    pub focus: Option<Focus>,
    pub options: Vec<BuildChoice>,
    pub builders: Vec<BuilderChoice>,
    pub focuses: Vec<FocusChoice>,
    pub blocked: Option<String>,
    pub refund: Option<Resources>,
    pub status: String,
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    if let Some(action) = heading(ctx) {
        return Some(action);
    }
    let action = match ctx.settlement.mode {
        SettlementMode::Overview => details::overview(ctx),
        SettlementMode::LocalActions => development::actions(ctx),
        SettlementMode::Rename => development::rename(ctx),
        SettlementMode::Resettle => development::destinations(ctx),
        SettlementMode::LocalReview => development::review(ctx),
        SettlementMode::Build | SettlementMode::Roads => choices::build(ctx),
        SettlementMode::Review => details::review(ctx),
        SettlementMode::Builders => choices::builders(ctx),
        SettlementMode::Order | SettlementMode::Cancel => details::order(ctx),
        SettlementMode::Focus => choices::focus(ctx),
    };
    if let Some(action) = action.or_else(|| tabs(ctx)) {
        return Some(action);
    }
    if ctx.settlement.mode != SettlementMode::Rename {
        lines(
            ctx,
            &ctx.settlement.status,
            vec2(112.0, 603.0),
            1056.0,
            1,
            BRASS,
        );
    }
    control(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        "back",
        true,
        false,
    )
    .then_some(UiAction::SettlementBack)
}

fn heading(ctx: &Context<'_>) -> Option<UiAction> {
    let name = ctx
        .campaign_view
        .and_then(|view| view.world.site(ctx.settlement.site?))
        .map(|site| site.name.as_str())
        .unwrap_or("");
    let title = format!("{} / {name}", ctx.text("settlement"));
    let title_width = if ctx.settlement.mode == SettlementMode::Overview {
        500.0
    } else {
        1056.0
    };
    text(
        ctx,
        &truncate_text_to_width_ex(&title, title_width, ctx.font(), 28.0),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    if let Some(balance) = ctx
        .campaign_view
        .and_then(|view| {
            view.factions
                .iter()
                .find(|faction| faction.id == view.observer)
        })
        .and_then(|faction| faction.resources)
    {
        body(
            ctx,
            &resources(ctx, balance),
            vec2(112.0, 123.0),
            18.0,
            BRASS,
        );
    }
    if ctx.settlement.mode == SettlementMode::Overview {
        if let Some(option) = &ctx.settlement.city_development {
            if control(
                ctx,
                Rect::new(652.0, 96.0, 210.0, 48.0),
                "develop_city",
                true,
                true,
            ) {
                return Some(UiAction::SelectLocalAction(LocalAction::DevelopCity));
            }
            let cost = truncate_text_to_width_ex(
                &resources(ctx, option.cost),
                292.0,
                ctx.body_font(),
                18.0,
            );
            let hint = option
                .blocked
                .as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| ctx.text("city_development_available"));
            body(ctx, &cost, vec2(876.0, 108.0), 18.0, BRASS);
            body(
                ctx,
                &truncate_text_to_width_ex(&hint, 292.0, ctx.body_font(), 18.0),
                vec2(876.0, 131.0),
                18.0,
                if option.blocked.is_some() {
                    BRASS
                } else {
                    MUTED
                },
            );
        }
    }
    None
}

fn tabs(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, (mode, key)) in [
        (SettlementMode::Overview, "settlement_overview"),
        (SettlementMode::Build, "settlement_build"),
        (SettlementMode::Roads, "settlement_roads"),
        (SettlementMode::Focus, "settlement_focus"),
        (SettlementMode::LocalActions, "local_actions"),
    ]
    .into_iter()
    .enumerate()
    {
        if control(
            ctx,
            Rect::new(112.0 + index as f32 * 214.0, 151.0, 198.0, 48.0),
            key,
            true,
            ctx.settlement.mode == mode,
        ) {
            return Some(UiAction::SettlementTab(mode));
        }
    }
    None
}

fn control(ctx: &Context<'_>, rect: Rect, key: &str, enabled: bool, primary: bool) -> bool {
    button(ctx, rect, &ctx.text(key), enabled, primary)
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

fn resources(ctx: &Context<'_>, value: Resources) -> String {
    format!(
        "{} {} · {} {} · {} {}",
        value.gold,
        ctx.text("gold"),
        value.wood,
        ctx.text("wood"),
        value.stone,
        ctx.text("stone")
    )
}

fn current_order<'a>(ctx: &'a Context<'_>) -> Option<&'a ConstructionOrder> {
    ctx.campaign_view?
        .construction
        .iter()
        .find(|order| Some(order.id) == ctx.settlement.order)
}

fn current_choice<'a>(ctx: &'a Context<'_>) -> Option<&'a BuildChoice> {
    let (target, kind) = ctx.settlement.choice?;
    ctx.settlement
        .options
        .iter()
        .find(|choice| choice.target == target && choice.option.kind == kind)
}

fn page_controls(ctx: &Context<'_>, count: usize) -> Option<UiAction> {
    let pages = count.div_ceil(SETTLEMENT_PAGE_SIZE).max(1);
    let page = ctx.settlement.page.min(pages - 1);
    centered(
        ctx,
        &format!("{} / {pages}", page + 1),
        vec2(826.0, 657.0),
        20.0,
        CREAM,
    );
    for (x, delta, key, enabled) in [
        (476.0, -1, "previous", page > 0),
        (1008.0, 1, "next", page + 1 < pages),
    ] {
        if control(ctx, Rect::new(x, 626.0, 160.0, 48.0), key, enabled, false) {
            return Some(UiAction::SettlementPage(delta));
        }
    }
    None
}

pub fn prepare_text(ctx: &Context<'_>) {
    let mut strings = vec![ctx.settlement.status.clone(), ctx.settlement.name.clone()];
    strings.extend(development::dynamic_text(ctx));
    if let Some(reason) = &ctx.settlement.blocked {
        strings.push(reason.clone());
    }
    for choice in &ctx.settlement.options {
        strings.push(labels::target_name(ctx, choice.target));
        if let Some(reason) = &choice.option.blocked {
            strings.push(reason.clone());
        }
    }
    for choice in ctx
        .settlement
        .builders
        .iter()
        .skip(ctx.settlement.page * 4)
        .take(4)
    {
        strings.push(choice.name.clone());
        strings.push(choice.location.clone());
        if let Some(reason) = &choice.blocked {
            strings.push(reason.clone());
        }
    }
    for choice in &ctx.settlement.focuses {
        if let Some(reason) = &choice.blocked {
            strings.push(reason.clone());
        }
    }
    if let Some(order) = current_order(ctx) {
        strings.push(order_status(ctx, order));
    }
    if let Some(font) = ctx.body_font() {
        let samples: Vec<_> = strings
            .iter()
            .flat_map(|s| [(18, s.as_str()), (20, s.as_str()), (23, s.as_str())])
            .collect();
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
}
