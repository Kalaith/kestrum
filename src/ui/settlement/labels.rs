//! Human labels for typed construction and focus state.

use super::*;
use kestrum::data::{economy::Habitation, world::Facility};
use kestrum::state::construction::ConstructionStatus;

pub fn kind_name(ctx: &Context<'_>, kind: ConstructionKind) -> String {
    ctx.text(match kind {
        ConstructionKind::Outpost => "construction_outpost",
        ConstructionKind::Road => "construction_road",
        ConstructionKind::RoadRepair => "construction_road_repair",
        ConstructionKind::Fort => "construction_fort",
        ConstructionKind::Facility(facility) => facility_key(facility),
    })
}

pub(super) fn facility_key(facility: Facility) -> &'static str {
    match facility {
        Facility::TrainingGround => "facility_training",
        Facility::Stable => "facility_stable",
        Facility::Infirmary => "facility_infirmary",
        Facility::Workshop => "facility_workshop",
        Facility::Temple => "facility_temple",
    }
}

pub fn focus_key(focus: Focus) -> &'static str {
    match focus {
        Focus::Growth => "focus_growth",
        Focus::Fortification => "focus_fortification",
        Focus::TroopTraining => "focus_training",
        Focus::Gold => "focus_gold",
        Focus::Wood => "focus_wood",
        Focus::Stone => "focus_stone",
    }
}

pub fn order_status(ctx: &Context<'_>, order: &ConstructionOrder) -> String {
    match &order.status {
        ConstructionStatus::Active => ctx.text("construction_active"),
        ConstructionStatus::Paused { reason } => {
            format!("{}: {reason}", ctx.text("construction_paused"))
        }
        ConstructionStatus::Completed { .. } => ctx.text("construction_completed"),
        ConstructionStatus::Cancelled { reason, .. } => {
            format!("{}: {reason}", ctx.text("construction_cancelled"))
        }
    }
}

pub(super) fn target_name(ctx: &Context<'_>, target: ConstructionTarget) -> String {
    let Some(view) = ctx.campaign_view else {
        return String::new();
    };
    let name = |id| view.world.site(id).map(|s| s.name.as_str()).unwrap_or("");
    match target {
        ConstructionTarget::Site(site) => name(site).into(),
        ConstructionTarget::Route(route) => view
            .world
            .route(route)
            .map(|route| format!("{} — {}", name(route.from), name(route.to)))
            .unwrap_or_default(),
    }
}

pub(super) fn habitation(ctx: &Context<'_>, value: Habitation) -> String {
    ctx.text(match value {
        Habitation::Unsettled => "habitation_unsettled",
        Habitation::Camp => "habitation_camp",
        Habitation::Outpost => "habitation_outpost",
        Habitation::Hamlet => "habitation_hamlet",
        Habitation::Village => "habitation_village",
        Habitation::Town => "habitation_town",
        Habitation::City => "habitation_city",
        Habitation::MajorCity => "habitation_major_city",
    })
}

pub(super) fn effect_key(kind: ConstructionKind) -> &'static str {
    match kind {
        ConstructionKind::Outpost => "construction_outpost_effect",
        ConstructionKind::Road => "construction_road_effect",
        ConstructionKind::RoadRepair => "construction_repair_effect",
        ConstructionKind::Fort => "construction_fort_effect",
        ConstructionKind::Facility(Facility::Stable) => "facility_stable_effect",
        ConstructionKind::Facility(Facility::Infirmary) => "facility_infirmary_effect",
        ConstructionKind::Facility(Facility::Workshop) => "facility_workshop_effect",
        ConstructionKind::Facility(Facility::TrainingGround) => "facility_training_effect",
        ConstructionKind::Facility(Facility::Temple) => "facility_temple_effect",
    }
}
