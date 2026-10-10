//! One collision-aware layout for map orders, supply and seasonal receipts.

use super::*;
use kestrum::{
    engine::VisibleCampaign,
    navigation::{MapNavigation, MapScope, MapSelection, MapView},
    state::{notifications::NotificationProjection, GameState, Screen},
};

const MARKER: f32 = 48.0;
const LABEL_WIDTH: f32 = 216.0;
const LABEL_HEIGHT: f32 = 64.0;
const VISUAL_WIDTH: f32 = MARKER + LABEL_WIDTH;
const HUD: [Rect; 10] = [
    Rect::new(24.0, 20.0, 194.0, 48.0),
    Rect::new(1788.0, 24.0, 108.0, 48.0),
    Rect::new(24.0, 1008.0, 48.0, 48.0),
    Rect::new(80.0, 1008.0, 48.0, 48.0),
    Rect::new(144.0, 1008.0, 148.0, 48.0),
    Rect::new(304.0, 1008.0, 148.0, 48.0),
    Rect::new(464.0, 1008.0, 132.0, 48.0),
    Rect::new(1712.0, 1008.0, 184.0, 48.0),
    Rect::new(1594.0, 1008.0, 108.0, 48.0),
    Rect::new(24.0, 936.0, 1100.0, 60.0),
];

#[derive(Clone, Copy)]
pub struct MapStatusInput<'a> {
    pub campaign: &'a VisibleCampaign,
    pub projection: Option<&'a NotificationProjection>,
    pub notifications: &'a super::super::NotificationSession,
    pub navigation: &'a MapNavigation,
    pub view: &'a MapView,
    pub movement: &'a super::super::MoveView,
    pub overview: Option<&'a kestrum::engine::MapOverview>,
    pub overview_ui: &'a super::super::OverviewView,
    pub state: &'a GameState,
}

