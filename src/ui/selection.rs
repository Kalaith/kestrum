//! Dismissible information about a selected public place.

use super::{components::*, world::is_anchor, Context, UiAction};
use kestrum::{
    data::world::{
        AnchorExpression, DiplomaticState, FactionId, MarkerId, MarkerLocation, Site, SiteId,
    },
    navigation::{MapNavigation, MapScope, MapSelection, MapView, WIDTH},
    state::{world::CampaignWorld, FactionStatus, Overlay},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

/// Drawing and input use the same side, opposite the selected map target.
pub fn bounds(navigation: &MapNavigation, world: &CampaignWorld, view: &MapView) -> Option<Rect> {
    let position = match navigation.selection()? {
        MapSelection::Marker(id) => world.marker(id)?.position,
        MapSelection::Site(id) => {
            let site = world.site(id)?;
            match navigation.scope() {
                MapScope::World => site.position,
                MapScope::Region(region) => world.region_site_position(region, id)?,
            }
        }
    };
    let left = view.project_normalized(position).x >= WIDTH * 0.5;
    Some(Rect::new(
        if left { 24.0 } else { WIDTH - 382.0 },
        180.0,
        358.0,
        620.0,
    ))
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let world = &campaign.world;
    let rect = bounds(ctx.navigation, world, ctx.view)?;
    let selected = ctx.navigation.selection()?;
    let (name, marker, site) = match selected {
        MapSelection::Marker(id) => {
            let marker = world.marker(id)?;
            let site = match marker.location {
                MarkerLocation::Site { site } => world.site(site),
                MarkerLocation::Region { .. } => None,
            };
            (marker.name.as_str(), marker, site)
        }
        MapSelection::Site(id) => {
            let site = world.site(id)?;
            (site.name.as_str(), world.marker(site.marker)?, Some(site))
        }
    };
    let mut sections = Vec::new();
    if campaign.observer_mode {
        observer_army_details(ctx, marker.id, site.map(|site| site.id), &mut sections);
    }
    if let Some(site) = site {
        site_details(ctx, site, &mut sections);
    } else {
        region_details(ctx, marker.id, &mut sections);
    }
    if campaign.observer_mode {
        observer_panel(ctx, rect, name, &sections);
    } else {
        panel(ctx, rect, name, sections);
    }
    controls(ctx, rect, marker.id, site)
}

fn controls(
    ctx: &Context<'_>,
    rect: Rect,
    marker: MarkerId,
    site: Option<&Site>,
) -> Option<UiAction> {
    let active = ctx.state.overlay == Overlay::None;
    let can_enter_region = ctx.navigation.scope() == MapScope::World
        && ctx
            .campaign_view
            .is_some_and(|campaign| campaign.world.is_region_available(marker));
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.observer_mode)
    {
        return observer_controls(ctx, rect, marker, site, can_enter_region);
    }
    if let Some(threat) = site.and_then(|site| {
        ctx.campaign_view?
            .threats
            .iter()
            .find(|threat| threat.site == site.id)
    }) {
        return threat_controls(ctx, rect, threat.id, marker, can_enter_region);
    }
    let siege = site.is_some_and(|site| {
        ctx.campaign_view
            .is_some_and(|view| view.sieges.iter().any(|siege| siege.site == site.id))
    });
    if let Some(site) = site.filter(|site| {
        siege || Some(site.controller) == ctx.campaign_view.map(|view| Some(view.observer))
    }) {
        return owned_controls(ctx, rect, site.id, siege, can_enter_region);
    }
    let owner = site.and_then(|site| site.controller).or_else(|| {
        ctx.campaign_view?
            .world
            .region_control(marker)?
            .political_owner
    });
    if let Some(owner) = owner.filter(|owner| {
        ctx.campaign_view
            .is_some_and(|campaign| *owner != campaign.observer)
    }) {
        if button(
            ctx,
            Rect::new(
                rect.x + 16.0,
                rect.y + rect.h - if can_enter_region { 174.0 } else { 118.0 },
                326.0,
                48.0,
            ),
            &ctx.text("kingdom"),
            active,
            true,
        ) {
            return Some(UiAction::OpenKingdom(Some(owner)));
        }
    }
    if can_enter_region
        && site.is_some()
        && button(
            ctx,
            Rect::new(rect.x + 16.0, rect.bottom() - 118.0, rect.w - 32.0, 48.0),
            &ctx.text("enter_region"),
            active,
            true,
        )
    {
        return Some(UiAction::EnterRegion(marker));
    }
    let (key, action, primary) = if let Some(site) = site {
        (
            "history",
            UiAction::OpenHistory(kestrum::state::history::HistorySubject::Site(site.id)),
            false,
        )
    } else {
        ("enter_region", UiAction::EnterRegion(marker), true)
    };
    if button(
        ctx,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 62.0, 192.0, 48.0),
        &ctx.text(key),
        active,
        primary,
    ) {
        return Some(action);
    }
    if button(
        ctx,
        Rect::new(rect.x + 220.0, rect.y + rect.h - 62.0, 122.0, 48.0),
        &ctx.text("close"),
        active,
        false,
    ) {
        return Some(UiAction::CloseSelection);
    }
    None
}

