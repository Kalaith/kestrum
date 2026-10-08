//! Observer event summaries and a complete per-step diagnostic JSONL file.

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use kestrum::state::{
    evidence::EvidenceKind,
    people::{PersonAssignment, PersonId},
};
use kestrum::{
    data::{world::FactionId, GameData},
    engine::ObserverStepOutcome,
    state::{
        battle::{BattleOutcome, BattleReport},
        campaign::DomainFactKind,
        development::DevelopmentReceipt,
        diplomacy::DiplomacyReceipt,
        history::{HistoryId, HistoryKind, HistoryRecord, LifeEvent},
        relationships::FamilyOrigin,
        FactionStatus, StrategicCampaign,
    },
};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(not(target_arch = "wasm32"))]
mod progression_audit;
#[cfg(not(target_arch = "wasm32"))]
use progression_audit::{faction_progression_counts, formation_service_audit};

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone)]
pub(super) struct ObserverSnapshot {
    round: u32,
    actor: Option<FactionId>,
    faction_names: BTreeMap<FactionId, String>,
    faction_statuses: BTreeMap<FactionId, FactionStatus>,
    site_controllers: BTreeMap<kestrum::data::world::SiteId, Option<FactionId>>,
    site_names: BTreeMap<kestrum::data::world::SiteId, String>,
    #[cfg(not(target_arch = "wasm32"))]
    people: BTreeSet<PersonId>,
    history_cursor: HistoryId,
    #[cfg(not(target_arch = "wasm32"))]
    battle_cursor: kestrum::state::battle::BattleId,
    wars: BTreeSet<[FactionId; 2]>,
}

impl ObserverSnapshot {
    pub(super) fn capture(campaign: &StrategicCampaign) -> Self {
        Self {
            round: campaign.completed_rounds,
            actor: match campaign.phase {
                kestrum::state::CampaignPhase::NpcTurn { faction, .. } => Some(faction),
                kestrum::state::CampaignPhase::PlayerTurn => None,
            },
            faction_names: campaign
                .factions
                .iter()
                .map(|(id, faction)| (*id, faction.name.clone()))
                .collect(),
            faction_statuses: campaign
                .factions
                .iter()
                .map(|(id, faction)| (*id, faction.status))
                .collect(),
            site_controllers: campaign
                .world
                .sites
                .iter()
                .map(|site| (site.id, site.controller))
                .collect(),
            site_names: campaign
                .world
                .sites
                .iter()
                .map(|site| (site.id, site.name.clone()))
                .collect(),
            #[cfg(not(target_arch = "wasm32"))]
            people: campaign.people.keys().copied().collect(),
            history_cursor: campaign.next_ids.history,
            #[cfg(not(target_arch = "wasm32"))]
            battle_cursor: campaign.next_ids.battle,
            wars: campaign
                .relations
                .iter()
                .filter(|relation| relation.state == kestrum::data::world::DiplomaticState::War)
                .map(|relation| relation.factions)
                .collect(),
        }
    }
}

#[derive(Default)]
pub(super) struct ObserverLog {
    pub(super) view: ui::ObserverEventLog,
    #[cfg(not(target_arch = "wasm32"))]
    file: Option<std::fs::File>,
}

impl ObserverLog {
    pub(super) fn reset(&mut self) {
        self.finish();
        self.view = Default::default();
    }

