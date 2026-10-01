//! Upgrade stacked legacy rosters without moving people between physical sites.

use crate::state::{people::PersonAssignment, StrategicCampaign};
use std::collections::BTreeMap;

pub(super) fn restore(campaign: &mut StrategicCampaign) {
    let mut members = BTreeMap::<_, Vec<_>>::new();
    for person in campaign.people.values() {
        if let PersonAssignment::Formation { formation } = person.assignment {
            members.entry(formation).or_default().push(person.id);
        }
    }
    let mut overflow = Vec::new();
    for (formation, mut people) in members {
        let Some(army) = campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
        else {
            continue;
        };
        let site = army.site;
        people.sort_by_key(|id| {
            (
                army.commander != Some(*id),
                campaign
                    .formations
                    .get(&formation)
                    .and_then(|unit| unit.battle_leader)
                    != Some(*id),
                !campaign.people[id].career.founding_lord,
                *id,
            )
        });
        for id in people.into_iter().skip(1) {
            campaign
                .people
                .get_mut(&id)
                .expect("legacy member")
                .assignment = PersonAssignment::Site { site };
            overflow.push((id, site));
        }
    }
    for (id, site) in overflow {
        if let Some(formation) =
            campaign.available_person_formation(campaign.people[&id].faction, site)
        {
            campaign
                .people
                .get_mut(&id)
                .expect("legacy member")
                .assignment = PersonAssignment::Formation { formation };
        }
    }
    for unit in campaign.formations.values_mut() {
        if unit.battle_leader.is_some_and(|id| {
            campaign.people.get(&id).is_none_or(|person| {
                person.assignment != (PersonAssignment::Formation { formation: unit.id })
            })
        }) {
            unit.battle_leader = None;
        }
    }
}
