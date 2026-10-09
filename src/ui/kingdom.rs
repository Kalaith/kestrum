//! Public relations and explicit sovereign decisions, opened only when needed.

use super::{components::*, Context, UiAction};
use kestrum::{
    data::world::{DiplomaticState, FactionId},
    engine::{DiplomacyFactionView, DiplomacyView},
    state::FactionStatus,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

pub const KINGDOM_PAGE_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KingdomIntent {
    DeclareWar,
    OfferPeace,
    AcceptPeace,
    DeclinePeace,
    Annex,
    Submission,
}

#[derive(Default)]
pub struct KingdomView {
    pub data: Option<DiplomacyView>,
    pub selected: Option<FactionId>,
    pub page: usize,
    pub intent: Option<KingdomIntent>,
    pub blocked: Option<String>,
    pub status: String,
}

impl KingdomView {
    pub fn is_save_boundary(&self) -> bool {
        self.data.as_ref().is_some_and(|view| {
            view.ending.is_some()
                || !view.incoming_offers.is_empty()
                || !view.pending_defeats.is_empty()
        })
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    text(ctx, &ctx.text("kingdom"), vec2(112.0, 82.0), 30.0, CREAM);
    lines(
        ctx,
        &ctx.text("kingdom_scope"),
        vec2(112.0, 120.0),
        1056.0,
        2,
        MUTED,
    );
    let result = if ctx.kingdom.intent.is_some() {
        review(ctx)
    } else {
        list(ctx).or_else(|| selected(ctx))
    };
    if result.is_some() {
        return result;
    }
    lines(
        ctx,
        &ctx.kingdom.status,
        vec2(112.0, 592.0),
        1056.0,
        1,
        BRASS,
    );
    if button(
        ctx,
        Rect::new(112.0, 626.0, 180.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::KingdomBack);
    }
    if ctx.kingdom.intent.is_none() {
        pages(ctx)
    } else {
        None
    }
}

fn list(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.kingdom.data.as_ref()?;
    for (index, faction) in view
        .factions
        .iter()
        .skip(ctx.kingdom.page * KINGDOM_PAGE_SIZE)
        .take(KINGDOM_PAGE_SIZE)
        .enumerate()
    {
        let y = 194.0 + index as f32 * 74.0;
        let rect = Rect::new(112.0, y, 430.0, 64.0);
        if ctx.kingdom.selected == Some(faction.id) {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.17, 0.24, 0.22, 1.0),
            );
        }
        let name = truncate_text_to_width_ex(&faction.name, 402.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(126.0, y + 25.0), 20.0, CREAM);
        let state = if view.pending_defeats.iter().any(|d| d.faction == faction.id) {
            ctx.text("kingdom_defeated_choice")
        } else if view
            .incoming_offers
            .iter()
            .any(|o| o.proposer == faction.id)
        {
            ctx.text("kingdom_peace_offered")
        } else {
            standing(ctx, faction)
        };
        body(ctx, &state, vec2(126.0, y + 50.0), 18.0, BRASS);
        if ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|p| rect.contains(p)) {
            return Some(UiAction::SelectKingdom(faction.id));
        }
    }
    None
}

fn selected(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.kingdom.data.as_ref()?;
    let faction = view
        .factions
        .iter()
        .find(|f| Some(f.id) == ctx.kingdom.selected)?;
    lines(ctx, &faction.name, vec2(592.0, 220.0), 576.0, 2, CREAM);
    body(
        ctx,
        &standing(ctx, faction),
        vec2(592.0, 280.0),
        20.0,
        BRASS,
    );
    if faction.status == FactionStatus::Independent {
        let help = format!("{}_help", faction.personality.text_key());
        lines(ctx, &ctx.text(&help), vec2(592.0, 310.0), 576.0, 1, MUTED);
    }
    if let Some(until) = faction.truce_until {
        lines(
            ctx,
            &format!(
                "{} {}",
                ctx.text("kingdom_truce_until"),
                until.saturating_add(1)
            ),
            vec2(592.0, 337.0),
            576.0,
            1,
            MUTED,
        );
    }
    let pending_defeat = view.pending_defeats.iter().any(|d| d.faction == faction.id);
    let incoming = view
        .incoming_offers
        .iter()
        .any(|o| o.proposer == faction.id);
    let choices: &[KingdomIntent] = if pending_defeat {
        &[KingdomIntent::Annex, KingdomIntent::Submission]
    } else if incoming {
        &[KingdomIntent::AcceptPeace, KingdomIntent::DeclinePeace]
    } else if faction.status != FactionStatus::Independent || view.ending.is_some() {
        &[]
    } else if faction.relation == DiplomaticState::War {
        &[KingdomIntent::OfferPeace]
    } else {
        &[KingdomIntent::DeclareWar]
    };
    for (index, intent) in choices.iter().enumerate() {
        if button(
            ctx,
            Rect::new(592.0, 370.0 + index as f32 * 60.0, 576.0, 48.0),
            &ctx.text(intent_key(*intent)),
            true,
            true,
        ) {
            return Some(UiAction::ReviewDiplomacy(*intent));
        }
    }
    let detail = if pending_defeat {
        ctx.text("kingdom_defeat_help")
    } else if incoming {
        ctx.text("kingdom_incoming_help")
    } else if let Some(intent) = choices.first() {
        match intent {
            KingdomIntent::DeclareWar => faction.declare_blocked.clone(),
            KingdomIntent::OfferPeace => faction.offer_blocked.clone(),
            _ => None,
        }
        .unwrap_or_else(|| ctx.text(intent_help(*intent)))
    } else {
        ctx.text("kingdom_inactive_help")
    };
    lines(ctx, &detail, vec2(592.0, 510.0), 576.0, 3, MUTED);
    None
}

