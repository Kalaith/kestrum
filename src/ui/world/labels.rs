//! Map names survive density through importance, zoom, and collision priority.

use super::*;
use kestrum::{
    data::{economy::Habitation, world::MarkerLocation},
    navigation::{place_map_labels, MapLabelCandidate, HEIGHT, WIDTH},
};
use std::collections::BTreeMap;

pub(super) fn draw(
    ctx: &Context<'_>,
    targets: &[MapTarget],
    panel: Option<Rect>,
    exploration: Option<&MapExploration>,
) {
    let (Some(campaign), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return;
    };
    let (candidates, names) = candidates(ctx, targets);
    let mut reserved: Vec<_> = panel.into_iter().collect();
    reserved.push(super::super::overview::attention_bounds(
        ctx.overview_ui,
        panel.is_some(),
        overview.attention.len(),
    ));
    if let Some(tutorial) = super::super::tutorial_bounds(ctx.state) {
        reserved.push(tutorial);
    }
    if !campaign.observer_mode && ctx.state.overlay == Overlay::None {
        reserved.extend(super::super::notification_reserved_rects(
            ctx.notifications,
            ctx.notification_projection,
        ));
    }
    reserved.extend(
        ctx.navigation
            .place_groups(&campaign.world, ctx.view)
            .iter()
            .map(|group| group.bounds()),
    );
    reserved.extend(
        ctx.navigation
            .army_targets(&campaign.world, ctx.view, &campaign.armies)
            .into_iter()
            .map(|target| armies::display_bounds(ctx, &target)),
    );
    let placed = place_map_labels(candidates, targets, &reserved, ctx.view.camera.zoom());
    reserved.extend(placed.iter().map(|label| label.bounds));
    kingdoms(ctx, targets, &mut reserved, exploration);
    for label in placed {
        let rect = label.bounds;
        if let Some(target) = targets
            .iter()
            .find(|target| target.selection == label.selection)
        {
            let edge = vec2(
                target.center.x.clamp(rect.x, rect.right()),
                target.center.y.clamp(rect.y, rect.bottom()),
            );
            let distance = target.center.distance(edge);
            if distance > 70.0 {
                let start = target.center.lerp(edge, 32.0 / distance);
                draw_line(start.x, start.y, edge.x, edge.y, 2.5, INK);
                draw_line(start.x, start.y, edge.x, edge.y, 1.0, BRASS);
            }
        }
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(INK.r, INK.g, INK.b, 0.92),
        );
        body(
            ctx,
            &names[&label.selection],
            vec2(rect.x + 7.0, rect.y + 20.0),
            18.0,
            CREAM,
        );
    }
}

fn kingdoms(
    ctx: &Context<'_>,
    targets: &[MapTarget],
    reserved: &mut Vec<Rect>,
    exploration: Option<&MapExploration>,
) {
    if ctx.navigation.scope() != MapScope::World
        || ctx.preferences.hide_labels
        || ctx.view.band() == MapScaleBand::Detail
    {
        return;
    }
    let (Some(view), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return;
    };
    let mut factions: Vec<_> = view.factions.iter().collect();
    factions.sort_by_key(|faction| (faction.id != view.observer, faction.id));
    for faction in factions {
        let owned: Vec<_> = targets
            .iter()
            .filter(|target| match target.selection {
                MapSelection::Marker(id) => overview.markers.get(&id).is_some_and(|summary| {
                    summary.political_known && summary.political_owner == Some(faction.id)
                }),
                MapSelection::Site(_) => false,
            })
            .collect();
        if owned.is_empty() {
            continue;
        }
        let name = truncate_text_to_width_ex(&faction.name, 218.0, ctx.body_font(), 18.0);
        let width = measure_text(&name, ctx.body_font(), 18, 1.0).width + 16.0;
        let center = owned.iter().map(|target| target.center).sum::<Vec2>() / owned.len() as f32;
        let candidates = std::iter::once(center)
            .chain(owned.iter().map(|target| target.center))
            .flat_map(|at| [at + vec2(0.0, -79.0), at + vec2(0.0, 57.0)]);
        let bounds = kingdom_bounds(
            ctx,
            targets,
            reserved,
            exploration,
            &owned,
            candidates,
            width,
        );
        if let Some(rect) = bounds {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(INK.r, INK.g, INK.b, 0.88),
            );
            draw_line(
                rect.x,
                rect.bottom(),
                rect.right(),
                rect.bottom(),
                2.0,
                faction_color(ctx, Some(faction.id)),
            );
            body(ctx, &name, vec2(rect.x + 8.0, rect.y + 20.0), 18.0, CREAM);
            reserved.push(rect);
        }
    }
}

