//! Current civilian conditions are independent of habitation, fortification and roles.

mod validation;
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, SiteId, SiteTag},
        GameData,
    },
    state::StrategicCampaign,
};
use serde::{Deserialize, Serialize};
pub(crate) use validation::valid_name;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteDevelopment {
    pub pressure: i32,
    pub displaced: u32,
    pub ruin_streak: u32,
    pub ruined: bool,
    pub ruination: u64,
    pub ruined_round: Option<u32>,
    pub lawless_rounds: u32,
    pub threat_created: bool,
    pub last_resolved_round: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivilIdentity {
    Agricultural,
    Military,
    Religious,
    Administrative,
    Frontier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DevelopmentReceipt {
    HabitationChanged {
        site: SiteId,
        owner: Option<FactionId>,
        from: Habitation,
        to: Habitation,
    },
    Ruined {
        site: SiteId,
        owner: Option<FactionId>,
        ruination: u64,
    },
    PopulationMoved {
        owner: FactionId,
        from: SiteId,
        to: SiteId,
        amount: u32,
        resettled: bool,
    },
    SiteRenamed {
        owner: FactionId,
        site: SiteId,
        old_name: String,
        new_name: String,
    },
    CapitalMoved {
        owner: FactionId,
        from: SiteId,
        to: SiteId,
    },
    HeadquartersMoved {
        owner: FactionId,
        from: SiteId,
        to: SiteId,
    },
}

impl StrategicCampaign {
    pub fn initialize_development(&mut self, data: &GameData) -> Result<(), String> {
        data.development.validate()?;
        let active: std::collections::BTreeSet<_> = self
            .world
            .sites
            .iter()
            .filter(|site| self.active_threat(site.id).is_some())
            .map(|site| site.id)
            .collect();
        for site in &self.world.sites {
            let state = self.world.development.entry(site.id).or_default();
            if site.tags.contains(&SiteTag::Ruins) {
                state.ruined = true;
                state.ruination = 1;
                state.ruined_round = Some(self.completed_rounds);
                state.threat_created = active.contains(&site.id);
            }
        }
        Ok(())
    }
    pub fn site_is_ruined(&self, site: SiteId) -> bool {
        self.world
            .development
            .get(&site)
            .is_some_and(|state| state.ruined)
    }
}