fn observer_controls(
    ctx: &Context<'_>,
    rect: Rect,
    marker: MarkerId,
    site: Option<&Site>,
    can_enter_region: bool,
) -> Option<UiAction> {
    if can_enter_region
        && site.is_some()
        && button(
            ctx,
            Rect::new(rect.x + 16.0, rect.bottom() - 118.0, rect.w - 32.0, 48.0),
            &ctx.text("enter_region"),
            ctx.state.overlay == Overlay::None,
            true,
        )
    {
        return Some(UiAction::EnterRegion(marker));
    }
    let action = if site.is_some() {
        UiAction::OpenObserverKingdoms
    } else {
        UiAction::EnterRegion(marker)
    };
    let key = if site.is_some() {
        "observer_kingdoms"
    } else {
        "enter_region"
    };
    if button(
        ctx,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 62.0, 192.0, 48.0),
        &ctx.text(key),
        ctx.state.overlay == Overlay::None,
        site.is_none(),
    ) {
        return Some(action);
    }
    if button(
        ctx,
        Rect::new(rect.x + 220.0, rect.y + rect.h - 62.0, 122.0, 48.0),
        &ctx.text("close"),
        ctx.state.overlay == Overlay::None,
        false,
    ) {
        return Some(UiAction::CloseSelection);
    }
    None
}

fn observer_army_details(
    ctx: &Context<'_>,
    marker: MarkerId,
    selected_site: Option<SiteId>,
    sections: &mut Vec<String>,
) {
    let Some(view) = ctx.campaign_view else {
        return;
    };
    let armies: Vec<_> = view
        .armies
        .iter()
        .filter(|army| {
            selected_site.map_or_else(
                || {
                    view.world
                        .site(army.site)
                        .is_some_and(|site| site.marker == marker)
                },
                |site| army.site == site,
            )
        })
        .collect();
    if armies.is_empty() {
        return;
    }
    sections.push(ctx.text("armies"));
    if selected_site.is_some() && armies.len() <= 4 {
        for army in armies {
            let owner = owner_name(ctx, Some(army.faction), "map_unknown_kingdom");
            let troops = ctx
                .overview
                .and_then(|overview| overview.armies.get(&army.id))
                .map_or(0, |summary| summary.troops);
            sections.push(format!(
                "{owner} · {}: {troops} {}",
                army.name,
                ctx.data.map.text("troops")
            ));
        }
        return;
    }
    for faction in &view.factions {
        let owned: Vec<_> = armies
            .iter()
            .filter(|army| army.faction == faction.id)
            .collect();
        if owned.is_empty() {
            continue;
        }
        let troops: u32 = owned
            .iter()
            .filter_map(|army| {
                ctx.overview
                    .and_then(|overview| overview.armies.get(&army.id))
                    .map(|summary| summary.troops)
            })
            .sum();
        sections.push(format!(
            "{}: {} {} · {troops} {}",
            faction.name,
            owned.len(),
            ctx.data.map.text("armies"),
            ctx.data.map.text("troops")
        ));
    }
}

