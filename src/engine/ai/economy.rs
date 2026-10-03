use super::*;
use crate::{
    data::{
        economy::{Habitation, Resources, TroopKind},
        world::{Facility, Geography, MilitaryLayer, SiteTag},
    },
    state::{
        construction::{ConstructionKind, ConstructionTarget, Focus},
        military::{Army, ArmyId},
    },
};

impl Planner<'_> {
    fn upkeep(&self) -> i64 {
        self.view
            .formations
            .iter()
            .map(|formation| self.data.economy.formations[&formation.kind].upkeep_gold)
            .fold(0_i64, i64::saturating_add)
    }

    fn reserve(&self, cost: Resources, extra: i64, emergency: bool) -> bool {
        let available = self.campaign.factions[&self.owner].resources;
        let reserve = if emergency {
            0
        } else {
            self.upkeep()
                .saturating_add(extra)
                .saturating_mul(i64::from(self.data.ai.reserve_upkeep_rounds))
        };
        available.gold.saturating_sub(cost.gold) >= reserve
            && available.gold >= cost.gold
            && available.wood >= cost.wood
            && available.stone >= cost.stone
    }

    pub(super) fn recruit(&self, emergency: bool) -> Option<AiDecision> {
        if self.campaign.factions[&self.owner].deficit {
            return None;
        }
        let mut armies: Vec<_> = self.view.armies.iter().collect();
        armies.sort_by_key(|army| {
            (
                army.formation_ids().count() >= self.data.ai.minimum_formations,
                army.id,
            )
        });
        for army in armies
            .iter()
            .filter(|army| army.formation_ids().count() < self.data.ai.minimum_formations)
        {
            if let Some(decision) = self.recruit_at(army.site, Some(army), emergency) {
                return Some(decision);
            }
        }
        let fronts = self
            .anchors()
            .iter()
            .filter(|site| {
                self.threatened(**site)
                    && self
                        .view
                        .world
                        .site(**site)
                        .is_some_and(|site| site.controller == Some(self.owner))
            })
            .count();
        let first_armies_full = armies
            .iter()
            .take(self.data.ai.target_armies)
            .all(|army| army.formation_ids().count() == 6);
        let target = if first_armies_full {
            self.data.ai.target_armies.max(fronts)
        } else {
            self.data.ai.target_armies
        };
        let established = armies
            .iter()
            .all(|army| army.formation_ids().count() >= self.data.ai.minimum_formations);
        if armies.len() < target && established {
            let mut sites: Vec<_> = self
                .view
                .world
                .sites
                .iter()
                .filter(|site| {
                    site.controller == Some(self.owner) && site.habitation >= Habitation::Outpost
                })
                .collect();
            sites.sort_by_key(|site| {
                (
                    site.id != self.campaign.factions[&self.owner].headquarters,
                    site.id,
                )
            });
            for site in sites {
                if let Some(decision) = self.recruit_at(site.id, None, emergency) {
                    return Some(decision);
                }
            }
        }
        for army in armies
            .iter()
            .take(target)
            .filter(|army| army.formation_ids().count() < 6)
        {
            if let Some(decision) = self.recruit_at(army.site, Some(army), emergency) {
                return Some(decision);
            }
        }
        None
    }

    fn recruit_at(
        &self,
        site: SiteId,
        army: Option<&&Army>,
        emergency: bool,
    ) -> Option<AiDecision> {
        let kinds: Vec<_> = army
            .into_iter()
            .flat_map(|army| army.formation_ids())
            .filter_map(|id| {
                self.view
                    .formations
                    .iter()
                    .find(|f| f.id == id)
                    .map(|f| f.kind)
            })
            .collect();
        let mut order = self.data.ai.recruitment_order.clone();
        order.sort_by_key(|kind| {
            (
                kinds.contains(kind),
                self.data
                    .ai
                    .recruitment_order
                    .iter()
                    .position(|item| item == kind),
            )
        });
        order.into_iter().find_map(|kind| {
            let definition = &self.data.economy.formations[&kind];
            if !self.reserve(definition.recruit_cost, definition.upkeep_gold, emergency) {
                return None;
            }
            self.choose(
                Command::Recruit {
                    site,
                    army: army.map(|army| army.id),
                    kind,
                },
                self.objective.clone(),
            )
        })
    }

    pub(super) fn construct(&self) -> Option<AiDecision> {
        let headquarters = self.campaign.factions[&self.owner].headquarters;
        if self
            .view
            .world
            .site(headquarters)
            .is_none_or(|site| site.controller != Some(self.owner))
        {
            for site in self
                .view
                .world
                .sites
                .iter()
                .filter(|site| site.controller == Some(self.owner))
            {
                if let Some(decision) = self.choose(
                    Command::RelocateHeadquarters { site: site.id },
                    self.objective.clone(),
                ) {
                    return Some(decision);
                }
            }
        }
        for army in &self.view.armies {
            let site = self.view.world.site(army.site)?;
            if site.controller != Some(self.owner) || !self.view.supplied_sites.contains(&site.id) {
                continue;
            }
            for order in self.view.construction.iter().filter(|order| {
                matches!(
                    order.status,
                    crate::state::construction::ConstructionStatus::Paused {
                        reason: crate::state::construction::ConstructionPause::BuilderMissing
                            | crate::state::construction::ConstructionPause::BuilderAway
                    }
                ) && order.builder != Some(army.id)
            }) {
                if let Some(decision) = self.choose(
                    Command::ReassignBuilder {
                        order: order.id,
                        builder: army.id,
                    },
                    self.objective.clone(),
                ) {
                    return Some(decision);
                }
            }
            if let Some(decision) = self.site_work(army) {
                return Some(decision);
            }
            for route in self
                .view
                .world
                .routes
                .iter()
                .filter(|route| route.from == site.id || route.to == site.id)
            {
                let kind = if route.road.improved {
                    ConstructionKind::RoadRepair
                } else {
                    ConstructionKind::Road
                };
                if let Some(decision) =
                    self.build(ConstructionTarget::Route(route.id), kind, army.id)
                {
                    return Some(decision);
                }
            }
        }
        self.focus()
    }

    pub(super) fn develop_city(&self) -> Option<AiDecision> {
        let cost = self.data.development.city_development.cost;
        if !self.reserve(cost, 0, false) {
            return None;
        }
        self.view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller == Some(self.owner)
                    && site.habitation >= self.data.development.city_development.minimum_habitation
                    && site.habitation < Habitation::City
            })
            .find_map(|site| {
                self.choose(
                    Command::DevelopCity { site: site.id },
                    self.objective.clone(),
                )
            })
    }

    fn site_work(&self, army: &Army) -> Option<AiDecision> {
        let site = self.view.world.site(army.site)?;
        let target = ConstructionTarget::Site(site.id);
        if site.habitation <= Habitation::Camp {
            if let Some(decision) = self.build(target, ConstructionKind::Outpost, army.id) {
                return Some(decision);
            }
        }
        if (site.id == self.campaign.factions[&self.owner].headquarters
            || self.anchors().contains(&site.id))
            && site.military == MilitaryLayer::None
        {
            if let Some(decision) = self.build(target, ConstructionKind::Fort, army.id) {
                return Some(decision);
            }
        }
        for (kind, facility) in [
            (TroopKind::Medics, Facility::Infirmary),
            (TroopKind::Riders, Facility::Stable),
            (TroopKind::SiegeEngines, Facility::Workshop),
        ] {
            if !self
                .view
                .formations
                .iter()
                .any(|formation| formation.kind == kind)
                && !site.facilities.contains(&facility)
            {
                if let Some(decision) =
                    self.build(target, ConstructionKind::Facility(facility), army.id)
                {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn build(
        &self,
        target: ConstructionTarget,
        kind: ConstructionKind,
        builder: ArmyId,
    ) -> Option<AiDecision> {
        let (cost, _) = crate::engine::construction::terms(self.data, kind);
        if !self.reserve(cost, 0, false) {
            return None;
        }
        self.choose(
            Command::StartConstruction {
                target,
                kind,
                builder,
            },
            self.objective.clone(),
        )
    }

    fn focus(&self) -> Option<AiDecision> {
        for site in self.view.world.sites.iter().filter(|site| {
            site.controller == Some(self.owner) && !self.view.world.focus.contains_key(&site.id)
        }) {
            let focus = if site.geography == Geography::Forest
                || site.tags.contains(&SiteTag::WoodSource)
            {
                Focus::Wood
            } else if site.geography == Geography::Hill || site.tags.contains(&SiteTag::StoneSource)
            {
                Focus::Stone
            } else {
                Focus::Gold
            };
            if let Some(decision) = self.choose(
                Command::SetFocus {
                    site: site.id,
                    focus,
                },
                self.objective.clone(),
            ) {
                return Some(decision);
            }
        }
        None
    }
}
