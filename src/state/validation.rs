//! Candidate-save invariants checked before replacing a live campaign.

use super::campaign::{
    CampaignPhase, DomainFactKind, FactionStatus, StrategicCampaign, STRATEGIC_VERSION,
};
use crate::data::{world::FactionId, GameData};
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub fn validate(&self, data: &GameData) -> Result<(), String> {
        require(
            self.version == STRATEGIC_VERSION,
            "version",
            "unsupported strategic schema",
        )?;
        require(
            self.content_version == data.rules.content_version
                && self.content_version == data.scenario.content_version,
            "content_version",
            "unsupported content rules",
        )?;
        require(
            data.presentation
                .start_year
                .checked_add(self.completed_rounds / 4)
                .is_some(),
            "completed_rounds",
            "calendar overflows the supported year",
        )?;
        require(
            self.rng.states().iter().all(|state| *state != 0),
            "rng",
            "zero stream state",
        )?;
        self.validate_factions(data)?;
        self.validate_diplomacy(data)?;
        self.validate_ai(data)?;
        self.validate_world(data)?;
        self.validate_military(data)?;
        self.validate_construction(data)?;
        self.validate_development(data)?;
        self.validate_threats(data)?;
        self.validate_sieges()?;
        self.validate_battles(data)?;
        self.validate_evidence(data)?;
        self.validate_history()?;
        self.validate_knowledge()?;
        self.validate_phase()?;
        self.validate_facts()?;
        self.validate_counters()
    }

    fn validate_factions(&self, data: &GameData) -> Result<(), String> {
        require(
            (data.rules.min_factions..=data.rules.max_factions).contains(&self.factions.len()),
            "factions",
            "unsupported faction count",
        )?;
        require(
            self.is_independent(self.player)
                || self
                    .diplomacy
                    .ending
                    .as_ref()
                    .is_some_and(|ending| ending.kind == super::diplomacy::EndingKind::Defeat),
            "player",
            "player must be an independent faction",
        )?;
        for (id, faction) in &self.factions {
            require(
                *id == faction.id && id.0 > 0,
                "factions.id",
                "mismatched or zero identity",
            )?;
            require(
                data.rules.valid_kingdom_name(&faction.name),
                "factions.name",
                "invalid kingdom name",
            )?;
            faction
                .resources
                .validate("campaign", "factions.resources")?;
            require(
                self.world.site(faction.headquarters).is_some()
                    && self.world.site(faction.capital).is_some(),
                "factions.headquarters/capital",
                "unknown physical site",
            )?;
            if let FactionStatus::Vassal { sovereign } = faction.status {
                require(
                    sovereign != *id && self.is_independent(sovereign),
                    "factions.sovereign",
                    "invalid independent sovereign",
                )?;
            }
        }
        let mut pairs = BTreeSet::new();
        for relation in &self.relations {
            let [first, second] = relation.factions;
            require(
                first < second
                    && self.factions.contains_key(&first)
                    && self.factions.contains_key(&second)
                    && pairs.insert([first, second]),
                "relations",
                "unknown, unordered or duplicate pair",
            )?;
        }
        require(
            pairs.len() == self.factions.len() * (self.factions.len() - 1) / 2,
            "relations",
            "every faction pair needs one relation",
        )
    }

    fn validate_world(&self, data: &GameData) -> Result<(), String> {
        require(
            self.world.sites.len() == data.scenario.sites.len()
                && self.world.routes.len() == data.scenario.routes.len()
                && self.world.markers.len() == data.scenario.markers.len(),
            "world",
            "fixed topology counts changed",
        )?;
        require(
            self.world
                .sites
                .windows(2)
                .all(|pair| pair[0].id < pair[1].id)
                && self
                    .world
                    .routes
                    .windows(2)
                    .all(|pair| pair[0].id < pair[1].id)
                && self
                    .world
                    .markers
                    .windows(2)
                    .all(|pair| pair[0].id < pair[1].id),
            "world",
            "IDs must be unique and ordered",
        )?;
        for marker in &self.world.markers {
            let mut original = data
                .scenario
                .marker(marker.id)
                .cloned()
                .ok_or("campaign.world.markers: unknown marker")?;
            if matches!(
                marker.location,
                crate::data::world::MarkerLocation::Site { .. }
            ) {
                original.name = marker.name.clone();
            }
            require(
                original == *marker && !marker.name.trim().is_empty(),
                "world.markers",
                "fixed marker or entrance mapping changed",
            )?;
        }
        for site in &self.world.sites {
            let original = data
                .scenario
                .site(site.id)
                .ok_or("campaign.world.sites: unknown site")?;
            require(
                site.marker == original.marker
                    && site.position == original.position
                    && site.geography == original.geography
                    && site.key == original.key
                    && site.tags == original.tags,
                "world.sites",
                "fixed site geography changed",
            )?;
            require(
                !site.name.trim().is_empty(),
                "world.sites.name",
                "empty site name",
            )?;
            require(
                site.controller
                    .is_none_or(|controller| self.factions.contains_key(&controller)),
                "world.sites.controller",
                "unknown faction",
            )?;
            let facilities: BTreeSet<_> = site.facilities.iter().collect();
            require(
                facilities.len() == site.facilities.len(),
                "world.sites.facilities",
                "duplicate facility",
            )?;
        }
        for route in &self.world.routes {
            let original = data
                .scenario
                .route(route.id)
                .ok_or("campaign.world.routes: unknown route")?;
            require(
                route.from == original.from
                    && route.to == original.to
                    && route.major_connection == original.major_connection
                    && route.terrain_cost == original.terrain_cost
                    && route.road.damage <= 100
                    && (route.road.improved || route.road.damage == 0),
                "world.routes",
                "fixed route changed or invalid road damage",
            )?;
        }
        self.validate_region_control()
    }

    fn validate_phase(&self) -> Result<(), String> {
        let unique: BTreeSet<_> = self.round_order.iter().copied().collect();
        require(
            self.round_order.first() == Some(&self.player)
                && unique.len() == self.round_order.len()
                && self
                    .round_order
                    .iter()
                    .all(|id| self.factions.contains_key(id))
                && self.round_order[1..]
                    .windows(2)
                    .all(|pair| pair[0] < pair[1]),
            "round_order",
            "must snapshot player first and distinct NPC IDs in order",
        )?;
        require(
            self.independent_order()
                .iter()
                .all(|id| unique.contains(id))
                && self.acted.is_subset(&unique),
            "acted",
            "unknown acted faction or missing independent phase",
        )?;
        let next = self
            .round_order
            .iter()
            .copied()
            .find(|id| self.is_independent(*id) && !self.acted.contains(id));
        require(
            self.diplomacy.is_blocked() || next == Some(self.active_faction()),
            "phase",
            "active faction is not next in the round",
        )?;
        if let CampaignPhase::NpcTurn { faction, .. } = self.phase {
            require(
                faction != self.player,
                "phase",
                "player cannot have an NPC phase",
            )?;
        }
        Ok(())
    }

    fn validate_facts(&self) -> Result<(), String> {
        require(
            self.consumed_sequence <= self.accepted_sequence,
            "consumed_sequence",
            "consumption is ahead of accepted actions",
        )?;
        let mut ids = BTreeSet::new();
        let mut actors = BTreeSet::<FactionId>::new();
        let mut previous_sequence = self.consumed_sequence;
        let mut previous_id = 0;
        for fact in &self.pending_facts {
            require(
                fact.id.0 > 0
                    && fact.id < self.next_ids.fact
                    && ids.insert(fact.id)
                    && fact.sequence >= previous_sequence
                    && fact.sequence > self.consumed_sequence
                    && fact.id.0 > previous_id
                    && fact.sequence <= self.accepted_sequence
                    && fact.completed_rounds == self.completed_rounds,
                "pending_facts",
                "invalid identity, sequence or date",
            )?;
            if let DomainFactKind::FactionPassed { faction } = fact.kind {
                require(
                    self.acted.contains(&faction) && actors.insert(faction),
                    "pending_facts.faction",
                    "duplicate or unacted faction",
                )?;
            } else {
                self.validate_fact_subject(fact)?;
            }
            previous_sequence = fact.sequence;
            previous_id = fact.id.0;
        }
        require(
            actors == self.acted,
            "pending_facts",
            "each acted phase must have exactly one unconsumed fact",
        )
    }

    fn validate_fact_subject(&self, fact: &super::campaign::DomainFact) -> Result<(), String> {
        match &fact.kind {
            DomainFactKind::DiplomacyChanged { receipt } => {
                self.validate_diplomacy_receipt(receipt, fact.completed_rounds)?;
            }
            DomainFactKind::DevelopmentChanged { receipt } => {
                self.validate_development_receipt(receipt)?;
            }
            DomainFactKind::SiegeChanged { siege, change } => {
                self.validate_siege_receipt(siege, *change, fact.completed_rounds)?;
            }
            DomainFactKind::ConstructionChanged { order } => {
                self.validate_construction_receipt(order, fact.completed_rounds)?
            }
            DomainFactKind::FocusChanged { faction, site, .. } => require(
                self.factions.contains_key(faction) && self.world.site(*site).is_some(),
                "pending_facts.focus",
                "invalid owner or site",
            )?,
            DomainFactKind::BattleResolved { battle, .. } => require(
                self.battles.get(battle).is_some_and(|report| {
                    report.sequence == fact.sequence
                        && report.completed_rounds == fact.completed_rounds
                }),
                "pending_facts.battle",
                "missing or mismatched encounter receipt",
            )?,
            DomainFactKind::FactionPassed { .. } => {}
            _ => self.validate_military_fact_subject(&fact.kind)?,
        }
        Ok(())
    }

    fn validate_military_fact_subject(&self, kind: &DomainFactKind) -> Result<(), String> {
        match kind {
            DomainFactKind::FormationRecruited {
                faction,
                army,
                formation,
                site,
                ..
            }
            | DomainFactKind::FormationDisbanded {
                faction,
                army,
                formation,
                site,
                ..
            } => require(
                self.factions.contains_key(faction)
                    && self.world.site(*site).is_some()
                    && army.0 > 0
                    && *army < self.next_ids.army
                    && formation.0 > 0
                    && *formation < self.next_ids.formation,
                "pending_facts.formation",
                "invalid historical faction, army, formation or site reference",
            )?,
            DomainFactKind::ArmiesMoved {
                faction,
                armies,
                path,
                spent,
                ..
            } => require(
                self.factions.contains_key(faction)
                    && !armies.is_empty()
                    && armies.windows(2).all(|pair| pair[0] < pair[1])
                    && armies.iter().all(|id| id.0 > 0 && *id < self.next_ids.army)
                    && *spent > 0
                    && path.len() >= 2
                    && path
                        .windows(2)
                        .all(|pair| self.world.connected_route(pair[0], pair[1]).is_some()),
                "pending_facts.movement",
                "invalid historical group or physical route",
            )?,
            DomainFactKind::FormationTransferred {
                faction,
                formation,
                from_army,
                to_army,
                site,
            } => require(
                self.factions.contains_key(faction)
                    && self.world.site(*site).is_some()
                    && formation.0 > 0
                    && *formation < self.next_ids.formation
                    && [from_army, to_army]
                        .iter()
                        .all(|id| id.0 > 0 && **id < self.next_ids.army),
                "pending_facts.transfer",
                "invalid historical formation or army reference",
            )?,
            DomainFactKind::PersonTransferred {
                faction,
                person,
                to_formation,
                site,
            } => require(
                self.factions.contains_key(faction)
                    && self.world.site(*site).is_some()
                    && person.0 > 0
                    && *person < self.next_ids.person
                    && to_formation.0 > 0
                    && *to_formation < self.next_ids.formation,
                "pending_facts.transfer",
                "invalid historical person or formation reference",
            )?,
            _ => return Err("campaign.pending_facts: unexpected military fact".into()),
        }
        Ok(())
    }

    fn validate_counters(&self) -> Result<(), String> {
        for (field, counter, maximum) in [
            (
                "faction",
                self.next_ids.faction.0,
                self.factions.keys().map(|id| id.0).max(),
            ),
            (
                "marker",
                self.next_ids.marker.0,
                self.world.markers.iter().map(|m| m.id.0).max(),
            ),
            (
                "site",
                self.next_ids.site.0,
                self.world.sites.iter().map(|s| s.id.0).max(),
            ),
            (
                "route",
                self.next_ids.route.0,
                self.world.routes.iter().map(|r| r.id.0).max(),
            ),
            (
                "army",
                self.next_ids.army.0,
                self.armies.keys().map(|id| id.0).max(),
            ),
            (
                "formation",
                self.next_ids.formation.0,
                self.formations.keys().map(|id| id.0).max(),
            ),
            (
                "person",
                self.next_ids.person.0,
                self.people.keys().map(|id| id.0).max(),
            ),
        ] {
            require(
                counter > maximum.unwrap_or(0),
                &format!("next_ids.{field}"),
                "counter must exceed every existing ID",
            )?;
        }
        require(
            self.next_ids.fact.0 > 0,
            "next_ids.fact",
            "counter must be positive",
        )
    }
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.{field}: {reason}"))
    }
}
