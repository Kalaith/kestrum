//! Release stranded builders and reconnect isolated forces through known land.
use super::*;
use crate::{
    data::world::DiplomaticState,
    engine::{movement::route_cost, MoveOrder},
    state::construction::{
        ConstructionKind, ConstructionPause, ConstructionStatus, ConstructionTarget,
    },
    state::military::ArmyId,
};
use std::collections::{BTreeMap, BTreeSet};

type RoutePath = (u32, Vec<SiteId>);

#[derive(Debug, Clone)]
struct Corridor {
    cost: u32,
    rival: Option<FactionId>,
    armies: Vec<ArmyId>,
    path: Vec<SiteId>,
}

impl Planner<'_> {
    /// Cancel only stale supply-paused work that still holds an own builder.
    pub(super) fn release_stranded_builders(&self) -> Option<AiDecision> {
        let current_round = self.campaign.completed_rounds;
        self.view
            .construction
            .iter()
            .filter(|order| {
                order.owner == self.owner
                    && order.is_open()
                    && !matches!(order.kind, ConstructionKind::Facility(_))
                    && matches!(
                        order.status,
                        ConstructionStatus::Paused {
                            reason: ConstructionPause::SupplyLost
                        }
                    )
                    && order.builder.is_some_and(|builder| {
                        self.view
                            .armies
                            .iter()
                            .any(|army| army.id == builder && army.faction == self.owner)
                    })
                    && self.target_is_unsupplied(order.target)
                    && current_round
                        .saturating_sub(order.last_progress_round.unwrap_or(order.created_round))
                        >= self.data.ai.objective_rounds
            })
            .map(|order| order.id)
            .find_map(|order| {
                self.choose(
                    Command::CancelConstruction { order },
                    self.objective.clone(),
                )
            })
    }

    /// Move isolated healthy armies toward known supply, opening only a proven corridor.
    pub(super) fn reconnect_supply(&self) -> Option<AiDecision> {
        let armies = self.movable_unsupplied_groups();
        if armies.is_empty() {
            return None;
        }

        let current_enemies = self.current_enemies();
        let mut existing: Vec<_> = armies
            .iter()
            .filter_map(|group| self.corridor(&self.routes, &current_enemies, group))
            .collect();
        existing.sort_by_key(corridor_key);
        for corridor in existing {
            if let Some(decision) = self.follow_corridor(corridor) {
                return Some(decision);
            }
        }

        if !self.can_open_corridor() {
            return None;
        }
        self.open_blocking_corridor(&armies, &current_enemies)
    }

    fn target_is_unsupplied(&self, target: ConstructionTarget) -> bool {
        let targets: Vec<_> = match target {
            ConstructionTarget::Site(site) => vec![site],
            ConstructionTarget::Route(route) => self
                .view
                .world
                .route(route)
                .map(|route| vec![route.from, route.to])
                .unwrap_or_default(),
        };
        !targets.is_empty()
            && targets
                .iter()
                .any(|site| !self.view.supplied_sites.contains(site))
    }

    fn current_enemies(&self) -> BTreeSet<FactionId> {
        self.campaign
            .relations
            .iter()
            .filter(|relation| {
                relation.state == DiplomaticState::War && relation.factions.contains(&self.owner)
            })
            .flat_map(|relation| relation.factions)
            .filter(|other| *other != self.owner)
            .collect()
    }

    fn movable_unsupplied_groups(&self) -> Vec<Vec<ArmyId>> {
        let mut groups = BTreeMap::<SiteId, Vec<ArmyId>>::new();
        for army in &self.view.armies {
            if !self.view.supplied_armies.contains(&army.id)
                && self.moving(army, false)
                && !self.weak(army)
            {
                groups.entry(army.site).or_default().push(army.id);
            }
        }
        groups.into_values().collect()
    }

    fn corridor(
        &self,
        routes: &ObservedRoutes,
        enemies: &BTreeSet<FactionId>,
        armies: &[ArmyId],
    ) -> Option<Corridor> {
        let origin = self.army_site(*armies.first()?)?;
        let supplied: BTreeSet<_> = self
            .view
            .supplied_sites
            .iter()
            .copied()
            .filter(|site| {
                self.view
                    .world
                    .site(*site)
                    .is_some_and(|record| record.controller == Some(self.owner))
            })
            .collect();
        let mut candidates = Vec::new();
        for destination in supplied.iter().copied() {
            let Some(path) = routes.path(origin, destination, false) else {
                continue;
            };
            if path.1.len() > 1 {
                candidates.push(Corridor {
                    cost: path.0,
                    rival: None,
                    armies: armies.to_vec(),
                    path: path.1,
                });
            }
        }
        if candidates.is_empty() {
            if let Some((cost, path)) = self.known_path(origin, &supplied, enemies) {
                candidates.push(Corridor {
                    cost,
                    rival: None,
                    armies: armies.to_vec(),
                    path,
                });
            }
        }
        candidates.sort_by_key(corridor_key);
        candidates.into_iter().next()
    }

    fn open_blocking_corridor(
        &self,
        groups: &[Vec<ArmyId>],
        current_enemies: &BTreeSet<FactionId>,
    ) -> Option<AiDecision> {
        let mut candidates = Vec::new();
        for rival in self.peaceful_rivals() {
            if !self.can_declare_for_recovery(rival) {
                continue;
            }
            let mut enemies = current_enemies.clone();
            enemies.insert(rival);
            let routes = ObservedRoutes::new(&self.view, self.data, &enemies);
            for group in groups {
                let Some(mut corridor) = self.corridor(&routes, &enemies, group) else {
                    continue;
                };
                if !self.path_crosses_faction(&corridor.path, rival)
                    || self
                        .corridor(&self.routes, current_enemies, group)
                        .is_some()
                    || !self.corridor_power_is_sufficient(group, &corridor.path)
                {
                    continue;
                }
                corridor.rival = Some(rival);
                candidates.push(corridor);
            }
        }
        candidates.sort_by_key(corridor_key);
        candidates.into_iter().find_map(|corridor| {
            self.choose(
                Command::DeclareWar {
                    faction: corridor.rival?,
                },
                self.objective.clone(),
            )
        })
    }

    fn follow_corridor(&self, corridor: Corridor) -> Option<AiDecision> {
        let origin = *corridor.path.first()?;
        let next = *corridor.path.get(1)?;
        if self.view.hostile_presence.contains(&next) && !self.advantage(&corridor.armies, next) {
            return None;
        }
        self.choose(
            Command::Move(MoveOrder {
                armies: corridor.armies,
                path: vec![origin, next],
            }),
            None,
        )
    }

    fn can_open_corridor(&self) -> bool {
        let faction = &self.campaign.factions[&self.owner];
        let established = self
            .view
            .armies
            .iter()
            .filter(|army| {
                army.formation_ids().count() >= self.data.ai.minimum_formations && !self.weak(army)
            })
            .count();
        let upkeep = self
            .view
            .formations
            .iter()
            .map(|formation| self.data.economy.formations[&formation.kind].upkeep_gold)
            .fold(0_i64, i64::saturating_add);
        let reserve = upkeep.saturating_mul(i64::from(self.data.ai.reserve_upkeep_rounds));
        !faction.deficit
            && faction.resources.gold >= reserve
            && established >= self.data.ai.target_armies
    }

    fn peaceful_rivals(&self) -> Vec<FactionId> {
        let mut rivals: Vec<_> = self
            .view
            .factions
            .iter()
            .map(|faction| faction.id)
            .filter(|rival| {
                *rival != self.owner
                    && self.campaign.is_independent(*rival)
                    && self.campaign.relations.iter().any(|relation| {
                        relation.factions.contains(&self.owner)
                            && relation.factions.contains(rival)
                            && relation.state == DiplomaticState::Peace
                    })
            })
            .collect();
        rivals.sort_unstable();
        rivals
    }

    fn can_declare_for_recovery(&self, rival: FactionId) -> bool {
        self.campaign
            .diplomacy
            .pair(self.owner, rival)
            .is_some_and(|pair| {
                pair.truce_until
                    .is_none_or(|until| self.campaign.completed_rounds >= until)
                    && pair.peace_since.is_some_and(|since| {
                        self.campaign.completed_rounds.saturating_sub(since)
                            >= self.data.ai.war_peace_rounds
                    })
            })
    }

    fn corridor_power_is_sufficient(&self, armies: &[ArmyId], path: &[SiteId]) -> bool {
        path.iter()
            .copied()
            .skip(1)
            .find(|site| self.view.hostile_presence.contains(site))
            .is_none_or(|site| self.advantage(armies, site))
    }

    fn path_crosses_faction(&self, path: &[SiteId], faction: FactionId) -> bool {
        path.iter().any(|site| {
            self.view
                .world
                .site(*site)
                .is_some_and(|record| record.controller == Some(faction))
        })
    }

    fn army_site(&self, army: ArmyId) -> Option<SiteId> {
        self.view
            .armies
            .iter()
            .find(|record| record.id == army)
            .map(|record| record.site)
    }

    fn known_path(
        &self,
        origin: SiteId,
        destinations: &BTreeSet<SiteId>,
        enemies: &BTreeSet<FactionId>,
    ) -> Option<RoutePath> {
        let mut edges: BTreeMap<SiteId, Vec<(SiteId, u32)>> = self
            .view
            .world
            .sites
            .iter()
            .map(|site| (site.id, Vec::new()))
            .collect();
        for route in &self.view.world.routes {
            let cost = route_cost(route, self.data);
            edges.entry(route.from).or_default().push((route.to, cost));
            edges.entry(route.to).or_default().push((route.from, cost));
        }
        for neighbors in edges.values_mut() {
            neighbors.sort_unstable();
        }

        let mut best = BTreeMap::from([(origin, (0_u32, vec![origin]))]);
        let mut settled = BTreeSet::new();
        loop {
            let (&site, (cost, path)) = best
                .iter()
                .filter(|(id, _)| !settled.contains(*id))
                .min_by(|left, right| left.1.cmp(right.1))?;
            let (cost, path) = (*cost, path.clone());
            settled.insert(site);
            if destinations.contains(&site) {
                return Some((cost, path));
            }
            for &(next, edge_cost) in edges.get(&site).into_iter().flatten() {
                if settled.contains(&next)
                    || self.view.world.contested_sites.contains(&next)
                    || self.view.threats.iter().any(|threat| threat.site == next)
                    || !self.controller_allows_transit(next, enemies)
                {
                    continue;
                }
                let Some(next_cost) = cost.checked_add(edge_cost) else {
                    continue;
                };
                let mut next_path = path.clone();
                next_path.push(next);
                let candidate = (next_cost, next_path);
                if best.get(&next).is_none_or(|old| candidate < *old) {
                    best.insert(next, candidate);
                }
            }
        }
    }

    fn controller_allows_transit(&self, site: SiteId, enemies: &BTreeSet<FactionId>) -> bool {
        self.view.world.site(site).is_some_and(|record| {
            record
                .controller
                .is_none_or(|controller| controller == self.owner || enemies.contains(&controller))
        })
    }
}

fn corridor_key(corridor: &Corridor) -> (u32, Option<FactionId>, Vec<ArmyId>, Vec<SiteId>) {
    (
        corridor.cost,
        corridor.rival,
        corridor.armies.clone(),
        corridor.path.clone(),
    )
}
