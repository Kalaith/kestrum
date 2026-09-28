//! K15 households retain real kinship and appoint qualified successors without copying lives.

use kestrum::{
    data::{
        progression::TrainingDiscipline,
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{ai, apply, Actor, Command},
    state::{
        people::{
            CompletedApprenticeship, PersonAssignment, PersonId, PersonRelationship, PersonStatus,
        },
        relationships::{
            FamilyLink, FamilyOrigin, HouseholdId, LegacyCategory, PersonFamily, SuccessorLink,
        },
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;
use std::collections::BTreeMap;

const HOME: SiteId = SiteId(1);
const OWNER: FactionId = FactionId(1);
const RIVAL: FactionId = FactionId(2);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn add_person(
    campaign: &mut StrategicCampaign,
    age: u32,
    assignment: PersonAssignment,
) -> PersonId {
    add_person_for(campaign, OWNER, age, assignment)
}

fn add_person_for(
    campaign: &mut StrategicCampaign,
    faction: FactionId,
    age: u32,
    assignment: PersonAssignment,
) -> PersonId {
    let id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(id.0 + 1);
    let mut person = campaign
        .people
        .values()
        .find(|person| person.faction == faction)
        .unwrap()
        .clone();
    person.id = id;
    person.faction = faction;
    person.name = format!("K15 Person {}", id.0);
    person.birth_round = i64::from(campaign.completed_rounds) - i64::from(age) * 4;
    person.service_start_round = 0;
    person.class = PersonClass::Recruit;
    person.assignment = assignment;
    person.movement_spent = 0;
    person.status = PersonStatus::Fit;
    person.career = Default::default();
    person.evidence = Default::default();
    campaign.people.insert(id, person);
    id
}

fn set_shared_seasons(
    campaign: &mut StrategicCampaign,
    first: PersonId,
    second: PersonId,
    count: u32,
) {
    let fact = PersonRelationship {
        shared_service_seasons: count,
        last_shared_service_round: (count > 0).then_some(campaign.completed_rounds),
        mutual_combat_rounds: 0,
        last_mutual_combat_round: None,
    };
    campaign
        .people
        .get_mut(&first)
        .unwrap()
        .career
        .relationships
        .insert(second, fact.clone());
    campaign
        .people
        .get_mut(&second)
        .unwrap()
        .career
        .relationships
        .insert(first, fact);
}

fn form_pair(data: &GameData, campaign: &mut StrategicCampaign) -> (PersonId, PersonId) {
    let first = add_person(campaign, 25, PersonAssignment::Site { site: HOME });
    let second = add_person(campaign, 27, PersonAssignment::Site { site: HOME });
    set_shared_seasons(campaign, first, second, 4);
    apply(
        campaign,
        data,
        Actor::Player,
        Command::FormHousehold {
            first,
            second,
            site: HOME,
        },
    )
    .unwrap();
    (first, second)
}

fn complete_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

fn prepared_birth(data: &GameData, seed: u64) -> (StrategicCampaign, HouseholdId) {
    let mut campaign = StrategicCampaign::new(data).unwrap();
    campaign.rng.people = SeededRng::new(seed);
    form_pair(data, &mut campaign);
    let household = *campaign.households.keys().next().unwrap();
    apply(
        &mut campaign,
        data,
        Actor::Player,
        Command::SetHouseholdChildraising {
            household,
            enabled: true,
        },
    )
    .unwrap();
    (campaign, household)
}

#[test]
fn household_formation_requires_four_recorded_seasons_and_is_atomic() {
    let (data, mut campaign) = fixture();
    let first = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let second = add_person(&mut campaign, 27, PersonAssignment::Site { site: HOME });
    set_shared_seasons(&mut campaign, first, second, 3);
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::FormHousehold {
            first,
            second,
            site: HOME
        },
    )
    .is_err());
    assert_eq!(campaign, before);

    set_shared_seasons(&mut campaign, first, second, 4);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::FormHousehold {
            first,
            second,
            site: HOME,
        },
    )
    .unwrap();
    campaign.validate(&data).unwrap();
    assert_eq!(campaign.households.len(), 1);
}

