//! Resource-aware recruitment targets and local production choices.

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
            .filter(|formation| formation.headcount > 0)
            .map(|formation| self.data.economy.formations[&formation.kind].upkeep_gold)
            .fold(0_i64, i64::saturating_add)
    }

    fn minimum_complement(&self) -> (Resources, i64) {
        self.data
            .ai
            .recruitment_order
            .iter()
            .take(self.data.ai.minimum_formations)
            .fold(
                (
                    Resources {
                        gold: 0,
                        wood: 0,
                        stone: 0,
                    },
                    0_i64,
                ),
                |(mut cost, upkeep), kind| {
                    let definition = &self.data.economy.formations[kind];
                    cost.gold = cost.gold.saturating_add(definition.recruit_cost.gold);
                    cost.wood = cost.wood.saturating_add(definition.recruit_cost.wood);
                    cost.stone = cost.stone.saturating_add(definition.recruit_cost.stone);
                    (cost, upkeep.saturating_add(definition.upkeep_gold))
                },
            )
    }

    fn growth_supported_by_income(&self) -> usize {
        let faction = &self.campaign.factions[&self.owner];
        if faction.deficit {
            return 0;
        }
        let Some(income) = faction
            .last_economy
            .as_ref()
            .filter(|income| income.completed_rounds == self.campaign.completed_rounds)
        else {
            return 0;
        };
        let (_, added_upkeep) = self.minimum_complement();
        let guarded_added_upkeep =
            added_upkeep.saturating_mul(i64::from(self.data.ai.reserve_upkeep_rounds));
        if guarded_added_upkeep == 0 {
            return 0;
        }
        let sustainable_income = income.income.gold.saturating_sub(self.upkeep()).max(0);
        usize::try_from(sustainable_income / guarded_added_upkeep).unwrap_or(0)
    }

    /// The maximum army count justified by developed land, supply needs and income.
    pub(super) fn recruitment_target(&self) -> usize {
        let rules = &self.data.ai;
        let base = rules.target_armies.min(rules.maximum_armies);
        let developed_sites = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller == Some(self.owner)
                    && site.habitation >= Habitation::Outpost
                    && !self.campaign.site_is_ruined(site.id)
            })
            .count();
        let holdings_target = developed_sites
            .saturating_add(rules.sites_per_army.saturating_sub(1))
            .checked_div(rules.sites_per_army)
            .unwrap_or(0)
            .max(base)
            .min(rules.maximum_armies);

        let armies = &self.view.armies;
        let first_base_full = armies
            .iter()
            .take(base)
            .all(|army| army.formation_ids().count() == 6);
        let threatened_fronts = if first_base_full {
            self.anchors()
                .iter()
                .filter(|site| {
                    self.threatened(**site)
                        && self
                            .view
                            .world
                            .site(**site)
                            .is_some_and(|site| site.controller == Some(self.owner))
                })
                .count()
                .min(rules.maximum_armies)
        } else {
            base
        };
        let supplied_core = self
            .view
            .world
            .sites
            .iter()
            .any(|site| self.can_replenish_at(site.id));
        // Each stranded army leaves its slot open for one supplied relief force,
        // on top of whatever target the realm would otherwise hold.
        let relief = if supplied_core {
            armies
                .iter()
                .filter(|army| !self.view.supplied_armies.contains(&army.id))
                .count()
        } else {
            0
        };
        // An outmatched sovereign adds one force at a time toward parity.
        let parity_target = if self.outmatched() {
            armies.len().saturating_add(1)
        } else {
            base
        };
        let desired = holdings_target
            .max(threatened_fronts)
            .max(parity_target)
            .saturating_add(relief)
            .saturating_add(self.temperament().extra_armies)
            .min(rules.maximum_armies);
        let supported = base
            .saturating_add(self.growth_supported_by_income())
            .min(rules.maximum_armies);
        desired.min(supported).max(base)
    }

    fn can_replenish_at(&self, site_id: SiteId) -> bool {
        let Some(site) = self.view.world.site(site_id) else {
            return false;
        };
        site.controller == Some(self.owner)
            && site.habitation >= Habitation::Outpost
            && !self.campaign.site_is_ruined(site_id)
            && !self.view.world.contested_sites.contains(&site_id)
            && self.view.supplied_sites.contains(&site_id)
            && !self
                .view
                .threats
                .iter()
                .any(|threat| threat.site == site_id)
    }

    fn can_reserve_minimum_army(&self, emergency: bool) -> bool {
        if emergency && self.view.armies.is_empty() {
            return true;
        }
        let (complement_cost, complement_upkeep) = self.minimum_complement();
        let reserve = self
            .upkeep()
            .saturating_add(complement_upkeep)
            .saturating_mul(i64::from(self.data.ai.reserve_upkeep_rounds));
        let available = self.campaign.factions[&self.owner].resources;
        available.gold.saturating_sub(complement_cost.gold) >= reserve
            && available.wood >= complement_cost.wood
            && available.stone >= complement_cost.stone
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
        let established = armies
            .iter()
            .filter(|army| self.can_replenish_at(army.site))
            .all(|army| army.formation_ids().count() >= self.data.ai.minimum_formations);
        let target = self.recruitment_target();
        if armies.len() < target && established && self.can_reserve_minimum_army(emergency) {
            let mut sites: Vec<_> = self
                .view
                .world
                .sites
                .iter()
                .filter(|site| self.can_replenish_at(site.id))
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
            .filter(|army| self.can_replenish_at(army.site))
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
        let cost = crate::engine::development::city_cost(self.campaign, self.data, self.owner);
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
