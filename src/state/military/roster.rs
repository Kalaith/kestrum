//! One named person shares each formation slot with its ordinary troops.

use super::FormationId;
use crate::{
    data::world::{FactionId, SiteId},
    state::{
        people::{Person, PersonAssignment},
        StrategicCampaign,
    },
};

impl StrategicCampaign {
    pub fn formation_person(&self, formation: FormationId) -> Option<&Person> {
        self.people
            .values()
            .find(|person| person.assignment == (PersonAssignment::Formation { formation }))
    }

    /// Prefer the least staffed local army, then its first unstaffed formation.
    pub fn available_person_formation(
        &self,
        faction: FactionId,
        site: SiteId,
    ) -> Option<FormationId> {
        self.armies
            .values()
            .filter(|army| army.faction == faction && army.site == site)
            .filter_map(|army| {
                let formation = army.formation_ids().find(|id| {
                    self.formations
                        .get(id)
                        .is_some_and(|unit| unit.headcount > 0)
                        && self.formation_person(*id).is_none()
                })?;
                let staffed = army
                    .formation_ids()
                    .filter(|id| self.formation_person(*id).is_some())
                    .count();
                Some((staffed, army.id, formation))
            })
            .min()
            .map(|(_, _, formation)| formation)
    }
}
