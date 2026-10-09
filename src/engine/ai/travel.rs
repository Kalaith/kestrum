//! Legal overland paths and deterministic objective ranking from observed facts.
use super::*;
use crate::{
    data::{
        economy::Habitation,
        world::{AnchorExpression, DiplomaticState, MarkerLocation},
    },
    engine::MoveOrder,
    state::military::Army,
};
use std::collections::BTreeSet;

impl Planner<'_> {
    pub(super) fn at_war(&self, other: FactionId) -> bool {
        other != self.owner
            && self.campaign.relations.iter().any(|relation| {
                relation.factions.contains(&self.owner)
                    && relation.factions.contains(&other)
                    && relation.state == DiplomaticState::War
            })
    }

    pub(super) fn path(
        &self,
        origin: SiteId,
        destination: SiteId,
        attack: bool,
    ) -> Option<(u32, Vec<SiteId>)> {
        self.routes.path(origin, destination, attack)
    }

    fn transit(&self, site: SiteId, destination: SiteId, attack: bool) -> bool {
        if self.view.threats.iter().any(|threat| threat.site == site) {
            return false;
        }
        if (self.view.hostile_presence.contains(&site)
            || self.view.world.contested_sites.contains(&site))
            && !(attack && site == destination)
        {
            return false;
        }
        self.view.world.site(site).is_some_and(|site| {
            site.controller
                .is_none_or(|owner| owner == self.owner || self.at_war(owner))
        })
    }

    pub(super) fn moving(&self, army: &Army, emergency: bool) -> bool {
        self.campaign
            .army_movement_remaining(army.id, self.data)
            .is_some_and(|remaining| remaining > 0)
            && (emergency
                || (!self.weak(army)
                    && !self
                        .view
                        .construction
                        .iter()
                        .any(|order| order.is_open() && order.builder == Some(army.id))))
            && !self
                .view
                .sieges
                .iter()
                .any(|siege| siege.own_armies.contains(&army.id))
    }

    pub(super) fn toward(
        &self,
        destination: SiteId,
        kind: AiObjectiveKind,
        emergency: bool,
    ) -> Option<AiDecision> {
        let mut candidates: Vec<_> = self
            .view
            .armies
            .iter()
            .filter(|army| self.moving(army, emergency))
            .filter(|army| kind != AiObjectiveKind::Border || !self.posted_border_post(army.site))
            .filter_map(|army| {
                self.path(
                    army.site,
                    destination,
                    matches!(kind, AiObjectiveKind::Attack | AiObjectiveKind::Defend),
                )
                .map(|(cost, path)| (cost, army.id, path))
            })
            .filter(|(_, _, path)| path.len() > 1)
            .collect();
        candidates.sort();
        candidates.into_iter().find_map(|(_, army, path)| {
            let next = path[1];
            if self.view.hostile_presence.contains(&next) && !self.advantage(&[army], next) {
                return None;
            }
            self.choose(
                Command::Move(MoveOrder {
                    armies: vec![army],
                    path: path[..2].to_vec(),
                }),
                Some(self.objective(destination, kind)),
            )
        })
    }

    pub(super) fn strategic_priority(&self, site: SiteId) -> u8 {
        if site == self.campaign.factions[&self.owner].headquarters {
            return 0;
        }
        if self.anchors().contains(&site) {
            return 1;
        }
        match self.view.world.site(site).map(|site| site.habitation) {
            Some(
                Habitation::Hamlet
                | Habitation::Village
                | Habitation::Town
                | Habitation::City
                | Habitation::MajorCity,
            ) => 2,
            Some(Habitation::Outpost) => 3,
            _ => 4,
        }
    }

    pub(super) fn anchors(&self) -> BTreeSet<SiteId> {
        fn collect(expression: &AnchorExpression, ids: &mut BTreeSet<SiteId>) {
            match expression {
                AnchorExpression::ControlledSite { site }
                | AnchorExpression::SuppliedEntrance { site } => {
                    ids.insert(*site);
                }
                AnchorExpression::All { conditions } | AnchorExpression::Any { conditions } => {
                    for condition in conditions {
                        collect(condition, ids);
                    }
                }
            }
        }
        let mut ids = BTreeSet::new();
        for marker in &self.view.world.markers {
            if let MarkerLocation::Region { anchors, .. } = &marker.location {
                collect(anchors, &mut ids);
            }
        }
        ids
    }

    pub(super) fn targets(&self, sites: impl Iterator<Item = SiteId>) -> Vec<SiteId> {
        let candidates = sites
            .filter_map(|site| {
                self.view
                    .armies
                    .iter()
                    .filter(|army| self.moving(army, false))
                    .filter_map(|army| self.path(army.site, site, true).map(|(cost, _)| cost))
                    .min()
                    .map(|travel_cost| AiTarget {
                        site,
                        travel_cost,
                        strategic_priority: self.strategic_priority(site),
                    })
            })
            .collect();
        rank_targets(self.campaign, &self.view, candidates)
            .into_iter()
            .map(|target| target.site)
            .collect()
    }

    pub(super) fn threat_approaches(
        &self,
        site: SiteId,
    ) -> Vec<(u32, crate::state::military::ArmyId, Vec<SiteId>)> {
        let mut approaches = Vec::new();
        for staging in self.view.world.adjacent_sites(site) {
            if !self.transit(staging, staging, false) {
                continue;
            }
            let Some(edge) = self.view.world.connected_route(staging, site) else {
                continue;
            };
            let final_cost = crate::engine::movement::route_cost(edge, self.data);
            for army in self
                .view
                .armies
                .iter()
                .filter(|army| self.moving(army, false))
            {
                if let Some((cost, path)) = self.path(army.site, staging, false) {
                    if let Some(total) = cost.checked_add(final_cost) {
                        approaches.push((total, army.id, path));
                    }
                }
            }
        }
        approaches.sort();
        approaches
    }

    pub(super) fn pursue(&self) -> Option<AiDecision> {
        let objective = self.objective.as_ref()?;
        match objective.kind {
            AiObjectiveKind::Threat => self.clear_threat_at(objective.site),
            AiObjectiveKind::Attack => self.attack_at(objective.site),
            _ => self.toward(
                objective.site,
                objective.kind,
                objective.kind == AiObjectiveKind::Defend,
            ),
        }
    }

    /// Unclaimed sites that touch supplied home territory within the realm's
    /// reach, with their route cost from the capital. Claims made here stay
    /// joined to the capital instead of opening exposed, disconnected holdings.
    pub(super) fn frontier(&self) -> Vec<(u32, SiteId)> {
        let headquarters = self.campaign.factions[&self.owner].headquarters;
        self.view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller.is_none()
                    && !self.view.hostile_presence.contains(&site.id)
                    && self
                        .view
                        .world
                        .adjacent_sites(site.id)
                        .into_iter()
                        .any(|neighbor| self.view.supplied_sites.contains(&neighbor))
            })
            .filter_map(|site| {
                let (cost, path) = self.path(headquarters, site.id, false)?;
                (path.len().saturating_sub(1) <= self.data.ai.expansion_reach)
                    .then_some((cost, site.id))
            })
            .collect()
    }

    pub(super) fn expand(&self) -> Option<AiDecision> {
        let home_weight = u64::from(self.data.ai.home_distance_percent);
        let mut candidates: Vec<_> = self
            .frontier()
            .into_iter()
            .filter_map(|(home, site)| {
                let travel = self
                    .view
                    .armies
                    .iter()
                    .filter(|army| self.moving(army, false))
                    .filter_map(|army| self.path(army.site, site, false).map(|(cost, _)| cost))
                    .min()?;
                // Distance from the capital outweighs the nearest army's walk,
                // so the realm fills in around home before reaching outward.
                let score = (u64::from(home) * home_weight / 100).saturating_add(travel.into());
                Some((score, self.strategic_priority(site), site))
            })
            .collect();
        candidates.sort_unstable();
        candidates
            .into_iter()
            .find_map(|(_, _, site)| self.toward(site, AiObjectiveKind::Expand, false))
    }

    pub(super) fn border_post(&self, id: SiteId) -> bool {
        self.view.world.site(id).is_some_and(|post| {
            post.controller == Some(self.owner)
                && self
                    .view
                    .world
                    .adjacent_sites(post.id)
                    .into_iter()
                    .any(|neighbor| {
                        self.view.hostile_presence.contains(&neighbor)
                            || self
                                .view
                                .threats
                                .iter()
                                .any(|threat| threat.site == neighbor)
                            || self.view.world.site(neighbor).is_some_and(|adjacent| {
                                adjacent.controller.is_some_and(|owner| self.at_war(owner))
                            })
                    })
        })
    }

    pub(super) fn posted_border_post(&self, id: SiteId) -> bool {
        self.border_post(id) && self.view.armies.iter().any(|army| army.site == id)
    }

    pub(super) fn border(&self) -> Option<AiDecision> {
        let borders = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                self.border_post(site.id)
                    && !self.view.armies.iter().any(|army| army.site == site.id)
            })
            .map(|site| site.id);
        self.targets(borders)
            .into_iter()
            .find_map(|site| self.toward(site, AiObjectiveKind::Border, false))
    }
}
