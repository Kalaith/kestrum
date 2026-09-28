//! Private application feedback and uninterrupted personal service chronology.

use kestrum::{
    data::{world::SiteId, GameData},
    engine::{action_notices, apply, Actor, Command},
    state::{
        military::ArmyId,
        people::{PersonId, PersonStatus},
        relationships::{FamilyLink, FamilyOrigin, PersonFamily},
        Campaign, StrategicCampaign,
    },
};

#[test]
fn application_announces_only_own_apprentices_and_departures() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let own = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: SiteId(1) },
    )
    .unwrap();
    assert_eq!(
        action_notices(&campaign, &data, campaign.player, &own).len(),
        1
    );
    let rival = *campaign
        .factions
        .keys()
        .find(|id| **id != campaign.player)
        .unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let home = campaign.factions[&rival].headquarters;
    let invited = apply(
        &mut campaign,
        &data,
        Actor::Npc(rival),
        Command::InviteApprentice { site: home },
    )
    .unwrap();
    assert!(!invited.new_people.is_empty());
    assert!(action_notices(&campaign, &data, campaign.player, &invited).is_empty());
    let commander = campaign
        .armies
        .values()
        .find(|army| army.faction == rival)
        .unwrap()
        .commander
        .unwrap();
    let retired = apply(
        &mut campaign,
        &data,
        Actor::Npc(rival),
        Command::RetirePerson {
            person: commander,
            site: home,
        },
    )
    .unwrap();
    assert!(!retired.succession.is_empty());
    assert!(action_notices(&campaign, &data, campaign.player, &retired).is_empty());
    assert!(!action_notices(&campaign, &data, rival, &retired).is_empty());
    let restored: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    restored.validate(&data).unwrap();
}

#[test]
fn later_family_transfers_keep_wounds_evidence_movement_and_original_service() {
    let data = GameData::load().unwrap();
    for origin in [
        FamilyOrigin::Birth,
        FamilyOrigin::AdoptedWard,
        FamilyOrigin::LocalApprentice,
    ] {
        let mut campaign = StrategicCampaign::new(&data).unwrap();
        campaign.completed_rounds = 8;
        campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
        let person = PersonId(1);
        let mut links = std::collections::BTreeMap::new();
        let link = match origin {
            FamilyOrigin::Birth => FamilyLink::BiologicalParent,
            FamilyOrigin::AdoptedWard => FamilyLink::AdoptiveGuardian,
            FamilyOrigin::LocalApprentice => FamilyLink::ApprenticeOf,
        };
        for _ in 0..2 {
            let id = campaign.next_ids.person;
            campaign.next_ids.person = PersonId(id.0 + 1);
            let mut parent = campaign.people[&person].clone();
            parent.id = id;
            parent.birth_round -= 80;
            campaign.people.insert(id, parent);
            if origin == FamilyOrigin::Birth || links.is_empty() {
                links.insert(id, link);
            }
        }
        campaign.people.get_mut(&person).unwrap().class =
            kestrum::data::world::PersonClass::Recruit;
        campaign.families.insert(
            person,
            PersonFamily {
                origin,
                origin_site: SiteId(1),
                household: None,
                links,
            },
        );
        let entry = campaign.people.get_mut(&person).unwrap();
        entry.status = PersonStatus::Wounded {
            since_round: 4,
            remaining_steps: 2,
        };
        entry.movement_spent = 2;
        let before = entry.clone();
        let formations = campaign.armies[&ArmyId(1)]
            .formation_ids()
            .collect::<Vec<_>>();
        for target in [formations[1], formations[0], formations[1]] {
            apply(
                &mut campaign,
                &data,
                Actor::Player,
                Command::TransferPerson {
                    person,
                    to_formation: target,
                },
            )
            .unwrap();
            let current = &campaign.people[&person];
            assert_eq!(current.service_start_round, before.service_start_round);
            assert_eq!(current.evidence, before.evidence);
            assert_eq!(current.status, before.status);
            assert_eq!(current.movement_spent, before.movement_spent);
            let restored: Campaign = serde_json::from_str(
                &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
            )
            .unwrap();
            campaign = restored.strategic().unwrap().clone();
            campaign.validate(&data).unwrap();
        }
    }
}
