//! Retained narratives carry their own labels; expired detail is a weak reference.

use super::*;
use crate::state::{campaign::DomainFactKind, StrategicCampaign};

impl StrategicCampaign {
    pub(crate) fn validate_history(&self) -> Result<(), String> {
        ensure(self.next_ids.history.0 > 0, "zero next history identity")?;
        let mut sources = BTreeSet::new();
        let mut battles = BTreeSet::new();
        for (id, record) in &self.history.events {
            ensure(*id == record.id, "mismatched history identity")?;
            self.validate_history_record(record)?;
            if let Some(source) = record.source_fact {
                ensure(sources.insert(source), "duplicate narrative source fact")?;
            }
            if let HistoryKind::Battle { battle, .. } = record.kind {
                ensure(battles.insert(battle), "duplicate battle narrative")?;
            }
        }
        ensure(
            self.battles.keys().all(|id| battles.contains(id)),
            "battle has no retained narrative",
        )?;
        for (id, summaries) in &self.history.site_notables {
            ensure(self.world.site(*id).is_some(), "unknown notable site")?;
            self.validate_notables(HistorySubject::Site(*id), summaries)?;
        }
        for (id, summaries) in &self.history.army_notables {
            ensure(id.0 > 0 && *id < self.next_ids.army, "invalid notable army")?;
            self.validate_notables(HistorySubject::Army(*id), summaries)?;
        }
        for (id, summaries) in &self.history.person_notables {
            ensure(
                id.0 > 0 && *id < self.next_ids.person,
                "invalid notable person",
            )?;
            self.validate_notables(HistorySubject::Person(*id), summaries)?;
        }
        Ok(())
    }

    fn validate_history_record(&self, record: &HistoryRecord) -> Result<(), String> {
        self.validate_history_header(record.id, record.completed_rounds, &record.visible_to)?;
        ensure(
            record
                .source_fact
                .is_none_or(|id| id.0 > 0 && id < self.next_ids.fact),
            "invalid source fact identity",
        )?;
        self.validate_history_labels(record)?;
        self.validate_history_kind(&record.kind, record.completed_rounds)?;
        ensure(
            valid_record_shape(record),
            "invalid participants for narrative kind",
        )?;
        match record.kind {
            HistoryKind::Diplomacy { ref receipt } => {
                self.validate_diplomacy_record(record, receipt)?
            }
            HistoryKind::Development { ref receipt } => {
                self.validate_development_record(record, receipt)?
            }
            HistoryKind::Battle { battle, .. } => {
                let report = self
                    .battles
                    .get(&battle)
                    .ok_or("history: retained battle detail is missing")?;
                ensure(
                    *record == HistoryRecord::battle(record.id, report, record.source_fact),
                    "battle narrative contradicts its witnessed receipt",
                )?;
            }
            HistoryKind::VeterancyEarned { formation, .. } => {
                ensure(
                    record.source_fact.is_none()
                        && record.visible_to.len() == 1
                        && record.formations.len() == 1
                        && record.formations[0].id == formation
                        && record.armies.len() == 1
                        && record.sites.len() == 1,
                    "invalid earned veterancy narrative",
                )?;
            }
            HistoryKind::Construction { ref order } => {
                self.validate_construction_record(record, order)?;
            }
            HistoryKind::Siege {
                defender, besieger, ..
            } => ensure(
                record.source_fact.is_some()
                    && record.visible_to == BTreeSet::from([defender, besieger]),
                "siege narrative needs its two participating factions",
            )?,
            _ => ensure(
                record.source_fact.is_some() && record.visible_to.len() == 1,
                "private action needs one owner and one source",
            )?,
        }
        if let Some(fact) = record
            .source_fact
            .and_then(|id| self.pending_facts.iter().find(|fact| fact.id == id))
        {
            ensure(
                record.completed_rounds == fact.completed_rounds
                    && source_matches(record, &fact.kind),
                "narrative contradicts its pending fact",
            )?;
        }
        Ok(())
    }

