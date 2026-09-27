//! Freeze actual travellers before casualties or later reassignment can change them.

use super::{MovementOutcome, StrategicCampaign};
use crate::state::{evidence::MovementService, people::PersonAssignment};
use std::collections::BTreeSet;

pub(in crate::engine) fn service_snapshot(
    before: &StrategicCampaign,
    moved: &MovementOutcome,
) -> MovementService {
    let formations = moved
        .armies
        .iter()
        .filter_map(|id| before.armies.get(id))
        .flat_map(|army| army.formation_ids())
        .collect::<BTreeSet<_>>();
    let people = before
        .people
        .values()
        .filter(|person| {
            person.is_alive()
                && matches!(person.assignment,
        PersonAssignment::Formation{formation} if formations.contains(&formation))
        })
        .map(|person| person.id)
        .collect();
    let routes = moved
        .path
        .windows(2)
        .filter_map(|pair| before.world.connected_route(pair[0], pair[1]))
        .map(|route| route.id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    MovementService {
        formations: formations.into_iter().collect(),
        people,
        routes,
    }
}
