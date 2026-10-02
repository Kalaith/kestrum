//! Participant-only siege choices never carry a live enemy faction or roster.

use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{
        actions::validate_command,
        projection::{visible_sieges, SiegeRole},
        Actor, Command,
    },
    state::{
        military::ArmyId,
        siege::{SiegeAction, SiegeId, SiegeOrder},
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeActionOption {
    pub action: SiegeAction,
    pub blocked: Option<String>,
    pub destinations: Vec<SiteId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeView {
    pub id: SiegeId,
    pub site: SiteId,
    pub role: SiegeRole,
    pub elapsed_steps: u32,
    pub wall_permille: u32,
    pub fort_damage: u32,
    pub own_armies: Vec<ArmyId>,
    pub supplied_armies: BTreeSet<ArmyId>,
    pub actions: Vec<SiegeActionOption>,
}

pub fn siege_view(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    site: SiteId,
) -> Option<SiegeView> {
    let summary = visible_sieges(campaign, observer)
        .into_iter()
        .find(|siege| siege.site == site)?;
    let supplied_armies = summary
        .own_armies
        .iter()
        .copied()
        .filter(|id| campaign.army_is_supplied(*id))
        .collect();
    let action_kinds = match summary.role {
        SiegeRole::Besieger => vec![
            SiegeAction::Maintain,
            SiegeAction::Assault,
            SiegeAction::Withdraw,
        ],
        SiegeRole::Defender => vec![SiegeAction::Sortie, SiegeAction::Escape],
        SiegeRole::Observer => Vec::new(),
    };
    let actions = action_kinds
        .into_iter()
        .map(|action| action_option(campaign, data, observer, site, &summary.own_armies, action))
        .collect();
    Some(SiegeView {
        id: summary.id,
        site,
        role: summary.role,
        elapsed_steps: summary.elapsed_steps,
        wall_permille: campaign.siege_wall_permille(site, &data.siege)?,
        fort_damage: summary.fort_damage,
        own_armies: summary.own_armies,
        supplied_armies,
        actions,
    })
}

fn action_option(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    armies: &[ArmyId],
    action: SiegeAction,
) -> SiegeActionOption {
    let destinations = if matches!(action, SiegeAction::Withdraw | SiegeAction::Escape) {
        super::destinations(campaign, owner, site)
    } else {
        Vec::new()
    };
    let order = SiegeOrder {
        site,
        action,
        armies: armies.to_vec(),
        destination: destinations.first().copied(),
    };
    let actor = if owner == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(owner)
    };
    let blocked = validate_command(campaign, actor, &Command::Siege(order.clone()))
        .and_then(|()| super::validate_order(campaign, data, owner, &order))
        .err()
        .map(|error| error.to_string());
    SiegeActionOption {
        action,
        blocked,
        destinations,
    }
}