    fn validate_construction_record(
        &self,
        record: &HistoryRecord,
        order: &super::super::construction::ConstructionOrder,
    ) -> Result<(), String> {
        use super::super::construction::ConstructionTarget;
        let expected = match order.target {
            ConstructionTarget::Site(site) => BTreeSet::from([site]),
            ConstructionTarget::Route(route) => {
                let edge = self
                    .world
                    .route(route)
                    .ok_or("history: unknown construction route")?;
                BTreeSet::from([edge.from, edge.to])
            }
        };
        ensure(
            record.source_fact.is_some()
                && record.visible_to == BTreeSet::from([order.owner])
                && record
                    .sites
                    .iter()
                    .map(|entry| entry.id)
                    .collect::<BTreeSet<_>>()
                    == expected,
            "construction narrative contradicts its location or owner",
        )
    }

    fn validate_history_labels(&self, record: &HistoryRecord) -> Result<(), String> {
        ensure(
            (!record.sites.is_empty() || matches!(record.kind, HistoryKind::Diplomacy { .. }))
                && ordered(record.sites.iter().map(|entry| entry.id))
                && record
                    .sites
                    .iter()
                    .all(|entry| self.world.site(entry.id).is_some() && valid_label(&entry.name)),
            "invalid dated site labels",
        )?;
        ensure(
            ordered(record.armies.iter().map(|entry| entry.id))
                && record.armies.iter().all(|entry| {
                    entry.id.0 > 0 && entry.id < self.next_ids.army && valid_label(&entry.name)
                }),
            "invalid dated army labels",
        )?;
        ensure(
            ordered(record.people.iter().map(|entry| entry.id))
                && record.people.iter().all(|entry| {
                    entry.id.0 > 0 && entry.id < self.next_ids.person && valid_label(&entry.name)
                }),
            "invalid dated person labels",
        )?;
        ensure(
            ordered(record.formations.iter().map(|entry| entry.id))
                && record
                    .formations
                    .iter()
                    .all(|entry| entry.id.0 > 0 && entry.id < self.next_ids.formation),
            "invalid dated formation labels",
        )
    }

    fn validate_history_header(
        &self,
        id: HistoryId,
        date: u32,
        visible: &BTreeSet<FactionId>,
    ) -> Result<(), String> {
        ensure(
            id.0 > 0 && id < self.next_ids.history && date <= self.completed_rounds,
            "invalid history identity or date",
        )?;
        ensure(
            !visible.is_empty() && visible.iter().all(|id| self.factions.contains_key(id)),
            "invalid observer set",
        )
    }

    fn validate_history_kind(&self, kind: &HistoryKind, date: u32) -> Result<(), String> {
        match kind {
            HistoryKind::Diplomacy { receipt } => self.validate_diplomacy_receipt(receipt, date),
            HistoryKind::Development { receipt } => self.validate_development_receipt(receipt),
            HistoryKind::Siege {
                siege,
                defender,
                besieger,
                change,
                elapsed_steps,
            } => ensure(
                siege.0 > 0
                    && *siege < self.next_ids.siege
                    && defender != besieger
                    && self.factions.contains_key(defender)
                    && self.factions.contains_key(besieger)
                    && u64::from(*elapsed_steps) <= u64::from(date) + 1
                    && (*change != super::super::siege::SiegeChange::Established
                        || *elapsed_steps == 0),
                "invalid siege narrative identity, factions or progress",
            ),
            HistoryKind::Battle { battle, .. } => ensure(
                battle.0 > 0 && *battle < self.next_ids.battle,
                "invalid historical battle identity",
            ),
            HistoryKind::VeterancyEarned {
                formation,
                tier,
                xp,
            } => ensure(
                formation.0 > 0
                    && *formation < self.next_ids.formation
                    && *tier != Veterancy::Ordinary
                    && *xp > 0,
                "invalid historical veterancy",
            ),
            HistoryKind::Construction { order } => self.validate_construction_receipt(order, date),
            _ => Ok(()),
        }
    }

