//! Explicit adjacent attacks, observer-safe offers, and one spawn per ruination.

mod projection;
pub use projection::{
    threat_preview, threat_view, visible_threats, ThreatArmyOption, ThreatPreview, ThreatView,
    VisibleThreat,
};

use super::{
    actions::record_fact, combat, movement, ActionOutcome, MovementBlock, MovementOutcome,
    RuleError,
};
use crate::{
    data::{
        threats::ThreatKind,
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        campaign::DomainFactKind,
        military::ArmyId,
        threat::{Threat, ThreatId, ThreatStatus},
        StrategicCampaign,
    },
};

pub(crate) fn validate_order(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    armies: &[ArmyId],
    id: ThreatId,
) -> Result<(), RuleError> {
    let (origin, remaining) = movement::validate_group(campaign, data, owner, armies)?;
    let threat = campaign
        .threats
        .get(&id)
        .filter(|threat| threat.status == ThreatStatus::Active)
        .ok_or_else(|| {
            RuleError::InvalidState(
                "That local threat has already been cleared or is unavailable.".into(),
            )
        })?;
    if campaign.sieges.contains_key(&origin) {
        return Err(RuleError::MovementBlocked {
            site: origin,
            reason: MovementBlock::SiegeExitRequired,
        });
    }
    let route = campaign
        .world
        .connected_route(origin, threat.site)
        .ok_or(RuleError::InvalidRoute)?;
    if let Some(reason) = movement::public_block(campaign, owner, threat.site) {
        return Err(RuleError::MovementBlocked {
            site: threat.site,
            reason,
        });
    }
    let cost = movement::route_cost(route, data);
    if cost > remaining {
        return Err(RuleError::MovementBlocked {
            site: threat.site,
            reason: MovementBlock::InsufficientMovement {
                required: cost,
                remaining,
            },
        });
    }
    if campaign
        .armies
        .values()
        .any(|army| army.site == threat.site)
        || campaign.sieges.contains_key(&threat.site)
    {
        return Err(RuleError::InvalidState(
            "Resolve the site's military occupants before attacking its local threat.".into(),
        ));
    }
    Ok(())
}

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    armies: &[ArmyId],
    id: ThreatId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    validate_order(campaign, data, owner, armies, id)?;
    let origin = campaign.armies[&armies[0]].site;
    let site = campaign.threats[&id].site;
    let cost = movement::route_cost(
        campaign
            .world
            .connected_route(origin, site)
            .expect("validated route"),
        data,
    );
    let mut armies = armies.to_vec();
    armies.sort();
    let mut moved = MovementOutcome {
        battle: None,
        armies,
        path: vec![origin, site],
        spent: cost,
        stop: None,
    };
    let receipt = movement::service_snapshot(campaign, &moved);
    movement::spend_edge(campaign, &moved.armies, site, cost)?;
    let battle = combat::resolve_threat(campaign, data, &moved.armies, origin, id)?;
    moved.battle = Some(battle);
    outcome.battle = Some(battle);
    outcome.movement = Some(moved);
    record_fact(
        campaign,
        outcome,
        DomainFactKind::BattleResolved {
            battle,
            movement: Some(receipt),
        },
    )
}

pub(crate) fn spawn_ruin(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    transition: u64,
) -> Result<ThreatId, RuleError> {
    let state = campaign
        .world
        .development
        .get(&site)
        .ok_or(RuleError::UnknownSite { site })?;
    if transition == 0
        || !state.ruined
        || state.ruination != transition
        || state.threat_created
        || campaign.active_threat(site).is_some()
        || campaign.sieges.contains_key(&site)
        || campaign.armies.values().any(|army| army.site == site)
        || campaign
            .threats
            .values()
            .any(|threat| threat.site == site && threat.ruination == Some(transition))
    {
        return Err(RuleError::InvalidState(
            "This ruination cannot create another local threat.".into(),
        ));
    }
    let id = campaign.next_ids.threat;
    let next = id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "threat identifiers",
    })?;
    // Current development preserves the latest transition/creation flag. Older
    // cleared spawned records are weak history references, so retention is bounded.
    campaign.threats.retain(|_, threat| {
        threat.site != site || threat.ruination.is_none() || threat.status == ThreatStatus::Active
    });
    campaign.threats.insert(
        id,
        Threat {
            id,
            site,
            kind: ThreatKind::Bandits,
            name: data.threats.definitions[&ThreatKind::Bandits].name.clone(),
            headcount: data.threats.definitions[&ThreatKind::Bandits].headcount,
            status: ThreatStatus::Active,
            ruination: Some(transition),
        },
    );
    campaign.next_ids.threat = ThreatId(next);
    Ok(id)
}
