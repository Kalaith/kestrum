//! K14 mentorship requires qualified people, physical contact and real seasons.

use kestrum::{
    data::{
        economy::Resources,
        progression::TrainingDiscipline,
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{ai, apply, career_options, mentorship_options, Actor, Command, MoveOrder},
    state::{
        mentorship::{MentorshipPauseReason, MentorshipStatus},
        people::{PersonAssignment, PersonId, PersonStatus},
        CampaignPhase, StrategicCampaign,
    },
};
use std::collections::BTreeMap;

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn add_person(
    campaign: &mut StrategicCampaign,
    id: u32,
    faction: u32,
    class: PersonClass,
    age: u32,
    assignment: PersonAssignment,
) {
    let mut person = campaign.people[&PersonId(faction)].clone();
    person.id = PersonId(id);
    person.faction = FactionId(faction);
    person.name = format!("Apprentice {id}");
    person.birth_round = i64::from(campaign.completed_rounds) - 4 * i64::from(age);
    person.service_start_round = 0;
    person.class = class;
    person.assignment = assignment;
    person.movement_spent = 0;
    person.status = PersonStatus::Fit;
    person.career = Default::default();
    person.evidence = Default::default();
    campaign.people.insert(person.id, person);
    campaign.next_ids.person = PersonId(campaign.next_ids.person.0.max(id + 1));
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

fn qualified_infantry_fixture() -> (GameData, StrategicCampaign, PersonId, PersonId) {
    let (data, mut campaign) = fixture();
    campaign.people.get_mut(&PersonId(1)).unwrap().class = PersonClass::Infantry;
    campaign.people.get_mut(&PersonId(1)).unwrap().birth_round = -104;
    campaign
        .people
        .get_mut(&PersonId(1))
        .unwrap()
        .service_start_round = 0;
    for _ in 0..4 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(
        campaign.people[&PersonId(1)]
            .career
            .discipline_service_seasons[&TrainingDiscipline::Infantry],
        4
    );
    add_person(
        &mut campaign,
        5,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    (data, campaign, PersonId(1), PersonId(5))
}

#[test]
fn qualified_class_and_four_real_service_seasons_unlock_one_learner() {
    let (data, mut campaign, mentor, learner) = qualified_infantry_fixture();
    campaign.validate(&data).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    add_person(
        &mut campaign,
        6,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner: PersonId(6),
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .is_err());
    assert_eq!(campaign.mentorships.len(), 1);
    let full_option = mentorship_options(&campaign, &data, PersonId(6))
        .unwrap()
        .into_iter()
        .find(|option| option.mentor == mentor && option.discipline == TrainingDiscipline::Infantry)
        .unwrap();
    assert!(!full_option.eligible);
    assert_eq!(
        full_option.reason.as_deref(),
        Some("This mentor is already teaching another learner")
    );

    let mut too_young = campaign.clone();
    too_young.people.get_mut(&mentor).unwrap().birth_round =
        i64::from(too_young.completed_rounds) - 4 * 25;
    assert!(apply(
        &mut too_young,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner: PersonId(6),
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .is_err());
}

#[test]
fn a_developed_infantry_student_can_choose_a_command_mentor_too() {
    let (data, mut campaign, _, learner) = qualified_infantry_fixture();
    campaign.people.get_mut(&learner).unwrap().class = PersonClass::Infantry;
    add_person(
        &mut campaign,
        6,
        1,
        PersonClass::Officer,
        30,
        PersonAssignment::Site { site: SiteId(1) },
    );
    campaign
        .people
        .get_mut(&PersonId(6))
        .unwrap()
        .career
        .discipline_service_seasons = BTreeMap::from([(TrainingDiscipline::Command, 4)]);
    assert!(mentorship_options(&campaign, &data, learner)
        .unwrap()
        .iter()
        .any(|option| {
            option.mentor == PersonId(6)
                && option.discipline == TrainingDiscipline::Command
                && option.eligible
        }));
}

#[test]
fn actual_site_contact_pauses_and_resumes_lessons_and_wounds_explainably() {
    let (data, mut campaign, mentor, learner) = qualified_infantry_fixture();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![kestrum::state::military::ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Paused {
            reason: MentorshipPauseReason::Apart
        }
    );
    finish_round(&mut campaign, &data);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![kestrum::state::military::ArmyId(1)],
            path: vec![SiteId(5), SiteId(1)],
        }),
    )
    .unwrap();
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Active
    );

    campaign.people.get_mut(&mentor).unwrap().status = PersonStatus::Wounded {
        since_round: campaign.completed_rounds,
        remaining_steps: 2,
    };
    campaign
        .armies
        .get_mut(&kestrum::state::military::ArmyId(1))
        .unwrap()
        .commander = None;
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Paused {
            reason: MentorshipPauseReason::Wounded
        }
    );
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.people[&mentor].status, PersonStatus::Fit);
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Active
    );

    campaign.people.get_mut(&mentor).unwrap().class = PersonClass::Medic;
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Paused {
            reason: MentorshipPauseReason::QualificationLost
        }
    );
    campaign.people.get_mut(&mentor).unwrap().class = PersonClass::Infantry;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    assert_eq!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Active
    );
}

