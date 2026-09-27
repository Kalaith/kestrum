//! One transactional work service for player and NPC builders.

mod commands;
mod progress;
mod query;
mod settlers;
pub(super) use commands::{cancel, reassign, set_focus, start};
pub(super) use progress::{reconcile, resolve};
pub use query::{construction_options, construction_refund, ConstructionOption};

use super::{recovery::SupplySnapshot, RuleError};
use crate::{
    data::{
        economy::{Habitation, OrderKind, Resources},
        world::{FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    state::{construction::*, military::ArmyId, StrategicCampaign},
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionBlock {
    InvalidTarget,
    NotOwned,
    Contested,
    Unsupplied,
    BuilderMissing,
    BuilderAway,
    BuilderBusy,
    DuplicateOrder,
    AlreadyBuilt,
    OutpostSiteRequired,
    SettlementRequired,
    MissingTag,
    RoadRequired,
    NoDamage,
    UnknownOrder,
    OrderNotOwned,
    OrderFinished,
    BuilderUnchanged,
    FacilityNeedsNoBuilder,
    FocusUnchanged,
}

impl fmt::Display for ConstructionBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
        Self::InvalidTarget=>"Choose a physical site or an existing route for this work.",
        Self::NotOwned=>"Control the work site and both road endpoints before placing this order.",
        Self::Contested=>"Construction cannot begin at a contested site.",Self::Unsupplied=>"Construction needs a friendly supply path to headquarters.",
        Self::BuilderMissing=>"Choose a nonempty friendly builder army.",Self::BuilderAway=>"The builder must be at the site or one road endpoint.",
        Self::BuilderBusy=>"That builder already supports unfinished work. Reassign or cancel that order first.",
        Self::DuplicateOrder=>"This site or route already has unfinished work.",Self::AlreadyBuilt=>"This improvement is already present.",
        Self::OutpostSiteRequired=>"An Outpost needs a controlled Unsettled or Camp site.",Self::SettlementRequired=>"This settlement is too small for the chosen improvement.",
        Self::MissingTag=>"This facility needs local horse access.",Self::RoadRequired=>"Road repair requires an improved road.",Self::NoDamage=>"This road has no damage to repair.",
        Self::UnknownOrder=>"That construction order is unavailable.",Self::OrderNotOwned=>"You can manage only your own construction orders.",
        Self::OrderFinished=>"Completed or cancelled work cannot be reopened.",Self::BuilderUnchanged=>"That army is already assigned to this work.",
        Self::FacilityNeedsNoBuilder=>"Facility work needs a builder only when it is placed.",Self::FocusUnchanged=>"This focus is already selected.",
    })
    }
}

fn blocked(reason: ConstructionBlock) -> RuleError {
    RuleError::Construction { reason }
}

pub(crate) fn terms(data: &GameData, kind: ConstructionKind) -> (Resources, u32) {
    match kind {
        ConstructionKind::Outpost => (
            data.economy.orders[&OrderKind::EstablishOutpost].cost,
            data.construction.outpost_steps,
        ),
        ConstructionKind::Road => (
            data.economy.orders[&OrderKind::ImproveRoad].cost,
            data.construction.road_steps,
        ),
        ConstructionKind::Fort => (
            data.economy.orders[&OrderKind::BuildFort].cost,
            data.construction.fort_steps,
        ),
        ConstructionKind::RoadRepair => (
            data.construction.road_repair.cost,
            data.construction.road_repair.steps,
        ),
        ConstructionKind::Facility(facility) => {
            let definition = &data.construction.facilities[&facility];
            (definition.cost, definition.steps)
        }
    }
}

fn sites(
    campaign: &StrategicCampaign,
    target: ConstructionTarget,
) -> Result<Vec<SiteId>, RuleError> {
    match target {
        ConstructionTarget::Site(id) if campaign.world.site(id).is_some() => Ok(vec![id]),
        ConstructionTarget::Route(id) => campaign
            .world
            .route(id)
            .map(|route| vec![route.from, route.to])
            .ok_or_else(|| blocked(ConstructionBlock::InvalidTarget)),
        _ => Err(blocked(ConstructionBlock::InvalidTarget)),
    }
}

fn validate_builder(
    campaign: &StrategicCampaign,
    owner: FactionId,
    target: ConstructionTarget,
    builder: ArmyId,
    except: Option<OrderId>,
) -> Result<(), RuleError> {
    let army = campaign
        .armies
        .get(&builder)
        .filter(|army| army.faction == owner && army.formation_ids().next().is_some())
        .ok_or_else(|| blocked(ConstructionBlock::BuilderMissing))?;
    if !sites(campaign, target)?.contains(&army.site) {
        return Err(blocked(ConstructionBlock::BuilderAway));
    }
    if campaign
        .construction
        .values()
        .any(|order| Some(order.id) != except && order.is_open() && order.builder == Some(builder))
    {
        return Err(blocked(ConstructionBlock::BuilderBusy));
    }
    Ok(())
}

fn owned_order(
    campaign: &StrategicCampaign,
    owner: FactionId,
    id: OrderId,
) -> Result<&ConstructionOrder, RuleError> {
    let order = campaign
        .construction
        .get(&id)
        .ok_or_else(|| blocked(ConstructionBlock::UnknownOrder))?;
    if order.owner != owner {
        return Err(blocked(ConstructionBlock::OrderNotOwned));
    }
    if !order.is_open() {
        return Err(blocked(ConstructionBlock::OrderFinished));
    }
    Ok(order)
}

fn afford(
    campaign: &StrategicCampaign,
    owner: FactionId,
    required: Resources,
) -> Result<(), RuleError> {
    let available = campaign
        .factions
        .get(&owner)
        .ok_or(RuleError::UnknownActor)?
        .resources;
    if required.gold > available.gold
        || required.wood > available.wood
        || required.stone > available.stone
    {
        return Err(RuleError::InsufficientResources {
            required,
            available,
        });
    }
    Ok(())
}
