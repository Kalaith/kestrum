//! Short topic pages flow by wrapped height within the centered help sheet.
use super::*;
use macroquad_toolkit::ui::wrap_text_ex;

const PAGES: &[&[&str]] = &[
    &["help_select", "help_region", "help_navigation"],
    &["help_turn", "help_claim", "help_scope", "help_menu"],
    &["help_army", "help_recruit", "help_slots"],
    &["help_economy", "help_disband", "help_leadership"],
    &["help_move", "help_route", "help_spent"],
    &["help_transfer", "help_recovery", "help_transfer_phase"],
    &["help_battle", "help_battle_reports", "help_wounds"],
    &["help_service", "help_careers", "help_history_knowledge"],
    &[
        "help_construction",
        "help_builder",
        "help_construction_refund",
    ],
    &["help_siege", "help_siege_choices", "help_relief"],
    &["help_development", "help_local_actions", "help_threats"],
    &[
        "help_diplomacy",
        "help_kingdom_decisions",
        "help_kingdom_ending",
    ],
    &["help_aging", "help_aging_choices", "help_retirement"],
    &[
        "help_mentorship",
        "help_mentorship_contact",
        "help_apprenticeship",
    ],
    &["help_households"],
    &["help_succession"],
    &["help_heirlooms"],
    &["help_notifications"],
    &["help_map_scales", "help_map_symbols", "help_map_contacts"],
];
pub const HELP_PAGE_COUNT: usize = PAGES.len();
pub const MAP_KEY_PAGE: usize = HELP_PAGE_COUNT - 1;

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.observer_mode)
    {
        let mut y = 230.0;
        let keys = if ctx.help_page == MAP_KEY_PAGE {
            &[
                "help_map_scales",
                "help_map_symbols",
                "observer_full_visibility",
            ][..]
        } else {
            &[
                "observer_full_visibility",
                "observer_tutorial",
                "observer_roster_help",
            ][..]
        };
        for key in keys {
            for line in wrap_text_ex(&ctx.text(key), 653.0, ctx.body_font(), 19.0) {
                body(ctx, &line, vec2(314.0, y), 19.0, CREAM);
                y += 28.0;
            }
            y += 14.0;
        }
        return None;
    }
    let legacy = ctx.state.campaign.is_some() && ctx.campaign_view.is_none();
    let keys = if legacy {
        &["legacy_read_only", "help_pan", "help_zoom", "help_menu"][..]
    } else {
        PAGES[ctx.help_page.min(HELP_PAGE_COUNT - 1)]
    };
    let mut y = 223.0;
    for key in keys {
        for line in wrap_text_ex(&ctx.text(key), 653.0, ctx.body_font(), 19.0) {
            body(ctx, &line, vec2(314.0, y), 19.0, CREAM);
            y += 28.0;
        }
        y += 14.0;
    }
    if legacy {
        return None;
    }
    if let Some(campaign) = ctx
        .state
        .campaign
        .as_ref()
        .and_then(kestrum::state::Campaign::strategic)
    {
        let key = if campaign.tutorial.is_complete() {
            "tutorial_restart"
        } else {
            "tutorial_resume"
        };
        if button(
            ctx,
            Rect::new(378.0, 40.0, 524.0, 48.0),
            &ctx.text(key),
            true,
            true,
        ) {
            return Some(UiAction::ReopenTutorial);
        }
    }
    if button(
        ctx,
        Rect::new(390.0, 548.0, 124.0, 48.0),
        &ctx.text("previous"),
        ctx.help_page > 0,
        false,
    ) {
        return Some(UiAction::HelpPage(-1));
    }
    if button(
        ctx,
        Rect::new(766.0, 548.0, 124.0, 48.0),
        &ctx.text("next"),
        ctx.help_page + 1 < HELP_PAGE_COUNT,
        false,
    ) {
        return Some(UiAction::HelpPage(1));
    }
    None
}
