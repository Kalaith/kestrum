//! Place, reassign and cancel prepaid work without partially changing balances.

use super::*;
use crate::state::campaign::DomainFactKind;

pub(in crate::engine) fn start(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    target: ConstructionTarget,
    kind: ConstructionKind,
    builder: ArmyId,
) -> Result<DomainFactKind, RuleError> {
    validate_start(campaign, data, owner, target, kind, builder)?;
    let (paid, required_steps) = terms(data, kind);
    let id = campaign.next_ids.order;
    campaign.next_ids.order = OrderId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "construction identifiers",
    })?);
    let order = ConstructionOrder {
        id,
        owner,
        kind,
        target,
        builder: if matches!(kind, ConstructionKind::Facility(_)) {
            None
        } else {
            Some(builder)
        },
        created_round: campaign.completed_rounds,
        progress: 0,
        required_steps,
        paid,
        status: ConstructionStatus::Active,
        last_progress_round: None,
    };
    let balance = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("validated owner")
        .resources;
    balance.gold -= paid.gold;
    balance.wood -= paid.wood;
    balance.stone -= paid.stone;
    campaign.construction.insert(id, order.clone());
    Ok(DomainFactKind::ConstructionChanged { order })
}

pub(in crate::engine) fn validate_start(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    target: ConstructionTarget,
    kind: ConstructionKind,
    builder: ArmyId,
) -> Result<(), RuleError> {
    let target_sites = sites(campaign, target)?;
    if !campaign.owns_construction_target(owner, target) {
        return Err(blocked(ConstructionBlock::NotOwned));
    }
    if campaign
        .construction
        .values()
        .any(|order| order.target == target && order.is_open())
    {
        return Err(blocked(ConstructionBlock::DuplicateOrder));
    }
    let supplied = campaign
        .world
        .supplied_sites(owner, campaign.factions[&owner].headquarters);
    if target_sites
        .iter()
        .any(|site| campaign.world.contested_sites.contains(site))
    {
        return Err(blocked(ConstructionBlock::Contested));
    }
    if target_sites.iter().any(|site| !supplied.contains(site)) {
        return Err(blocked(ConstructionBlock::Unsupplied));
    }
    validate_improvement(campaign, data, target, kind)?;
    validate_builder(campaign, owner, target, builder, None)?;
    afford(campaign, owner, terms(data, kind).0)
}

fn validate_improvement(
    campaign: &StrategicCampaign,
    data: &GameData,
    target: ConstructionTarget,
    kind: ConstructionKind,
) -> Result<(), RuleError> {
    match (target, kind) {
        (ConstructionTarget::Site(id), ConstructionKind::Outpost) => {
            if campaign.world.site(id).expect("valid site").habitation > Habitation::Camp {
                return Err(blocked(ConstructionBlock::OutpostSiteRequired));
            }
        }
        (ConstructionTarget::Site(id), ConstructionKind::Fort) => {
            let site = campaign.world.site(id).expect("valid site");
            if site.habitation < Habitation::Outpost {
                return Err(blocked(ConstructionBlock::SettlementRequired));
            }
            if site.military != MilitaryLayer::None {
                return Err(blocked(ConstructionBlock::AlreadyBuilt));
            }
        }
        (ConstructionTarget::Site(id), ConstructionKind::Facility(facility)) => {
            let site = campaign.world.site(id).expect("valid site");
            let definition = &data.construction.facilities[&facility];
            if site.facilities.contains(&facility) {
                return Err(blocked(ConstructionBlock::AlreadyBuilt));
            }
            if site.habitation < definition.minimum_habitation {
                return Err(blocked(ConstructionBlock::SettlementRequired));
            }
            if definition
                .required_tag
                .is_some_and(|tag| !site.tags.contains(&tag))
            {
                return Err(blocked(ConstructionBlock::MissingTag));
            }
        }
        (ConstructionTarget::Route(id), ConstructionKind::Road) => {
            if campaign.world.route(id).expect("valid route").road.improved {
                return Err(blocked(ConstructionBlock::AlreadyBuilt));
            }
        }
        (ConstructionTarget::Route(id), ConstructionKind::RoadRepair) => {
            let road = &campaign.world.route(id).expect("valid route").road;
            if !road.improved {
                return Err(blocked(ConstructionBlock::RoadRequired));
            }
            if road.damage == 0 {
                return Err(blocked(ConstructionBlock::NoDamage));
            }
        }
        _ => return Err(blocked(ConstructionBlock::InvalidTarget)),
    }
    Ok(())
}

pub(in crate::engine) fn cancel(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    id: OrderId,
) -> Result<DomainFactKind, RuleError> {
    let refund = construction_refund(campaign, data, owner, id)?;
    let balance = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("valid owner")
        .resources;
    for (current, amount) in [
        (&mut balance.gold, refund.gold),
        (&mut balance.wood, refund.wood),
        (&mut balance.stone, refund.stone),
    ] {
        *current = current.checked_add(amount).ok_or(RuleError::Overflow {
            field: "construction refund",
        })?;
    }
    let order = campaign.construction.get_mut(&id).expect("validated order");
    order.status = ConstructionStatus::Cancelled {
        completed_rounds: campaign.completed_rounds,
        reason: CancellationReason::Player,
    };
    let fact = DomainFactKind::ConstructionChanged {
        order: order.clone(),
    };
    campaign.trim_terminal_orders();
    Ok(fact)
}

pub(in crate::engine) fn reassign(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    id: OrderId,
    builder: ArmyId,
) -> Result<DomainFactKind, RuleError> {
    let order = owned_order(campaign, owner, id)?;
    if matches!(order.kind, ConstructionKind::Facility(_)) {
        return Err(blocked(ConstructionBlock::FacilityNeedsNoBuilder));
    }
    if order.builder == Some(builder) {
        return Err(blocked(ConstructionBlock::BuilderUnchanged));
    }
    validate_builder(campaign, owner, order.target, builder, Some(id))?;
    let order = campaign.construction.get_mut(&id).expect("validated order");
    order.builder = Some(builder);
    order.status = ConstructionStatus::Active;
    Ok(DomainFactKind::ConstructionChanged {
        order: order.clone(),
    })
}

pub(in crate::engine) fn set_focus(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    focus: Focus,
) -> Result<DomainFactKind, RuleError> {
    if !campaign.owns_construction_target(owner, ConstructionTarget::Site(site)) {
        return Err(blocked(ConstructionBlock::NotOwned));
    }
    if campaign.world.focus.get(&site) == Some(&focus) {
        return Err(blocked(ConstructionBlock::FocusUnchanged));
    }
    let cost = data.economy.orders[&OrderKind::ChangeFocus].cost;
    afford(campaign, owner, cost)?;
    let balance = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("validated owner")
        .resources;
    balance.gold -= cost.gold;
    balance.wood -= cost.wood;
    balance.stone -= cost.stone;
    campaign.world.focus.insert(site, focus);
    Ok(DomainFactKind::FocusChanged {
        faction: owner,
        site,
        focus,
    })
}
