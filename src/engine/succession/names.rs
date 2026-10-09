//! Generated names stay distinct among the living without changing the draw.

use crate::{data::GameData, state::StrategicCampaign};
use std::collections::BTreeSet;

/// Draw a given and family name; if a living person already holds that name,
/// step deterministically through the remaining pairs to the next free one.
pub(crate) fn fresh_person_name(campaign: &mut StrategicCampaign, data: &GameData) -> String {
    let names = &data.human_names;
    let given = campaign.rng.people.below(names.given_names.len());
    let family = campaign.rng.people.below(names.family_names.len());
    let taken: BTreeSet<_> = campaign
        .people
        .values()
        .filter(|person| person.is_alive())
        .map(|person| person.name.as_str())
        .collect();
    let families = names.family_names.len();
    let total = names.given_names.len() * families;
    let pair = |index: usize| {
        format!(
            "{} {}",
            names.given_names[index / families],
            names.family_names[index % families]
        )
    };
    let first = given * families + family;
    (0..total)
        .map(|offset| pair((first + offset) % total))
        .find(|name| !taken.contains(name.as_str()))
        .unwrap_or_else(|| pair(first))
}