    pub(super) fn start(&mut self, seed: u64, capture: bool) {
        self.reset();
        self.start_platform(seed, capture);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_platform(&mut self, seed: u64, capture: bool) {
        if capture {
            self.view.status = Some(
                "Capture preview; native play writes every observer step to a JSONL file.".into(),
            );
        } else {
            self.open_native(seed);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn start_platform(&mut self, _seed: u64, capture: bool) {
        self.view.status = Some(if capture {
            "Capture preview; native play writes every observer step to a JSONL file.".into()
        } else {
            "Browser builds keep this event feed in memory; native play writes a JSONL file.".into()
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_native(&mut self, seed: u64) {
        let stamp = unix_millis();
        let filename = format!("observer_logs/observer-{seed}-{stamp}.jsonl");
        let Some(path) = macroquad_toolkit::persistence::get_app_data_path("kestrum", &filename)
        else {
            self.view.status = Some("Could not find the Kestrum app-data directory.".into());
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                self.view.status = Some(format!("Could not create observer log folder: {error}"));
                return;
            }
        }
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
        {
            Ok(file) => {
                self.view.file_path = Some(path.display().to_string());
                self.file = Some(file);
                self.append(&json!({
                    "record": "session_started",
                    "written_at_unix_ms": unix_millis(),
                    "seed": seed,
                    "format_version": 2
                }));
            }
            Err(error) => {
                self.view.status = Some(format!(
                    "Could not open observer log {}: {error}",
                    path.display()
                ));
            }
        }
    }

    pub(super) fn record(
        &mut self,
        before: &ObserverSnapshot,
        after: &StrategicCampaign,
        step: &ObserverStepOutcome,
        data: &GameData,
    ) {
        let life_records = new_life_records(before, after);
        let events = notable_events(before, after, step, &life_records);
        for event in &events {
            self.view.add(after.completed_rounds, event.clone());
        }
        self.record_file(before, after, step, data, &events, &life_records);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn record_file(
        &mut self,
        before: &ObserverSnapshot,
        after: &StrategicCampaign,
        step: &ObserverStepOutcome,
        data: &GameData,
        events: &[String],
        life_records: &[&HistoryRecord],
    ) {
        let actor = before.actor;
        let mut record = json!({
            "record": "observer_step",
            "written_at_unix_ms": unix_millis(),
            "round_before": before.round,
            "round_after": after.completed_rounds,
            "actor": actor.map(|id| json!({
                "id": id,
                "name": before.faction_names.get(&id)
            })),
            "action": &step.action,
            "diagnostics": &step.diagnostics,
            "accepted_sequence": step.outcome.accepted_sequence,
            "round_completed": step.outcome.round_completed,
            "action_result_debug": format!("{:?}", step.outcome),
            "facts": &step.outcome.facts,
            "consumed_facts": &step.outcome.consumed_facts,
            "outcome_new_people": step.outcome.new_people,
            "outcome_new_people_is_complete": false,
            "people_added_in_accepted_step": added_people(before, after),
            "life_events": life_records.iter().map(|record| life_event_audit(after, record)).collect::<Vec<_>>(),
            "person_combat_events": new_person_combat_events(before, after),
            "notable_events": events,
            "state": campaign_snapshot(before, after, data)
        });
        if step.outcome.round_completed {
            record["formation_service_audit"] = formation_service_audit(after, data);
            record["faction_progression_counts"] = faction_progression_counts(after);
        }
        self.append(&record);
    }

    #[cfg(target_arch = "wasm32")]
    fn record_file(
        &mut self,
        _before: &ObserverSnapshot,
        _after: &StrategicCampaign,
        _step: &ObserverStepOutcome,
        _data: &GameData,
        _events: &[String],
        _life_records: &[&HistoryRecord],
    ) {
    }

    pub(super) fn record_failure(
        &mut self,
        before: &ObserverSnapshot,
        campaign: &StrategicCampaign,
        data: &GameData,
        error: &str,
    ) {
        self.record_failure_file(before, campaign, data, error);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn record_failure_file(
        &mut self,
        before: &ObserverSnapshot,
        campaign: &StrategicCampaign,
        data: &GameData,
        error: &str,
    ) {
        self.append(&json!({
            "record": "observer_step_failed",
            "written_at_unix_ms": unix_millis(),
            "round": before.round,
            "actor": before.actor.map(|id| json!({
                "id": id,
                "name": before.faction_names.get(&id)
            })),
            "error": error,
            "state": campaign_snapshot(before, campaign, data)
        }));
    }

    #[cfg(target_arch = "wasm32")]
    fn record_failure_file(
        &mut self,
        _before: &ObserverSnapshot,
        _campaign: &StrategicCampaign,
        _data: &GameData,
        _error: &str,
    ) {
    }

    pub(super) fn finish(&mut self) {
        self.finish_platform();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn finish_platform(&mut self) {
        if self.file.is_some() {
            self.append(&json!({
                "record": "session_ended",
                "written_at_unix_ms": unix_millis()
            }));
            self.file = None;
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn finish_platform(&mut self) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn append(&mut self, record: &Value) {
        if let Some(file) = &mut self.file {
            use std::io::Write;
            let result = serde_json::to_vec(record)
                .map_err(|error| error.to_string())
                .and_then(|mut line| {
                    line.push(b'\n');
                    file.write_all(&line)
                        .and_then(|_| file.flush())
                        .map_err(|error| error.to_string())
                });
            if let Err(error) = result {
                self.file = None;
                self.view.status = Some(format!("Observer log write failed: {error}"));
            }
        }
    }
}

fn new_life_records<'a>(
    before: &ObserverSnapshot,
    campaign: &'a StrategicCampaign,
) -> Vec<&'a HistoryRecord> {
    campaign
        .history
        .events
        .range(before.history_cursor..)
        .map(|(_, record)| record)
        .filter(|record| matches!(record.kind, HistoryKind::Life { .. }))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn added_people(before: &ObserverSnapshot, campaign: &StrategicCampaign) -> Vec<Value> {
    campaign
        .people
        .iter()
        .filter(|(id, _)| !before.people.contains(id))
        .map(|(_, person)| person_progression_audit(campaign, person.id))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn life_event_audit(campaign: &StrategicCampaign, record: &HistoryRecord) -> Value {
    let HistoryKind::Life {
        owner,
        person,
        event,
    } = &record.kind
    else {
        return Value::Null;
    };
    json!({
        "history_id": record.id,
        "round": record.completed_rounds,
        "owner": owner,
        "person": person,
        "event": event,
        "site": record.sites.first(),
        "person_state": person_progression_audit(campaign, *person)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn person_progression_audit(campaign: &StrategicCampaign, id: PersonId) -> Value {
    let Some(person) = campaign.people.get(&id) else {
        return json!({ "id": id, "missing_after_step": true });
    };
    let assigned_formation = match person.assignment {
        PersonAssignment::Formation { formation } => Some(formation),
        _ => None,
    };
    let assigned_army = assigned_formation.and_then(|formation| {
        campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| json!({ "id": army.id, "name": army.name, "site": army.site }))
    });
    json!({
        "id": person.id,
        "name": person.name,
        "faction": person.faction,
        "age_years": person.age_years(campaign.completed_rounds),
        "class": person.class,
        "alive": person.is_alive(),
        "retired": person.career.retired,
        "assignment": person.assignment,
        "assigned_formation": assigned_formation,
        "assigned_army": assigned_army,
        "emergence": person.career.emergence,
        "personal_qualifying_participation": person.evidence.counts
            .get(&EvidenceKind::MeaningfulEncounter).copied().unwrap_or_default(),
        "recognition": person.career.recognition
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn new_person_combat_events(before: &ObserverSnapshot, campaign: &StrategicCampaign) -> Vec<Value> {
    campaign
        .battles
        .range(before.battle_cursor..)
        .flat_map(|(battle_id, report)| {
            report.person_events.iter().map(move |event| {
                json!({
                    "battle": battle_id,
                    "round": report.completed_rounds,
                    "site": report.site,
                    "person": event.person,
                    "outcome": event.outcome,
                    "person_state": person_progression_audit(campaign, event.person)
                })
            })
        })
        .collect()
}

fn life_event_line(campaign: &StrategicCampaign, record: &HistoryRecord) -> Option<String> {
    let HistoryKind::Life { person, event, .. } = &record.kind else {
        return None;
    };
    let name = campaign.people.get(person)?.name.as_str();
    match event {
        LifeEvent::Emerged { troop } => Some(format!("Apprentice {name} emerged with {troop:?}")),
        LifeEvent::Arrived { origin } => Some(match origin {
            FamilyOrigin::Birth => format!("{name} was born"),
            FamilyOrigin::AdoptedWard => format!("{name} joined as an adopted ward"),
            FamilyOrigin::LocalApprentice => format!("{name} joined as a local apprentice"),
        }),
        LifeEvent::Recognized { epithet, .. } => {
            Some(format!("{name} was recognized as {epithet}"))
        }
        LifeEvent::Retired => Some(format!("{name} retired")),
        LifeEvent::NaturalDeath => Some(format!("{name} died")),
        _ => None,
    }
}

fn notable_events(
    before: &ObserverSnapshot,
    after: &StrategicCampaign,
    step: &ObserverStepOutcome,
    life_records: &[&HistoryRecord],
) -> Vec<String> {
    let mut events = Vec::new();
    for (site_id, controller) in &before.site_controllers {
        let Some(site) = after.world.site(*site_id) else {
            continue;
        };
        let old = *controller;
        let new = site.controller;
        if old == new {
            continue;
        }
        let place = before
            .site_names
            .get(site_id)
            .map_or(site.name.as_str(), String::as_str);
        match (old, new) {
            (None, Some(owner)) => events.push(format!(
                "{} took control of {place}",
                faction_name(after, owner)
            )),
            (Some(old_owner), Some(new_owner)) => events.push(format!(
                "{} captured {place} from {}",
                faction_name(after, new_owner),
                faction_name(after, old_owner)
            )),
            (Some(old_owner), None) => events.push(format!(
                "{} lost control of {place}",
                faction_name(after, old_owner)
            )),
            (None, None) => {}
        }
    }
    for (id, status) in &before.faction_statuses {
        if after
            .factions
            .get(id)
            .is_some_and(|faction| faction.status != *status)
        {
            let name = faction_name(after, *id);
            let result = match after.factions[id].status {
                FactionStatus::Eliminated => format!("{name} was eliminated"),
                FactionStatus::Vassal { sovereign } => format!(
                    "{name} became subordinate to {}",
                    faction_name(after, sovereign)
                ),
                FactionStatus::Independent => format!("{name} regained independence"),
            };
            events.push(result);
        }
    }
    for faction in after.factions.values() {
        if !before.faction_statuses.contains_key(&faction.id) {
            events.push(format!("New kingdom founded: {}", faction.name));
        }
    }
    for relation in &after.relations {
        if relation.state == kestrum::data::world::DiplomaticState::War
            && !before.wars.contains(&relation.factions)
        {
            events.push(format!(
                "War declared: {} and {}",
                faction_name(after, relation.factions[0]),
                faction_name(after, relation.factions[1])
            ));
        }
    }
    for record in life_records {
        if let Some(event) = life_event_line(after, record) {
            events.push(event);
        }
    }
    for fact in &step.outcome.facts {
        if let Some(event) = fact_event(after, &fact.kind) {
            events.push(event);
        }
    }
    if step.outcome.round_completed {
        events.push(format!("Season {} completed", after.completed_rounds));
    }
    let mut seen = BTreeSet::new();
    events.retain(|event| seen.insert(event.clone()));
    events
}

fn fact_event(campaign: &StrategicCampaign, fact: &DomainFactKind) -> Option<String> {
    match fact {
        DomainFactKind::DiplomacyChanged { receipt } => match receipt {
            DiplomacyReceipt::WarDeclared { factions } => Some(format!(
                "War declared: {} and {}",
                faction_name(campaign, factions[0]),
                faction_name(campaign, factions[1])
            )),
            DiplomacyReceipt::PeaceOffered {
                proposer,
                recipient,
            } => Some(format!(
                "{} offered peace to {}",
                faction_name(campaign, *proposer),
                faction_name(campaign, *recipient)
            )),
            DiplomacyReceipt::PeaceRejected {
                proposer,
                recipient,
            } => Some(format!(
                "{} rejected peace from {}",
                faction_name(campaign, *recipient),
                faction_name(campaign, *proposer)
            )),
            DiplomacyReceipt::PeaceAgreed { factions, .. } => Some(format!(
                "Peace agreed between {} and {}",
                faction_name(campaign, factions[0]),
                faction_name(campaign, factions[1])
            )),
            DiplomacyReceipt::DefeatPending { faction, victor } => Some(format!(
                "{} was defeated by {}",
                faction_name(campaign, *faction),
                faction_name(campaign, *victor)
            )),
            DiplomacyReceipt::FactionResolved {
                faction, victor, ..
            } => Some(format!(
                "{} was resolved by {}",
                faction_name(campaign, *faction),
                (*victor).map_or_else(|| "no victor".into(), |id| faction_name(campaign, id))
            )),
            DiplomacyReceipt::CampaignEnded { ending } => {
                Some(format!("Observer campaign ended: {:?}", ending.kind))
            }
            DiplomacyReceipt::ArmyWithdrawn { .. } => None,
        },
        DomainFactKind::DevelopmentChanged { receipt } => match receipt {
            DevelopmentReceipt::HabitationChanged { site, from, to, .. } => Some(format!(
                "{} grew from {:?} to {:?}",
                site_name(campaign, *site),
                from,
                to
            )),
            DevelopmentReceipt::Ruined { site, .. } => {
                Some(format!("{} was ruined", site_name(campaign, *site)))
            }
            DevelopmentReceipt::PopulationMoved { .. }
            | DevelopmentReceipt::SiteRenamed { .. }
            | DevelopmentReceipt::CapitalMoved { .. }
            | DevelopmentReceipt::HeadquartersMoved { .. } => None,
        },
        DomainFactKind::BattleResolved { battle, .. } => {
            campaign.battles.get(battle).map(battle_event)
        }
        DomainFactKind::FormationRecruited {
            faction,
            site,
            troop,
            ..
        } => Some(format!(
            "{} raised {:?} at {}",
            faction_name(campaign, *faction),
            troop,
            site_name(campaign, *site)
        )),
        DomainFactKind::FormationDisbanded {
            faction,
            site,
            troop,
            ..
        } => Some(format!(
            "{} disbanded {:?} at {}",
            faction_name(campaign, *faction),
            troop,
            site_name(campaign, *site)
        )),
        DomainFactKind::SiegeChanged { siege, change } => Some(format!(
            "Siege {:?} at {} ({})",
            change,
            site_name(campaign, siege.site),
            faction_name(campaign, siege.besieger)
        )),
        DomainFactKind::ConstructionChanged { .. }
        | DomainFactKind::FocusChanged { .. }
        | DomainFactKind::FactionPassed { .. }
        | DomainFactKind::ArmiesMoved { .. }
        | DomainFactKind::FormationTransferred { .. }
        | DomainFactKind::PersonTransferred { .. } => None,
    }
}

fn battle_event(report: &BattleReport) -> String {
    let attacker = &report.attacker.name;
    let defender = report.defender.name();
    match report.outcome {
        BattleOutcome::AttackerVictory => {
            format!("{attacker} defeated {defender} at {}", report.site_name)
        }
        BattleOutcome::DefenderVictory => {
            format!("{defender} held {} against {attacker}", report.site_name)
        }
        BattleOutcome::Stalemate => format!(
            "Battle at {} between {attacker} and {defender} ended in a stalemate",
            report.site_name
        ),
        BattleOutcome::MutualDestruction => {
            format!("Both sides were destroyed at {}", report.site_name)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn campaign_snapshot(
    before: &ObserverSnapshot,
    campaign: &StrategicCampaign,
    data: &GameData,
) -> Value {
    let factions: Vec<_> =
        campaign
            .factions
            .values()
            .map(|faction| {
                let armies: Vec<_> = campaign
                .armies
                .values()
                .filter(|army| army.faction == faction.id)
                .map(|army| {
                    let formations: Vec<_> = army
                        .formation_ids()
                        .filter_map(|id| campaign.formations.get(&id))
                        .map(|formation| json!({
                            "id": formation.id,
                            "kind": formation.kind,
                            "headcount": formation.headcount,
                            "capacity": formation.capacity,
                            "movement_spent": formation.movement_spent
                        }))
                        .collect();
                    json!({
                        "id": army.id,
                        "name": &army.name,
                        "site": army.site,
                        "site_name": site_name(campaign, army.site),
                        "supplied": campaign.army_is_supplied(army.id),
                        "movement_remaining": campaign.army_movement_remaining(army.id, data),
                        "in_siege": campaign.sieges.get(&army.site).is_some_and(|siege| {
                            siege.besieging.contains(&army.id) || siege.defending.contains(&army.id)
                        }),
                        "formations": formations
                    })
                })
                .collect();
                let owned_sites: Vec<_> = campaign
                    .world
                    .sites
                    .iter()
                    .filter(|site| site.controller == Some(faction.id))
                    .map(|site| site.id)
                    .collect();
                let ai = campaign.ai.factions.get(&faction.id);
                json!({
                    "id": faction.id,
                    "name": faction.name,
                    "status": faction.status,
                    "headquarters": faction.headquarters,
                    "capital": faction.capital,
                    "resources": &faction.resources,
                    "deficit": faction.deficit,
                    "owned_sites": owned_sites,
                    "armies": armies,
                    "ai_objective": ai.and_then(|state| state.objective.as_ref()),
                    "ai_accepted_commands": ai.map(|state| state.accepted_commands),
                    "ai_rejected_intents": ai.map(|state| state.rejected.len())
                })
            })
            .collect();
    json!({
        "phase": campaign.phase,
        "active_faction": campaign.active_faction(),
        "accepted_sequence": campaign.accepted_sequence,
        "factions": factions,
        "territory": campaign.world.sites.iter().map(|site| json!({
            "site": site.id,
            "controller": site.controller,
            "contested": campaign.world.contested_sites.contains(&site.id)
        })).collect::<Vec<_>>(),
        "wars": campaign.relations.iter().filter(|relation| {
            relation.state == kestrum::data::world::DiplomaticState::War
        }).map(|relation| relation.factions).collect::<Vec<_>>(),
        "sieges": &campaign.sieges,
        "threats": campaign.threats.iter().map(|(id, threat)| json!({
            "id": id,
            "site": threat.site,
            "kind": threat.kind,
            "status": &threat.status
        })).collect::<Vec<_>>(),
        "construction": &campaign.construction,
        "movement_plans": &campaign.movement_plans,
        "pending_offers": &campaign.diplomacy.pending_offers,
        "pending_defeats": &campaign.diplomacy.pending_defeats,
        "pending_battle": campaign.pending_battle.as_ref().map(|battle| battle.report.id),
        "control_changes_this_step": campaign.world.sites.iter().filter_map(|site| {
            let old = before.site_controllers.get(&site.id).copied().flatten();
            (old != site.controller).then_some(json!({
                "site": site.id,
                "name": site.name,
                "from": old,
                "to": site.controller
            }))
        }).collect::<Vec<_>>()
    })
}

fn faction_name(campaign: &StrategicCampaign, faction: FactionId) -> String {
    campaign.factions.get(&faction).map_or_else(
        || format!("Faction #{}", faction.0),
        |entry| entry.name.clone(),
    )
}

fn site_name(campaign: &StrategicCampaign, site: kestrum::data::world::SiteId) -> &str {
    campaign
        .world
        .site(site)
        .map_or("unknown place", |entry| entry.name.as_str())
}

#[cfg(not(target_arch = "wasm32"))]
fn unix_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis())
}