#[test]
fn parent_child_and_shared_guardian_pairs_cannot_form_households() {
    let (data, mut campaign) = fixture();
    let guardian = add_person(&mut campaign, 50, PersonAssignment::Site { site: HOME });
    let first_ward = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let second_ward = add_person(&mut campaign, 24, PersonAssignment::Site { site: HOME });
    for ward in [first_ward, second_ward] {
        campaign.families.insert(
            ward,
            PersonFamily {
                origin: FamilyOrigin::AdoptedWard,
                origin_site: HOME,
                household: None,
                links: BTreeMap::from([(guardian, FamilyLink::AdoptiveGuardian)]),
            },
        );
        set_shared_seasons(&mut campaign, guardian, ward, 4);
    }
    set_shared_seasons(&mut campaign, first_ward, second_ward, 4);

    for (first, second) in [(guardian, first_ward), (first_ward, second_ward)] {
        let before = campaign.clone();
        assert!(apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::FormHousehold {
                first,
                second,
                site: HOME,
            },
        )
        .is_err());
        assert_eq!(campaign, before);
    }
}

#[test]
fn adopted_ward_uses_one_population_unit_and_receives_no_inherited_records() {
    let (data, mut campaign) = fixture();
    let guardian = add_person(&mut campaign, 28, PersonAssignment::Site { site: HOME });
    let population = campaign.world.population[&HOME];
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AdoptWard {
            guardian,
            site: HOME,
        },
    )
    .unwrap();
    let ward = outcome.new_people[0];
    let person = &campaign.people[&ward];
    assert_eq!(campaign.world.population[&HOME], population - 1);
    assert_eq!(person.age_years(campaign.completed_rounds), 8);
    assert_eq!(person.class, PersonClass::Recruit);
    assert_eq!(
        person.assignment,
        PersonAssignment::Dependent { site: HOME }
    );
    assert!(person.career == Default::default());
    assert!(person.evidence.counts.is_empty());
    assert_eq!(campaign.families[&ward].origin, FamilyOrigin::AdoptedWard);
    assert_eq!(
        campaign.families[&ward].links[&guardian],
        FamilyLink::AdoptiveGuardian
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn local_training_and_field_service_keep_their_separate_age_gates() {
    let (data, mut campaign) = fixture();
    let guardian = add_person(&mut campaign, 28, PersonAssignment::Site { site: HOME });
    let ward = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AdoptWard {
            guardian,
            site: HOME,
        },
    )
    .unwrap()
    .new_people[0];
    campaign.people.get_mut(&ward).unwrap().birth_round = -64;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AssignTrainee {
            person: ward,
            site: HOME,
        },
    )
    .unwrap();
    assert_eq!(
        campaign.people[&ward].assignment,
        PersonAssignment::Trainee { site: HOME }
    );
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EnterService {
            person: ward,
            formation: None
        },
    )
    .is_err());

    campaign.people.get_mut(&ward).unwrap().birth_round = -68;
    campaign.people.get_mut(&ward).unwrap().assignment = PersonAssignment::Dependent { site: HOME };
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EnterService {
            person: ward,
            formation: None,
        },
    )
    .unwrap();
    assert_eq!(
        campaign.people[&ward].assignment,
        PersonAssignment::Site { site: HOME }
    );
    assert_eq!(
        campaign.people[&ward].service_start_round,
        campaign.completed_rounds
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn invite_apprentice_spends_gold_and_population_once_per_year() {
    let (data, mut campaign) = fixture();
    assert!(campaign.world.population[&HOME] > 0);
    let gold = campaign.factions[&OWNER].resources.gold;
    let population = campaign.world.population[&HOME];
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: HOME },
    )
    .unwrap();
    let apprentice = outcome.new_people[0];
    assert_eq!(campaign.world.population[&HOME], population - 1);
    assert_eq!(
        campaign.factions[&OWNER].resources.gold,
        gold - data.households.apprentice_cost_gold
    );
    assert_eq!(
        campaign.people[&apprentice].age_years(campaign.completed_rounds),
        17
    );
    assert_eq!(campaign.people[&apprentice].class, PersonClass::Recruit);
    assert_eq!(
        campaign.families[&apprentice].origin,
        FamilyOrigin::LocalApprentice
    );
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: HOME },
    )
    .is_err());
    campaign.validate(&data).unwrap();
}

