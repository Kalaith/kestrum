//! Local interventions share the settlement sheet and ordinary previews.
use super::*;
use macroquad_toolkit::ui::text_entry::keyboard_keys;

pub(super) fn actions(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.settlement.development.as_ref()?;
    for (index, action) in [
        LocalAction::DevelopCity,
        LocalAction::Rename,
        LocalAction::Resettle,
        LocalAction::MoveCapital,
        LocalAction::RelocateHeadquarters,
    ]
    .into_iter()
    .enumerate()
    {
        let y = 205.0 + index as f32 * 76.0;
        if control(
            ctx,
            Rect::new(112.0, y, 292.0, 48.0),
            action_key(action),
            true,
            action == LocalAction::DevelopCity,
        ) {
            return Some(UiAction::SelectLocalAction(action));
        }
        let cost = if action == LocalAction::DevelopCity {
            ctx.settlement
                .city_development
                .as_ref()
                .map(|option| option.cost)
                .unwrap_or(Resources {
                    gold: 0,
                    wood: 0,
                    stone: 0,
                })
        } else {
            action_cost(view, action)
        };
        let reason = match action {
            LocalAction::DevelopCity => ctx
                .settlement
                .city_development
                .as_ref()
                .and_then(|option| option.blocked.as_deref()),
            LocalAction::MoveCapital => view.capital_blocked.as_deref(),
            LocalAction::RelocateHeadquarters => view.headquarters_blocked.as_deref(),
            _ => None,
        };
        let detail = format!(
            "{} · {}",
            resources(ctx, cost),
            reason
                .map(str::to_owned)
                .unwrap_or_else(|| ctx.text(help_key(action)))
        );
        lines(
            ctx,
            &detail,
            vec2(436.0, y + 19.0),
            732.0,
            3,
            if reason.is_some() { BRASS } else { MUTED },
        );
    }
    None
}

pub(super) fn destinations(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, destination) in ctx
        .settlement
        .destinations
        .iter()
        .skip(ctx.settlement.page * SETTLEMENT_PAGE_SIZE)
        .take(SETTLEMENT_PAGE_SIZE)
        .enumerate()
    {
        let y = 238.0 + index as f32 * 80.0;
        body(
            ctx,
            &truncate_text_to_width_ex(&destination.name, 820.0, ctx.body_font(), 20.0),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        if let Some(reason) = &destination.blocked {
            lines(ctx, reason, vec2(112.0, y + 25.0), 820.0, 2, BRASS);
        }
        if control(
            ctx,
            Rect::new(966.0, y - 18.0, 202.0, 48.0),
            "select",
            true,
            false,
        ) {
            return Some(UiAction::SelectResettleDestination(destination.site));
        }
    }
    if ctx.settlement.destinations.is_empty() {
        lines(
            ctx,
            &ctx.text(
                if ctx
                    .settlement
                    .development
                    .as_ref()
                    .is_some_and(|view| view.displaced == 0)
                {
                    "resettle_no_people"
                } else {
                    "resettle_no_destination"
                },
            ),
            vec2(112.0, 245.0),
            1056.0,
            3,
            BRASS,
        );
    }
    page_controls(ctx, ctx.settlement.destinations.len())
}

pub(super) fn rename(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(154.0, 210.0, 972.0, 58.0, Color::new(0.13, 0.20, 0.19, 1.0));
    body(
        ctx,
        &truncate_text_to_width_ex(&ctx.settlement.name, 932.0, ctx.body_font(), 23.0),
        vec2(174.0, 249.0),
        23.0,
        CREAM,
    );
    if let Ok(keys) = keyboard_keys(
        Rect::new(154.0, 280.0, 972.0, 290.0),
        ctx.settlement.keyboard_page,
    ) {
        for key in keys {
            if input_key(ctx, key.rect, &key.label) {
                return Some(UiAction::EditPlaceName(key.action));
            }
        }
    }
    let note = if !ctx.settlement.status.is_empty() {
        ctx.settlement.status.clone()
    } else {
        ctx.settlement
            .blocked
            .clone()
            .unwrap_or_else(|| format!("{} / 40", ctx.settlement.name.chars().count()))
    };
    lines(ctx, &note, vec2(154.0, 593.0), 972.0, 1, BRASS);
    confirm(ctx)
}

