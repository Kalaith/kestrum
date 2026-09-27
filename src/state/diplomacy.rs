//! Durable political agreements and outcomes independent of pruned narratives.

mod validation;
use crate::{
    data::world::{DiplomaticState, FactionId, SiteId},
    state::{military::ArmyId, StrategicCampaign},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignDiplomacy {
    pub pairs: Vec<PairDiplomacy>,
    pub losses: Vec<SiteLoss>,
    pub last_attackers: BTreeMap<FactionId, FactionId>,
    pub pending_offers: Vec<PeaceOffer>,
    pub pending_defeats: Vec<PendingDefeat>,
    pub ending: Option<CampaignEnding>,
    /// False only for older saves whose retained records cannot establish a full era timeline.
    #[serde(default)]
    pub era_history_complete: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairDiplomacy {
    pub factions: [FactionId; 2],
    pub peace_since: Option<u32>,
    pub truce_until: Option<u32>,
    pub last_offer_round: Option<u32>,
    #[serde(default)]
    pub war_started_round: Option<u32>,
    #[serde(default)]
    pub war_ended_round: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteLoss {
    pub faction: FactionId,
    pub victor: FactionId,
    pub site: SiteId,
    pub completed_rounds: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeaceOffer {
    pub proposer: FactionId,
    pub recipient: FactionId,
    pub completed_rounds: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingDefeat {
    pub faction: FactionId,
    pub victor: FactionId,
    pub completed_rounds: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefeatResolution {
    Annex,
    Submission,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndingKind {
    Victory,
    Defeat,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignEnding {
    pub kind: EndingKind,
    pub completed_rounds: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiplomacyReceipt {
    WarDeclared {
        factions: [FactionId; 2],
    },
    PeaceOffered {
        proposer: FactionId,
        recipient: FactionId,
    },
    PeaceRejected {
        proposer: FactionId,
        recipient: FactionId,
    },
    PeaceAgreed {
        factions: [FactionId; 2],
        truce_until: u32,
    },
    ArmyWithdrawn {
        faction: FactionId,
        army: ArmyId,
        from: SiteId,
        to: SiteId,
    },
    DefeatPending {
        faction: FactionId,
        victor: FactionId,
    },
    FactionResolved {
        faction: FactionId,
        victor: Option<FactionId>,
        resolution: DefeatResolution,
    },
    CampaignEnded {
        ending: CampaignEnding,
    },
}

impl CampaignDiplomacy {
    pub fn pair(&self, a: FactionId, b: FactionId) -> Option<&PairDiplomacy> {
        let pair = ordered_pair(a, b);
        self.pairs.iter().find(|entry| entry.factions == pair)
    }
    pub fn has_pending_decision(&self) -> bool {
        !self.pending_offers.is_empty() || !self.pending_defeats.is_empty()
    }
    pub fn is_blocked(&self) -> bool {
        self.ending.is_some() || !self.pending_offers.is_empty() || !self.pending_defeats.is_empty()
    }
}
pub(crate) fn ordered_pair(a: FactionId, b: FactionId) -> [FactionId; 2] {
    if a < b {
        [a, b]
    } else {
        [b, a]
    }
}

impl StrategicCampaign {
    pub(crate) fn initialize_diplomacy(&mut self) {
        self.diplomacy = CampaignDiplomacy {
            pairs: self
                .relations
                .iter()
                .map(|relation| PairDiplomacy {
                    factions: relation.factions,
                    peace_since: (relation.state == DiplomaticState::Peace)
                        .then_some(self.completed_rounds),
                    truce_until: None,
                    last_offer_round: None,
                    war_started_round: (relation.state == DiplomaticState::War)
                        .then_some(self.completed_rounds),
                    war_ended_round: None,
                })
                .collect(),
            era_history_complete: true,
            ..Default::default()
        };
    }
    pub fn defeat_eligible(&self, faction: FactionId) -> bool {
        !self.world.sites.iter().any(|site| {
            site.controller == Some(faction)
                && site.habitation >= crate::data::economy::Habitation::Outpost
                && !self.site_is_ruined(site.id)
        }) && !self
            .armies
            .values()
            .any(|army| army.faction == faction && !army.is_empty())
    }
}
