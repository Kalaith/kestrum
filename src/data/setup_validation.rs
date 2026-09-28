//! Validate authored faction grants without instantiating simulation state.

use super::economy::{Economy, Habitation, TroopKind};
use super::rules::CampaignRules;
use super::validation::{require, unique};
use super::world::*;

impl Scenario {
    pub(super) fn validate_setup(
        &self,
        rules: &CampaignRules,
        economy: &Economy,
    ) -> Result<(), String> {
        let valid_count = if self.kind == ScenarioKind::RosemarchPrototype {
            self.factions.len() == rules.default_factions
        } else {
            (rules.min_factions..=rules.max_factions).contains(&self.factions.len())
        };
        require(
            SOURCE,
            "factions",
            valid_count,
            "faction count is outside the supported setup",
        )?;
        require(
            SOURCE,
            "player",
            self.factions
                .iter()
                .any(|faction| faction.id == self.player),
            "must resolve to a faction",
        )?;
        unique(
            SOURCE,
            "factions.headquarters",
            self.factions.iter().map(|faction| faction.headquarters),
        )?;
        for faction in &self.factions {
            self.validate_faction(faction, rules)?;
            faction
                .resources
                .resolve(economy)
                .validate(SOURCE, &format!("factions[{}].resources", faction.id.0))?;
            for troop in &faction.starting_formations {
                require(
                    SOURCE,
                    &format!("factions[{}].starting_formations", faction.id.0),
                    economy.formations.contains_key(troop),
                    "missing economy formation definition",
                )?;
            }
            require(
                SOURCE,
                &format!("factions[{}].headquarters", faction.id.0),
                self.reachable_sites(faction.headquarters).len() == self.sites.len(),
                "HQ must reach every physical site",
            )?;
        }
        self.validate_relations()
    }

    fn validate_faction(
        &self,
        faction: &FactionSetup,
        rules: &CampaignRules,
    ) -> Result<(), String> {
        let field = format!("factions[{}]", faction.id.0);
        require(
            SOURCE,
            &format!("{field}.name"),
            rules.valid_kingdom_name(&faction.name),
            "must be trimmed, nonempty and at most 32 Unicode characters",
        )?;
        for (key, name) in [
            ("army_name", &faction.army_name),
            ("founder.name", &faction.founder.name),
        ] {
            require(
                SOURCE,
                &format!("{field}.{key}"),
                !name.is_empty()
                    && name.trim() == name
                    && name.chars().count() <= 64
                    && !name.chars().any(char::is_control),
                "must be trimmed, nonempty and at most 64 characters without controls",
            )?;
        }
        require(
            SOURCE,
            &format!("{field}.emblem"),
            rules
                .emblems
                .iter()
                .any(|emblem| emblem.id == faction.emblem),
            "emblem must resolve to an authored symbol",
        )?;
        let hq = self
            .site(faction.headquarters)
            .ok_or_else(|| format!("{SOURCE}: {field}.headquarters: missing physical site"))?;
        require(
            SOURCE,
            &format!("{field}.headquarters"),
            self.marker(hq.marker)
                .is_some_and(|marker| match &marker.location {
                    MarkerLocation::Site { site } => *site == hq.id,
                    MarkerLocation::Region { sites, .. } => sites.contains(&hq.id),
                }),
            "HQ must be a physical site belonging to its world marker",
        )?;
        require(
            SOURCE,
            &format!("{field}.headquarters"),
            hq.habitation == Habitation::Village
                && hq.controller == Some(faction.id)
                && hq.facilities.contains(&Facility::TrainingGround)
                && hq.tags.contains(&SiteTag::HorseAccess),
            "HQ must be a controlled Village with training ground and horse access",
        )?;
        require(
            SOURCE,
            &format!("{field}.capital"),
            faction.capital == faction.headquarters,
            "founding capital must be the HQ",
        )?;
        require(
            SOURCE,
            &format!("{field}.starting_formations"),
            faction.starting_formations
                == [TroopKind::Warriors, TroopKind::Spearmen, TroopKind::Archers],
            "P01 grants full Warriors, Spearmen and Archers in that order",
        )?;
        require(
            SOURCE,
            &format!("{field}.founder"),
            faction.founder.age_years == 24
                && faction.founder.class == PersonClass::Officer
                && faction.founder.attached_to == TroopKind::Warriors
                && faction.founder.commander,
            "P01 grants an age-24 Officer attached to Warriors and appointed commander",
        )
    }

    fn validate_relations(&self) -> Result<(), String> {
        let expected = self.factions.len() * (self.factions.len() - 1) / 2;
        require(
            SOURCE,
            "relations",
            self.relations.len() == expected,
            "each faction pair needs one symmetric relation",
        )?;
        unique(
            SOURCE,
            "relations.factions",
            self.relations.iter().map(|relation| {
                (
                    relation.factions[0].min(relation.factions[1]),
                    relation.factions[0].max(relation.factions[1]),
                )
            }),
        )?;
        for relation in &self.relations {
            require(
                SOURCE,
                "relations.factions",
                relation.factions[0] != relation.factions[1]
                    && relation
                        .factions
                        .iter()
                        .all(|id| self.factions.iter().any(|faction| faction.id == *id)),
                "relations must name two distinct existing factions",
            )?;
        }
        Ok(())
    }
}
