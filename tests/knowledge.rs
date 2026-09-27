//! K08 visibility is enforced by the same services used by reports, maps and links.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/relations.rs"]
mod relations;
use relations::sync_relations;

use kestrum::{
    data::{
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{
        apply, battle_reports, hostile_presence, known_people, movement_preview, person_knowledge,
        project, Actor, Command, MoveOrder, PersonKnowledge,
    },
    state::{
        battle::BattleId,
        knowledge::ObservedCondition,
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId, PersonStatus},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use std::collections::BTreeSet;

#[path = "support/knowledge.rs"]
mod support;
use support::*;

#[test]
fn precontact_exposes_only_local_hostile_presence_and_public_route_information() {
    let (data, mut campaign) = fixture();
    assert_eq!(
        hostile_presence(&campaign, FactionId(1)),
        BTreeSet::from([SiteId(10)])
    );
    let visible = project(&campaign, FactionId(1)).unwrap();
    assert!(visible
        .armies
        .iter()
        .all(|army| army.faction == FactionId(1)));
    assert!(visible
        .formations
        .iter()
        .all(|formation| formation.faction == FactionId(1)));
    assert!(visible
        .people
        .iter()
        .all(|person| person.faction == FactionId(1)));
    assert!(visible.battles.is_empty());
    assert!(person_knowledge(&campaign, FactionId(1), PersonId(3)).is_none());
    let preview =
        movement_preview(&campaign, &data, FactionId(1), &[ArmyId(1)], SiteId(10)).unwrap();
    assert_eq!(preview.observed_hostile_sites, BTreeSet::from([SiteId(10)]));
    campaign.people.get_mut(&PersonId(3)).unwrap().name = "Private new name".into();
    campaign.armies.get_mut(&ArmyId(3)).unwrap().name = "Private army name".into();
    campaign
        .formations
        .get_mut(&FormationId(7))
        .unwrap()
        .headcount = 1;
    campaign
        .factions
        .get_mut(&FactionId(3))
        .unwrap()
        .resources
        .gold += 99;
    assert_eq!(project(&campaign, FactionId(1)).unwrap(), visible);
    assert_eq!(
        movement_preview(&campaign, &data, FactionId(1), &[ArmyId(1)], SiteId(10)).unwrap(),
        preview
    );
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(8);
    assert_eq!(
        hostile_presence(&campaign, FactionId(1)),
        BTreeSet::from([SiteId(8)])
    );
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(3);
    assert!(hostile_presence(&campaign, FactionId(1)).is_empty());
    // A distant destination does not gain future sight along the previewed path.
    let remote = movement_preview(&campaign, &data, FactionId(1), &[ArmyId(1)], SiteId(3)).unwrap();
    assert!(remote.observed_hostile_sites.is_empty());
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(8);
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(3)])
        .unwrap()
        .state = DiplomaticState::Peace;
    sync_relations(&mut campaign);
    assert!(hostile_presence(&campaign, FactionId(1)).is_empty());
    assert!(hostile_presence(&campaign, FactionId(99)).is_empty());
}

#[test]
fn an_actual_battle_reveals_only_participating_people_to_its_two_observers() {
    let (data, mut campaign) = fixture();
    let report = fight(&mut campaign, &data);
    let observed = snapshot(&campaign, FactionId(1), PersonId(3));
    assert_eq!(observed.id, PersonId(3));
    assert_eq!(observed.name, "Enemy witness 003");
    assert_eq!(observed.army, ArmyId(3));
    assert_eq!(observed.army_name, report.defender.armies()[0].name);
    assert_eq!(
        (observed.battle, observed.completed_rounds, observed.site),
        (report.id, 0, SiteId(10))
    );
    assert_eq!(observed.condition, ObservedCondition::Fit);
    assert_eq!(
        snapshot(&campaign, FactionId(3), PersonId(1)).name,
        "Own witness"
    );
    assert!(person_knowledge(&campaign, FactionId(1), PersonId(5)).is_none());
    assert!(person_knowledge(&campaign, FactionId(2), PersonId(3)).is_none());
    assert!(person_knowledge(&campaign, FactionId(99), PersonId(3)).is_none());
    assert!(matches!(
        person_knowledge(&campaign, FactionId(1), PersonId(1)),
        Some(PersonKnowledge::CurrentOwn(_))
    ));
    assert_eq!(
        battle_reports(&campaign, FactionId(1)),
        vec![report.clone()]
    );
    assert_eq!(battle_reports(&campaign, FactionId(3)), vec![report]);
    assert!(battle_reports(&campaign, FactionId(2)).is_empty());
    campaign.validate(&data).unwrap();
    assert_corrupt_snapshots(&data, &campaign);
}