fn review(ctx: &Context<'_>) -> Option<UiAction> {
    let intent = ctx.kingdom.intent?;
    let view = ctx.kingdom.data.as_ref()?;
    let faction = view
        .factions
        .iter()
        .find(|f| Some(f.id) == ctx.kingdom.selected)?;
    text(
        ctx,
        &ctx.text(intent_key(intent)),
        vec2(112.0, 220.0),
        28.0,
        CREAM,
    );
    lines(ctx, &faction.name, vec2(112.0, 268.0), 1056.0, 2, BRASS);
    lines(
        ctx,
        &ctx.text(intent_help(intent)),
        vec2(112.0, 352.0),
        1056.0,
        4,
        CREAM,
    );
    if let Some(reason) = &ctx.kingdom.blocked {
        lines(ctx, reason, vec2(112.0, 496.0), 1056.0, 3, BRASS);
    }
    button(
        ctx,
        Rect::new(812.0, 626.0, 356.0, 48.0),
        &ctx.text("confirm_local_action"),
        ctx.kingdom.blocked.is_none(),
        true,
    )
    .then_some(UiAction::ConfirmDiplomacy)
}

fn pages(ctx: &Context<'_>) -> Option<UiAction> {
    let count = ctx
        .kingdom
        .data
        .as_ref()?
        .factions
        .len()
        .div_ceil(KINGDOM_PAGE_SIZE)
        .max(1);
    body(
        ctx,
        &format!("{} / {}", ctx.kingdom.page + 1, count),
        vec2(907.0, 658.0),
        18.0,
        MUTED,
    );
    for (x, key, delta, enabled) in [
        (592.0, "previous", -1, ctx.kingdom.page > 0),
        (988.0, "next", 1, ctx.kingdom.page + 1 < count),
    ] {
        if button(
            ctx,
            Rect::new(x, 626.0, 180.0, 48.0),
            &ctx.text(key),
            enabled,
            false,
        ) {
            return Some(UiAction::KingdomPage(delta));
        }
    }
    None
}

/// Relation plus the ruler's temperament while the kingdom still acts on its own.
fn standing(ctx: &Context<'_>, faction: &DiplomacyFactionView) -> String {
    let relation = relation(ctx, faction);
    if faction.status == FactionStatus::Independent {
        format!("{relation} · {}", ctx.text(faction.personality.text_key()))
    } else {
        relation
    }
}

fn relation(ctx: &Context<'_>, faction: &DiplomacyFactionView) -> String {
    ctx.text(match faction.status {
        FactionStatus::Eliminated => "kingdom_eliminated",
        FactionStatus::Vassal { .. } => "kingdom_subordinate",
        FactionStatus::Independent => match faction.relation {
            DiplomaticState::War => "war",
            DiplomaticState::Peace => "peace",
        },
    })
}

pub fn intent_key(intent: KingdomIntent) -> &'static str {
    match intent {
        KingdomIntent::DeclareWar => "declare_war",
        KingdomIntent::OfferPeace => "offer_peace",
        KingdomIntent::AcceptPeace => "accept_peace",
        KingdomIntent::DeclinePeace => "decline_peace",
        KingdomIntent::Annex => "annex",
        KingdomIntent::Submission => "accept_submission",
    }
}

fn intent_help(intent: KingdomIntent) -> &'static str {
    match intent {
        KingdomIntent::DeclareWar => "declare_war_help",
        KingdomIntent::OfferPeace => "offer_peace_help",
        KingdomIntent::AcceptPeace => "accept_peace_help",
        KingdomIntent::DeclinePeace => "decline_peace_help",
        KingdomIntent::Annex => "annex_help",
        KingdomIntent::Submission => "submission_help",
    }
}

pub(super) fn lines(
    ctx: &Context<'_>,
    label: &str,
    at: Vec2,
    width: f32,
    limit: usize,
    color: Color,
) {
    for (index, line) in wrap_text_ex(label, width, ctx.body_font(), 19.0)
        .into_iter()
        .take(limit)
        .enumerate()
    {
        body(ctx, &line, at + vec2(0.0, index as f32 * 27.0), 19.0, color);
    }
}

pub fn prepare_text(ctx: &Context<'_>) {
    let Some(font) = ctx.body_font() else {
        return;
    };
    let mut strings = vec![ctx.kingdom.status.as_str()];
    strings.extend(ctx.kingdom.blocked.as_deref());
    if let Some(view) = &ctx.kingdom.data {
        for faction in view
            .factions
            .iter()
            .filter(|f| Some(f.id) == ctx.kingdom.selected)
            .chain(
                view.factions
                    .iter()
                    .skip(ctx.kingdom.page * KINGDOM_PAGE_SIZE)
                    .take(KINGDOM_PAGE_SIZE),
            )
        {
            strings.push(&faction.name);
            strings.extend(faction.declare_blocked.as_deref());
            strings.extend(faction.offer_blocked.as_deref());
        }
    }
    let samples: Vec<_> = strings
        .iter()
        .flat_map(|text| [18, 19, 20].map(|size| (size, *text)))
        .collect();
    macroquad_toolkit::ui::prepare_font_text(font, &samples);
}
