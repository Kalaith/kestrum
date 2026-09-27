use super::super::{actions::validate_command, Actor, Command};
use super::*;
use crate::data::{economy::Resources, world::RouteId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleThreat {
    pub id: ThreatId,
    pub site: SiteId,
    pub kind: ThreatKind,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatArmyOption {
    pub id: ArmyId,
    pub name: String,
    pub origin: SiteId,
    pub blocked: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatView {
    pub threat: VisibleThreat,
    pub armies: Vec<ThreatArmyOption>,
    pub reward: Resources,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreatPreview {
    pub threat: VisibleThreat,
    pub armies: Vec<ArmyId>,
    pub origin: SiteId,
    pub route: RouteId,
    pub cost: u32,
    pub remaining: u32,
    pub reward: Resources,
}

pub fn visible_threats(campaign: &StrategicCampaign, observer: FactionId) -> Vec<VisibleThreat> {
    campaign
        .threats
        .values()
        .filter(|threat| {
            threat.status == ThreatStatus::Active
                && campaign.armies.values().any(|army| {
                    army.faction == observer
                        && (army.site == threat.site
                            || campaign
                                .world
                                .connected_route(army.site, threat.site)
                                .is_some())
                })
        })
        .map(|threat| VisibleThreat {
            id: threat.id,
            site: threat.site,
            kind: threat.kind,
            name: threat.name.clone(),
        })
        .collect()
}

pub fn threat_view(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    id: ThreatId,
) -> Option<ThreatView> {
    let threat = visible_threats(campaign, observer)
        .into_iter()
        .find(|threat| threat.id == id)?;
    let armies = campaign
        .armies
        .values()
        .filter(|army| army.faction == observer)
        .map(|army| ThreatArmyOption {
            id: army.id,
            name: army.name.clone(),
            origin: army.site,
            blocked: threat_preview(campaign, data, observer, &[army.id], id)
                .err()
                .map(|error| error.to_string()),
        })
        .collect();
    Some(ThreatView {
        reward: data.threats.definitions[&threat.kind].reward,
        threat,
        armies,
    })
}

pub fn threat_preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    armies: &[ArmyId],
    id: ThreatId,
) -> Result<ThreatPreview, RuleError> {
    let threat = visible_threats(campaign, observer)
        .into_iter()
        .find(|threat| threat.id == id)
        .ok_or_else(|| {
            RuleError::InvalidState("No local threat has been observed there.".into())
        })?;
    let actor = if observer == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(observer)
    };
    validate_command(
        campaign,
        actor,
        &Command::ClearThreat {
            armies: armies.to_vec(),
            threat: id,
        },
    )?;
    validate_order(campaign, data, observer, armies, id)?;
    let (origin, remaining) = movement::validate_group(campaign, data, observer, armies)?;
    let route = campaign
        .world
        .connected_route(origin, threat.site)
        .expect("validated");
    let mut armies = armies.to_vec();
    armies.sort();
    Ok(ThreatPreview {
        reward: data.threats.definitions[&threat.kind].reward,
        threat,
        armies,
        origin,
        route: route.id,
        cost: movement::route_cost(route, data),
        remaining,
    })
}