#[test]
fn last_encounter_stays_unchanged_through_hidden_changes_until_real_contact() {
    let (data, mut campaign) = fixture();
    fight(&mut campaign, &data);
    let original = campaign.clone();
    let knowledge = person_knowledge(&campaign, FactionId(1), PersonId(3));
    let report = battle_reports(&campaign, FactionId(1));
    let remote = campaign.people.get_mut(&PersonId(3)).unwrap();
    remote.name = "Unseen later identity".into();
    remote.assignment = PersonAssignment::Site { site: SiteId(3) };
    remote.status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 1,
    };
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(11);
    campaign
        .formations
        .get_mut(&FormationId(7))
        .unwrap()
        .headcount = 37;
    campaign.validate(&data).unwrap();
    assert_eq!(
        person_knowledge(&campaign, FactionId(1), PersonId(3)),
        knowledge
    );
    let remote = campaign.people.get_mut(&PersonId(3)).unwrap();
    remote.status = PersonStatus::Dead {
        completed_rounds: 0,
        site: SiteId(3),
    };
    remote.assignment = PersonAssignment::Dead;
    remote.movement_spent = 0;
    campaign.validate(&data).unwrap();
    assert_eq!(
        person_knowledge(&campaign, FactionId(1), PersonId(3)),
        knowledge
    );
    assert_eq!(battle_reports(&campaign, FactionId(1)), report);
    assert_eq!(known_people(&campaign, FactionId(1), "Unseen", 0).total, 0);
    // A separate continuation meets the same living person again after a season.
    let mut reunion = original;
    reunion.people.get_mut(&PersonId(3)).unwrap().name = "Witness now recognized".into();
    finish_round(&mut reunion, &data);
    let next = fight(&mut reunion, &data);
    let observed = snapshot(&reunion, FactionId(1), PersonId(3));
    assert_eq!(observed.name, "Witness now recognized");
    assert_eq!((observed.battle, observed.completed_rounds), (next.id, 1));
    assert_eq!(reunion.battles[&BattleId(1)], report[0]);
}

#[test]
fn linked_views_and_bounded_search_resolve_only_own_or_witnessed_identity() {
    let (data, mut campaign) = fixture();
    for id in 6..60 {
        let mut person = campaign.people[&PersonId(3)].clone();
        person.id = PersonId(id);
        person.name = format!("Enemy witness {id:03}");
        campaign.people.insert(person.id, person);
    }
    campaign.next_ids.person = PersonId(60);
    fight(&mut campaign, &data);
    let first = known_people(&campaign, FactionId(1), " enemy WITNESS ", 0);
    let second = known_people(&campaign, FactionId(1), "Enemy witness", 1);
    assert_eq!(
        (
            first.total,
            first.people.len(),
            second.total,
            second.people.len()
        ),
        (55, 50, 55, 5)
    );
    assert!(first
        .people
        .iter()
        .all(|person| matches!(person, PersonKnowledge::LastEncountered { .. })));
    assert_eq!(first.people[0].id(), PersonId(3));
    assert_eq!(second.people.last().unwrap().id(), PersonId(59));
    let before = campaign.clone();
    assert!(known_people(&campaign, FactionId(1), "Hidden career", 0)
        .people
        .is_empty());
    assert!(known_people(&campaign, FactionId(1), "", usize::MAX)
        .people
        .is_empty());
    assert!(known_people(&campaign, FactionId(2), "Enemy witness", 0)
        .people
        .is_empty());
    assert!(person_knowledge(&campaign, FactionId(1), PersonId(999)).is_none());
    for _ in 0..3 {
        assert!(matches!(
            person_knowledge(&campaign, FactionId(1), PersonId(3)),
            Some(PersonKnowledge::LastEncountered {
                available_report: Some(BattleId(1)),
                ..
            })
        ));
    }
    assert_eq!(campaign, before);
    assert_history_visibility(&data, &campaign);
}

#[test]
fn forgetting_reports_preserves_dated_labels_and_loadable_bounded_knowledge() {
    let (data, mut campaign) = fixture();
    for invalid in ["age", "count", "page", "departed_age", "departed_count"] {
        let mut malformed = data.clone();
        match invalid {
            "age" => malformed.history.knowledge_max_age_rounds = 81,
            "count" => malformed.history.knowledge_max_entries = 10_001,
            "page" => malformed.history.page_size = 0,
            "departed_age" => malformed.history.departed_max_age_rounds = 81,
            _ => malformed.history.departed_max_entries = 2_001,
        }
        assert!(malformed.validate().is_err(), "{invalid}");
    }
    fight(&mut campaign, &data);
    assert_earlier_save(&data, &campaign);
    let mut count_limited = campaign.clone();
    let mut limited = data.clone();
    limited.history.knowledge_max_entries = 1;
    finish_round(&mut count_limited, &limited);
    assert!(person_knowledge(&count_limited, FactionId(1), PersonId(3)).is_none());
    assert!(person_knowledge(&count_limited, FactionId(3), PersonId(1)).is_some());
    // Advance real seasonal boundaries so recent service and history prune together.
    finish_round(&mut campaign, &data);
    assert_expired_earlier_save(&data, &campaign);
    let expected = snapshot(&campaign, FactionId(1), PersonId(3));
    while campaign.completed_rounds < 41 {
        finish_round(&mut campaign, &data);
    }
    assert!(campaign.battles.is_empty());
    assert_eq!(snapshot(&campaign, FactionId(1), PersonId(3)), expected);
    assert!(matches!(
        person_knowledge(&campaign, FactionId(1), PersonId(3)),
        Some(PersonKnowledge::LastEncountered {
            available_report: None,
            ..
        })
    ));
    campaign.people.remove(&PersonId(3));
    campaign.validate(&data).unwrap();
    assert_eq!(snapshot(&campaign, FactionId(1), PersonId(3)), expected);
    assert_catalogue(&data, &campaign);
    while campaign.completed_rounds < 80 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(snapshot(&campaign, FactionId(1), PersonId(3)), expected);
    finish_round(&mut campaign, &data);
    assert!(person_knowledge(&campaign, FactionId(1), PersonId(3)).is_none());
    assert!(campaign.knowledge.observers.is_empty());
    assert_catalogue(&data, &campaign);
    assert_departed_budgets(&data);
}
