//! Read-only map summaries of discovered places and visible forces.

use super::VisibleCampaign;
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, MarkerId, MarkerLocation, Site, SiteId},
    },
    state::military::{Army, ArmyId},
};
use std::collections::{BTreeMap, BTreeSet};

/// Per-kind counts are known affected sites, never hostile armies or troop strength.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapDanger {
    pub threats: usize,
    pub sieges: usize,
    pub hostile_contacts: usize,
}

impl MapDanger {
    /// Number of known conditions; one site can contribute more than one kind.
    pub fn count(self) -> usize {
        self.threats + self.sieges + self.hostile_contacts
    }

    pub fn any(self) -> bool {
        self.count() > 0
    }

    fn include(&mut self, other: Self) {
        self.threats += other.threats;
        self.sieges += other.sieges;
        self.hostile_contacts += other.hostile_contacts;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteOverview {
    pub controller: Option<FactionId>,
    pub political_owner: Option<FactionId>,
    /// False distinguishes withheld regional claims from known neutral land.
    pub political_known: bool,
    pub contested: bool,
    /// Lasting occupation pressure, or foreign physical control inside a known claim.
    pub occupied: bool,
    pub habitation: Habitation,
    pub capital: bool,
    pub danger: MapDanger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerOverview {
    pub political_owner: Option<FactionId>,
    pub political_known: bool,
    pub contested: bool,
    /// Describes only discovered internal sites; it makes no claim about unseen land.
    pub mixed_control: bool,
    pub occupied: bool,
    pub capital: bool,
    pub habitation: Habitation,
    pub danger: MapDanger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmyMapStatus {
    Idle,
    Queued,
    Siege,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmyOverview {
    pub faction: FactionId,
    pub site: SiteId,
    pub name: String,
    pub troops: u32,
    pub status: ArmyMapStatus,
    pub supplied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttentionTarget {
    Site(SiteId),
    Army(ArmyId),
}

/// Ordering is the strategic urgency used by the compact map list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttentionKind {
    Siege,
    HostileContact,
    LocalThreat,
    RuinRisk,
    DeclineRisk,
    Contested,
    Occupied,
    Unsupplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapAttention {
    pub target: AttentionTarget,
    pub kind: AttentionKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapOverview {
    pub sites: BTreeMap<SiteId, SiteOverview>,
    pub markers: BTreeMap<MarkerId, MarkerOverview>,
    pub armies: BTreeMap<ArmyId, ArmyOverview>,
    pub attention: Vec<MapAttention>,
}

/// Independent discovery filtering makes this safe for both projection entry points.
/// Full observer map views may summarize every faction's visible records.
pub fn map_overview(view: &VisibleCampaign) -> MapOverview {
    let sites: BTreeMap<_, _> = view
        .world
        .sites
        .iter()
        .filter(|site| view.known_sites.contains(&site.id))
        .map(|site| (site.id, site_overview(view, site)))
        .collect();
    let markers = view
        .world
        .markers
        .iter()
        .filter_map(|marker| {
            marker_overview(view, marker.id, &sites).map(|summary| (marker.id, summary))
        })
        .collect();
    let armies: BTreeMap<_, _> = view
        .armies
        .iter()
        .filter(|army| {
            (view.full_map_visibility || army.faction == view.observer)
                && sites.contains_key(&army.site)
        })
        .map(|army| (army.id, army_overview(view, army)))
        .collect();
    let attention = attention(view, &sites, &armies);
    MapOverview {
        sites,
        markers,
        armies,
        attention,
    }
}

#[derive(Default)]
struct PoliticalClaim {
    known: bool,
    owner: Option<FactionId>,
    contested: bool,
}

fn political_claim(view: &VisibleCampaign, marker: MarkerId) -> PoliticalClaim {
    match view.world.marker(marker).map(|marker| &marker.location) {
        Some(MarkerLocation::Site { site }) if view.known_sites.contains(site) => PoliticalClaim {
            known: true,
            owner: view.world.site(*site).and_then(|site| site.controller),
            contested: false,
        },
        Some(MarkerLocation::Region { sites, .. })
            if sites.iter().all(|site| view.known_sites.contains(site)) =>
        {
            view.world
                .region_control(marker)
                .map_or_else(PoliticalClaim::default, |control| PoliticalClaim {
                    known: true,
                    owner: control.political_owner,
                    contested: control.contested,
                })
        }
        _ => PoliticalClaim::default(),
    }
}

fn site_overview(view: &VisibleCampaign, site: &Site) -> SiteOverview {
    let claim = political_claim(view, site.marker);
    let occupied = view
        .world
        .occupation
        .get(&site.id)
        .is_some_and(|pressure| *pressure > 0)
        || claim.owner.is_some_and(|owner| {
            site.controller
                .is_some_and(|controller| controller != owner)
        });
    SiteOverview {
        controller: site.controller,
        political_owner: claim.owner,
        political_known: claim.known,
        contested: view.world.contested_sites.contains(&site.id),
        occupied,
        habitation: site.habitation,
        capital: view
            .factions
            .iter()
            .any(|faction| faction.capital == Some(site.id)),
        danger: MapDanger {
            threats: usize::from(view.threats.iter().any(|threat| threat.site == site.id)),
            sieges: usize::from(view.sieges.iter().any(|siege| siege.site == site.id)),
            hostile_contacts: usize::from(view.hostile_presence.contains(&site.id)),
        },
    }
}

fn marker_overview(
    view: &VisibleCampaign,
    marker: MarkerId,
    sites: &BTreeMap<SiteId, SiteOverview>,
) -> Option<MarkerOverview> {
    let local: Vec<_> = view
        .world
        .sites
        .iter()
        .filter(|site| site.marker == marker)
        .filter_map(|site| sites.get(&site.id))
        .collect();
    if local.is_empty() {
        return None;
    }
    let claim = political_claim(view, marker);
    let mut danger = MapDanger::default();
    for summary in &local {
        danger.include(summary.danger);
    }
    Some(MarkerOverview {
        political_owner: claim.owner,
        political_known: claim.known,
        contested: claim.contested || local.iter().any(|site| site.contested),
        mixed_control: local
            .iter()
            .map(|site| site.controller)
            .collect::<BTreeSet<_>>()
            .len()
            > 1,
        occupied: local.iter().any(|site| site.occupied),
        capital: local.iter().any(|site| site.capital),
        habitation: local
            .iter()
            .map(|site| site.habitation)
            .max()
            .unwrap_or(Habitation::Unsettled),
        danger,
    })
}

fn army_overview(view: &VisibleCampaign, army: &Army) -> ArmyOverview {
    let status = if view.sieges.iter().any(|siege| {
        siege.own_armies.contains(&army.id)
            || siege.defending_armies.contains(&army.id)
            || siege.besieging_armies.contains(&army.id)
    }) {
        ArmyMapStatus::Siege
    } else if view
        .movement_plans
        .iter()
        .any(|plan| plan.armies.contains(&army.id))
    {
        ArmyMapStatus::Queued
    } else {
        ArmyMapStatus::Idle
    };
    ArmyOverview {
        faction: army.faction,
        site: army.site,
        name: army.name.clone(),
        troops: army
            .formation_ids()
            .filter_map(|id| {
                view.formations
                    .iter()
                    .find(|formation| formation.id == id && formation.faction == army.faction)
            })
            .map(|formation| formation.headcount)
            .sum(),
        status,
        supplied: view.supplied_armies.contains(&army.id),
    }
}

fn attention(
    view: &VisibleCampaign,
    sites: &BTreeMap<SiteId, SiteOverview>,
    armies: &BTreeMap<ArmyId, ArmyOverview>,
) -> Vec<MapAttention> {
    let mut result = Vec::new();
    for (&id, site) in sites {
        let concerns_observer = view.full_map_visibility
            || site.controller == Some(view.observer)
            || site.political_owner == Some(view.observer);
        let kind = if site.danger.sieges > 0 {
            Some(AttentionKind::Siege)
        } else if site.danger.hostile_contacts > 0 {
            Some(AttentionKind::HostileContact)
        } else if site.danger.threats > 0 {
            Some(AttentionKind::LocalThreat)
        } else if concerns_observer && site.contested {
            Some(AttentionKind::Contested)
        } else if concerns_observer && site.occupied {
            Some(AttentionKind::Occupied)
        } else {
            None
        };
        if let Some(kind) = kind {
            result.push(MapAttention {
                target: AttentionTarget::Site(id),
                kind,
            });
        }
    }
    result.extend(
        armies
            .iter()
            .filter(|(_, army)| !army.supplied)
            .map(|(&id, _)| MapAttention {
                target: AttentionTarget::Army(id),
                kind: AttentionKind::Unsupplied,
            }),
    );
    result.sort_by_key(|entry| (entry.kind, entry.target));
    result
}