impl<'a> MapStatusInput<'a> {
    pub fn from_context(ctx: &'a Context<'_>) -> Option<Self> {
        Some(Self {
            campaign: ctx.campaign_view?,
            projection: ctx.notification_projection,
            notifications: ctx.notifications,
            navigation: ctx.navigation,
            view: ctx.view,
            movement: ctx.movement,
            overview: ctx.overview,
            overview_ui: ctx.overview_ui,
            state: ctx.state,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusAnchor {
    Site(MapSelection),
    Route(kestrum::data::world::RouteId),
}

#[derive(Debug, Clone)]
pub enum StatusContent {
    Work {
        order: kestrum::state::construction::OrderId,
        count: usize,
    },
    Supply {
        count: usize,
    },
    Receipt {
        ids: Vec<kestrum::state::notifications::NotificationId>,
        title: String,
        category: kestrum::state::notifications::NotificationCategory,
        priority: kestrum::state::notifications::NotificationPriority,
        active: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum StatusIntent {
    Work(SiteId),
    Supply(SiteId),
    Receipt(kestrum::state::notifications::NotificationId),
}

#[derive(Debug, Clone)]
pub struct StatusMarker {
    pub anchor_point: Vec2,
    pub bounds: Rect,
    pub target: Rect,
    pub content: StatusContent,
    pub intent: StatusIntent,
}

pub(super) struct StatusCandidate {
    pub at: Vec2,
    pub content: StatusContent,
    pub intent: StatusIntent,
}

#[derive(Debug, Clone, Default)]
pub struct MapStatusLayout {
    markers: Vec<StatusMarker>,
    status_rects: Vec<Rect>,
}

impl MapStatusLayout {
    pub fn build(input: &MapStatusInput<'_>) -> Self {
        if input.state.screen != Screen::Campaign
            || input.state.overlay != kestrum::state::Overlay::None
            || input.campaign.observer_mode
        {
            return Self::default();
        }
        let mut reserved = base_reserved(input);
        let mut markers = Vec::new();
        let mut candidates: Vec<_> = orders::candidates(input)
            .into_iter()
            .chain(receipts::candidates(input))
            .collect();
        candidates.sort_by_key(StatusCandidate::priority);
        for candidate in candidates {
            let Some((bounds, target)) = place(candidate.at, &reserved) else {
                continue;
            };
            reserved.push(bounds);
            markers.push(StatusMarker {
                anchor_point: candidate.at,
                bounds,
                target,
                content: candidate.content,
                intent: candidate.intent,
            });
        }
        let status_rects = markers.iter().map(|marker| marker.bounds).collect();
        Self {
            markers,
            status_rects,
        }
    }

    pub fn reserved_rects(&self) -> &[Rect] {
        &self.status_rects
    }

    pub fn control_targets(&self) -> impl Iterator<Item = Rect> + '_ {
        self.markers.iter().map(|marker| marker.target)
    }

    pub fn control_bounds(&self) -> impl Iterator<Item = Rect> + '_ {
        self.markers.iter().map(|marker| marker.bounds)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.markers
            .iter()
            .any(|marker| marker.bounds.contains(point))
    }

    fn tapped(&self, ctx: &Context<'_>) -> Option<StatusIntent> {
        if !ctx.pointer.released {
            return None;
        }
        let origin = ctx.origin?;
        self.markers
            .iter()
            .find(|marker| {
                marker.bounds.contains(origin) && marker.bounds.contains(ctx.pointer.position)
            })
            .map(|marker| marker.intent)
    }
}

pub fn layout_for_context(ctx: &Context<'_>) -> MapStatusLayout {
    MapStatusInput::from_context(ctx)
        .as_ref()
        .map_or_else(MapStatusLayout::default, MapStatusLayout::build)
}

pub fn map_control_contains(point: Vec2, input: MapStatusInput<'_>) -> bool {
    MapStatusLayout::build(&input).contains(point)
}

pub fn layout(input: MapStatusInput<'_>) -> MapStatusLayout {
    MapStatusLayout::build(&input)
}

pub fn action(ctx: &Context<'_>) -> Option<UiAction> {
    let input = MapStatusInput::from_context(ctx)?;
    match MapStatusLayout::build(&input).tapped(ctx)? {
        StatusIntent::Work(site) => Some(UiAction::OpenSettlement(site)),
        StatusIntent::Supply(site) => match input.navigation.scope() {
            MapScope::World => Some(UiAction::OpenSettlement(site)),
            MapScope::Region(_) => Some(UiAction::SelectMap(MapSelection::Site(site))),
        },
        StatusIntent::Receipt(id) => Some(UiAction::Notification(
            super::super::NotificationAction::OpenMapGroup(id),
        )),
    }
}

pub(super) fn draw(ctx: &Context<'_>, layout: &MapStatusLayout) {
    for marker in &layout.markers {
        let (glyph, first, second, color) = marker_text(ctx, marker);
        let edge = vec2(
            marker
                .anchor_point
                .x
                .clamp(marker.bounds.x, marker.bounds.right()),
            marker
                .anchor_point
                .y
                .clamp(marker.bounds.y, marker.bounds.bottom()),
        );
        draw_line(
            marker.anchor_point.x,
            marker.anchor_point.y,
            edge.x,
            edge.y,
            2.0,
            INK,
        );
        draw_line(
            marker.anchor_point.x,
            marker.anchor_point.y,
            edge.x,
            edge.y,
            1.0,
            color,
        );
        draw_rectangle(
            marker.bounds.x,
            marker.bounds.y,
            marker.bounds.w,
            marker.bounds.h,
            INK,
        );
        draw_rectangle_lines(
            marker.bounds.x,
            marker.bounds.y,
            marker.bounds.w,
            marker.bounds.h,
            2.0,
            color,
        );
        centered(
            ctx,
            &glyph,
            vec2(
                marker.target.x + marker.target.w * 0.5,
                marker.target.y + 31.0,
            ),
            24.0,
            color,
        );
        let label_x = if marker.target.x <= marker.bounds.x + 1.0 {
            marker.target.right() + 6.0
        } else {
            marker.bounds.x + 6.0
        };
        let first = truncate_text_to_width_ex(&first, LABEL_WIDTH - 12.0, ctx.body_font(), 16.0);
        body(
            ctx,
            &first,
            vec2(label_x, marker.bounds.y + 24.0),
            16.0,
            CREAM,
        );
        if !second.is_empty() {
            let second =
                truncate_text_to_width_ex(&second, LABEL_WIDTH - 12.0, ctx.body_font(), 16.0);
            body(
                ctx,
                &second,
                vec2(label_x, marker.bounds.y + 48.0),
                16.0,
                color,
            );
        }
    }
}

fn marker_text(ctx: &Context<'_>, marker: &StatusMarker) -> (String, String, String, Color) {
    match &marker.content {
        StatusContent::Work { order, count } => {
            let order = ctx
                .campaign_view
                .into_iter()
                .flat_map(|view| &view.construction)
                .find(|candidate| candidate.id == *order);
            let Some(order) = order else {
                return ("W".into(), String::new(), String::new(), BRASS);
            };
            let work = ctx.text(work_key(order.kind));
            let count = if *count > 1 {
                format!(" ×{count}")
            } else {
                String::new()
            };
            let progress = ctx
                .text("map_work_progress")
                .replace("{work}", &work)
                .replace("{progress}", &order.progress.to_string())
                .replace("{steps}", &order.required_steps.to_string())
                .replace("{count}", &count);
            let paused = match order.status {
                kestrum::state::construction::ConstructionStatus::Paused { reason } => {
                    let why = ctx.text(pause_key(reason));
                    ctx.text("map_work_paused").replace("{reason}", &why)
                }
                _ => String::new(),
            };
            (
                "W".into(),
                progress,
                paused,
                Color::new(0.96, 0.75, 0.41, 1.0),
            )
        }
        StatusContent::Supply { count } => {
            let count = if *count > 1 {
                format!(" ×{count}")
            } else {
                String::new()
            };
            (
                "!".into(),
                ctx.text("map_place_supply_cutoff")
                    .replace("{count}", &count),
                String::new(),
                Color::new(0.96, 0.58, 0.43, 1.0),
            )
        }
        StatusContent::Receipt {
            ids,
            title,
            category,
            priority,
            active,
        } => {
            let category_key = match category {
                kestrum::state::notifications::NotificationCategory::People => {
                    "map_category_people"
                }
                kestrum::state::notifications::NotificationCategory::Places => {
                    "map_category_places"
                }
                kestrum::state::notifications::NotificationCategory::Security => {
                    "map_category_security"
                }
                kestrum::state::notifications::NotificationCategory::Orders => {
                    "map_category_orders"
                }
                kestrum::state::notifications::NotificationCategory::Military => {
                    "map_category_military"
                }
                kestrum::state::notifications::NotificationCategory::EconomyDiplomacy => {
                    "map_category_economy"
                }
                kestrum::state::notifications::NotificationCategory::Remembrance => {
                    "map_category_remembrance"
                }
            };
            let summary = if ctx.view.band() == MapScaleBand::Overview {
                format!("{} ×{}", ctx.text(category_key), ids.len())
            } else {
                format!("{} ×{}", title, ids.len())
            };
            let color = if *active
                || matches!(
                    priority,
                    kestrum::state::notifications::NotificationPriority::Urgent
                        | kestrum::state::notifications::NotificationPriority::Warning
                ) {
                Color::new(0.96, 0.58, 0.43, 1.0)
            } else {
                BRASS
            };
            ("•".into(), summary, String::new(), color)
        }
    }
}

fn work_key(kind: kestrum::state::construction::ConstructionKind) -> &'static str {
    use kestrum::state::construction::ConstructionKind::*;
    match kind {
        Outpost => "construction_outpost",
        Road => "construction_road",
        RoadRepair => "construction_road_repair",
        Fort => "construction_fort",
        Facility(kestrum::data::world::Facility::TrainingGround) => "facility_training",
        Facility(kestrum::data::world::Facility::Stable) => "facility_stable",
        Facility(kestrum::data::world::Facility::Infirmary) => "facility_infirmary",
        Facility(kestrum::data::world::Facility::Workshop) => "facility_workshop",
        Facility(kestrum::data::world::Facility::Temple) => "facility_temple",
    }
}

fn pause_key(reason: kestrum::state::construction::ConstructionPause) -> &'static str {
    use kestrum::state::construction::ConstructionPause::*;
    match reason {
        BuilderMissing => "construction_pause_builder_missing",
        BuilderAway => "construction_pause_builder_away",
        SupplyLost => "construction_pause_supply_lost",
        Contested => "construction_pause_contested",
        Combat => "construction_pause_combat",
        NoSettlers => "construction_pause_no_settlers",
        Threat => "construction_pause_threat",
        Ruined => "construction_pause_ruined",
    }
}

fn base_reserved(input: &MapStatusInput<'_>) -> Vec<Rect> {
    let campaign = input.campaign;
    let world = &campaign.world;
    let panel = if input.state.overlay == kestrum::state::Overlay::None
        && input.notifications.is_open
        && !campaign.observer_mode
    {
        super::super::notification_reserved_rects(input.notifications, input.projection)
            .last()
            .copied()
    } else if input.movement.stage == super::super::MoveStage::Map {
        Some(super::super::movement_panel_bounds(
            input.movement,
            input.navigation,
            world,
            input.view,
        ))
    } else {
        super::super::selection_bounds(input.navigation, world, input.view)
    };
    let mut reserved = Vec::from(HUD);
    if let Some(panel) = panel {
        reserved.push(panel);
    }
    reserved.push(super::super::overview::attention_bounds(
        input.overview_ui,
        panel.is_some(),
        input
            .overview
            .map_or(0, |overview| overview.attention.len()),
    ));
    reserved.push(super::super::feedback_bounds(input.state));
    if let Some(tutorial) = super::super::tutorial_bounds(input.state) {
        reserved.push(tutorial);
    }
    reserved.extend(super::super::notification_reserved_rects(
        input.notifications,
        input.projection,
    ));
    reserved.extend(
        input
            .navigation
            .targets(world, input.view)
            .iter()
            .map(MapTarget::bounds),
    );
    reserved.extend(
        input
            .navigation
            .place_groups(world, input.view)
            .iter()
            .map(|group| group.bounds()),
    );
    reserved.extend(
        input
            .navigation
            .army_targets(world, input.view, &campaign.armies)
            .iter()
            .map(|target| army_bounds(input, target)),
    );
    reserved
}

fn place(anchor: Vec2, reserved: &[Rect]) -> Option<(Rect, Rect)> {
    if !kestrum::navigation::MAP_RECT.contains(anchor) {
        return None;
    }
    let candidates = [
        (vec2(anchor.x + 26.0, anchor.y - LABEL_HEIGHT * 0.5), false),
        (
            vec2(
                anchor.x - 26.0 - VISUAL_WIDTH,
                anchor.y - LABEL_HEIGHT * 0.5,
            ),
            true,
        ),
        (vec2(anchor.x - 24.0, anchor.y + 26.0), false),
        (vec2(anchor.x - 24.0, anchor.y - 26.0 - LABEL_HEIGHT), false),
    ];
    candidates.into_iter().find_map(|(at, reversed)| {
        let bounds = Rect::new(at.x, at.y, VISUAL_WIDTH, LABEL_HEIGHT);
        let target = if reversed {
            Rect::new(bounds.right() - MARKER, bounds.y + 8.0, MARKER, MARKER)
        } else {
            Rect::new(bounds.x, bounds.y + 8.0, MARKER, MARKER)
        };
        (bounds.x >= 8.0
            && bounds.right() <= kestrum::navigation::WIDTH - 8.0
            && bounds.y >= 82.0
            && bounds.bottom() <= kestrum::navigation::HEIGHT - 114.0
            && !reserved.iter().any(|rect| rect.overlaps(&bounds)))
        .then_some((bounds, target))
    })
}

impl StatusCandidate {
    fn priority(&self) -> u8 {
        match &self.content {
            StatusContent::Receipt {
                priority, active, ..
            } => match (*active, *priority) {
                (true, kestrum::state::notifications::NotificationPriority::Urgent) => 0,
                (true, kestrum::state::notifications::NotificationPriority::Warning) => 1,
                (_, kestrum::state::notifications::NotificationPriority::Urgent) => 2,
                (_, kestrum::state::notifications::NotificationPriority::Warning) => 3,
                _ => 4,
            },
            StatusContent::Work { .. } => 5,
            StatusContent::Supply { .. } => 6,
        }
    }
}

fn army_bounds(input: &MapStatusInput<'_>, target: &kestrum::navigation::ArmyTarget) -> Rect {
    if input.view.band() == MapScaleBand::Overview {
        return target.bounds;
    }
    let center = target.bounds.center();
    Rect::new(center.x - 57.0, center.y - 24.0, 114.0, 60.0)
}
