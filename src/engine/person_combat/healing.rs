//! Free person healing shares the pre-boundary physical supply snapshot.

use crate::{
    engine::{person_site, recovery::SupplySnapshot},
    state::{
        people::{PersonId, PersonStatus},
        StrategicCampaign,
    },
};

pub(in crate::engine) fn heal_wounds(campaign: &mut StrategicCampaign, supply: &SupplySnapshot) {
    let eligible: Vec<PersonId> = campaign
        .people
        .values()
        .filter(|person| {
            matches!(person.status, PersonStatus::Wounded { .. })
                && campaign.is_independent(person.faction)
                && person_site(campaign, person.id)
                    .is_some_and(|site| supply.contains(person.faction, site))
        })
        .map(|person| person.id)
        .collect();
    for id in eligible {
        let person = campaign
            .people
            .get_mut(&id)
            .expect("eligible wounded person");
        if let PersonStatus::Wounded {
            since_round,
            remaining_steps,
        } = person.status
        {
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
}
