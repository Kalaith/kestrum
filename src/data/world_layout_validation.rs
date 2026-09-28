//! Production map invariants are checked before a seed can instantiate a campaign.

use super::{economy::Economy, rules::CampaignRules, validation::require, world::*};
use std::collections::{BTreeSet, VecDeque};
mod atlas;

impl WorldLayout {
    pub fn validate(&self, rules: &CampaignRules, economy: &Economy) -> Result<(), String> {
        require(
            LAYOUT_SOURCE,
            "schema_version",
            self.schema_version == 1,
            "unsupported version",
        )?;
        require(
            LAYOUT_SOURCE,
            "content_version",
            self.content_version == rules.content_version,
            "incompatible campaign rules",
        )?;
        let authored = Scenario {
            schema_version: self.schema_version,
            content_version: self.content_version,
            kind: ScenarioKind::Production,
            name: "Production layout".into(),
            seed: 1,
            difficulty: rules.difficulty,
            player: FactionId(1),
            markers: self.markers.clone(),
            sites: self.sites.clone(),
            routes: self.routes.clone(),
            factions: Vec::new(),
            relations: Vec::new(),
        };
        authored.validate_geography(economy)?;
        self.validate_atlas()?;
        self.validate_counts()?;
        self.validate_connected_graph()?;
        self.validate_headquarters_candidates()?;
        self.validate_initial_content_coverage()
    }

    fn validate_counts(&self) -> Result<(), String> {
        let regions = self
            .markers
            .iter()
            .filter(|marker| matches!(marker.location, MarkerLocation::Region { .. }))
            .count();
        require(
            LAYOUT_SOURCE,
            "markers",
            self.markers.len() == 80 && regions == 8,
            "requires 80 major locations, including exactly eight regions",
        )?;
        require(
            LAYOUT_SOURCE,
            "sites",
            self.sites.len() == 152 && self.markers.len() - regions == 72,
            "requires 152 physical sites: 80 regional and 72 single-site locations",
        )?;
        require(
            LAYOUT_SOURCE,
            "routes",
            !self.routes.is_empty(),
            "must include traversable land routes",
        )
    }

    fn validate_connected_graph(&self) -> Result<(), String> {
        let start = self.sites.first().map(|site| site.id);
        let reached = start
            .map(|site| self.reachable_sites(site).len())
            .unwrap_or(0);
        require(
            LAYOUT_SOURCE,
            "routes",
            reached == self.sites.len(),
            "every physical site and major location must be reachable",
        )
    }

    fn validate_headquarters_candidates(&self) -> Result<(), String> {
        require(
            LAYOUT_SOURCE,
            "headquarters_candidates",
            self.headquarters_candidates.len() == 8,
            "exactly eight reserved starts are required",
        )?;
        let unique: BTreeSet<_> = self.headquarters_candidates.iter().copied().collect();
        require(
            LAYOUT_SOURCE,
            "headquarters_candidates",
            unique.len() == self.headquarters_candidates.len(),
            "candidate starts must be unique",
        )?;
        for candidate in &self.headquarters_candidates {
            let site = self.site(*candidate).ok_or_else(|| {
                format!("{LAYOUT_SOURCE}: unknown headquarters candidate {candidate:?}")
            })?;
            let marker = self.marker(site.marker);
            require(
                LAYOUT_SOURCE,
                "headquarters_candidates",
                matches!(marker.map(|marker| &marker.location), Some(MarkerLocation::Site { site: single }) if single == candidate)
                    && site.habitation == crate::data::economy::Habitation::Village
                    && site.controller.is_none()
                    && site.tags.contains(&SiteTag::HorseAccess),
                "each reserved start must be a neutral Village single site with horse access",
            )?;
            let neighbors: BTreeSet<_> = self
                .routes
                .iter()
                .filter_map(|route| route.other_endpoint(*candidate))
                .collect();
            require(
                LAYOUT_SOURCE,
                "headquarters_candidates",
                neighbors.len() >= 2
                    && neighbors.iter().all(|id| {
                        self.site(*id)
                            .is_some_and(|neighbor| neighbor.controller.is_none())
                    }),
                "each reserved start needs two distinct neutral neighboring threat sites",
            )?;
            for resource in [SiteTag::WoodSource, SiteTag::StoneSource] {
                require(
                    LAYOUT_SOURCE,
                    "headquarters_candidates",
                    self.sites.iter().any(|site| {
                        site.tags.contains(&resource)
                            && self
                                .distance(*candidate, site.id)
                                .is_some_and(|edges| edges <= 3)
                    }),
                    "each start needs accessible wood and stone within three physical routes",
                )?;
            }
        }
        for (index, left) in self.headquarters_candidates.iter().enumerate() {
            for right in self.headquarters_candidates.iter().skip(index + 1) {
                require(
                    LAYOUT_SOURCE,
                    "headquarters_candidates",
                    self.distance(*left, *right).is_some_and(|edges| edges >= 4),
                    "reserved starts must be separated by at least four physical routes",
                )?;
            }
        }
        Ok(())
    }

    fn validate_initial_content_coverage(&self) -> Result<(), String> {
        let geographies: BTreeSet<_> = self.sites.iter().map(|site| site.geography).collect();
        require(
            LAYOUT_SOURCE,
            "sites.geography",
            [Geography::Valley, Geography::Coast, Geography::Pass, Geography::Plains,
                Geography::Forest, Geography::Hill, Geography::River, Geography::Marsh]
                .iter()
                .all(|geography| geographies.contains(geography)),
            "ordinary land layout must cover valley, coast, pass, plains, forest, hill, river and marsh terrain",
        )
    }

    fn distance(&self, start: SiteId, target: SiteId) -> Option<usize> {
        let mut seen = BTreeSet::from([start]);
        let mut pending = VecDeque::from([(start, 0usize)]);
        while let Some((site, distance)) = pending.pop_front() {
            if site == target {
                return Some(distance);
            }
            for next in self
                .routes
                .iter()
                .filter_map(|route| route.other_endpoint(site))
            {
                if seen.insert(next) {
                    pending.push_back((next, distance + 1));
                }
            }
        }
        None
    }
}
