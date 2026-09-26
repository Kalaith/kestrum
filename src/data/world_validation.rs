//! Referential integrity, reciprocal boundaries, and feasible region anchors.

use super::economy::Economy;
use super::rules::CampaignRules;
use super::validation::{require, unique};
use super::world::*;
use std::collections::BTreeSet;

impl Scenario {
    pub fn validate(&self, rules: &CampaignRules, economy: &Economy) -> Result<(), String> {
        require(
            SOURCE,
            "schema_version",
            self.schema_version == 1,
            "unsupported version",
        )?;
        require(
            SOURCE,
            "content_version",
            self.content_version == rules.content_version,
            "incompatible campaign rules",
        )?;
        require(
            SOURCE,
            "name",
            !self.name.trim().is_empty(),
            "must not be empty",
        )?;
        require(
            SOURCE,
            "difficulty",
            self.difficulty == rules.difficulty,
            "unsupported difficulty",
        )?;
        // This named fixture is distinct from K17's 80-marker production layout.
        require(
            SOURCE,
            "markers",
            self.markers.len() == 5,
            "Rosemarch requires five major markers",
        )?;
        require(
            SOURCE,
            "sites",
            self.sites.len() == 14,
            "Rosemarch requires fourteen physical sites",
        )?;
        self.validate_identities()?;
        self.validate_sites(economy)?;
        self.validate_routes()?;
        for marker in &self.markers {
            self.validate_marker(marker)?;
        }
        self.validate_setup(rules, economy)
    }

    fn validate_identities(&self) -> Result<(), String> {
        unique(SOURCE, "sites.id", self.sites.iter().map(|site| site.id))?;
        unique(
            SOURCE,
            "markers.id",
            self.markers.iter().map(|marker| marker.id),
        )?;
        unique(
            SOURCE,
            "routes.id",
            self.routes.iter().map(|route| route.id),
        )?;
        unique(
            SOURCE,
            "factions.id",
            self.factions.iter().map(|faction| faction.id),
        )?;
        unique(SOURCE, "sites.key", self.sites.iter().map(|site| &site.key))?;
        unique(
            SOURCE,
            "markers.key",
            self.markers.iter().map(|marker| &marker.key),
        )?;
        for (field, valid) in [
            ("sites.id", self.sites.iter().all(|site| site.id.0 > 0)),
            (
                "markers.id",
                self.markers.iter().all(|marker| marker.id.0 > 0),
            ),
            ("routes.id", self.routes.iter().all(|route| route.id.0 > 0)),
            (
                "factions.id",
                self.factions.iter().all(|faction| faction.id.0 > 0),
            ),
        ] {
            require(SOURCE, field, valid, "IDs must be positive")?;
        }
        Ok(())
    }

    fn validate_sites(&self, economy: &Economy) -> Result<(), String> {
        for site in &self.sites {
            let field = format!("sites[{}]", site.id.0);
            validate_label_position(&field, &site.key, &site.name, site.position)?;
            require(
                SOURCE,
                &format!("{field}.habitation"),
                economy.settlement_income.contains_key(&site.habitation),
                "missing economy tier",
            )?;
            require(
                SOURCE,
                &format!("{field}.controller"),
                site.controller
                    .is_none_or(|id| self.factions.iter().any(|faction| faction.id == id)),
                "unknown faction",
            )?;
            unique(SOURCE, &format!("{field}.tags"), &site.tags)?;
            unique(SOURCE, &format!("{field}.facilities"), &site.facilities)?;
            let marker = self
                .marker(site.marker)
                .ok_or_else(|| format!("{SOURCE}: {field}.marker: missing {:?}", site.marker))?;
            let contained = match &marker.location {
                MarkerLocation::Site { site: endpoint } => *endpoint == site.id,
                MarkerLocation::Region { sites, .. } => sites.contains(&site.id),
            };
            require(
                SOURCE,
                &format!("{field}.marker"),
                contained,
                "marker must refer back to this physical site",
            )?;
        }
        Ok(())
    }

    fn validate_routes(&self) -> Result<(), String> {
        let mut edges = BTreeSet::new();
        for route in &self.routes {
            let field = format!("routes[{}]", route.id.0);
            let from = self.site(route.from).ok_or_else(|| {
                format!("{SOURCE}: {field}.from: missing endpoint {:?}", route.from)
            })?;
            let to = self
                .site(route.to)
                .ok_or_else(|| format!("{SOURCE}: {field}.to: missing endpoint {:?}", route.to))?;
            require(
                SOURCE,
                &field,
                from.id != to.id,
                "self-routes are not traversable connections",
            )?;
            require(
                SOURCE,
                &field,
                edges.insert((from.id.min(to.id), from.id.max(to.id))),
                "duplicate undirected endpoints",
            )?;
            let expected = (from.marker != to.marker).then_some([from.marker, to.marker]);
            require(
                SOURCE,
                &format!("{field}.major_connection"),
                route.major_connection == expected,
                "must match physical endpoint parents in order",
            )?;
            require(
                SOURCE,
                &format!("{field}.terrain_cost"),
                (2..=4).contains(&route.terrain_cost),
                "supported P04 base costs are 2..=4",
            )?;
            require(
                SOURCE,
                &format!("{field}.road.damage"),
                route.road.damage <= 100,
                "must be within 0..=100",
            )?;
            require(
                SOURCE,
                &format!("{field}.road"),
                route.road.improved || route.road.damage == 0,
                "an unimproved route cannot have road damage",
            )?;
        }
        Ok(())
    }