pub(super) fn review(ctx: &Context<'_>) -> Option<UiAction> {
    let action = ctx.settlement.local_action?;
    if action == LocalAction::DevelopCity {
        return city_review(ctx);
    }
    let view = ctx.settlement.development.as_ref()?;
    body(
        ctx,
        &ctx.text(action_key(action)),
        vec2(112.0, 240.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &ctx.text(help_key(action)),
        vec2(112.0, 280.0),
        1056.0,
        3,
        CREAM,
    );
    body(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("cost"),
            resources(ctx, action_cost(view, action))
        ),
        vec2(112.0, 370.0),
        20.0,
        BRASS,
    );
    if let Some(destination) = ctx
        .settlement
        .destination
        .and_then(|id| ctx.settlement.destinations.iter().find(|d| d.site == id))
    {
        lines(
            ctx,
            &format!("{}: {}", ctx.text("siege_exit"), destination.name),
            vec2(112.0, 412.0),
            1056.0,
            2,
            CREAM,
        );
    }
    if let Some(reason) = &ctx.settlement.blocked {
        lines(ctx, reason, vec2(112.0, 491.0), 1056.0, 3, BRASS);
    }
    confirm(ctx)
}

fn city_review(ctx: &Context<'_>) -> Option<UiAction> {
    let option = ctx.settlement.city_development.as_ref()?;
    body(
        ctx,
        &ctx.text("develop_city"),
        vec2(112.0, 240.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &ctx.text("develop_city_help"),
        vec2(112.0, 280.0),
        1056.0,
        3,
        CREAM,
    );
    body(
        ctx,
        &format!("{}: {}", ctx.text("cost"), resources(ctx, option.cost)),
        vec2(112.0, 370.0),
        20.0,
        BRASS,
    );
    if let Some(reason) = &ctx.settlement.blocked {
        lines(ctx, reason, vec2(112.0, 491.0), 1056.0, 3, BRASS);
    }
    confirm(ctx)
}

fn confirm(ctx: &Context<'_>) -> Option<UiAction> {
    control(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        "confirm_local_action",
        ctx.settlement.blocked.is_none(),
        true,
    )
    .then_some(UiAction::ConfirmLocalAction)
}

pub(super) fn action_key(action: LocalAction) -> &'static str {
    match action {
        LocalAction::DevelopCity => "develop_city",
        LocalAction::Rename => "rename_place",
        LocalAction::Resettle => "resettle",
        LocalAction::MoveCapital => "move_capital",
        LocalAction::RelocateHeadquarters => "relocate_hq",
    }
}

fn help_key(action: LocalAction) -> &'static str {
    match action {
        LocalAction::DevelopCity => "develop_city_help",
        LocalAction::Rename => "rename_place_help",
        LocalAction::Resettle => "resettle_help",
        LocalAction::MoveCapital => "move_capital_help",
        LocalAction::RelocateHeadquarters => "relocate_hq_help",
    }
}

fn action_cost(view: &kestrum::engine::DevelopmentView, action: LocalAction) -> Resources {
    match action {
        LocalAction::DevelopCity => Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
        LocalAction::Rename => Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
        LocalAction::Resettle => view.resettle_cost,
        LocalAction::MoveCapital => view.capital_cost,
        LocalAction::RelocateHeadquarters => view.headquarters_cost,
    }
}

pub(super) fn dynamic_text(ctx: &Context<'_>) -> Vec<String> {
    let mut text = Vec::new();
    if let Some(reason) = ctx
        .settlement
        .city_development
        .as_ref()
        .and_then(|option| option.blocked.as_ref())
    {
        text.push(reason.clone());
    }
    if let Some(view) = &ctx.settlement.development {
        text.extend(view.causes.iter().map(|cause| cause.label.clone()));
        text.extend(view.growth_blocked.iter().cloned());
        text.extend(view.capital_blocked.iter().cloned());
        text.extend(view.headquarters_blocked.iter().cloned());
    }
    for destination in ctx
        .settlement
        .destinations
        .iter()
        .skip(ctx.settlement.page * 4)
        .take(4)
    {
        text.push(destination.name.clone());
        text.extend(destination.blocked.iter().cloned());
    }
    text
}