fn threat_controls(
    ctx: &Context<'_>,
    rect: Rect,
    threat: kestrum::state::threat::ThreatId,
    marker: MarkerId,
    can_enter_region: bool,
) -> Option<UiAction> {
    if can_enter_region
        && button(
            ctx,
            Rect::new(rect.x + 16.0, rect.bottom() - 118.0, rect.w - 32.0, 48.0),
            &ctx.text("enter_region"),
            ctx.state.overlay == Overlay::None,
            true,
        )
    {
        return Some(UiAction::EnterRegion(marker));
    }
    for (x, key, action) in [
        (16.0, "clear_threat", UiAction::OpenThreat(threat)),
        (220.0, "close", UiAction::CloseSelection),
    ] {
        if button(
            ctx,
            Rect::new(
                rect.x + x,
                rect.y + rect.h - 62.0,
                if x < 100.0 { 192.0 } else { 122.0 },
                48.0,
            ),
            &ctx.text(key),
            ctx.state.overlay == Overlay::None,
            x < 100.0,
        ) {
            return Some(action);
        }
    }
    None
}

fn owned_controls(
    ctx: &Context<'_>,
    rect: Rect,
    site: SiteId,
    siege: bool,
    can_enter_region: bool,
) -> Option<UiAction> {
    let primary = if siege {
        ("siege", UiAction::OpenSiege(site))
    } else {
        ("settlement_manage", UiAction::OpenSettlement(site))
    };
    let mut actions = vec![
        primary,
        ("armies", UiAction::OpenArmies(site)),
        (
            "history",
            UiAction::OpenHistory(kestrum::state::history::HistorySubject::Site(site)),
        ),
    ];
    if can_enter_region {
        actions.push((
            "enter_region",
            UiAction::EnterRegion(ctx.campaign_view?.world.site(site)?.marker),
        ));
    }
    actions.push(("close", UiAction::CloseSelection));
    for (index, (key, action)) in actions.into_iter().enumerate() {
        let bounds = if can_enter_region && index == 4 {
            Rect::new(rect.x + 16.0, rect.bottom() - 62.0, rect.w - 32.0, 48.0)
        } else {
            Rect::new(
                rect.x + 16.0 + (index % 2) as f32 * 170.0,
                rect.bottom() - if can_enter_region { 174.0 } else { 118.0 }
                    + (index / 2) as f32 * 56.0,
                154.0,
                48.0,
            )
        };
        if button(
            ctx,
            bounds,
            &ctx.text(key),
            ctx.state.overlay == Overlay::None,
            index == 0,
        ) {
            return Some(action);
        }
    }
    None
}

fn panel(ctx: &Context<'_>, rect: Rect, name: &str, sections: Vec<String>) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(INK.r, INK.g, INK.b, 0.98),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    let mut y = rect.y + 33.0;
    for line in wrap_text_ex(name, rect.w - 32.0, ctx.font(), 24.0) {
        text(ctx, &line, vec2(rect.x + 16.0, y), 24.0, CREAM);
        y += 29.0;
    }
    draw_line(
        rect.x + 16.0,
        y - 10.0,
        rect.x + rect.w - 16.0,
        y - 10.0,
        1.0,
        BRASS,
    );
    y += 14.0;
    for section in sections {
        for line in wrap_text_ex(&section, rect.w - 32.0, ctx.body_font(), 18.0) {
            body(ctx, &line, vec2(rect.x + 16.0, y), 18.0, CREAM);
            y += 23.0;
        }
        y += 10.0;
    }
}

