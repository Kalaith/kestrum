//! Treatment receipts use people present before recovery, never newly healed medics.

use super::*;
use crate::{
    data::world::FounderClass,
    state::people::{PersonAssignment, PersonStatus},
};

pub(crate) fn recovery_medics(campaign: &StrategicCampaign) -> Vec<(PersonId, FactionId, SiteId)> {
    campaign
        .people
        .values()
        .filter(|person| person.class == FounderClass::Medic && person.status == PersonStatus::Fit)
        .filter_map(|person| {
            let site = match person.assignment {
                PersonAssignment::Site { site } => Some(site),
                PersonAssignment::Formation { formation } => campaign
                    .armies
                    .values()
                    .find(|army| army.formation_ids().any(|id| id == formation))
                    .map(|army| army.site),
                PersonAssignment::Dead => None,
            }?;
            Some((person.id, person.faction, site))
        })
        .collect()
}

pub(crate) fn record_recovery(
    campaign: &mut StrategicCampaign,
    medics: &[(PersonId, FactionId, SiteId)],
) -> Result<(), RuleError> {
    for (id, faction, site) in medics {
        let treated = campaign.factions[faction]
            .last_recovery
            .as_ref()
            .is_some_and(|receipt| {
                receipt.completed_rounds == campaign.completed_rounds
                    && receipt
                        .entries
                        .iter()
                        .any(|entry| entry.site == *site && entry.restored > 0)
            });
        if treated {
            increment(
                &mut campaign
                    .people
                    .get_mut(id)
                    .expect("present recovery medic")
                    .evidence
                    .counts,
                EvidenceKind::TreatedWounded,
            )?;
        }
    }
    Ok(())
}