    fn validate_notables(
        &self,
        subject: HistorySubject,
        summaries: &[NotableSummary],
    ) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for summary in summaries {
            self.validate_history_header(
                summary.id,
                summary.completed_rounds,
                &summary.visible_to,
            )?;
            self.validate_history_kind(&summary.kind, summary.completed_rounds)?;
            if let HistoryKind::Construction { order } = &summary.kind {
                ensure(
                    summary.visible_to == BTreeSet::from([order.owner]),
                    "construction notable has the wrong observer",
                )?;
            }
            if let HistoryKind::Siege {
                defender, besieger, ..
            } = summary.kind
            {
                ensure(
                    summary.visible_to == BTreeSet::from([defender, besieger]),
                    "siege notable has the wrong observers",
                )?;
            }
            ensure(seen.insert(summary.id), "duplicate notable identity")?;
            ensure(
                summary.site.as_ref().is_none_or(|entry| {
                    self.world.site(entry.id).is_some() && valid_label(&entry.name)
                }),
                "invalid notable site label",
            )?;
            if let Some(record) = self.history.events.get(&summary.id) {
                ensure(
                    record.concerns(subject) && *summary == record.notable(),
                    "notable contradicts its retained narrative",
                )?;
            }
        }
        Ok(())
    }
}

fn valid_record_shape(record: &HistoryRecord) -> bool {
    match record.kind {
        HistoryKind::Diplomacy { .. } => record.people.is_empty() && record.formations.is_empty(),
        HistoryKind::Development { .. } => {
            record.armies.is_empty() && record.people.is_empty() && record.formations.is_empty()
        }
        HistoryKind::Battle { .. } => true, // The complete immutable receipt is compared separately.
        HistoryKind::Siege { .. } => {
            record.sites.len() == 1
                && record.armies.is_empty()
                && record.people.is_empty()
                && record.formations.is_empty()
        }
        HistoryKind::Recruited { troop } | HistoryKind::Disbanded { troop } => {
            record.sites.len() == 1
                && record.armies.len() == 1
                && record.people.is_empty()
                && record.formations.len() == 1
                && record.formations[0].kind == troop
        }
        HistoryKind::Moved => !record.armies.is_empty(),
        HistoryKind::FormationTransferred => {
            record.sites.len() == 1
                && (1..=2).contains(&record.armies.len())
                && record.formations.len() == 1
                && record.people.is_empty()
        }
        HistoryKind::PersonTransferred => {
            record.sites.len() == 1
                && record.armies.is_empty()
                && record.people.len() == 1
                && record.formations.len() == 1
        }
        HistoryKind::VeterancyEarned { .. } => record.people.is_empty(),
        HistoryKind::Construction { ref order } => {
            record.people.is_empty()
                && record.formations.is_empty()
                && record.armies.iter().map(|entry| entry.id).eq(order.builder)
        }
        HistoryKind::FocusChanged { .. } => {
            record.sites.len() == 1
                && record.armies.is_empty()
                && record.people.is_empty()
                && record.formations.is_empty()
        }
    }
}

fn source_matches(record: &HistoryRecord, fact: &DomainFactKind) -> bool {
    match fact {
        DomainFactKind::DiplomacyChanged { receipt } => {
            record.kind
                == HistoryKind::Diplomacy {
                    receipt: receipt.clone(),
                }
        }
        DomainFactKind::DevelopmentChanged { receipt } => {
            record.kind
                == HistoryKind::Development {
                    receipt: receipt.clone(),
                }
        }
        DomainFactKind::SiegeChanged { siege, change } => {
            record.kind
                == HistoryKind::Siege {
                    siege: siege.id,
                    defender: siege.defender,
                    besieger: siege.besieger,
                    change: *change,
                    elapsed_steps: siege.elapsed_steps,
                }
                && record.visible_to == BTreeSet::from([siege.defender, siege.besieger])
                && record.sites.len() == 1
                && record.sites[0].id == siege.site
        }
        DomainFactKind::ConstructionChanged { order } => {
            record.kind
                == HistoryKind::Construction {
                    order: order.clone(),
                }
                && record.visible_to == BTreeSet::from([order.owner])
        }
        DomainFactKind::FocusChanged {
            faction,
            site,
            focus,
        } => {
            record.kind == HistoryKind::FocusChanged { focus: *focus }
                && record.visible_to == BTreeSet::from([*faction])
                && record.sites.len() == 1
                && record.sites[0].id == *site
        }
        _ => military_source_matches(record, fact),
    }
}