fn observer_panel(ctx: &Context<'_>, rect: Rect, name: &str, sections: &[String]) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(INK.r, INK.g, INK.b, 0.98),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    let title = truncate_text_to_width_ex(name, rect.w - 32.0, ctx.font(), 24.0);
    text(ctx, &title, vec2(rect.x + 16.0, rect.y + 34.0), 24.0, CREAM);
    draw_line(
        rect.x + 16.0,
        rect.y + 47.0,
        rect.right() - 16.0,
        rect.y + 47.0,
        1.0,
        BRASS,
    );
    let mut y = rect.y + 72.0;
    let bottom = rect.bottom() - 82.0;
    for section in sections {
        for line in wrap_text_ex(section, rect.w - 32.0, ctx.body_font(), 18.0) {
            if y + 23.0 > bottom {
                return;
            }
            body(ctx, &line, vec2(rect.x + 16.0, y), 18.0, CREAM);
            y += 23.0;
        }
        y += 4.0;
    }
}

fn region_details(ctx: &Context<'_>, id: MarkerId, sections: &mut Vec<String>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    let Some(marker) = world.marker(id) else {
        return;
    };
    let MarkerLocation::Region {
        sites,
        entrances,
        anchors,
    } = &marker.location
    else {
        return;
    };
    if let Some(control) = world.region_control(id) {
        sections.push(format!(
            "{}: {}",
            ctx.text("political_claim"),
            owner_relation(ctx, control.political_owner, "unclaimed")
        ));
        border_help(ctx, control.political_owner, sections);
        sections.push(ctx.text(if control.contested {
            "contested"
        } else {
            "secure_control"
        }));
    }
    let counts = world
        .controller_counts(id)
        .into_iter()
        .map(|(owner, count)| format!("{} {count}", owner_name(ctx, owner, "uncontrolled")))
        .collect::<Vec<_>>()
        .join("; ");
    sections.push(format!(
        "{} ({} {}): {counts}",
        ctx.text("local_control"),
        sites.len(),
        ctx.text("region_sites")
    ));
    if !campaign.observer_mode {
        if world.region_control(id).is_some() {
            sections.push(format!(
                "{}: {}",
                ctx.text("anchor_requirements"),
                anchor_description(ctx, anchors)
            ));
        } else {
            sections.push(ctx.text("map_unexplored_region"));
        }
        let mut gates: Vec<_> = entrances
            .iter()
            .filter_map(|entry| world.site(entry.site))
            .map(|site| site.name.as_str())
            .collect();
        gates.sort_unstable();
        gates.dedup();
        sections.push(format!("{}: {}", ctx.text("entrances"), gates.join(", ")));
    }
}

fn site_details(ctx: &Context<'_>, site: &Site, sections: &mut Vec<String>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    if let Some(threat) = campaign
        .threats
        .iter()
        .find(|threat| threat.site == site.id)
    {
        sections.push(threat.name.clone());
    }
    if campaign.hostile_presence.contains(&site.id) {
        sections.push(ctx.text("hostile_presence"));
    }
    sections.push(format!(
        "{}: {}",
        ctx.text("local_control"),
        owner_relation(ctx, site.controller, "uncontrolled")
    ));
    border_help(ctx, site.controller, sections);
    if campaign.sieges.iter().any(|siege| siege.site == site.id) {
        sections.push(ctx.text("siege_underway"));
    } else if world.contested_sites.contains(&site.id) {
        sections.push(ctx.text("contested"));
    }
    if let Some(control) = world.region_control(site.marker) {
        sections.push(format!(
            "{}: {}{}",
            ctx.text("political_claim"),
            owner_relation(ctx, control.political_owner, "unclaimed"),
            if control.contested {
                format!(" ({})", ctx.text("contested"))
            } else {
                String::new()
            }
        ));
    }
    if !campaign.observer_mode && site.controller == Some(campaign.observer) {
        sections.push(ctx.text(if campaign.supplied_sites.contains(&site.id) {
            "site_supplied"
        } else {
            "site_unsupplied"
        }));
    }
    if !campaign.observer_mode {
        site_connections(ctx, site, world, sections);
    }
}

