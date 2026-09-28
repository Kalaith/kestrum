//! Peaceful service establishes familiarity without inventing combat experience.
use kestrum::{
    data::{world::SiteId, GameData},
    engine::{apply, Actor, Command, MoveOrder},
    state::{
        military::ArmyId,
        people::{PersonAssignment, PersonId, PersonStatus},
        Campaign, StrategicCampaign,
    },
};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn colleague(campaign: &mut StrategicCampaign, assignment: PersonAssignment) -> PersonId {
    let id = campaign.next_ids.person;
    campaign.next_ids.person.0 += 1;
    let mut person = campaign.people[&PersonId(1)].clone();
    person.id = id;
    person.name = format!("Colleague {}", id.0);
    person.assignment = assignment;
    person.career = Default::default();
    person.evidence = Default::default();
    campaign.people.insert(id, person);
    id
}

fn season(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

#[test]
fn four_stationary_seasons_enable_a_real_household_across_a_reload() {
    let (data, mut campaign) = fixture();
    let second = colleague(&mut campaign, PersonAssignment::Site { site: SiteId(1) });
    let command = Command::FormHousehold {
        first: PersonId(1),
        second,
        site: SiteId(1),
    };
    for round in 0..4 {
        assert!(apply(&mut campaign, &data, Actor::Player, command.clone()).is_err());
        season(&mut campaign, &data);
        assert_eq!(
            campaign.people[&second].career.relationships[&PersonId(1)].shared_service_seasons,
            round + 1
        );
        let loaded: Campaign = serde_json::from_str(
            &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
        )
        .unwrap();
        campaign = loaded.strategic().unwrap().clone();
    }
    apply(&mut campaign, &data, Actor::Player, command).unwrap();
    assert_eq!(campaign.households.len(), 1);
    assert!(campaign.people[&second].evidence.counts.is_empty());
    assert_eq!(
        campaign.people[&second].career.relationships[&PersonId(1)].mutual_combat_rounds,
        0
    );
}

#[test]
fn remote_sites_and_pre_service_dependents_do_not_receive_stationary_credit() {
    let (data, mut campaign) = fixture();
    let remote = colleague(&mut campaign, PersonAssignment::Site { site: SiteId(5) });
    let apprentice = apply(
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
    season(&mut campaign, &data);
    assert!(campaign.people[&remote].career.relationships.is_empty());
    assert!(campaign.people[&apprentice].career.relationships.is_empty());
}

#[test]
fn movement_and_stationary_contact_share_one_season_stamp() {
    let (data, mut campaign) = fixture();
    let assignment = campaign.people[&PersonId(1)].assignment;
    let second = colleague(&mut campaign, assignment);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    season(&mut campaign, &data);
    let relation = &campaign.people[&second].career.relationships[&PersonId(1)];
    assert_eq!(
        (
            relation.shared_service_seasons,
            relation.last_shared_service_round
        ),
        (1, Some(0))
    );
}

#[test]
fn wounded_and_retired_people_are_not_counted_as_active_shared_service() {
    let (data, mut campaign) = fixture();
    let wounded = colleague(&mut campaign, PersonAssignment::Site { site: SiteId(1) });
    campaign.people.get_mut(&wounded).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 2,
    };
    let retired = colleague(&mut campaign, PersonAssignment::Site { site: SiteId(1) });
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RetirePerson {
            person: retired,
            site: SiteId(1),
        },
    )
    .unwrap();
    season(&mut campaign, &data);
    assert!(campaign.people[&wounded].career.relationships.is_empty());
    assert!(campaign.people[&retired].career.relationships.is_empty());
}

#[test]
fn stationary_contact_retains_symmetric_bounded_relationships() {
    let (mut data, mut campaign) = fixture();
    data.progression.relationships_per_person = 2;
    for _ in 0..4 {
        colleague(&mut campaign, PersonAssignment::Site { site: SiteId(1) });
    }
    season(&mut campaign, &data);
    for person in campaign.people.values() {
        assert!(person.career.relationships.len() <= 2);
        for (other, relation) in &person.career.relationships {
            assert_eq!(
                relation,
                &campaign.people[other].career.relationships[&person.id]
            );
        }
    }
    campaign.validate(&data).unwrap();
}
