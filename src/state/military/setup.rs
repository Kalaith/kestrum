//! Authored founding grants create real entities once, only for a new campaign.

use super::{Army, ArmyId, Formation, FormationId};
use crate::{
    data::GameData,
    state::{
        people::{Person, PersonAssignment, PersonId, PersonStatus},
        StrategicCampaign,
    },
};

impl StrategicCampaign {
    pub(crate) fn instantiate_starting_military(&mut self, data: &GameData) -> Result<(), String> {
        let mut setups: Vec<_> = data.scenario.factions.iter().collect();
        setups.sort_by_key(|setup| setup.id);
        for setup in setups {
            let army_id = self.next_ids.army;
            self.next_ids.army = ArmyId(increment(army_id.0)?);
            let mut army = Army {
                id: army_id,
                faction: setup.id,
                site: setup.headquarters,
                name: setup.army_name.clone(),
                slots: [None; 6],
                commander: None,
            };
            let mut founder_formation = None;
            for (slot, kind) in setup.starting_formations.iter().copied().enumerate() {
                let formation_id = self.next_ids.formation;
                self.next_ids.formation = FormationId(increment(formation_id.0)?);
                let capacity = data.economy.formations[&kind].capacity;
                self.formations.insert(
                    formation_id,
                    Formation {
                        service: Default::default(),
                        id: formation_id,
                        faction: setup.id,
                        kind,
                        headcount: capacity,
                        capacity,
                        movement_spent: 0,
                        created_round: 0,
                    },
                );
                army.slots[slot] = Some(formation_id);
                if kind == setup.founder.attached_to {
                    founder_formation = Some(formation_id);
                }
            }
            let person_id = self.next_ids.person;
            self.next_ids.person = PersonId(increment(person_id.0)?);
            self.people.insert(
                person_id,
                Person {
                    evidence: Default::default(),
                    id: person_id,
                    faction: setup.id,
                    name: setup.founder.name.clone(),
                    birth_round: -i64::from(setup.founder.age_years) * 4,
                    service_start_round: 0,
                    class: setup.founder.class,
                    assignment: PersonAssignment::Formation {
                        formation: founder_formation
                            .ok_or("Founder's formation grant is absent")?,
                    },
                    movement_spent: 0,
                    status: PersonStatus::Fit,
                },
            );
            army.commander = setup.founder.commander.then_some(person_id);
            self.armies.insert(army_id, army);
        }
        Ok(())
    }
}

fn increment(value: u32) -> Result<u32, String> {
    value
        .checked_add(1)
        .ok_or_else(|| "campaign.next_ids: identifier space exhausted".into())
}
