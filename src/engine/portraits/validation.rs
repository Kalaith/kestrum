//! Campaign-wide identity and reservation consistency checks for portraits.

use crate::{
    data::portraits::{AppearanceDescriptor, PortraitCatalog, PortraitCatalogIndex},
    state::{
        battle::{BattleArmyReport, BattleDefender, BattleReport},
        campaign::StrategicCampaign,
        knowledge::CampaignKnowledge,
        notifications::{
            NotificationDetail, NotificationSubjectSnapshot, PersonNotificationSnapshot,
        },
        people::PersonId,
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// Validate every retained portrait against the catalog and its campaign reservation.
pub fn validate_campaign(
    campaign: &StrategicCampaign,
    catalog: &PortraitCatalog,
) -> Result<(), String> {
    catalog.validate()?;
    let index = catalog.validation_index()?;
    campaign
        .appearance_registry
        .validate_with_index(campaign.seed, catalog, &index)?;
    validate_reservations(campaign)?;

    let mut appearances = BTreeMap::new();
    for (key, person) in &campaign.people {
        if *key != person.id {
            return Err(format!(
                "people[{}]: map key does not match person ID",
                key.0
            ));
        }
        validate_identity(
            person.id,
            &person.appearance,
            "people",
            campaign,
            &index,
            &mut appearances,
        )?;
    }
    validate_knowledge(&campaign.knowledge, campaign, &index, &mut appearances)?;
    for (id, report) in &campaign.battles {
        validate_report(
            report,
            &format!("battles[{}]", id.0),
            campaign,
            &index,
            &mut appearances,
        )?;
    }
    if let Some(pending) = &campaign.pending_battle {
        validate_report(
            &pending.report,
            "pending_battle.report",
            campaign,
            &index,
            &mut appearances,
        )?;
    }
    validate_notifications(campaign, &index, &mut appearances)?;
    validate_retained_reservations(campaign, &appearances)?;
    Ok(())
}

fn validate_reservations(campaign: &StrategicCampaign) -> Result<(), String> {
    let next_person = campaign.next_ids.person.0;
    if next_person == 0 {
        return Err("appearance_registry: next person ID must be nonzero".into());
    }
    let allocated_limit = u64::from(next_person - 1);
    let mut allocation_count = 0_u64;
    for (signature, reservation) in &campaign.appearance_registry.reservations {
        if reservation.first_person.0 >= next_person
            || reservation.last_person.0 >= next_person
            || reservation.first_person > reservation.last_person
            || reservation.last_round > campaign.completed_rounds
        {
            return Err(format!(
                "appearance_registry.reservations[{signature}]: person or round counter is outside campaign history"
            ));
        }
        allocation_count = allocation_count
            .checked_add(u64::from(reservation.reuse_count))
            .ok_or_else(|| "appearance_registry: allocation total overflow".to_owned())?;
    }
    if allocation_count > allocated_limit {
        return Err(
            "appearance_registry: reservation allocation count exceeds issued person IDs".into(),
        );
    }
    Ok(())
}

fn validate_identity(
    id: PersonId,
    descriptor: &AppearanceDescriptor,
    path: &str,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    if id.0 == 0 || id.0 >= campaign.next_ids.person.0 {
        return Err(format!(
            "{path}[{}]: person ID is outside campaign identity history",
            id.0
        ));
    }
    index
        .validate_descriptor(descriptor)
        .map_err(|error| format!("{path}[{}].appearance: {error}", id.0))?;
    let canonical = index
        .signature_for(descriptor)
        .map_err(|error| format!("{path}[{}].appearance: {error}", id.0))?;
    if canonical != descriptor.signature
        || !campaign
            .appearance_registry
            .reservations
            .contains_key(&descriptor.signature)
    {
        return Err(format!(
            "{path}[{}].appearance: signature is not reserved by this campaign",
            id.0
        ));
    }
    if let Some(previous) = appearances.insert(id, descriptor.clone()) {
        if previous != *descriptor {
            return Err(format!(
                "appearance: retained copies of person {} disagree",
                id.0
            ));
        }
    }
    Ok(())
}

fn validate_retained_reservations(
    campaign: &StrategicCampaign,
    appearances: &BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    let mut retained = BTreeMap::<&str, BTreeSet<PersonId>>::new();
    for (id, descriptor) in appearances {
        let reservation = campaign
            .appearance_registry
            .reservations
            .get(&descriptor.signature)
            .ok_or_else(|| format!("appearance[{}]: missing signature reservation", id.0))?;
        if *id < reservation.first_person || *id > reservation.last_person {
            return Err(format!(
                "appearance[{}]: person is outside the signature reservation range",
                id.0
            ));
        }
        retained
            .entry(&descriptor.signature)
            .or_default()
            .insert(*id);
    }
    for (signature, ids) in retained {
        let reservation = campaign
            .appearance_registry
            .reservations
            .get(signature)
            .ok_or_else(|| {
                format!("appearance_registry.reservations[{signature}]: missing reservation")
            })?;
        let retained_count = u64::try_from(ids.len())
            .map_err(|_| "appearance: retained identity count overflow".to_owned())?;
        if retained_count > u64::from(reservation.reuse_count) {
            return Err(format!(
                "appearance_registry.reservations[{signature}]: retained identities exceed recorded uses"
            ));
        }
    }
    Ok(())
}

fn validate_knowledge(
    knowledge: &CampaignKnowledge,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    for observer in knowledge.observers.values() {
        for (key, person) in &observer.people {
            if *key != person.id {
                return Err(format!(
                    "knowledge.observers.people[{}]: map key does not match person ID",
                    key.0
                ));
            }
            validate_identity(
                person.id,
                &person.appearance,
                "knowledge.observers.people",
                campaign,
                index,
                appearances,
            )?;
        }
    }
    Ok(())
}

fn validate_report(
    report: &BattleReport,
    path: &str,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    validate_side_people(
        &report.attacker.armies,
        &format!("{path}.attacker"),
        campaign,
        index,
        appearances,
    )?;
    if let BattleDefender::Faction(side) = &report.defender {
        validate_side_people(
            &side.armies,
            &format!("{path}.defender"),
            campaign,
            index,
            appearances,
        )?;
    }
    Ok(())
}

fn validate_side_people(
    armies: &[BattleArmyReport],
    path: &str,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    for (army_index, army) in armies.iter().enumerate() {
        for person in &army.people {
            validate_identity(
                person.id,
                &person.appearance,
                &format!("{path}.armies[{army_index}].people"),
                campaign,
                index,
                appearances,
            )?;
        }
    }
    Ok(())
}

fn validate_notifications(
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    for receipt in &campaign.notifications.receipts {
        if let Some(subject) = &receipt.subject {
            validate_notification_subject(
                subject,
                &format!("notifications.receipts[{}].subject", receipt.id.0),
                campaign,
                index,
                appearances,
            )?;
        }
        match &receipt.detail {
            NotificationDetail::Person { person, .. } => validate_notification_person(
                person,
                &format!("notifications.receipts[{}].detail.person", receipt.id.0),
                campaign,
                index,
                appearances,
            )?,
            NotificationDetail::Remembrance { subject, .. } => validate_notification_subject(
                subject,
                &format!("notifications.receipts[{}].detail.subject", receipt.id.0),
                campaign,
                index,
                appearances,
            )?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_notification_subject(
    subject: &NotificationSubjectSnapshot,
    path: &str,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    if let NotificationSubjectSnapshot::Person(person) = subject {
        validate_notification_person(person, path, campaign, index, appearances)?;
    }
    Ok(())
}

fn validate_notification_person(
    person: &PersonNotificationSnapshot,
    path: &str,
    campaign: &StrategicCampaign,
    index: &PortraitCatalogIndex<'_>,
    appearances: &mut BTreeMap<PersonId, AppearanceDescriptor>,
) -> Result<(), String> {
    if let Some(descriptor) = &person.appearance {
        validate_identity(person.id, descriptor, path, campaign, index, appearances)?;
    }
    Ok(())
}
