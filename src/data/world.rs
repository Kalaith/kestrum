//! Durable authored topology and setup grants; these are not campaign entities.

use super::economy::{Economy, Habitation, Resources, TroopKind};
use super::rules::{Difficulty, Emblem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCE: &str = "assets/data/scenarios/rosemarch.json";
pub const LAYOUT_SOURCE: &str = "assets/data/world_layout.json";

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u32);
    };
}

stable_id!(SiteId);
stable_id!(MarkerId);
stable_id!(RouteId);
stable_id!(FactionId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    RosemarchPrototype,
    Production,
}

/// Authored production geography. Campaign ownership and ordinary settlement
/// contents are assigned only when a player confirms a seeded setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldLayout {
    pub layout_revision: u32,
    pub legacy_marker_positions: std::collections::BTreeMap<MarkerId, [f32; 2]>,
    pub atlas_paths: std::collections::BTreeMap<RouteId, AtlasPath>,
    pub schema_version: u32,
    pub content_version: u32,
    pub default_seed: u64,
    pub markers: Vec<MajorMarker>,
    pub sites: Vec<Site>,
    pub routes: Vec<Route>,
    pub headquarters_candidates: Vec<SiteId>,
}

/// Authored display geometry; movement still follows the fixed route endpoints and cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlasPath {
    pub waypoints: Vec<[f32; 2]>,
    pub bridges: Vec<[f32; 2]>,
}

impl WorldLayout {
    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.iter().find(|site| site.id == id)
    }

    pub fn marker(&self, id: MarkerId) -> Option<&MajorMarker> {
        self.markers.iter().find(|marker| marker.id == id)
    }

    pub fn reachable_sites(&self, start: SiteId) -> BTreeSet<SiteId> {
        let mut visited = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(site) = pending.pop() {
            if self.site(site).is_some() && visited.insert(site) {
                pending.extend(
                    self.routes
                        .iter()
                        .filter_map(|route| route.other_endpoint(site)),
                );
            }
        }
        visited
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Geography {
    Valley,
    Coast,
    Marsh,
    Pass,
    Island,
    Plains,
    Forest,
    Hill,
    River,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteTag {
    Ruins,
    Bridge,
    Pass,
    HorseAccess,
    WoodSource,
    StoneSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Facility {
    TrainingGround,
    Stable,
    Infirmary,
    Workshop,
    Temple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilitaryLayer {
    None,
    Fort,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub id: SiteId,
    pub key: String,
    pub name: String,
    pub marker: MarkerId,
    /// Normalized coordinates within the containing map (world or region).
    pub position: [f32; 2],
    pub geography: Geography,
    pub tags: Vec<SiteTag>,
    pub controller: Option<FactionId>,
    pub habitation: Habitation,
    pub military: MilitaryLayer,
    pub facilities: Vec<Facility>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MajorMarker {
    pub id: MarkerId,
    pub key: String,
    pub name: String,
    pub position: [f32; 2],
    pub location: MarkerLocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MarkerLocation {
    Site {
        site: SiteId,
    },
    Region {
        sites: Vec<SiteId>,
        entrances: Vec<Entrance>,
        anchors: AnchorExpression,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entrance {
    pub route: RouteId,
    pub site: SiteId,
}

/// Positive all/any expressions; secure control and supplied gates are separate facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "condition", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnchorExpression {
    All { conditions: Vec<AnchorExpression> },
    Any { conditions: Vec<AnchorExpression> },
    ControlledSite { site: SiteId },
    SuppliedEntrance { site: SiteId },
}

impl AnchorExpression {
    /// Evaluate supplied facts only. K04 owns actual control/supply projections.
    pub fn is_satisfied(&self, held: &BTreeSet<SiteId>, supplied: &BTreeSet<SiteId>) -> bool {
        match self {
            Self::All { conditions } => {
                !conditions.is_empty()
                    && conditions
                        .iter()
                        .all(|condition| condition.is_satisfied(held, supplied))
            }
            Self::Any { conditions } => conditions
                .iter()
                .any(|condition| condition.is_satisfied(held, supplied)),
            Self::ControlledSite { site } => held.contains(site),
            Self::SuppliedEntrance { site } => held.contains(site) && supplied.contains(site),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Road {
    pub improved: bool,
    pub damage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub id: RouteId,
    pub from: SiteId,
    pub to: SiteId,
    /// Present exactly when crossing between major markers, in endpoint order.
    pub major_connection: Option<[MarkerId; 2]>,
    pub terrain_cost: u32,
    pub road: Road,
}

impl Route {
    pub fn other_endpoint(&self, site: SiteId) -> Option<SiteId> {
        if site == self.from {
            Some(self.to)
        } else if site == self.to {
            Some(self.from)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceGrant {
    EconomyStartingResources,
}

impl ResourceGrant {
    pub fn resolve(self, economy: &Economy) -> Resources {
        match self {
            Self::EconomyStartingResources => economy.starting_resources,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonClass {
    Recruit,
    Infantry,
    Archer,
    Scout,
    Cavalry,
    Medic,
    Officer,
}

/// Founding grants retain their historical public name while all tracked people
/// use the complete ordinary human career roster.
pub type FounderClass = PersonClass;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FounderGrant {
    pub name: String,
    pub age_years: u32,
    pub class: FounderClass,
    pub attached_to: TroopKind,
    pub commander: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionSetup {
    pub id: FactionId,
    pub name: String,
    pub army_name: String,
    pub emblem: Emblem,
    pub headquarters: SiteId,
    pub capital: SiteId,
    pub resources: ResourceGrant,
    /// One HQ army of full formations, granted without recruitment charges.
    pub starting_formations: Vec<TroopKind>,
    pub founder: FounderGrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticState {
    War,
    Peace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub factions: [FactionId; 2],
    pub state: DiplomaticState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub content_version: u32,
    pub kind: ScenarioKind,
    pub name: String,
    pub seed: u64,
    pub difficulty: Difficulty,
    pub player: FactionId,
    pub markers: Vec<MajorMarker>,
    pub sites: Vec<Site>,
    pub routes: Vec<Route>,
    pub factions: Vec<FactionSetup>,
    pub relations: Vec<Relation>,
}

impl Scenario {
    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.iter().find(|site| site.id == id)
    }

    pub fn marker(&self, id: MarkerId) -> Option<&MajorMarker> {
        self.markers.iter().find(|marker| marker.id == id)
    }

    pub fn route(&self, id: RouteId) -> Option<&Route> {
        self.routes.iter().find(|route| route.id == id)
    }

    /// Topology only: ignores present control and does not imply current supply.
    pub fn reachable_sites(&self, start: SiteId) -> BTreeSet<SiteId> {
        let mut visited = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(site) = pending.pop() {
            if self.site(site).is_some() && visited.insert(site) {
                pending.extend(
                    self.routes
                        .iter()
                        .filter_map(|route| route.other_endpoint(site)),
                );
            }
        }
        visited
    }
}
