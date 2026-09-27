//! Free person healing shares the pre-boundary physical supply snapshot.

use crate::{
    data::world::{FactionId, SiteId},
    engine::{person_site, recovery::SupplySnapshot},
    state::{
        people::{PersonId, PersonStatus},
        StrategicCampaign,
    },
};

pub(in crate::engine) fn heal_wounds(
    campaign: &mut StrategicCampaign,
    supply: &SupplySnapshot,
) -> Vec<(FactionId, SiteId)> {
    let eligible: Vec<PersonId> = campaign
        .people
        .values()
        .filter(|person| {
            matches!(person.status, PersonStatus::Wounded { .. })
                && campaign.is_independent(person.faction)
                && person_site(campaign, person.id).is_some_and(|site| {
                    supply.contains(person.faction, site) && !campaign.sieges.contains_key(&site)
                })
        })
        .map(|person| person.id)
        .collect();
    let mut treated = Vec::new();
    for id in eligible {
        let Some(site) = person_site(campaign, id) else {
            continue;
        };
        let person = campaign
            .people
            .get_mut(&id)
            .expect("eligible wounded person");
        if let PersonStatus::Wounded {
            since_round,
            remaining_steps,
        } = person.status
        {
            treated.push((person.faction, site));
            person.status = if remaining_steps == 1 {
                PersonStatus::Fit
            } else {
                PersonStatus::Wounded {
                    since_round,
                    remaining_steps: remaining_steps - 1,
                }
            };
        }
    }
    treated
}