#[test]
fn spring_births_are_sparse_deterministic_and_resume_without_replay() {
    let (data, _) = fixture();
    let (seed, mut initial) = (1..=100)
        .find_map(|seed| {
            let (mut campaign, household) = prepared_birth(&data, seed);
            for _ in 0..4 {
                complete_round(&mut campaign, &data);
            }
            campaign
                .families
                .values()
                .any(|family| {
                    family.household == Some(household) && family.origin == FamilyOrigin::Birth
                })
                .then_some((seed, campaign))
        })
        .expect("at least one fixed seed succeeds at the authored 20% chance");
    let household = *initial.households.keys().next().unwrap();
    let first = initial.households[&household].partners[0];
    let mut replay = prepared_birth(&data, seed).0;
    for _ in 0..4 {
        complete_round(&mut replay, &data);
    }
    assert_eq!(initial, replay);
    assert_eq!(initial.households[&household].last_attempted_year, Some(1));
    let children = initial
        .families
        .iter()
        .filter(|(_, family)| {
            family.household == Some(household) && family.origin == FamilyOrigin::Birth
        })
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
    let child = &initial.people[&children[0]];
    assert_eq!(child.class, PersonClass::Recruit);
    assert_eq!(child.assignment, PersonAssignment::Dependent { site: HOME });
    assert!(child.career == Default::default());
    assert!(child.evidence.counts.is_empty());
    assert!(initial.known_blood_relation(first, children[0]));
    let encoded = serde_json::to_value(Campaign::Strategic(Box::new(initial.clone()))).unwrap();
    let restored: Campaign = serde_json::from_value(encoded).unwrap();
    assert_eq!(restored.strategic(), Some(&initial));
    for _ in 0..4 {
        complete_round(&mut initial, &data);
    }
    assert_eq!(
        initial
            .families
            .values()
            .filter(|family| family.household == Some(household)
                && family.origin == FamilyOrigin::Birth)
            .count(),
        1,
        "the eight-season gap prevents a second child next Spring"
    );
}

#[test]
fn adopted_household_successors_keep_their_own_skills_and_command_successors_stay_local() {
    let (data, mut campaign) = fixture();
    let guardian = add_person(&mut campaign, 35, PersonAssignment::Site { site: HOME });
    let ward = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AdoptWard {
            guardian,
            site: HOME,
        },
    )
    .unwrap()
    .new_people[0];
    campaign.people.get_mut(&ward).unwrap().birth_round = -68;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DesignateSuccessor {
            predecessor: guardian,
            successor: ward,
            category: LegacyCategory::Household,
            link: SuccessorLink::Adopted,
        },
    )
    .unwrap();
    assert!(campaign.people[&ward].career == Default::default());
    campaign.validate(&data).unwrap();

    let commander = PersonId(1);
    campaign
        .armies
        .get_mut(&kestrum::state::military::ArmyId(1))
        .unwrap()
        .commander = Some(commander);
    let formation = campaign.armies[&kestrum::state::military::ArmyId(1)]
        .formation_ids()
        .next()
        .unwrap();
    campaign.completed_rounds = 4;
    let pupil = add_person(&mut campaign, 24, PersonAssignment::Formation { formation });
    let mentor = campaign.people.get_mut(&pupil).unwrap();
    mentor
        .career
        .mentorship_seasons
        .insert(TrainingDiscipline::Command, 4);
    mentor
        .career
        .completed_mentors
        .insert(CompletedApprenticeship {
            mentor: commander,
            discipline: TrainingDiscipline::Command,
            started_round: 0,
            completed_round: 4,
        });
    set_shared_seasons(&mut campaign, commander, pupil, 4);
    let career_before_retirement = campaign.people[&pupil].career.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DesignateSuccessor {
            predecessor: commander,
            successor: pupil,
            category: LegacyCategory::Command,
            link: SuccessorLink::Martial,
        },
    )
    .unwrap();
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RetirePerson {
            person: commander,
            site: HOME,
        },
    )
    .unwrap();
    assert_eq!(
        campaign.armies[&kestrum::state::military::ArmyId(1)].commander,
        Some(pupil)
    );
    assert_eq!(outcome.succession.len(), 1);
    assert_eq!(outcome.succession[0].successor, Some(pupil));
    assert_eq!(campaign.people[&pupil].career, career_before_retirement);
    campaign.validate(&data).unwrap();
}