fn candidates(
    ctx: &Context<'_>,
    targets: &[MapTarget],
) -> (Vec<MapLabelCandidate>, BTreeMap<MapSelection, String>) {
    let (Some(campaign), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return (Vec::new(), BTreeMap::new());
    };
    let mut names = BTreeMap::new();
    let labels = targets
        .iter()
        .filter_map(|target| {
            let (name, capital, region, danger, owned, habitation) = match target.selection {
                MapSelection::Marker(id) => {
                    let marker = campaign.world.marker(id)?;
                    let summary = overview.markers.get(&id)?;
                    (
                        marker.name.as_str(),
                        summary.capital,
                        matches!(marker.location, MarkerLocation::Region { .. }),
                        summary.danger.any(),
                        summary.political_owner == Some(campaign.observer),
                        summary.habitation,
                    )
                }
                MapSelection::Site(id) => {
                    let site = campaign.world.site(id)?;
                    let summary = overview.sites.get(&id)?;
                    (
                        site.name.as_str(),
                        summary.capital,
                        false,
                        summary.danger.any(),
                        summary.controller == Some(campaign.observer),
                        summary.habitation,
                    )
                }
            };
            let selected = ctx.navigation.selection() == Some(target.selection);
            if ctx.preferences.hide_labels && !selected && !capital {
                return None;
            }
            let eligible = match ctx.view.band() {
                MapScaleBand::Overview => {
                    selected || capital || danger || region || habitation >= Habitation::City
                }
                MapScaleBand::Campaign => {
                    selected
                        || capital
                        || danger
                        || region
                        || owned
                        || habitation >= Habitation::Hamlet
                }
                MapScaleBand::Detail => true,
            };
            if !eligible {
                return None;
            }
            let identity = if capital {
                format!("{} · {name}", ctx.data.map.text("capital"))
            } else {
                name.to_owned()
            };
            let fitted_identity =
                truncate_text_to_width_ex(&identity, 216.0, ctx.body_font(), 18.0);
            let width = measure_text(&fitted_identity, ctx.body_font(), 18, 1.0).width + 14.0;
            names.insert(target.selection, fitted_identity);
            let priority = if selected {
                100
            } else if capital {
                95
            } else if danger {
                90
            } else if region {
                80
            } else if owned {
                60
            } else {
                30
            };
            Some(MapLabelCandidate {
                selection: target.selection,
                center: target.center,
                width,
                priority,
                minimum_zoom: 0.0,
            })
        })
        .collect();
    (labels, names)
}

fn kingdom_bounds(
    ctx: &Context<'_>,
    targets: &[MapTarget],
    reserved: &[Rect],
    exploration: Option<&MapExploration>,
    owned: &[&MapTarget],
    candidates: impl Iterator<Item = Vec2>,
    width: f32,
) -> Option<Rect> {
    candidates
        .map(|at| {
            Rect::new(
                (at.x - width * 0.5).clamp(12.0, WIDTH - 12.0 - width),
                at.y,
                width,
                27.0,
            )
        })
        .find(|rect| {
            rect.y >= 92.0
                && rect.bottom() <= HEIGHT - 160.0
                && !reserved.iter().any(|reserved| reserved.overlaps(rect))
                && !targets.iter().any(|target| target.bounds().overlaps(rect))
                && targets
                    .iter()
                    .min_by(|a, b| {
                        a.center
                            .distance_squared(rect.center())
                            .total_cmp(&b.center.distance_squared(rect.center()))
                    })
                    .is_some_and(|nearest| {
                        owned
                            .iter()
                            .any(|target| target.selection == nearest.selection)
                    })
                && [rect.point(), vec2(rect.right(), rect.bottom())]
                    .into_iter()
                    .all(|point| {
                        let atlas = ctx
                            .view
                            .camera
                            .screen_to_world(kestrum::navigation::MAP_RECT, point)
                            .unwrap_or(point);
                        exploration.is_none_or(|area| area.opacity(atlas) < 0.04)
                            && ctx.data.map.is_atlas_land([
                                atlas.x / ctx.view.extent().x,
                                atlas.y / ctx.view.extent().y,
                            ])
                    })
        })
}