fn site_connections(
    ctx: &Context<'_>,
    site: &Site,
    world: &CampaignWorld,
    sections: &mut Vec<String>,
) {
    if let Some(marker) = world.marker(site.marker) {
        if let MarkerLocation::Region {
            anchors, entrances, ..
        } = &marker.location
        {
            if is_anchor(anchors, site.id) {
                sections.push(ctx.text("anchor"));
            }
            let external: Vec<_> = entrances
                .iter()
                .filter(|entrance| entrance.site == site.id)
                .filter_map(|entrance| world.entrance(marker.id, entrance.route))
                .filter_map(|crossing| world.site(crossing.external_site))
                .map(|site| site.name.as_str())
                .collect();
            if !external.is_empty() {
                sections.push(format!("{}: {}", ctx.text("gate"), external.join(", ")));
            }
        }
    }
    let connected: Vec<_> = world
        .routes
        .iter()
        .filter_map(|route| route.other_endpoint(site.id))
        .filter_map(|id| world.site(id))
        .filter(|connected| {
            world.region_control(site.marker).is_none() || connected.marker == site.marker
        })
        .map(|site| site.name.as_str())
        .collect();
    sections.push(if connected.is_empty() {
        ctx.text("no_connections")
    } else {
        format!("{}: {}", ctx.text("connections"), connected.join(", "))
    });
}

fn owner_name(ctx: &Context<'_>, owner: Option<FactionId>, empty: &str) -> String {
    owner
        .and_then(|id| {
            ctx.campaign_view?
                .factions
                .iter()
                .find(|faction| faction.id == id)
        })
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| ctx.text(empty))
}

fn owner_relation(ctx: &Context<'_>, owner: Option<FactionId>, empty: &str) -> String {
    let name = owner_name(ctx, owner, empty);
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.observer_mode)
    {
        return name;
    }
    let Some(faction) = ctx.kingdom.data.as_ref().and_then(|view| {
        view.factions
            .iter()
            .find(|faction| Some(faction.id) == owner)
    }) else {
        return name;
    };
    let key = match faction.status {
        FactionStatus::Eliminated => "kingdom_eliminated",
        FactionStatus::Vassal { .. } => "kingdom_subordinate",
        FactionStatus::Independent => match faction.relation {
            DiplomaticState::War => "war",
            DiplomaticState::Peace => "peace",
        },
    };
    format!("{name} · {}", ctx.text(key))
}

fn border_help(ctx: &Context<'_>, owner: Option<FactionId>, sections: &mut Vec<String>) {
    if ctx
        .campaign_view
        .is_some_and(|campaign| campaign.observer_mode)
    {
        return;
    }
    let Some(faction) = ctx.kingdom.data.as_ref().and_then(|view| {
        view.factions.iter().find(|faction| {
            Some(faction.id) == owner && faction.status == FactionStatus::Independent
        })
    }) else {
        return;
    };
    sections.push(ctx.text(match faction.relation {
        DiplomaticState::Peace => "peaceful_border_help",
        DiplomaticState::War => "war_border_help",
    }));
}

fn anchor_description(ctx: &Context<'_>, expression: &AnchorExpression) -> String {
    match expression {
        AnchorExpression::All { conditions } => conditions
            .iter()
            .map(|condition| anchor_description(ctx, condition))
            .collect::<Vec<_>>()
            .join(" + "),
        AnchorExpression::Any { conditions } => format!(
            "({})",
            conditions
                .iter()
                .map(|condition| anchor_description(ctx, condition))
                .collect::<Vec<_>>()
                .join(" / ")
        ),
        AnchorExpression::ControlledSite { site } => site_name(ctx, *site),
        AnchorExpression::SuppliedEntrance { site } => {
            format!("{} {}", ctx.text("supplied_gate"), site_name(ctx, *site))
        }
    }
}

fn site_name(ctx: &Context<'_>, id: SiteId) -> String {
    ctx.campaign_view
        .and_then(|campaign| campaign.world.site(id))
        .map(|site| site.name.clone())
        .unwrap_or_default()
}
