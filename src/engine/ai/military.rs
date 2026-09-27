use super::*;
use crate::{
    engine::{MoveOrder, SiegeRole},
    state::{
        military::{Army, ArmyId},
        siege::{SiegeAction, SiegeOrder},
    },
};
use std::collections::BTreeMap;

impl Planner<'_> {
    pub(super) fn threatened(&self, site: SiteId) -> bool {
        self.view.hostile_presence.contains(&site)
            || self
                .view
                .world
                .adjacent_sites(site)
                .into_iter()
                .any(|id| self.view.hostile_presence.contains(&id))
    }
    pub(super) fn threatened_headquarters(&self) -> bool {
        self.threatened(self.campaign.factions[&self.owner].headquarters)
    }

    pub(super) fn defend(&self) -> Option<AiDecision> {
        let headquarters = self.campaign.factions[&self.owner].headquarters;
        let anchors = self.anchors();
        let mut targets: Vec<_> = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller == Some(self.owner)
                    && (site.id == headquarters || anchors.contains(&site.id))
                    && self.threatened(site.id)
            })
            .map(|site| site.id)
            .collect();
        targets.sort_by_key(|site| (*site != headquarters, *site));
        targets.into_iter().find_map(|site| {
            // A known weaker force blocking the HQ approach can be driven off.
            // Waiting at an unreachable defensive objective would waste the phase.
            std::iter::once(site)
                .chain(self.view.world.adjacent_sites(site))
                .filter(|enemy| self.view.hostile_presence.contains(enemy))
                .find_map(|enemy| self.attack_at(enemy))
                .map(|mut decision| {
                    decision.objective = Some(self.objective(site, AiObjectiveKind::Defend));
                    decision
                })
                .or_else(|| self.toward(site, AiObjectiveKind::Defend, true))
        })
    }

    pub(super) fn weak(&self, army: &Army) -> bool {
        let (heads, capacity) = army
            .formation_ids()
            .filter_map(|id| {
                self.view
                    .formations
                    .iter()
                    .find(|formation| formation.id == id)
            })
            .fold((0_u64, 0_u64), |(heads, capacity), f| {
                (
                    heads + u64::from(f.headcount),
                    capacity + u64::from(f.capacity),
                )
            });
        heads * 100 < capacity * u64::from(self.data.ai.retreat_capacity_percent)
    }

    pub(super) fn retreat(&self) -> Option<AiDecision> {
        for siege in &self.view.sieges {
            let view = crate::engine::siege_view(self.campaign, self.data, self.owner, siege.site)?;
            if !view.own_armies.iter().any(|id| {
                self.view
                    .armies
                    .iter()
                    .find(|army| army.id == *id)
                    .is_some_and(|army| self.weak(army))
            }) {
                continue;
            }
            let action = if view.role == SiegeRole::Defender {
                SiegeAction::Escape
            } else {
                SiegeAction::Withdraw
            };
            let Some(option) = view
                .actions
                .iter()
                .find(|option| option.action == action && option.blocked.is_none())
            else {
                continue;
            };
            let mut destinations = option.destinations.clone();
            destinations.sort_by_key(|site| (!self.view.supplied_sites.contains(site), *site));
            for destination in destinations {
                let command = Command::Siege(SiegeOrder {
                    site: view.site,
                    action,
                    armies: view.own_armies.clone(),
                    destination: Some(destination),
                });
                if let Some(decision) = self.choose(command, self.objective.clone()) {
                    return Some(decision);
                }
            }
        }
        for army in self.view.armies.iter().filter(|army| {
            self.weak(army)
                && !self.view.supplied_sites.contains(&army.site)
                && self.moving(army, true)
        }) {
            let mut routes: Vec<_> = self
                .view
                .supplied_sites
                .iter()
                .filter_map(|site| {
                    self.path(army.site, *site, false)
                        .map(|(cost, path)| (cost, *site, path))
                })
                .collect();
            routes.sort();
            for (_, _, path) in routes.into_iter().filter(|(_, _, path)| path.len() > 1) {
                if let Some(decision) = self.choose(
                    Command::Move(MoveOrder {
                        armies: vec![army.id],
                        path: path[..2].to_vec(),
                    }),
                    self.objective.clone(),
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    pub(super) fn clear_threat(&self) -> Option<AiDecision> {
        self.targets(self.view.threats.iter().map(|threat| threat.site))
            .into_iter()
            .find_map(|site| self.clear_threat_at(site))
    }
    pub(super) fn clear_threat_at(&self, site: SiteId) -> Option<AiDecision> {
        let threat = self
            .view
            .threats
            .iter()
            .find(|threat| threat.site == site)?;
        let mut groups: BTreeMap<SiteId, Vec<ArmyId>> = BTreeMap::new();
        for army in self.view.armies.iter().filter(|army| {
            self.moving(army, false) && self.view.world.connected_route(army.site, site).is_some()
        }) {
            groups.entry(army.site).or_default().push(army.id);
        }
        groups.into_values().find_map(|armies| {
            self.choose(
                Command::ClearThreat {
                    armies,
                    threat: threat.id,
                },
                Some(self.objective(site, AiObjectiveKind::Threat)),
            )
        })
    }

    pub(super) fn attack(&self) -> Option<AiDecision> {
        for siege in &self.view.sieges {
            let view = crate::engine::siege_view(self.campaign, self.data, self.owner, siege.site)?;
            if self.advantage(&view.own_armies, view.site) {
                let action = if view.role == SiegeRole::Defender {
                    SiegeAction::Sortie
                } else {
                    SiegeAction::Assault
                };
                if let Some(decision) = self.choose(
                    Command::Siege(SiegeOrder {
                        site: view.site,
                        action,
                        armies: view.own_armies,
                        destination: None,
                    }),
                    Some(self.objective(view.site, AiObjectiveKind::Attack)),
                ) {
                    return Some(decision);
                }
            }
        }
        let observed_empty = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller.is_some_and(|owner| self.at_war(owner))
                    && self.view.armies.iter().any(|army| {
                        self.view
                            .world
                            .connected_route(army.site, site.id)
                            .is_some()
                    })
                    && !self
                        .view
                        .threats
                        .iter()
                        .any(|threat| threat.site == site.id)
            })
            .map(|site| site.id);
        self.targets(
            self.view
                .hostile_presence
                .iter()
                .copied()
                .chain(observed_empty),
        )
        .into_iter()
        .find_map(|site| self.attack_at(site))
    }
    pub(super) fn attack_at(&self, site: SiteId) -> Option<AiDecision> {
        let mut groups: BTreeMap<SiteId, Vec<ArmyId>> = BTreeMap::new();
        for army in self.view.armies.iter().filter(|army| {
            self.moving(army, false) && self.view.world.connected_route(army.site, site).is_some()
        }) {
            groups.entry(army.site).or_default().push(army.id);
        }
        groups.into_iter().find_map(|(origin, armies)| {
            (!self.view.hostile_presence.contains(&site) || self.advantage(&armies, site))
                .then(|| {
                    self.choose(
                        Command::Move(MoveOrder {
                            armies,
                            path: vec![origin, site],
                        }),
                        Some(self.objective(site, AiObjectiveKind::Attack)),
                    )
                })
                .flatten()
        })
    }

    pub(super) fn advantage(&self, armies: &[ArmyId], site: SiteId) -> bool {
        let own: u128 = armies
            .iter()
            .filter_map(|id| self.view.armies.iter().find(|army| army.id == *id))
            .map(|army| {
                let leadership = u128::from(
                    self.campaign
                        .army_leadership_permille(army.id, self.data)
                        .unwrap_or(0),
                );
                army.formation_ids()
                    .filter_map(|id| {
                        self.view
                            .formations
                            .iter()
                            .find(|formation| formation.id == id)
                    })
                    .map(|formation| {
                        u128::from(formation.headcount)
                            * u128::from(self.data.troops.formations[&formation.kind].attack)
                            * leadership
                            * u128::from(formation.service.tier.permille(&self.data.progression))
                    })
                    .sum::<u128>()
            })
            .sum();
        let mut observations = BTreeMap::new();
        for report in &self.view.battles {
            for side in report
                .faction_sides()
                .filter(|side| self.at_war(side.faction))
            {
                for army in &side.armies {
                    let key = (report.completed_rounds, report.sequence, report.id);
                    if observations.get(&army.id).is_none_or(|(old, _)| key > *old) {
                        observations.insert(army.id, (key, army));
                    }
                }
            }
        }
        let enemy: u128 = observations
            .values()
            .filter(|(_, army)| army.final_site == Some(site))
            .map(|(_, army)| {
                army.formations
                    .iter()
                    .map(|f| {
                        u128::from(f.end)
                            * u128::from(self.data.troops.formations[&f.kind].attack)
                            * u128::from(army.leadership_permille)
                            * u128::from(f.veterancy_permille)
                    })
                    .sum::<u128>()
            })
            .sum();
        enemy > 0 && own * 100 > enemy * u128::from(self.data.ai.attack_advantage_percent)
    }
}