#[test]
fn a_paused_learner_can_end_the_link_before_choosing_another_mentor() {
    let (data, mut campaign, mentor, learner) = qualified_infantry_fixture();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    campaign.people.get_mut(&mentor).unwrap().status = PersonStatus::Wounded {
        since_round: campaign.completed_rounds,
        remaining_steps: 2,
    };
    campaign
        .armies
        .get_mut(&kestrum::state::military::ArmyId(1))
        .unwrap()
        .commander = None;
    finish_round(&mut campaign, &data);
    assert!(matches!(
        campaign.mentorships[&learner].status,
        MentorshipStatus::Paused {
            reason: MentorshipPauseReason::Wounded
        }
    ));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EndMentorship { learner },
    )
    .unwrap();

    assert!(!campaign.mentorships.contains_key(&learner));
}

#[test]
fn two_seasons_open_the_p18_route_and_four_leave_a_dated_legacy_tie_without_battle_facts() {
    let (data, mut campaign, mentor, learner) = qualified_infantry_fixture();
    let before = campaign.people[&learner].evidence.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.people[&learner].career.mentorship_seasons[&TrainingDiscipline::Infantry],
        1
    );
    finish_round(&mut campaign, &data);
    let options = career_options(&campaign, &data, learner).unwrap();
    assert!(options
        .iter()
        .any(|option| option.class == PersonClass::Infantry && option.eligible));
    assert_eq!(campaign.people[&learner].evidence, before);

    finish_round(&mut campaign, &data);
    finish_round(&mut campaign, &data);
    assert!(!campaign.mentorships.contains_key(&learner));
    let tie = campaign.people[&learner]
        .career
        .completed_mentors
        .iter()
        .next()
        .unwrap();
    assert_eq!(tie.mentor, mentor);
    assert_eq!(tie.discipline, TrainingDiscipline::Infantry);
    assert_eq!(tie.started_round, 4);
    assert_eq!(tie.completed_round, 8);
    assert_eq!(campaign.people[&learner].evidence, before);
}