#[test]
fn k14_save_migration_adds_empty_relationship_state_without_consuming_people_rng() {
    let (data, campaign) = fixture();
    let expected_rng = campaign.rng.clone();
    let mut old = serde_json::to_value(&campaign).unwrap();
    for field in [
        "households",
        "families",
        "successors",
        "apprentice_last_invited_year",
    ] {
        old.as_object_mut().unwrap().remove(field);
    }
    old.pointer_mut("/next_ids")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("household");
    let migrated: Campaign = serde_json::from_value(old).unwrap();
    let migrated = migrated.strategic().unwrap();
    assert!(migrated.households.is_empty());
    assert!(migrated.families.is_empty());
    assert!(migrated.successors.is_empty());
    assert_eq!(migrated.rng, expected_rng);
    migrated.validate(&data).unwrap();
}

#[test]
fn rival_household_formation_uses_the_same_validated_command() {
    let (data, mut campaign) = fixture();
    let first = add_person_for(
        &mut campaign,
        RIVAL,
        25,
        PersonAssignment::Site { site: SiteId(2) },
    );
    let second = add_person_for(
        &mut campaign,
        RIVAL,
        27,
        PersonAssignment::Site { site: SiteId(2) },
    );
    set_shared_seasons(&mut campaign, first, second, 4);
    campaign.apprentice_last_invited_year.insert(RIVAL, 0);
    campaign.factions.get_mut(&RIVAL).unwrap().resources.gold = 0;
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), RIVAL);
    let decision = ai::propose(&campaign, &data, RIVAL).unwrap();
    assert_eq!(
        decision.command,
        Command::FormHousehold {
            first,
            second,
            site: SiteId(2)
        }
    );
}

// Review options must explain the same rejection as the authoritative transaction.
fn review_selection(first: PersonId, second: PersonId) -> kestrum::engine::HouseholdSelection {
    kestrum::engine::HouseholdSelection {
        site: HOME,
        first: Some(first),
        second: Some(second),
        category: LegacyCategory::Item,
        link: SuccessorLink::Adopted,
    }
}

fn compare_review(
    data: &GameData,
    campaign: &StrategicCampaign,
    action: kestrum::engine::HouseholdAction,
    selection: kestrum::engine::HouseholdSelection,
    expected_reason: Option<&str>,
) -> StrategicCampaign {
    let before = campaign.clone();
    let option = kestrum::engine::household_option(campaign, data, OWNER, action, selection);
    assert_eq!(campaign, &before);
    let mut after = campaign.clone();
    let command = option
        .command
        .expect("complete selections produce a reviewable command");
    let result = apply(&mut after, data, Actor::Player, command);
    match expected_reason {
        Some(expected) => {
            let blocked = option.blocked.expect("blocked reason");
            assert!(blocked.contains(expected), "{blocked}");
            assert_eq!(result.unwrap_err().to_string(), blocked);
            assert_eq!(after, before);
        }
        None => {
            assert_eq!(option.blocked, None);
            result.unwrap();
        }
    }
    after
}

#[test]
fn partnership_review_explains_familiarity_age_and_existing_households() {
    use kestrum::engine::HouseholdAction::Form;
    let (data, mut campaign) = fixture();
    let first = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let second = add_person(&mut campaign, 27, PersonAssignment::Site { site: HOME });
    let selected = review_selection(first, second);
    compare_review(&data, &campaign, Form, selected, Some("these two have 0"));
    for _ in 0..4 {
        complete_round(&mut campaign, &data);
    }
    let formed = compare_review(&data, &campaign, Form, selected, None);
    compare_review(&data, &formed, Form, selected, Some("active household"));
    campaign.people.get_mut(&second).unwrap().birth_round = -60;
    compare_review(&data, &campaign, Form, selected, Some("at least 18"));
}

