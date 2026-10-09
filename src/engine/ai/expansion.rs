//! Weighing unclaimed land: distance from home, worth, and whether it joins supply.
use super::*;
use crate::data::world::{MilitaryLayer, Site, SiteTag};
use std::collections::BTreeSet;

/// Unclaimed land this realm would take, before choosing which army walks there.
pub(super) struct Claim {
    pub site: SiteId,
    /// Route cost from the capital.
    pub home: u32,
    /// Touches supplied territory, so the new holding stays joined to the capital.
    pub connected: bool,
    pub value: u32,
}

impl Planner<'_> {
    /// Authored worth of a site's settlement, resources and works.
    fn site_value(&self, site: &Site, anchors: &BTreeSet<SiteId>) -> u32 {
        let rules = &self.data.ai.site_value;
        let tags: u32 = site
            .tags
            .iter()
            .map(|tag| match tag {
                SiteTag::WoodSource => rules.wood_source,
                SiteTag::StoneSource => rules.stone_source,
                SiteTag::Bridge | SiteTag::Pass => rules.crossing,
                SiteTag::HorseAccess => rules.horse_access,
                SiteTag::Ruins => 0,
            })
            .sum();
        let facilities = u32::try_from(site.facilities.len()).unwrap_or(u32::MAX);
        rules
            .habitation
            .get(&site.habitation)
            .copied()
            .unwrap_or(0)
            .saturating_add(tags)
            .saturating_add(rules.facility.saturating_mul(facilities))
            .saturating_add(if site.military == MilitaryLayer::Fort {
                rules.fort
            } else {
                0
            })
            .saturating_add(if anchors.contains(&site.id) {
                rules.region_anchor
            } else {
                0
            })
    }

    /// Unclaimed land within the realm's reach. Land joined to supplied
    /// territory is always a candidate; a detached holding invites invasion,
    /// so it is considered only when worth the temperament's threshold.
    /// A realm without a comfortable income surplus over upkeep reaches further.
    pub(super) fn claims(&self) -> Vec<Claim> {
        let faction = &self.campaign.factions[&self.owner];
        let needs_land = faction.deficit
            || faction.last_economy.as_ref().is_some_and(|statement| {
                i128::from(statement.income.gold) * 100
                    < i128::from(statement.upkeep_due) * i128::from(self.data.ai.surplus_percent)
            });
        let temperament = self.temperament();
        let anchors = self.anchors();
        self.view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller.is_none() && !self.view.hostile_presence.contains(&site.id)
            })
            .filter_map(|site| {
                let connected = self
                    .view
                    .world
                    .adjacent_sites(site.id)
                    .into_iter()
                    .any(|neighbor| self.view.supplied_sites.contains(&neighbor));
                let value = self.site_value(site, &anchors);
                if !connected && value < temperament.detached_claim_value {
                    return None;
                }
                let (home, path) = self.path(faction.headquarters, site.id, false)?;
                (needs_land || path.len().saturating_sub(1) <= temperament.expansion_reach)
                    .then_some(Claim {
                        site: site.id,
                        home,
                        connected,
                        value,
                    })
            })
            .collect()
    }

    pub(super) fn expand(&self) -> Option<AiDecision> {
        let rules = &self.data.ai;
        let home_weight = i64::from(self.temperament().home_distance_percent);
        let mut candidates: Vec<_> = self
            .claims()
            .into_iter()
            .filter_map(|claim| {
                let travel = self
                    .view
                    .armies
                    .iter()
                    .filter(|army| self.moving(army, false))
                    .filter_map(|army| {
                        self.path(army.site, claim.site, false)
                            .map(|(cost, _)| cost)
                    })
                    .min()?;
                // Distance from the capital outweighs the nearest army's walk,
                // so the realm fills in around home; worth pulls it toward
                // settlements and resources, and a detached claim must pay
                // for leaving the supply network.
                let detached = if claim.connected {
                    0
                } else {
                    i64::from(rules.detached_cost)
                };
                let score =
                    i64::from(claim.home) * home_weight / 100 + i64::from(travel) + detached
                        - i64::from(claim.value) * i64::from(rules.value_cost);
                Some((score, self.strategic_priority(claim.site), claim.site))
            })
            .collect();
        candidates.sort_unstable();
        candidates
            .into_iter()
            .find_map(|(_, _, site)| self.toward(site, AiObjectiveKind::Expand, false))
    }
}