    fn validate_marker(&self, marker: &MajorMarker) -> Result<(), String> {
        let field = format!("markers[{}]", marker.id.0);
        validate_label_position(&field, &marker.key, &marker.name, marker.position)?;
        match &marker.location {
            MarkerLocation::Site { site } => {
                require(
                    SOURCE,
                    &format!("{field}.site"),
                    self.site(*site).is_some_and(|physical| {
                        physical.marker == marker.id && physical.position == marker.position
                    }),
                    "single-site marker must resolve to its physical site and position",
                )?;
            }
            MarkerLocation::Region {
                sites,
                entrances,
                anchors,
            } => {
                require(
                    SOURCE,
                    &format!("{field}.sites"),
                    sites.len() == 10,
                    "Rosemarch requires ten internal sites",
                )?;
                unique(SOURCE, &format!("{field}.sites"), sites)?;
                for id in sites {
                    require(
                        SOURCE,
                        &format!("{field}.sites"),
                        self.site(*id).is_some_and(|site| site.marker == marker.id),
                        "internal site must resolve and refer back to region",
                    )?;
                }
                self.validate_entrances(marker.id, entrances)?;
                validate_anchor(
                    anchors,
                    &AnchorScope {
                        field: &field,
                        sites,
                        entrances,
                    },
                )?;
                self.validate_region_connectivity(marker.id, sites)?;
                for faction in &self.factions {
                    let reachable = self.reachable_sites(faction.headquarters);
                    require(
                        SOURCE,
                        &format!("{field}.anchors"),
                        anchors.is_satisfied(&reachable, &reachable),
                        "anchor expression is not satisfiable from every HQ",
                    )?;
                }
            }
        }
        Ok(())
    }

    fn validate_entrances(&self, region: MarkerId, entrances: &[Entrance]) -> Result<(), String> {
        let field = format!("markers[{}].entrances", region.0);
        unique(SOURCE, &field, entrances.iter().map(|gate| gate.route))?;
        let gates: BTreeSet<_> = entrances.iter().map(|gate| gate.site).collect();
        require(
            SOURCE,
            &field,
            gates.len() >= 2,
            "a region needs at least two distinct entrances",
        )?;
        for gate in entrances {
            let route = self
                .route(gate.route)
                .ok_or_else(|| format!("{SOURCE}: {field}: missing route {:?}", gate.route))?;
            let outside = route.other_endpoint(gate.site).and_then(|id| self.site(id));
            require(
                SOURCE,
                &field,
                self.site(gate.site)
                    .is_some_and(|site| site.marker == region)
                    && outside.is_some_and(|site| site.marker != region),
                "entrance must map a boundary route to its internal endpoint",
            )?;
        }
        for route in &self.routes {
            if route
                .major_connection
                .is_some_and(|markers| markers.contains(&region))
            {
                let internal = [route.from, route.to]
                    .into_iter()
                    .find(|id| self.site(*id).is_some_and(|site| site.marker == region));
                require(
                    SOURCE,
                    &field,
                    entrances
                        .iter()
                        .any(|gate| gate.route == route.id && Some(gate.site) == internal),
                    "external route is missing its reverse entrance mapping",
                )?;
            }
        }
        Ok(())
    }

    fn validate_region_connectivity(
        &self,
        region: MarkerId,
        sites: &[SiteId],
    ) -> Result<(), String> {
        let mut visited = BTreeSet::new();
        let mut pending = sites.first().copied().into_iter().collect::<Vec<_>>();
        while let Some(site) = pending.pop() {
            if visited.insert(site) {
                pending.extend(
                    self.routes
                        .iter()
                        .filter(|route| route.major_connection.is_none())
                        .filter_map(|route| route.other_endpoint(site))
                        .filter(|id| sites.contains(id)),
                );
            }
        }
        require(
            SOURCE,
            &format!("markers[{}].sites", region.0),
            visited.len() == sites.len(),
            "internal graph must be connected without external shortcuts",
        )
    }
}

fn validate_label_position(
    field: &str,
    key: &str,
    name: &str,
    position: [f32; 2],
) -> Result<(), String> {
    require(
        SOURCE,
        &format!("{field}.key"),
        !key.is_empty() && key.trim() == key,
        "must be nonempty and trimmed",
    )?;
    require(
        SOURCE,
        &format!("{field}.name"),
        !name.trim().is_empty(),
        "must not be empty",
    )?;
    require(
        SOURCE,
        &format!("{field}.position"),
        position
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value)),
        "must contain finite normalized coordinates",
    )
}

struct AnchorScope<'a> {
    field: &'a str,
    sites: &'a [SiteId],
    entrances: &'a [Entrance],
}

fn validate_anchor(anchor: &AnchorExpression, scope: &AnchorScope<'_>) -> Result<(), String> {
    let field = format!("{}.anchors", scope.field);
    match anchor {
        AnchorExpression::All { conditions } | AnchorExpression::Any { conditions } => {
            require(
                SOURCE,
                &field,
                !conditions.is_empty(),
                "all/any conditions must not be empty",
            )?;
            for condition in conditions {
                validate_anchor(condition, scope)?;
            }
        }
        AnchorExpression::ControlledSite { site } | AnchorExpression::SuppliedEntrance { site } => {
            require(
                SOURCE,
                &field,
                scope.sites.contains(site),
                "anchor site must resolve inside its region",
            )?;
            if matches!(anchor, AnchorExpression::SuppliedEntrance { .. }) {
                require(
                    SOURCE,
                    &field,
                    scope.entrances.iter().any(|gate| gate.site == *site),
                    "supplied entrance must be a mapped gate",
                )?;
            }
        }
    }
    Ok(())
}