#[test]
fn review_handles_missing_people_ownership_and_phase_without_mutation() {
    use kestrum::engine::{household_option, HouseholdAction::Form};
    let (data, mut campaign) = fixture();
    let first = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let second = add_person(&mut campaign, 27, PersonAssignment::Site { site: HOME });
    let mut selected = review_selection(first, second);
    selected.second = None;
    let option = household_option(&campaign, &data, OWNER, Form, selected);
    assert!(option.command.is_none());
    assert!(option.blocked.unwrap().contains("Choose a second"));
    selected.second = campaign
        .people
        .values()
        .find(|person| person.faction == RIVAL)
        .map(|person| person.id);
    let option = household_option(&campaign, &data, OWNER, Form, selected);
    assert!(option.command.is_none());
    assert!(option.blocked.is_some());
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    compare_review(
        &data,
        &campaign,
        Form,
        review_selection(first, second),
        Some("turn"),
    );
}

#[test]
fn adoption_review_shows_capacity_and_training_review_shows_age() {
    use kestrum::engine::HouseholdAction::{Adopt, Service, Train};
    let (data, mut campaign) = fixture();
    let guardian = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let selected = review_selection(guardian, guardian);
    campaign = compare_review(&data, &campaign, Adopt, selected, None);
    let ward = *campaign.families.keys().next().unwrap();
    campaign = compare_review(&data, &campaign, Adopt, selected, None);
    compare_review(&data, &campaign, Adopt, selected, Some("two dependent"));
    let selected = review_selection(ward, guardian);
    compare_review(
        &data,
        &campaign,
        Train,
        selected,
        Some("thirteen to sixteen"),
    );
    compare_review(&data, &campaign, Service, selected, Some("seventeen"));
    for _ in 0..20 {
        complete_round(&mut campaign, &data);
    }
    campaign = compare_review(&data, &campaign, Train, selected, None);
    for _ in 0..16 {
        complete_round(&mut campaign, &data);
    }
    compare_review(&data, &campaign, Service, selected, None);
}

#[test]
fn apprenticeship_review_shows_deficit_cost_and_annual_limit() {
    use kestrum::engine::HouseholdAction::Invite;
    let (data, mut campaign) = fixture();
    let first = campaign
        .people
        .values()
        .find(|p| p.faction == OWNER)
        .unwrap()
        .id;
    let selected = review_selection(first, first);
    campaign.factions.get_mut(&OWNER).unwrap().deficit = true;
    compare_review(&data, &campaign, Invite, selected, Some("shortfall"));
    campaign.factions.get_mut(&OWNER).unwrap().deficit = false;
    campaign.factions.get_mut(&OWNER).unwrap().resources.gold = 0;
    compare_review(&data, &campaign, Invite, selected, Some("needs 20 Gold"));
    campaign.factions.get_mut(&OWNER).unwrap().resources.gold = 100;
    campaign = compare_review(&data, &campaign, Invite, selected, None);
    compare_review(
        &data,
        &campaign,
        Invite,
        selected,
        Some("one apprentice per year"),
    );
}

#[test]
fn succession_review_explains_missing_relationship_and_used_category() {
    use kestrum::engine::HouseholdAction::{Adopt, Designate};
    let (data, mut campaign) = fixture();
    let first = add_person(&mut campaign, 25, PersonAssignment::Site { site: HOME });
    let second = add_person(&mut campaign, 27, PersonAssignment::Site { site: HOME });
    compare_review(
        &data,
        &campaign,
        Designate,
        review_selection(first, second),
        Some("not established"),
    );
    campaign = compare_review(
        &data,
        &campaign,
        Adopt,
        review_selection(first, second),
        None,
    );
    let ward = *campaign.families.keys().next().unwrap();
    let selected = review_selection(first, ward);
    campaign = compare_review(&data, &campaign, Designate, selected, None);
    compare_review(&data, &campaign, Designate, selected, Some("one successor"));
    let mut command = selected;
    command.category = LegacyCategory::Command;
    compare_review(
        &data,
        &campaign,
        Designate,
        command,
        Some("current commander"),
    );
}
