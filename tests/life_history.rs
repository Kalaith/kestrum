//! Accepted life transitions produce one private, shared history record.
use kestrum::{
    data::{
        progression::{EpithetFact, TrainingDiscipline},
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{action_notices, apply, history_page, ActionOutcome, Actor, Command, HistoryFilter},
    state::{
        history::{HistoryKind, HistoryRecord, HistorySubject, LifeEvent},
        military::{ArmyId, FormationId},
        people::{PersonId, PersonStatus},
        Campaign, StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn season(campaign: &mut StrategicCampaign, data: &GameData) -> ActionOutcome {
    let round = campaign.completed_rounds;
    loop {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        let outcome = apply(campaign, data, actor, Command::EndTurn).unwrap();
        if campaign.completed_rounds != round {
            return outcome;
        }
    }
}

fn life(campaign: &StrategicCampaign, person: PersonId) -> Vec<&HistoryRecord> {
    campaign
        .history
        .events
        .values()
        .filter(
            |record| matches!(record.kind, HistoryKind::Life { person: id, .. } if id == person),
        )
        .collect()
}

fn assert_shared(
    campaign: &StrategicCampaign,
    record: &HistoryRecord,
    subjects: &[HistorySubject],
) {
    for subject in subjects {
        let page = history_page(
            campaign,
            campaign.player,
            &HistoryFilter {
                subject: Some(*subject),
                event: Some(record.id),
                ..Default::default()
            },
        );
        assert_eq!(page.entries.as_slice(), std::slice::from_ref(record));
    }
    assert!(history_page(
        campaign,
        FactionId(2),
        &HistoryFilter {
            event: Some(record.id),
            ..Default::default()
        }
    )
    .entries
    .is_empty());
}

#[test]
fn local_apprenticeship_class_and_retirement_leave_one_shared_dated_trace() {
    let (data, mut campaign) = fixture();
    campaign.people.get_mut(&PersonId(1)).unwrap().class = PersonClass::Infantry;
    campaign.people.get_mut(&PersonId(1)).unwrap().birth_round = -104;
    for _ in 0..4 {
        season(&mut campaign, &data);
    }
    let learner = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: SiteId(1) },
    )
    .unwrap()
    .new_people[0];
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            learner,
            mentor: PersonId(1),
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    for _ in 0..4 {
        season(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: learner,
            to_formation: FormationId(2),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: learner,
            class: PersonClass::Infantry,
            site: SiteId(1),
        },
    )
    .unwrap();
    season(&mut campaign, &data);
    let outcome = season(&mut campaign, &data);
    let notices = action_notices(&campaign, &data, campaign.player, &outcome);
    assert!(notices
        .iter()
        .any(|line| line.contains("Completed training as Infantry")));
    assert!(action_notices(&campaign, &data, FactionId(2), &outcome).is_empty());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RetirePerson {
            person: learner,
            site: SiteId(1),
        },
    )
    .unwrap();
    let records = life(&campaign, learner);
    assert_eq!(records.len(), 5);
    assert!(records.iter().any(|record| matches!(
        record.kind,
        HistoryKind::Life {
            event: LifeEvent::MentorshipCompleted { .. },
            ..
        }
    )));
    let retired = records.last().unwrap();
    assert_shared(
        &campaign,
        retired,
        &[
            HistorySubject::Person(learner),
            HistorySubject::Site(SiteId(1)),
            HistorySubject::Army(ArmyId(1)),
        ],
    );
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap(),
    )
    .unwrap();
    assert_eq!(loaded.strategic().unwrap(), &campaign);
    season(&mut campaign, &data);
    assert_eq!(life(&campaign, learner).len(), 5);
}

#[test]
fn recognition_is_announced_once_and_observers_cannot_read_private_milestones() {
    use kestrum::state::evidence::EvidenceKind;
    let (data, mut campaign) = fixture();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.evidence.counts.extend([
        (EvidenceKind::Battle, 3),
        (EvidenceKind::MeaningfulEncounter, 3),
        (EvidenceKind::DefendedAnchor, 1),
    ]);
    person
        .evidence
        .service_by_troop
        .insert(kestrum::data::economy::TroopKind::Warriors, 3);
    person
        .career
        .notable_sites
        .insert(EpithetFact::DefendedAnchor, SiteId(1));
    let outcome = season(&mut campaign, &data);
    assert_eq!(
        action_notices(&campaign, &data, campaign.player, &outcome)
            .iter()
            .filter(|line| line.contains("Recognized as"))
            .count(),
        1
    );
    let recognized = life(&campaign, PersonId(1))[0].clone();
    assert_shared(
        &campaign,
        &recognized,
        &[
            HistorySubject::Person(PersonId(1)),
            HistorySubject::Site(SiteId(1)),
            HistorySubject::Army(ArmyId(1)),
        ],
    );
    let before = campaign.clone();
    for _ in 0..3 {
        history_page(&campaign, campaign.player, &HistoryFilter::default());
    }
    assert_eq!(campaign, before);
    let next = season(&mut campaign, &data);
    assert!(!action_notices(&campaign, &data, campaign.player, &next)
        .iter()
        .any(|line| line.contains("Recognized as")));
    let mut forged = campaign.clone();
    forged
        .history
        .events
        .get_mut(&recognized.id)
        .unwrap()
        .visible_to
        .insert(FactionId(2));
    assert!(forged.validate(&data).is_err());
}

