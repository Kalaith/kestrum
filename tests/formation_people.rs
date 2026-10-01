//! Named people share slots with troops, never with another named person.

use kestrum::{
    data::{world::SiteId, GameData},
    engine::{apply, preview, Actor, Command, RuleError},
    state::{
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        Campaign, StrategicCampaign,
    },
};
use std::collections::BTreeSet;

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn legacy(campaign: &StrategicCampaign) -> Campaign {
    let mut value = serde_json::to_value(campaign).unwrap();
    value.as_object_mut().unwrap().remove("roster_version");
    serde_json::from_value(value).unwrap()
}

fn member(campaign: &mut StrategicCampaign, formation: FormationId) -> PersonId {
    let mut person = campaign.people[&PersonId(1)].clone();
    person.id = campaign.next_ids.person;
    campaign.next_ids.person.0 += 1;
    person.name = format!("Companion {}", person.id.0);
    person.career = Default::default();
    person.evidence = Default::default();
    person.assignment = PersonAssignment::Formation { formation };
    let id = person.id;
    campaign.people.insert(id, person);
    id
}

#[test]
fn transfers_and_previews_reject_another_named_person_atomically() {
    let (data, mut campaign) = fixture();
    let person = member(&mut campaign, FormationId(2));
    let before = campaign.clone();
    let command = Command::TransferPerson {
        person,
        to_formation: FormationId(1),
    };
    assert_eq!(
        preview(&campaign, &data, Actor::Player, command.clone()),
        Err(RuleError::PersonSlotOccupied)
    );
    assert_eq!(
        apply(&mut campaign, &data, Actor::Player, command),
        Err(RuleError::PersonSlotOccupied)
    );
    assert_eq!(campaign, before);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person,
            to_formation: FormationId(3),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.formation_person(FormationId(3)).unwrap().id,
        person
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn field_service_requires_a_troops_only_slot() {
    let (data, mut campaign) = fixture();
    let person = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AdoptWard {
            guardian: PersonId(1),
            site: SiteId(1),
        },
    )
    .unwrap()
    .new_people[0];
    campaign.people.get_mut(&person).unwrap().birth_round = -68;
    campaign.people.get_mut(&person).unwrap().class = kestrum::data::world::PersonClass::Recruit;
    let before = campaign.clone();
    let command = Command::EnterService {
        person,
        formation: Some(FormationId(1)),
    };
    assert_eq!(
        apply(&mut campaign, &data, Actor::Player, command),
        Err(RuleError::PersonSlotOccupied)
    );
    assert_eq!(campaign, before);
}

#[test]
fn eight_legacy_members_use_distinct_local_slots_in_multiple_armies() {
    let (data, mut campaign) = fixture();
    for id in [ArmyId(2), ArmyId(3)] {
        let army = campaign.armies.get_mut(&id).unwrap();
        army.site = SiteId(1);
        army.faction = campaign.player;
        army.commander = None;
        for formation in army.formation_ids().collect::<Vec<_>>() {
            campaign.formations.get_mut(&formation).unwrap().faction = campaign.player;
        }
    }
    for id in [PersonId(2), PersonId(3)] {
        campaign.people.get_mut(&id).unwrap().assignment =
            PersonAssignment::Site { site: SiteId(1) };
    }
    for _ in 0..7 {
        member(&mut campaign, FormationId(1));
    }
    let before = campaign.clone();
    let upgraded = legacy(&campaign);
    let restored = upgraded.strategic().unwrap();
    restored.validate(&data).unwrap();
    let assignments: BTreeSet<_> = restored
        .people
        .values()
        .filter(|person| person.faction == restored.player)
        .filter_map(|person| match person.assignment {
            PersonAssignment::Formation { formation } => Some(formation),
            _ => None,
        })
        .collect();
    assert_eq!(assignments.len(), 8);
    assert_eq!(
        restored.armies[&ArmyId(1)].commander,
        before.armies[&ArmyId(1)].commander
    );
    for id in [ArmyId(1), ArmyId(2), ArmyId(3)] {
        assert!(restored.armies[&id]
            .formation_ids()
            .any(|id| restored.formation_person(id).is_some()));
    }
    assert_eq!(restored.formations, before.formations);
    assert_eq!(restored.rng, before.rng);
    assert_eq!(restored.battles, before.battles);
    assert_eq!(restored.next_ids, before.next_ids);
    let roundtrip: Campaign =
        serde_json::from_str(&serde_json::to_string(&upgraded).unwrap()).unwrap();
    assert_eq!(roundtrip, upgraded);
}

#[test]
fn legacy_overflow_waits_at_its_site_without_teleporting_or_creating_troops() {
    let (data, mut campaign) = fixture();
    let original = campaign.people.len();
    for _ in 0..7 {
        member(&mut campaign, FormationId(1));
    }
    let upgraded = legacy(&campaign);
    let restored = upgraded.strategic().unwrap();
    restored.validate(&data).unwrap();
    assert_eq!(restored.people.len(), original + 7);
    assert_eq!(
        restored
            .people
            .values()
            .filter(|person| person.faction == restored.player
                && matches!(person.assignment, PersonAssignment::Formation { .. }))
            .count(),
        3
    );
    assert_eq!(
        restored
            .people
            .values()
            .filter(|person| person.faction == restored.player
                && person.assignment == (PersonAssignment::Site { site: SiteId(1) }))
            .count(),
        5
    );
    assert_eq!(restored.formations, campaign.formations);
    assert_eq!(restored.armies, campaign.armies);
}

#[test]
fn modern_duplicate_assignments_remain_invalid_instead_of_being_migrated() {
    let (data, mut campaign) = fixture();
    member(&mut campaign, FormationId(1));
    assert!(campaign
        .validate(&data)
        .unwrap_err()
        .contains("only one named person"));
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    assert!(loaded.validate(&data).is_err());
}

#[test]
fn disbanding_uses_a_free_person_slot_or_preserves_the_person_at_the_site() {
    let (data, mut campaign) = fixture();
    let companion = member(&mut campaign, FormationId(2));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.people[&PersonId(1)].assignment,
        PersonAssignment::Formation {
            formation: FormationId(3)
        }
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(3),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.people[&PersonId(1)].assignment,
        PersonAssignment::Site { site: SiteId(1) }
    );
    assert_eq!(
        campaign.formation_person(FormationId(2)).unwrap().id,
        companion
    );
    campaign.validate(&data).unwrap();
}
