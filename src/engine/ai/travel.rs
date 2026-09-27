use super::*;
use crate::{
    data::{
        economy::Habitation,
        world::{AnchorExpression, DiplomaticState, MarkerLocation},
    },
    engine::MoveOrder,
    state::military::Army,
};
use std::collections::{BTreeMap, BTreeSet};

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
        let mut best = BTreeMap::from([(origin, (0_u32, vec![origin]))]);
        let mut settled = BTreeSet::new();
        loop {
            let (&site, (cost, path)) = best
                .iter()
                .filter(|(id, _)| !settled.contains(*id))
                .min_by(|a, b| a.1.cmp(b.1))?;
            let (cost, path) = (*cost, path.clone());
            if site == destination {
                return Some((cost, path));
            }
            settled.insert(site);
            for next in self.view.world.adjacent_sites(site) {
                if settled.contains(&next) || !self.transit(next, destination, attack) {
                    continue;
                }
                let route = self.view.world.connected_route(site, next)?;
                let next_cost =
                    cost.checked_add(crate::engine::movement::route_cost(route, self.data))?;
                let mut next_path = path.clone();
                next_path.push(next);
                let candidate = (next_cost, next_path);
                if best.get(&next).is_none_or(|old| candidate < *old) {
                    best.insert(next, candidate);
                }
            }
        }
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
        let mut targets: Vec<_> = sites
            .filter_map(|site| {
                self.view
                    .armies
                    .iter()
                    .filter(|army| self.moving(army, false))
                    .filter_map(|army| self.path(army.site, site, true).map(|(cost, _)| cost))
                    .min()
                    .map(|cost| (cost, self.strategic_priority(site), site))
            })
            .collect();
        targets.sort();
        targets.into_iter().map(|(_, _, site)| site).collect()
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

    pub(super) fn expand(&self) -> Option<AiDecision> {
        self.targets(
            self.view
                .world
                .sites
                .iter()
                .filter(|site| {
                    site.controller.is_none() && !self.view.hostile_presence.contains(&site.id)
                })
                .map(|site| site.id),
        )
        .into_iter()
        .find_map(|site| self.toward(site, AiObjectiveKind::Expand, false))
    }

    pub(super) fn border(&self) -> Option<AiDecision> {
        let borders = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller == Some(self.owner)
                    && self
                        .view
                        .world
                        .adjacent_sites(site.id)
                        .into_iter()
                        .any(|next| {
                            self.view.world.site(next).is_some_and(|site| {
                                site.controller.is_some_and(|owner| owner != self.owner)
                            })
                        })
            })
            .map(|site| site.id);
        self.targets(borders)
            .into_iter()
            .find_map(|site| self.toward(site, AiObjectiveKind::Border, false))
    }
}
