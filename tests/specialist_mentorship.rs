//! Specialist lessons supplement real treatment and physical travel.
use kestrum::{
    data::{
        progression::TrainingDiscipline,
        world::{Facility, PersonClass, SiteId},
        GameData,
    },
    engine::{apply, career_options, Actor, Command, MoveOrder},
    state::{
        evidence::EvidenceKind,
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId, PersonStatus},
        Campaign, StrategicCampaign,
    },
};

const MENTOR: PersonId = PersonId(1);

fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let owner = campaign.active_faction();
        let actor = if owner == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(owner)
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

fn add_person(campaign: &mut StrategicCampaign) -> PersonId {
    let id = campaign.next_ids.person;
    campaign.next_ids.person.0 += 1;
    let mut person = campaign.people[&MENTOR].clone();
    person.id = id;
    person.name = format!("Pupil {}", id.0);
    person.class = PersonClass::Recruit;
    person.career = Default::default();
    person.evidence = Default::default();
    person.assignment = PersonAssignment::Site { site: SiteId(1) };
    campaign.people.insert(id, person);
    id
}

fn fixture(discipline: TrainingDiscipline) -> (GameData, StrategicCampaign, PersonId) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap();
    site.facilities.push(Facility::Infirmary);
    let mentor = campaign.people.get_mut(&MENTOR).unwrap();
    mentor.birth_round = -120;
    mentor.class = if discipline == TrainingDiscipline::Medicine {
        PersonClass::Medic
    } else {
        PersonClass::Scout
    };
    mentor.assignment = PersonAssignment::Site { site: SiteId(1) };
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    for _ in 0..4 {
        finish(&mut campaign, &data);
    }
    let learner = add_person(&mut campaign);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: MENTOR,
            learner,
            discipline,
        },
    )
    .unwrap();
    (data, campaign, learner)
}

fn eligible(
    campaign: &StrategicCampaign,
    data: &GameData,
    learner: PersonId,
    class: PersonClass,
) -> bool {
    career_options(campaign, data, learner)
        .unwrap()
        .iter()
        .find(|option| option.class == class)
        .unwrap()
        .eligible
}

fn treatments(campaign: &StrategicCampaign, person: PersonId) -> u32 {
    campaign.people[&person]
        .evidence
        .counts
        .get(&EvidenceKind::TreatedWounded)
        .copied()
        .unwrap_or(0)
}

#[test]
fn medical_pupil_assists_real_healing_and_can_complete_the_medic_course() {
    let (data, mut campaign, learner) = fixture(TrainingDiscipline::Medicine);
    let patient = add_person(&mut campaign);
    campaign.people.get_mut(&patient).unwrap().status = PersonStatus::Wounded {
        since_round: 4,
        remaining_steps: 1,
    };
    finish(&mut campaign, &data);
    assert_eq!(campaign.people[&patient].status, PersonStatus::Fit);
    assert_eq!(treatments(&campaign, learner), 1);
    assert_eq!(treatments(&campaign, MENTOR), 1);
    assert!(!eligible(&campaign, &data, learner, PersonClass::Medic));
    finish(&mut campaign, &data);
    assert_eq!(
        treatments(&campaign, learner),
        1,
        "no second receipt without a patient"
    );
    assert!(eligible(&campaign, &data, learner, PersonClass::Medic));
    for _ in 0..2 {
        finish(&mut campaign, &data);
    }
    assert!(!campaign.mentorships.contains_key(&learner));
    let restored: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap(),
    )
    .unwrap();
    restored.validate(&data).unwrap();
    campaign = restored.strategic().unwrap().clone();
    assert!(eligible(&campaign, &data, learner, PersonClass::Medic));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: learner,
            class: PersonClass::Medic,
            site: SiteId(1),
        },
    )
    .unwrap();
    for _ in 0..2 {
        finish(&mut campaign, &data);
    }
    assert_eq!(campaign.people[&learner].class, PersonClass::Medic);
}

#[test]
fn lessons_without_patients_never_invent_treatment_or_unlock_medicine() {
    let (data, mut campaign, learner) = fixture(TrainingDiscipline::Medicine);
    for _ in 0..4 {
        finish(&mut campaign, &data);
    }
    assert_eq!(treatments(&campaign, learner), 0);
    assert!(!eligible(&campaign, &data, learner, PersonClass::Medic));
    assert_eq!(campaign.people[&learner].career.completed_mentors.len(), 1);
}

#[test]
fn medical_assistance_also_records_real_formation_recovery() {
    let (data, mut campaign, learner) = fixture(TrainingDiscipline::Medicine);
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.headcount -= 20;
    let before = formation.headcount;
    finish(&mut campaign, &data);
    assert!(campaign.formations[&FormationId(1)].headcount > before);
    assert_eq!(treatments(&campaign, learner), 1);
    assert_eq!(treatments(&campaign, MENTOR), 1);
}

#[test]
fn unavailable_assistants_cannot_take_credit_even_if_healed_at_the_boundary() {
    for condition in ["wounded", "apart", "mentor_wounded", "facility", "ended"] {
        let (data, mut campaign, learner) = fixture(TrainingDiscipline::Medicine);
        let patient = add_person(&mut campaign);
        campaign.people.get_mut(&patient).unwrap().status = PersonStatus::Wounded {
            since_round: 4,
            remaining_steps: 1,
        };
        match condition {
            "wounded" => {
                campaign.people.get_mut(&learner).unwrap().status = PersonStatus::Wounded {
                    since_round: 4,
                    remaining_steps: 1,
                }
            }
            "apart" => {
                campaign.people.get_mut(&learner).unwrap().assignment =
                    PersonAssignment::Site { site: SiteId(5) }
            }
            "mentor_wounded" => {
                campaign.people.get_mut(&MENTOR).unwrap().status = PersonStatus::Wounded {
                    since_round: 4,
                    remaining_steps: 1,
                }
            }
            "facility" => campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == SiteId(1))
                .unwrap()
                .facilities
                .retain(|facility| *facility != Facility::Infirmary),
            _ => {
                apply(
                    &mut campaign,
                    &data,
                    Actor::Player,
                    Command::EndMentorship { learner },
                )
                .unwrap();
            }
        }
        finish(&mut campaign, &data);
        assert_eq!(treatments(&campaign, learner), 0, "{condition}");
    }
}

#[test]
fn scouting_lessons_reduce_but_do_not_replace_the_need_to_travel_distinct_routes() {
    let (data, mut campaign, learner) = fixture(TrainingDiscipline::Scouting);
    for _ in 0..4 {
        finish(&mut campaign, &data);
    }
    assert!(!eligible(&campaign, &data, learner, PersonClass::Scout));
    assert!(campaign.people[&learner]
        .evidence
        .traversed_routes
        .is_empty());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: learner,
            to_formation: FormationId(1),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: [1, 5].map(SiteId).to_vec(),
        }),
    )
    .unwrap();
    finish(&mut campaign, &data);
    assert!(!eligible(&campaign, &data, learner, PersonClass::Scout));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: [5, 6].map(SiteId).to_vec(),
        }),
    )
    .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(campaign.people[&learner].evidence.traversed_routes.len(), 2);
    assert!(eligible(&campaign, &data, learner, PersonClass::Scout));
    campaign
        .people
        .get_mut(&learner)
        .unwrap()
        .career
        .mentorship_seasons
        .clear();
    assert!(
        !eligible(&campaign, &data, learner, PersonClass::Scout),
        "two routes alone do not suffice"
    );
}
