//! Observer-only paths, cached for the lifetime of one planning decision.
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::VisibleCampaign,
};

type Path = (u32, Vec<SiteId>);
type Paths = BTreeMap<SiteId, Path>;

/// A snapshot of legal travel. Hostile/contested sites may be destinations of
/// attacks, but never intermediate steps. Hidden armies are absent from this
/// graph. Rebuild it after any accepted command changes the observer's view.
pub struct ObservedRoutes {
    edges: BTreeMap<SiteId, Vec<(SiteId, u32)>>,
    terminals: BTreeSet<SiteId>,
    forbidden: BTreeSet<SiteId>,
    cache: RefCell<BTreeMap<(SiteId, bool), Paths>>,
}

impl ObservedRoutes {
    pub fn new(view: &VisibleCampaign, data: &GameData, enemies: &BTreeSet<FactionId>) -> Self {
        let mut edges: BTreeMap<_, Vec<_>> = view
            .world
            .sites
            .iter()
            .map(|site| (site.id, Vec::new()))
            .collect();
        for route in &view.world.routes {
            let (left, right) = (route.from, route.to);
            let cost = crate::engine::movement::route_cost(route, data);
            edges.entry(left).or_default().push((right, cost));
            edges.entry(right).or_default().push((left, cost));
        }
        for neighbors in edges.values_mut() {
            neighbors.sort_unstable();
        }
        let forbidden = view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller
                    .is_some_and(|owner| owner != view.observer && !enemies.contains(&owner))
                    || view.threats.iter().any(|threat| threat.site == site.id)
            })
            .map(|site| site.id)
            .collect();
        Self {
            edges,
            terminals: view
                .hostile_presence
                .union(&view.world.contested_sites)
                .copied()
                .collect(),
            forbidden,
            cache: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn path(&self, origin: SiteId, destination: SiteId, attack: bool) -> Option<Path> {
        let key = (origin, attack);
        let mut cache = self.cache.borrow_mut();
        cache
            .entry(key)
            .or_insert_with(|| self.paths_from(origin, attack))
            .get(&destination)
            .cloned()
    }

    fn paths_from(&self, origin: SiteId, attack: bool) -> Paths {
        let mut best = BTreeMap::from([(origin, (0_u32, vec![origin]))]);
        let mut settled = BTreeSet::new();
        loop {
            let Some((&site, (cost, path))) = best
                .iter()
                .filter(|(id, _)| !settled.contains(*id))
                .min_by(|a, b| a.1.cmp(b.1))
            else {
                return best;
            };
            let (cost, path) = (*cost, path.clone());
            settled.insert(site);
            if site != origin && self.terminals.contains(&site) {
                continue;
            }
            for &(next, edge_cost) in self.edges.get(&site).into_iter().flatten() {
                if settled.contains(&next)
                    || self.forbidden.contains(&next)
                    || (!attack && self.terminals.contains(&next))
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
}
