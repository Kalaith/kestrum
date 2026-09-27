//! Validated, local transfer of a mundane item to one existing custodian.

use crate::{
    data::world::FactionId,
    engine::{Command, RuleError},
    state::{
        legacy::{person_site, LegacyItemCustody, LegacyItemId},
        people::PersonId,
        StrategicCampaign,
    },
};

pub(super) fn validate(
    campaign: &StrategicCampaign,
    owner: FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    let Command::TransferLegacyItem { item, to } = command else {
        return Ok(());
    };
    let item = campaign
        .legacy_items
        .get(item)
        .filter(|item| item.faction == owner)
        .ok_or_else(|| error("That heirloom is not available to this faction."))?;
    let recipient = campaign
        .people
        .get(to)
        .filter(|person| person.faction == owner && person.is_alive())
        .ok_or_else(|| error("Choose a living person of this faction."))?;
    let site = person_site(campaign, *to)
        .ok_or_else(|| error("The recipient needs a current physical location."))?;
    let custody_site = match item.custody {
        LegacyItemCustody::Person(holder) => {
            let current = campaign
                .people
                .get(&holder)
                .filter(|person| person.is_alive() && person.faction == owner)
                .ok_or_else(|| error("The current holder is no longer available."))?;
            if holder == *to {
                return Err(error("That person already holds this heirloom."));
            }
            person_site(campaign, current.id)
                .ok_or_else(|| error("The current holder has no physical location."))?
        }
        LegacyItemCustody::SiteEstate(estate) => {
            if campaign
                .world
                .site(estate)
                .is_none_or(|place| place.controller != Some(owner))
            {
                return Err(error(
                    "Recover an estate heirloom from a site your faction controls.",
                ));
            }
            estate
        }
    };
    if custody_site != site || recipient.faction != item.faction {
        return Err(error(
            "Heirlooms can move only between local faction members.",
        ));
    }
    Ok(())
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    item: LegacyItemId,
    to: PersonId,
) -> Result<(), RuleError> {
    let entry = campaign
        .legacy_items
        .get_mut(&item)
        .filter(|item| item.faction == owner)
        .ok_or_else(|| error("That heirloom is not available to this faction."))?;
    entry.custody = LegacyItemCustody::Person(to);
    Ok(())
}

fn error(reason: &str) -> RuleError {
    RuleError::Legacy(reason.into())
}
