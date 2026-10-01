//! Removing a formation preserves people and never reuses a military identity.

use super::FormationId;
use crate::state::{
    people::{PersonAssignment, PersonStatus},
    StrategicCampaign,
};

impl StrategicCampaign {
    /// Used after disbanding or resolved zero-headcount destruction. This also
    /// accepts the transient zero record, which is invalid in a committed save.
    pub fn remove_formation(&mut self, id: FormationId) -> Result<(), String> {
        let formation = self.formations.get(&id).ok_or("Unknown formation")?;
        let mut owners = self
            .armies
            .values()
            .filter(|army| army.formation_ids().any(|entry| entry == id));
        let army = owners
            .next()
            .ok_or("Formation is not assigned to an army")?;
        if owners.next().is_some() || army.faction != formation.faction {
            return Err("Formation has inconsistent army membership".into());
        }
        let (army_id, faction, site) = (army.id, army.faction, army.site);
        let mut candidate = self.clone();
        candidate.formations.remove(&id);
        let army = candidate.armies.get_mut(&army_id).ok_or("Unknown army")?;
        for slot in &mut army.slots {
            if *slot == Some(id) {
                *slot = None;
            }
        }
        if army.is_empty() {
            candidate.armies.remove(&army_id);
        }
        let recipient = candidate.available_person_formation(faction, site);
        let assignment = recipient.map_or(PersonAssignment::Site { site }, |formation| {
            PersonAssignment::Formation { formation }
        });
        for person in candidate.people.values_mut() {
            if person.assignment == (PersonAssignment::Formation { formation: id }) {
                person.assignment = assignment;
            }
        }
        for army in candidate.armies.values_mut() {
            let valid = army
                .commander
                .and_then(|id| candidate.people.get(&id))
                .is_some_and(|person| {
                    person.faction == army.faction
                        && person.status == PersonStatus::Fit
                        && matches!(person.assignment, PersonAssignment::Formation { formation }
                        if army.formation_ids().any(|id| id == formation))
                });
            if !valid {
                army.commander = None;
            }
        }
        candidate.reconcile_movement_plans();
        *self = candidate;
        Ok(())
    }
}
