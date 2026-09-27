//! A siege partitions armies at one physical site; walls remain site property.

mod validation;

use super::{military::ArmyId, StrategicCampaign};
use crate::data::{
    siege::SiegeRules,
    world::{FactionId, SiteId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SiegeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiegeChange {
    Established,
    Reinforced,
    Progressed,
    Lifted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiegeAction {
    Maintain,
    Assault,
    Withdraw,
    Sortie,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeOrder {
    pub site: SiteId,
    pub action: SiegeAction,
    pub armies: Vec<ArmyId>,
    pub destination: Option<SiteId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Siege {
    pub id: SiegeId,
    pub site: SiteId,
    pub defender: FactionId,
    pub besieger: FactionId,
    pub defending: Vec<ArmyId>,
    pub besieging: Vec<ArmyId>,
    pub started_round: u32,
    pub elapsed_steps: u32,
    pub last_progress_round: Option<u32>,
}

impl StrategicCampaign {
    pub fn siege_wall_permille(&self, site: SiteId, rules: &SiegeRules) -> Option<u32> {
        let siege = self.sieges.get(&site)?;
        let damage = self.world.fort_damage.get(&site).copied().unwrap_or(0);
        Some(rules.wall_permille(siege.elapsed_steps, damage, false))
    }

    /// A siege endpoint can supply besiegers, but never relays to sites beyond it.
    pub fn army_is_supplied(&self, id: ArmyId) -> bool {
        let Some(army) = self.armies.get(&id) else {
            return false;
        };
        let supplied = self.supplied_sites(army.faction);
        if let Some(siege) = self.sieges.get(&army.site) {
            siege.besieger == army.faction
                && self
                    .world
                    .adjacent_sites(army.site)
                    .iter()
                    .any(|site| supplied.contains(site))
        } else {
            supplied.contains(&army.site)
        }
    }
}