#[test]
fn medical_lessons_require_an_infirmary_and_actual_same_site_contact() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 4;
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .facilities
        .push(kestrum::data::world::Facility::Infirmary);
    let mentor = campaign.people.get_mut(&PersonId(1)).unwrap();
    mentor.class = PersonClass::Medic;
    mentor.birth_round = -100;
    mentor.service_start_round = 0;
    mentor.career.discipline_service_seasons = BTreeMap::from([(TrainingDiscipline::Medicine, 4)]);
    add_person(
        &mut campaign,
        5,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    campaign.validate(&data).unwrap();
    assert!(mentorship_options(&campaign, &data, PersonId(5))
        .unwrap()
        .iter()
        .any(|option| option.discipline == TrainingDiscipline::Medicine && option.eligible));

    campaign.people.get_mut(&PersonId(5)).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(5) };
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: PersonId(1),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Medicine,
        },
    )
    .is_err());

    let facilities = &mut campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .facilities;
    let infirmary = facilities
        .iter()
        .position(|facility| *facility == kestrum::data::world::Facility::Infirmary)
        .unwrap();
    facilities.remove(infirmary);
    campaign.people.get_mut(&PersonId(5)).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(1) };
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: PersonId(1),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Medicine,
        },
    )
    .is_err());
}

#[test]
fn riding_lessons_pause_without_horse_access_and_resume_when_restored() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 4;
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .facilities
        .push(kestrum::data::world::Facility::Stable);
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .tags
        .clear();
    let mentor = campaign.people.get_mut(&PersonId(1)).unwrap();
    mentor.class = PersonClass::Cavalry;
    mentor.birth_round = -116;
    mentor.service_start_round = 0;
    mentor.career.discipline_service_seasons = BTreeMap::from([(TrainingDiscipline::Riding, 4)]);
    add_person(
        &mut campaign,
        5,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    let unavailable = mentorship_options(&campaign, &data, PersonId(5)).unwrap();
    assert!(unavailable.iter().any(|option| {
        option.mentor == PersonId(1)
            && option.discipline == TrainingDiscipline::Riding
            && !option.eligible
            && option.reason.as_deref() == Some("Riding lessons require local horse access")
    }));
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: PersonId(1),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Riding,
        },
    )
    .is_err());
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .tags
        .push(kestrum::data::world::SiteTag::HorseAccess);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: PersonId(1),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Riding,
        },
    )
    .unwrap();
    assert_eq!(
        campaign.mentorships[&PersonId(5)].status,
        MentorshipStatus::Active
    );
}

#[test]
fn npc_and_player_share_validated_mentorship_opportunities() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 4;
    let mentor = campaign.people.get_mut(&PersonId(2)).unwrap();
    mentor.class = PersonClass::Officer;
    mentor.birth_round = -116;
    mentor.service_start_round = 0;
    mentor.career.discipline_service_seasons = BTreeMap::from([(TrainingDiscipline::Command, 4)]);
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == FactionId(2))
        .unwrap()
        .id;
    let site = campaign.armies[&army].site;
    add_person(
        &mut campaign,
        5,
        2,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site },
    );
    campaign.factions.get_mut(&FactionId(2)).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    campaign.factions.get_mut(&FactionId(2)).unwrap().deficit = true;
    campaign.validate(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert!(matches!(
        campaign.phase,
        CampaignPhase::NpcTurn {
            faction: FactionId(2),
            ..
        }
    ));
    let decision = ai::propose(&campaign, &data, FactionId(2)).unwrap();
    assert_eq!(
        decision.command,
        Command::StartMentorship {
            mentor: PersonId(2),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Command,
        }
    );
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        decision.command,
    )
    .unwrap();
    assert_eq!(campaign.mentorships[&PersonId(5)].mentor, PersonId(2));

    let mut player_campaign = StrategicCampaign::new(&data).unwrap();
    player_campaign.completed_rounds = 4;
    let player_mentor = player_campaign.people.get_mut(&PersonId(1)).unwrap();
    player_mentor.class = PersonClass::Officer;
    player_mentor.birth_round = -116;
    player_mentor.career.discipline_service_seasons =
        BTreeMap::from([(TrainingDiscipline::Command, 4)]);
    add_person(
        &mut player_campaign,
        5,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    apply(
        &mut player_campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: PersonId(1),
            learner: PersonId(5),
            discipline: TrainingDiscipline::Command,
        },
    )
    .unwrap();
    assert_eq!(
        player_campaign.mentorships[&PersonId(5)].mentor,
        PersonId(1)
    );
}