fn military_source_matches(record: &HistoryRecord, fact: &DomainFactKind) -> bool {
    let has_owner = |owner| record.visible_to == BTreeSet::from([owner]);
    let sites = || {
        record
            .sites
            .iter()
            .map(|entry| entry.id)
            .collect::<BTreeSet<_>>()
    };
    let armies = || {
        record
            .armies
            .iter()
            .map(|entry| entry.id)
            .collect::<BTreeSet<_>>()
    };
    let formation = |id, troop| record.formations == vec![FormationLabel { id, kind: troop }];
    match fact {
        DomainFactKind::BattleResolved { battle, .. } => {
            matches!(record.kind, HistoryKind::Battle { battle: id, .. } if id == *battle)
        }
        DomainFactKind::FormationRecruited {
            faction,
            army,
            formation: id,
            site,
            troop,
        } => {
            record.kind == HistoryKind::Recruited { troop: *troop }
                && has_owner(*faction)
                && sites() == BTreeSet::from([*site])
                && armies() == BTreeSet::from([*army])
                && formation(*id, *troop)
        }
        DomainFactKind::FormationDisbanded {
            faction,
            army,
            formation: id,
            site,
            troop,
        } => {
            record.kind == HistoryKind::Disbanded { troop: *troop }
                && has_owner(*faction)
                && sites() == BTreeSet::from([*site])
                && armies() == BTreeSet::from([*army])
                && formation(*id, *troop)
        }
        DomainFactKind::ArmiesMoved {
            faction,
            armies: moved,
            path,
            movement,
            ..
        } => {
            record.kind == HistoryKind::Moved
                && has_owner(*faction)
                && sites() == path.iter().copied().collect()
                && armies() == moved.iter().copied().collect()
                && movement_labels_match(record, movement.as_ref())
        }
        DomainFactKind::FormationTransferred {
            faction,
            formation,
            from_army,
            to_army,
            site,
        } => {
            record.kind == HistoryKind::FormationTransferred
                && has_owner(*faction)
                && sites() == BTreeSet::from([*site])
                && armies() == BTreeSet::from([*from_army, *to_army])
                && record.formations.len() == 1
                && record.formations[0].id == *formation
        }
        DomainFactKind::PersonTransferred {
            faction,
            person,
            to_formation,
            site,
        } => {
            record.kind == HistoryKind::PersonTransferred
                && has_owner(*faction)
                && sites() == BTreeSet::from([*site])
                && record.people.len() == 1
                && record.people[0].id == *person
                && record.formations.len() == 1
                && record.formations[0].id == *to_formation
        }
        _ => false,
    }
}

fn movement_labels_match(
    record: &HistoryRecord,
    movement: Option<&crate::state::evidence::MovementService>,
) -> bool {
    match movement {
        Some(receipt) => {
            record
                .people
                .iter()
                .map(|entry| entry.id)
                .eq(receipt.people.iter().copied())
                && record
                    .formations
                    .iter()
                    .map(|entry| entry.id)
                    .eq(receipt.formations.iter().copied())
        }
        None => record.people.is_empty() && record.formations.is_empty(),
    }
}

fn ordered<T: Ord + Copy>(values: impl Iterator<Item = T>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|old| old >= value) {
            return false;
        }
        previous = Some(value);
    }
    true
}

fn valid_label(name: &str) -> bool {
    !name.trim().is_empty()
}

fn ensure(valid: bool, message: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("history: {message}"))
    }
}