pub(super) fn summary(ctx: &Context<'_>) {
    let Some(view) = &ctx.settlement.development else {
        return;
    };
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let Some(site) = campaign.world.site(view.site) else {
        return;
    };
    let tier = if view.ruined {
        ctx.text("place_ruined")
    } else {
        labels::habitation(ctx, view.habitation)
    };
    body(ctx, &tier, vec2(112.0, 236.0), 20.0, CREAM);
    lines(
        ctx,
        &format!(
            "{}: {} / {} · {}: {}",
            ctx.text("settlement_population"),
            view.population,
            view.capacity,
            ctx.text("occupation"),
            view.occupation
        ),
        vec2(112.0, 264.0),
        470.0,
        1,
        MUTED,
    );
    lines(
        ctx,
        &format!(
            "{}: {:+} · {}: {}",
            ctx.text("development_pressure"),
            view.pressure,
            ctx.text("development_next"),
            view.contribution
                .map(|value| format!("{value:+}"))
                .unwrap_or_else(|| ctx.text("development_unknown"))
        ),
        vec2(112.0, 294.0),
        470.0,
        1,
        CREAM,
    );
    let mut causes = view
        .causes
        .iter()
        .map(|cause| format!("{} {:+}", cause.label, cause.amount))
        .collect::<Vec<_>>()
        .join(" · ");
    if view.safe != Some(true) {
        if !causes.is_empty() {
            causes.push_str(" · ");
        }
        causes.push_str(&ctx.text(if view.safe.is_some() {
            "place_unsafe"
        } else {
            "place_safety_unknown"
        }));
    }
    lines(ctx, &causes, vec2(112.0, 323.0), 470.0, 3, MUTED);
    let blocker = view.growth_blocked.clone().unwrap_or_else(|| {
        format!(
            "{}: {}",
            ctx.text("development_cap"),
            labels::habitation(ctx, view.maximum_habitation)
        )
    });
    lines(ctx, &blocker, vec2(112.0, 401.0), 470.0, 2, BRASS);
    infrastructure(ctx, view, site);
}

fn infrastructure(
    ctx: &Context<'_>,
    view: &kestrum::engine::DevelopmentView,
    site: &kestrum::data::world::Site,
) {
    lines(
        ctx,
        &format!(
            "{}: {}% · {}: {} · {}: {}%",
            ctx.text("settlement_damage"),
            view.structural_damage,
            ctx.text("settlement_fortification"),
            ctx.text(
                if site.military == kestrum::data::world::MilitaryLayer::Fort {
                    "construction_fort"
                } else {
                    "none"
                }
            ),
            ctx.text("siege_fort_damage"),
            view.fort_damage
        ),
        vec2(112.0, 455.0),
        470.0,
        2,
        MUTED,
    );
    let facilities = site
        .facilities
        .iter()
        .map(|f| ctx.text(labels::facility_key(*f)))
        .collect::<Vec<_>>()
        .join(", ");
    lines(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("settlement_facilities"),
            if facilities.is_empty() {
                ctx.text("none")
            } else {
                facilities
            }
        ),
        vec2(112.0, 505.0),
        470.0,
        2,
        MUTED,
    );
    lines(
        ctx,
        &format!(
            "{}: {} · {}: {} / 4",
            ctx.text("displaced_population"),
            view.displaced,
            ctx.text("ruin_conditions"),
            view.ruin_streak
        ),
        vec2(112.0, 558.0),
        470.0,
        2,
        MUTED,
    );
}

pub(super) fn income(ctx: &Context<'_>) {
    let entry = ctx
        .campaign_view
        .and_then(|campaign| campaign.factions.iter().find(|f| f.id == campaign.observer))
        .and_then(|f| f.last_economy.as_ref())
        .and_then(|s| s.settlement_inputs.as_ref())
        .and_then(|s| {
            s.sites
                .iter()
                .find(|site| Some(site.site) == ctx.settlement.site)
        });
    let Some(entry) = entry else {
        lines(
            ctx,
            &ctx.text("site_income_unknown"),
            vec2(652.0, 236.0),
            516.0,
            2,
            MUTED,
        );
        return;
    };
    lines(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("last_income"),
            resources(ctx, entry.income)
        ),
        vec2(652.0, 236.0),
        516.0,
        1,
        CREAM,
    );
    let focus = entry
        .focus
        .map(|focus| ctx.text(focus_key(focus)))
        .unwrap_or_else(|| ctx.text("none"));
    let detail = if entry.ruined || entry.besieged || entry.local_threat {
        ctx.text("site_income_stopped")
    } else {
        format!(
            "{} ×{}% · {} ×{}% · {}",
            ctx.text("income_damage"),
            entry.damage_percent,
            ctx.text("occupation"),
            entry.occupation_percent,
            focus
        )
    };
    lines(ctx, &detail, vec2(652.0, 262.0), 516.0, 1, MUTED);
}
