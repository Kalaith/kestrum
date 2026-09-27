//! Dismissible information about a selected public place.

use super::{components::*, world::is_anchor, Context, UiAction};
use kestrum::{
    data::world::{AnchorExpression, FactionId, MarkerId, MarkerLocation, Site, SiteId},
    navigation::{MapNavigation, MapSelection, MapView, WIDTH},
    state::{world::CampaignWorld, Overlay},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::wrap_text_ex;

/// Drawing and input use the same side, opposite the selected map target.
pub fn bounds(navigation: &MapNavigation, world: &CampaignWorld, view: &MapView) -> Option<Rect> {
    let position = match navigation.selection()? {
        MapSelection::Marker(id) => world.marker(id)?.position,
        MapSelection::Site(id) => world.site(id)?.position,
    };
    let left = view.project_normalized(position).x >= WIDTH * 0.5;
    Some(Rect::new(
        if left { 24.0 } else { 898.0 },
        96.0,
        358.0,
        532.0,
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
    if let Some(site) = site {
        site_details(ctx, site, &mut sections);
    } else {
        region_details(ctx, marker.id, &mut sections);
    }
    panel(ctx, rect, name, sections);
    controls(ctx, rect, marker.id, site)
}

fn controls(
    ctx: &Context<'_>,
    rect: Rect,
    marker: MarkerId,
    site: Option<&Site>,
) -> Option<UiAction> {
    let active = ctx.state.overlay == Overlay::None;
    if let Some(threat) = site.and_then(|site| {
        ctx.campaign_view?
            .threats
            .iter()
            .find(|threat| threat.site == site.id)
    }) {
        for (x, key, action) in [
            (16.0, "clear_threat", UiAction::OpenThreat(threat.id)),
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
                active,
                x < 100.0,
            ) {
                return Some(action);
            }
        }
        return None;
    }
    let siege = site.is_some_and(|site| {
        ctx.campaign_view
            .is_some_and(|view| view.sieges.iter().any(|siege| siege.site == site.id))
    });
    if let Some(site) = site.filter(|site| {
        siege || Some(site.controller) == ctx.campaign_view.map(|view| Some(view.observer))
    }) {
        let primary = if siege {
            ("siege", UiAction::OpenSiege(site.id))
        } else {
            ("settlement_manage", UiAction::OpenSettlement(site.id))
        };
        for (index, (key, action)) in [
            primary,
            ("armies", UiAction::OpenArmies(site.id)),
            (
                "history",
                UiAction::OpenHistory(kestrum::state::history::HistorySubject::Site(site.id)),
            ),
            ("close", UiAction::CloseSelection),
        ]
        .into_iter()
        .enumerate()
        {
            let button_rect = Rect::new(
                rect.x + 16.0 + (index % 2) as f32 * 170.0,
                rect.y + rect.h - 118.0 + (index / 2) as f32 * 56.0,
                154.0,
                48.0,
            );
            if button(ctx, button_rect, &ctx.text(key), active, index == 0) {
                return Some(action);
            }
        }
        return None;
    }
    let (key, action) = if let Some(site) = site {
        (
            "history",
            UiAction::OpenHistory(kestrum::state::history::HistorySubject::Site(site.id)),
        )
    } else {
        ("enter_region", UiAction::EnterRegion(marker))
    };
    if button(
        ctx,
        Rect::new(rect.x + 16.0, rect.y + rect.h - 62.0, 192.0, 48.0),
        &ctx.text(key),
        active,
        site.is_none(),
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
            owner_name(ctx, control.political_owner, "unclaimed")
        ));
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
    sections.push(format!(
        "{}: {}",
        ctx.text("anchor_requirements"),
        anchor_description(ctx, anchors)
    ));
    sections.push(ctx.text("anchors_help"));
    let mut gates: Vec<_> = entrances
        .iter()
        .filter_map(|entry| world.site(entry.site))
        .map(|site| site.name.as_str())
        .collect();
    gates.sort_unstable();
    gates.dedup();
    sections.push(format!("{}: {}", ctx.text("entrances"), gates.join(", ")));
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
        owner_name(ctx, site.controller, "uncontrolled")
    ));
    if campaign.sieges.iter().any(|siege| siege.site == site.id) {
        sections.push(ctx.text("siege_underway"));
    } else if world.contested_sites.contains(&site.id) {
        sections.push(ctx.text("contested"));
    }
    if let Some(control) = world.region_control(site.marker) {
        sections.push(format!(
            "{}: {}{}",
            ctx.text("political_claim"),
            owner_name(ctx, control.political_owner, "unclaimed"),
            if control.contested {
                format!(" ({})", ctx.text("contested"))
            } else {
                String::new()
            }
        ));
    }
    sections.push(ctx.text(if campaign.supplied_sites.contains(&site.id) {
        "site_supplied"
    } else {
        "site_unsupplied"
    }));
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
