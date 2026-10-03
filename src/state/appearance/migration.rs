//! Deterministic migration of retained identities in pre-portrait campaign saves.

use crate::{
    data::portraits::{AppearanceDescriptor, PortraitCatalog, APPEARANCE_SCHEMA_VERSION},
    engine::portraits::allocate_from_registry,
    state::{
        appearance::{
            derive_appearance_salt, AppearanceRegistry, APPEARANCE_REGISTRY_SCHEMA_VERSION,
        },
        people::{PersonId, PersonStatus},
    },
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct IdentityScan {
    ids: BTreeSet<PersonId>,
    living: BTreeSet<PersonId>,
    descriptors: BTreeMap<PersonId, AppearanceDescriptor>,
    records_with_appearance: usize,
    records_without_appearance: usize,
    notification_descriptors_without_registry: bool,
}

/// Initialize only wholly legacy saves; partial portrait state is a compatibility error.
pub fn migrate_legacy(value: &mut Value) -> Result<(), String> {
    let root = value
        .as_object()
        .ok_or_else(|| "appearance migration: campaign must be an object".to_owned())?;
    let mut scan = IdentityScan::default();
    visit_records(value, |record, path| scan_record(record, path, &mut scan))?;
    scan_notification_descriptors(value, &mut scan)?;
    let has_registry = root.contains_key("appearance_registry");

    if has_registry {
        if scan.records_without_appearance != 0 {
            return Err(
                "appearance migration: registry exists but a retained identity has no descriptor"
                    .into(),
            );
        }
        validate_modern_fields(root, &scan)?;
        return Ok(());
    }
    if scan.records_with_appearance != 0 {
        return Err(
            "appearance migration: descriptors exist without an appearance registry".into(),
        );
    }
    if scan.notification_descriptors_without_registry {
        return Err(
            "appearance migration: notification descriptors exist without an appearance registry"
                .into(),
        );
    }

    migrate_absent_state(value, &scan)
}

/// Notifications may contain optional, observer-safe person snapshots. Older
/// notifications legitimately omit appearance, so they are never assigned or
/// included in the identity union. A present descriptor, however, proves that
/// the save contains partial modern portrait state and requires its registry.
fn scan_notification_descriptors(root: &Value, scan: &mut IdentityScan) -> Result<(), String> {
    let Some(notifications) = root.get("notifications") else {
        return Ok(());
    };
    if notifications.is_null() {
        return Ok(());
    }
    let notifications = object(notifications, "campaign.notifications")?;
    let Some(receipts) = notifications.get("receipts") else {
        return Ok(());
    };
    let receipts = array(receipts, "campaign.notifications.receipts")?;
    for (index, receipt) in receipts.iter().enumerate() {
        let path = format!("campaign.notifications.receipts[{index}]");
        let receipt = object(receipt, &path)?;
        if let Some(subject) = receipt.get("subject") {
            scan_notification_subject(subject, &format!("{path}.subject"), scan)?;
        }
        if let Some(detail) = receipt.get("detail") {
            scan_notification_detail(detail, &format!("{path}.detail"), scan)?;
        }
    }
    Ok(())
}

fn scan_notification_subject(
    subject: &Value,
    path: &str,
    scan: &mut IdentityScan,
) -> Result<(), String> {
    let Some(subject) = subject.as_object() else {
        // `subject` is optional and old receipts may store it as null.
        return Ok(());
    };
    if subject.get("kind").and_then(Value::as_str) == Some("person") {
        if let Some(snapshot) = subject.get("snapshot") {
            scan_notification_person(snapshot, &format!("{path}.snapshot"), scan)?;
        }
    }
    Ok(())
}

fn scan_notification_detail(
    detail: &Value,
    path: &str,
    scan: &mut IdentityScan,
) -> Result<(), String> {
    let Some(detail) = detail.as_object() else {
        return Ok(());
    };
    match detail.get("kind").and_then(Value::as_str) {
        Some("person") => {
            if let Some(person) = detail.get("person") {
                scan_notification_person(person, &format!("{path}.person"), scan)?;
            }
        }
        Some("remembrance") => {
            if let Some(subject) = detail.get("subject") {
                scan_notification_subject(subject, &format!("{path}.subject"), scan)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn scan_notification_person(
    person: &Value,
    path: &str,
    scan: &mut IdentityScan,
) -> Result<(), String> {
    let person = object(person, path)?;
    if person
        .get("appearance")
        .is_some_and(|appearance| !appearance.is_null())
    {
        scan.notification_descriptors_without_registry = true;
    }
    Ok(())
}

fn validate_modern_fields(root: &Map<String, Value>, scan: &IdentityScan) -> Result<(), String> {
    let registry: AppearanceRegistry = serde_json::from_value(
        root.get("appearance_registry")
            .cloned()
            .ok_or_else(|| "appearance migration: missing registry".to_owned())?,
    )
    .map_err(|error| format!("appearance_registry: {error}"))?;
    if registry.schema_version != APPEARANCE_REGISTRY_SCHEMA_VERSION {
        return Err("appearance_registry: unsupported schema version".into());
    }
    let seed = required_u64(root, "seed", "campaign")?;
    if registry.appearance_salt != derive_appearance_salt(seed) {
        return Err("appearance_registry: salt does not match campaign seed".into());
    }
    if scan.descriptors.len() != scan.ids.len() && !scan.ids.is_empty() {
        return Err("appearance migration: retained identity descriptors are incomplete".into());
    }
    Ok(())
}

fn migrate_absent_state(value: &mut Value, scan: &IdentityScan) -> Result<(), String> {
    let root = value
        .as_object()
        .ok_or_else(|| "appearance migration: campaign must be an object".to_owned())?;
    let seed = required_u64(root, "seed", "campaign")?;
    let round = required_u32(root, "completed_rounds", "campaign")?;
    let catalog = PortraitCatalog::load_frozen()?;
    catalog.validate()?;
    let mut registry = AppearanceRegistry::for_campaign_seed(
        seed,
        catalog.catalog_revision,
        catalog.allocation_revision,
    );
    let mut assignments = BTreeMap::new();
    let mut living = Vec::new();

    for id in &scan.ids {
        let descriptor = allocate_from_registry(&mut registry, &catalog, *id, round, &living)?;
        if scan.living.contains(id) {
            living.push(descriptor.clone());
        }
        assignments.insert(*id, descriptor);
    }

    visit_records_mut(value, |record, path| {
        let id = record_id(record, path)?;
        let descriptor = assignments.get(&id).ok_or_else(|| {
            format!(
                "appearance migration: no assignment exists for identity {}",
                id.0
            )
        })?;
        let fields = record
            .as_object_mut()
            .ok_or_else(|| format!("{path}: identity must be an object"))?;
        fields.insert(
            "appearance".into(),
            serde_json::to_value(descriptor)
                .map_err(|error| format!("appearance migration: {error}"))?,
        );
        Ok(())
    })?;
    value
        .as_object_mut()
        .ok_or_else(|| "appearance migration: campaign must be an object".to_owned())?
        .insert(
            "appearance_registry".into(),
            serde_json::to_value(registry)
                .map_err(|error| format!("appearance migration: {error}"))?,
        );
    Ok(())
}

fn scan_record(record: &Value, path: &str, scan: &mut IdentityScan) -> Result<(), String> {
    let id = record_id(record, path)?;
    scan.ids.insert(id);
    let fields = record
        .as_object()
        .ok_or_else(|| format!("{path}: identity must be an object"))?;
    if let Some(value) = fields.get("appearance") {
        let descriptor: AppearanceDescriptor = serde_json::from_value(value.clone())
            .map_err(|error| format!("{path}.appearance: {error}"))?;
        validate_descriptor_shape(&descriptor, path)?;
        scan.records_with_appearance += 1;
        if let Some(previous) = scan.descriptors.insert(id, descriptor.clone()) {
            if previous != descriptor {
                return Err(format!(
                    "appearance migration: retained copies of person {} disagree",
                    id.0
                ));
            }
        }
    } else {
        scan.records_without_appearance += 1;
    }
    if path.starts_with("campaign.people[") {
        let status = fields
            .get("status")
            .cloned()
            .map(serde_json::from_value::<PersonStatus>)
            .transpose()
            .map_err(|error| format!("{path}.status: {error}"))?
            .unwrap_or_default();
        if !matches!(status, PersonStatus::Dead { .. }) {
            scan.living.insert(id);
        }
    }
    Ok(())
}

fn validate_descriptor_shape(descriptor: &AppearanceDescriptor, path: &str) -> Result<(), String> {
    if descriptor.schema_version != APPEARANCE_SCHEMA_VERSION
        || descriptor.catalog_revision == 0
        || descriptor.allocation_revision == 0
        || descriptor.signature.is_empty()
        || [
            &descriptor.rig_id,
            &descriptor.face_id,
            &descriptor.nose_id,
            &descriptor.eyes_id,
            &descriptor.hair_id,
            &descriptor.skin_palette_id,
            &descriptor.hair_palette_id,
            &descriptor.eye_palette_id,
        ]
        .iter()
        .any(|part| part.is_empty())
    {
        return Err(format!(
            "{path}.appearance: malformed or unsupported descriptor"
        ));
    }
    Ok(())
}

fn record_id(record: &Value, path: &str) -> Result<PersonId, String> {
    let id = record
        .get("id")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| format!("{path}.id: expected a nonzero person ID"))?;
    Ok(PersonId(id))
}

fn required_u64(object: &Map<String, Value>, key: &str, path: &str) -> Result<u64, String> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{path}.{key}: expected an unsigned integer"))
}

fn required_u32(object: &Map<String, Value>, key: &str, path: &str) -> Result<u32, String> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{path}.{key}: expected an unsigned 32-bit integer"))
}

fn visit_records(
    root: &Value,
    mut visit: impl FnMut(&Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let campaign = object(root, "campaign")?;
    if let Some(people) = campaign.get("people") {
        visit_identity_map(people, "campaign.people", &mut visit)?;
    }
    if let Some(knowledge) = campaign.get("knowledge") {
        let knowledge = object(knowledge, "campaign.knowledge")?;
        if let Some(observers) = knowledge.get("observers") {
            let observers = object(observers, "campaign.knowledge.observers")?;
            for (observer, state) in observers {
                let path = format!("campaign.knowledge.observers[{observer}]");
                let state = object(state, &path)?;
                if let Some(people) = state.get("people") {
                    visit_identity_map(people, &format!("{path}.people"), &mut visit)?;
                }
            }
        }
    }
    if let Some(battles) = campaign.get("battles") {
        let battles = object(battles, "campaign.battles")?;
        for (id, report) in battles {
            visit_report(report, &format!("campaign.battles[{id}]"), &mut visit)?;
        }
    }
    if let Some(pending) = campaign.get("pending_battle") {
        if !pending.is_null() {
            let pending = object(pending, "campaign.pending_battle")?;
            let report = pending
                .get("report")
                .ok_or_else(|| "campaign.pending_battle.report: missing report".to_owned())?;
            visit_report(report, "campaign.pending_battle.report", &mut visit)?;
        }
    }
    Ok(())
}

fn visit_identity_map(
    value: &Value,
    path: &str,
    visit: &mut impl FnMut(&Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let entries = object(value, path)?;
    for (key, record) in entries {
        let id = key
            .parse::<u32>()
            .ok()
            .filter(|id| id.to_string() == *key && *id != 0)
            .ok_or_else(|| format!("{path}[{key}]: invalid person ID key"))?;
        let record_path = format!("{path}[{key}]");
        if record_id(record, &record_path)? != PersonId(id) {
            return Err(format!("{record_path}: map key does not match person ID"));
        }
        visit(record, &record_path)?;
    }
    Ok(())
}

fn visit_report(
    value: &Value,
    path: &str,
    visit: &mut impl FnMut(&Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let report = object(value, path)?;
    let attacker = report
        .get("attacker")
        .ok_or_else(|| format!("{path}.attacker: missing battle side"))?;
    visit_side(attacker, &format!("{path}.attacker"), visit)?;
    let defender = object(
        report
            .get("defender")
            .ok_or_else(|| format!("{path}.defender: missing battle defender"))?,
        &format!("{path}.defender"),
    )?;
    match defender.get("kind").and_then(Value::as_str) {
        Some("faction") => {
            let side_path = format!("{path}.defender.side");
            let side = defender
                .get("side")
                .ok_or_else(|| format!("{side_path}: missing faction side"))?;
            visit_side(side, &side_path, visit)
        }
        Some("threat") => Ok(()),
        Some(kind) => Err(format!("{path}.defender.kind: unsupported value {kind}")),
        None => Err(format!("{path}.defender.kind: missing discriminator")),
    }
}

fn visit_side(
    value: &Value,
    path: &str,
    visit: &mut impl FnMut(&Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let side = object(value, path)?;
    let armies = array(
        side.get("armies")
            .ok_or_else(|| format!("{path}.armies: missing armies"))?,
        &format!("{path}.armies"),
    )?;
    for (army_index, army) in armies.iter().enumerate() {
        let army_path = format!("{path}.armies[{army_index}]");
        let army = object(army, &army_path)?;
        let people = array(
            army.get("people")
                .ok_or_else(|| format!("{army_path}.people: missing people"))?,
            &format!("{army_path}.people"),
        )?;
        for (person_index, person) in people.iter().enumerate() {
            visit(person, &format!("{army_path}.people[{person_index}]"))?;
        }
    }
    Ok(())
}

fn visit_records_mut(
    root: &mut Value,
    mut visit: impl FnMut(&mut Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let campaign = object_mut(root, "campaign")?;
    if let Some(people) = campaign.get_mut("people") {
        visit_identity_map_mut(people, "campaign.people", &mut visit)?;
    }
    if let Some(knowledge) = campaign.get_mut("knowledge") {
        let knowledge = object_mut(knowledge, "campaign.knowledge")?;
        if let Some(observers) = knowledge.get_mut("observers") {
            let observers = object_mut(observers, "campaign.knowledge.observers")?;
            for (observer, state) in observers {
                let path = format!("campaign.knowledge.observers[{observer}]");
                let state = object_mut(state, &path)?;
                if let Some(people) = state.get_mut("people") {
                    visit_identity_map_mut(people, &format!("{path}.people"), &mut visit)?;
                }
            }
        }
    }
    if let Some(battles) = campaign.get_mut("battles") {
        let battles = object_mut(battles, "campaign.battles")?;
        for (id, report) in battles {
            visit_report_mut(report, &format!("campaign.battles[{id}]"), &mut visit)?;
        }
    }
    if let Some(pending) = campaign.get_mut("pending_battle") {
        if !pending.is_null() {
            let pending = object_mut(pending, "campaign.pending_battle")?;
            let report = pending
                .get_mut("report")
                .ok_or_else(|| "campaign.pending_battle.report: missing report".to_owned())?;
            visit_report_mut(report, "campaign.pending_battle.report", &mut visit)?;
        }
    }
    Ok(())
}

fn visit_identity_map_mut(
    value: &mut Value,
    path: &str,
    visit: &mut impl FnMut(&mut Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let entries = object_mut(value, path)?;
    for (key, record) in entries {
        let record_path = format!("{path}[{key}]");
        visit(record, &record_path)?;
    }
    Ok(())
}

fn visit_report_mut(
    value: &mut Value,
    path: &str,
    visit: &mut impl FnMut(&mut Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let report = object_mut(value, path)?;
    let attacker = report
        .get_mut("attacker")
        .ok_or_else(|| format!("{path}.attacker: missing battle side"))?;
    visit_side_mut(attacker, &format!("{path}.attacker"), visit)?;
    let defender = object_mut(
        report
            .get_mut("defender")
            .ok_or_else(|| format!("{path}.defender: missing battle defender"))?,
        &format!("{path}.defender"),
    )?;
    match defender.get("kind").and_then(Value::as_str) {
        Some("faction") => {
            let side_path = format!("{path}.defender.side");
            let side = defender
                .get_mut("side")
                .ok_or_else(|| format!("{side_path}: missing faction side"))?;
            visit_side_mut(side, &side_path, visit)
        }
        Some("threat") => Ok(()),
        Some(kind) => Err(format!("{path}.defender.kind: unsupported value {kind}")),
        None => Err(format!("{path}.defender.kind: missing discriminator")),
    }
}

fn visit_side_mut(
    value: &mut Value,
    path: &str,
    visit: &mut impl FnMut(&mut Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let side = object_mut(value, path)?;
    let armies = array_mut(
        side.get_mut("armies")
            .ok_or_else(|| format!("{path}.armies: missing armies"))?,
        &format!("{path}.armies"),
    )?;
    for (army_index, army) in armies.iter_mut().enumerate() {
        let army_path = format!("{path}.armies[{army_index}]");
        let army = object_mut(army, &army_path)?;
        let people = array_mut(
            army.get_mut("people")
                .ok_or_else(|| format!("{army_path}.people: missing people"))?,
            &format!("{army_path}.people"),
        )?;
        for (person_index, person) in people.iter_mut().enumerate() {
            visit(person, &format!("{army_path}.people[{person_index}]"))?;
        }
    }
    Ok(())
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path}: expected an object"))
}

fn object_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Map<String, Value>, String> {
    value
        .as_object_mut()
        .ok_or_else(|| format!("{path}: expected an object"))
}

fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{path}: expected an array"))
}

fn array_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Vec<Value>, String> {
    value
        .as_array_mut()
        .ok_or_else(|| format!("{path}: expected an array"))
}