#[test]
fn natural_death_records_its_actual_date_place_and_released_army() {
    let (data, mut campaign) = fixture();
    campaign.people.get_mut(&PersonId(1)).unwrap().birth_round = -239;
    let seed = (1..100_000)
        .find(|seed| SeededRng::new(*seed).below(1000) < 20)
        .unwrap();
    campaign.rng.people = SeededRng::new(seed);
    season(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&PersonId(1)].status,
        PersonStatus::Dead { .. }
    ));
    let record = life(&campaign, PersonId(1))
        .into_iter()
        .find(|record| {
            matches!(
                record.kind,
                HistoryKind::Life {
                    event: LifeEvent::NaturalDeath,
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(record.completed_rounds, 1);
    assert_shared(
        &campaign,
        record,
        &[
            HistorySubject::Person(PersonId(1)),
            HistorySubject::Site(SiteId(1)),
            HistorySubject::Army(ArmyId(1)),
        ],
    );
}

#[test]
fn real_household_and_adoption_share_the_partners_and_guardian_chronicles() {
    let (data, mut campaign) = fixture();
    let second = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: SiteId(1) },
    )
    .unwrap()
    .new_people[0];
    for _ in 0..12 {
        season(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::FormHousehold {
            first: PersonId(1),
            second,
            site: SiteId(1),
        },
    )
    .unwrap();
    let formed = life(&campaign, PersonId(1))
        .into_iter()
        .find(|record| {
            matches!(
                record.kind,
                HistoryKind::Life {
                    event: LifeEvent::HouseholdFormed { .. },
                    ..
                }
            )
        })
        .unwrap();
    assert_shared(
        &campaign,
        formed,
        &[
            HistorySubject::Person(PersonId(1)),
            HistorySubject::Person(second),
            HistorySubject::Site(SiteId(1)),
        ],
    );
    let child = apply(
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
    let record = life(&campaign, child)[0];
    assert_shared(
        &campaign,
        record,
        &[
            HistorySubject::Person(PersonId(1)),
            HistorySubject::Person(child),
            HistorySubject::Site(SiteId(1)),
        ],
    );
}

#[test]
fn narrative_pruning_keeps_current_careers_and_does_not_recreate_old_events() {
    let (mut data, mut campaign) = fixture();
    let person = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: SiteId(1) },
    )
    .unwrap()
    .new_people[0];
    let career = campaign.people[&person].career.clone();
    data.history.detail_max_age_rounds = 1;
    for _ in 0..3 {
        season(&mut campaign, &data);
    }
    assert!(life(&campaign, person).is_empty());
    assert_eq!(campaign.people[&person].career.course, career.course);
    assert_eq!(campaign.people[&person].class, PersonClass::Recruit);
    assert!(campaign.history.person_notables[&person]
        .iter()
        .any(|record| matches!(
            record.kind,
            HistoryKind::Life {
                event: LifeEvent::Arrived { .. },
                ..
            }
        )));
    let next = campaign.next_ids.history;
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    assert_eq!(loaded.strategic().unwrap().next_ids.history, next);
}

#[test]
fn a_ward_ages_into_real_service_and_records_the_entry_once() {
    let (data, mut campaign) = fixture();
    let ward = apply(
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
    for _ in 0..20 {
        season(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AssignTrainee {
            person: ward,
            site: SiteId(1),
        },
    )
    .unwrap();
    for _ in 0..16 {
        season(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EnterService {
            person: ward,
            formation: Some(FormationId(2)),
        },
    )
    .unwrap();
    let entered = life(&campaign, ward)
        .into_iter()
        .find(|record| {
            matches!(
                record.kind,
                HistoryKind::Life {
                    event: LifeEvent::ServiceEntered,
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(entered.completed_rounds, 36);
    assert_shared(
        &campaign,
        entered,
        &[
            HistorySubject::Person(ward),
            HistorySubject::Site(SiteId(1)),
            HistorySubject::Army(ArmyId(1)),
        ],
    );
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EnterService {
            person: ward,
            formation: None
        }
    )
    .is_err());
    assert_eq!(campaign, before);
}

#[test]
fn spring_birth_and_chosen_household_departure_have_supported_causes() {
    let (data, mut campaign) = fixture();
    let second = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: SiteId(1) },
    )
    .unwrap()
    .new_people[0];
    for _ in 0..12 {
        season(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::FormHousehold {
            first: PersonId(1),
            second,
            site: SiteId(1),
        },
    )
    .unwrap();
    let household = *campaign.households.keys().next().unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetHouseholdChildraising {
            household,
            enabled: true,
        },
    )
    .unwrap();
    let seed = (1..100_000)
        .find(|seed| SeededRng::new(*seed).below(1000) < 200)
        .unwrap();
    campaign.rng.people = SeededRng::new(seed);
    for _ in 0..3 {
        season(&mut campaign, &data);
    }
    let born = season(&mut campaign, &data).new_people[0];
    let record = life(&campaign, born)[0];
    assert!(matches!(
        record.kind,
        HistoryKind::Life {
            event: LifeEvent::Arrived {
                origin: kestrum::state::relationships::FamilyOrigin::Birth
            },
            ..
        }
    ));
    assert_eq!(record.completed_rounds, 16);
    assert_shared(
        &campaign,
        record,
        &[
            HistorySubject::Person(PersonId(1)),
            HistorySubject::Person(second),
            HistorySubject::Person(born),
        ],
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EndHousehold { household },
    )
    .unwrap();
    assert!(life(&campaign, PersonId(1)).iter().any(|record| matches!(
        record.kind,
        HistoryKind::Life {
            event: LifeEvent::HouseholdEnded {
                reason: kestrum::state::relationships::HouseholdEndReason::Chosen,
                ..
            },
            ..
        }
    )));
}
