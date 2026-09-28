//! K14 ages on campaign boundaries and releases field, teaching and site duties.

use kestrum::{
    data::{
        progression::FormationSpecialization,
        world::{FactionId, FounderClass, MilitaryLayer, PersonClass, SiteId},
        GameData,
    },
    engine::{apply, development_view, Actor, Command},
    state::{
        evidence::EvidenceKind,
        mentorship::{Mentorship, MentorshipStatus},
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonCourse, PersonId, PersonSiteRole, PersonStatus},
        siege::{Siege, SiegeId},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::{persistence::encode_slot, rng::SeededRng};

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
    person.name = format!("K14 Person {id}");
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

fn seed_for_rolls(first_survives: bool, second_survives: bool) -> (u64, usize, usize) {
    (1..100_000)
        .find_map(|seed| {
            let mut rng = SeededRng::new(seed);
            let first = rng.below(1000);
            let second = rng.below(1000);
            ((first >= 20) == first_survives && (second >= 50) == second_survives)
                .then_some((seed, first, second))
        })
        .expect("find a fixed deterministic pair of birthday rolls")
}

#[test]
fn birthdays_follow_the_four_season_calendar_and_k13_saves_default_new_fields() {
    let (data, mut campaign) = fixture();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.birth_round = -239;
    person.service_start_round = 0;
    assert_eq!(person.age_years(0), 59);
    campaign.rng.people = SeededRng::new(7);
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.people[&PersonId(1)].age_years(1), 60);

    let mut invalid = campaign.clone();
    invalid.people.get_mut(&PersonId(1)).unwrap().birth_round = 8;
    assert!(invalid.validate(&data).is_err());

    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    value.as_object_mut().unwrap().remove("mentorships");
    for person in value["people"].as_object_mut().unwrap().values_mut() {
        let career = person["career"].as_object_mut().unwrap();
        for field in [
            "discipline_service_seasons",
            "mentorship_seasons",
            "completed_mentors",
            "site_role",
            "automatic_retirement_round",
        ] {
            career.remove(field);
        }
    }
    let encoded = encode_slot("strategic_v2", &value, "2").unwrap();
    let migrated = kestrum::state::persistence::load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert!(migrated.mentorships.is_empty());
    assert!(migrated.people.values().all(|person| {
        person.career.discipline_service_seasons.is_empty()
            && person.career.mentorship_seasons.is_empty()
            && person.career.completed_mentors.is_empty()
            && person.career.site_role.is_none()
            && person.career.automatic_retirement_round.is_none()
    }));
}

#[test]
fn wounds_need_supplied_unbesieged_steps_and_can_recover_at_a_local_site() {
    let (data, mut supplied) = fixture();
    add_person(
        &mut supplied,
        5,
        1,
        PersonClass::Medic,
        24,
        PersonAssignment::Site { site: SiteId(1) },
    );
    supplied.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    supplied.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 2,
    };
    apply(
        &mut supplied,
        &data,
        Actor::Player,
        Command::RecoverPersonAtSite {
            person: PersonId(1),
            site: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(
        supplied.people[&PersonId(1)].assignment,
        PersonAssignment::Site { site: SiteId(1) }
    );
    finish_round(&mut supplied, &data);
    assert!(matches!(
        supplied.people[&PersonId(1)].status,
        PersonStatus::Wounded {
            remaining_steps: 1,
            ..
        }
    ));
    assert_eq!(
        supplied.people[&PersonId(5)].evidence.counts[&EvidenceKind::TreatedWounded],
        1
    );
    finish_round(&mut supplied, &data);
    assert_eq!(supplied.people[&PersonId(1)].status, PersonStatus::Fit);
    assert_eq!(
        supplied.people[&PersonId(5)].evidence.counts[&EvidenceKind::TreatedWounded],
        2
    );
    assert_eq!(
        supplied.people[&PersonId(5)].career.notable_sites
            [&kestrum::data::progression::EpithetFact::TreatedWounded],
        SiteId(1)
    );

    let (data, mut remote_medic) = fixture();
    add_person(
        &mut remote_medic,
        5,
        1,
        PersonClass::Medic,
        24,
        PersonAssignment::Site { site: SiteId(5) },
    );
    remote_medic.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    remote_medic.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 2,
    };
    finish_round(&mut remote_medic, &data);
    finish_round(&mut remote_medic, &data);
    assert_eq!(
        remote_medic.people[&PersonId(5)]
            .evidence
            .counts
            .get(&EvidenceKind::TreatedWounded)
            .copied()
            .unwrap_or(0),
        0
    );

    let (data, mut besieged) = fixture();
    let faction_two = FactionId(2);
    apply(
        &mut besieged,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: faction_two,
        },
    )
    .unwrap();
    besieged
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap()
        .military = MilitaryLayer::Fort;
    besieged.world.contested_sites.insert(SiteId(1));
    let besieger = besieged
        .armies
        .values()
        .find(|army| army.faction == faction_two)
        .unwrap()
        .id;
    besieged.armies.get_mut(&besieger).unwrap().site = SiteId(1);
    besieged.sieges.insert(
        SiteId(1),
        Siege {
            id: SiegeId(1),
            site: SiteId(1),
            defender: FactionId(1),
            besieger: faction_two,
            defending: vec![ArmyId(1)],
            besieging: vec![besieger],
            started_round: 0,
            elapsed_steps: 0,
            last_progress_round: None,
        },
    );
    besieged.next_ids.siege = SiegeId(2);
    besieged.people.get_mut(&PersonId(1)).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(1) };
    besieged.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    besieged.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 2,
    };
    besieged.validate(&data).unwrap();
    finish_round(&mut besieged, &data);
    assert!(matches!(
        besieged.people[&PersonId(1)].status,
        PersonStatus::Wounded {
            remaining_steps: 2,
            ..
        }
    ));
}

#[test]
fn elders_retain_command_until_choice_and_governors_add_one_pressure() {
    let (data, mut campaign) = fixture();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.class = FounderClass::Officer;
    person.birth_round = -223;
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    campaign.validate(&data).unwrap();
    finish_round(&mut campaign, &data);
    let elder = &campaign.people[&PersonId(1)];
    assert_eq!(elder.age_years(campaign.completed_rounds), 56);
    assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(PersonId(1)));
    let command_power = campaign.army_leadership_permille(ArmyId(1), &data).unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    let passenger_power = campaign.army_leadership_permille(ArmyId(1), &data).unwrap();
    assert!(command_power > passenger_power);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::AppointGovernor {
            person: PersonId(1),
            site: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
    assert_eq!(
        campaign.people[&PersonId(1)].career.site_role,
        Some(PersonSiteRole::Governor)
    );
    let view = development_view(&campaign, &data, FactionId(1), SiteId(1)).unwrap();
    assert!(view
        .causes
        .iter()
        .any(|cause| cause.label == "Governor" && cause.amount == 1));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RetirePerson {
            person: PersonId(1),
            site: SiteId(1),
        },
    )
    .unwrap();
    let retired = &campaign.people[&PersonId(1)];
    assert!(retired.career.retired);
    assert_eq!(retired.career.site_role, None);
    assert_eq!(
        retired.assignment,
        PersonAssignment::Site { site: SiteId(1) }
    );
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetCommander {
            army: ArmyId(1),
            person: Some(PersonId(1)),
        },
    )
    .is_err());
}

#[test]
fn birthdays_roll_once_in_person_id_order_and_death_releases_every_duty() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 4;
    let (seed, first_roll, second_roll) = seed_for_rolls(false, true);
    campaign.rng.people = SeededRng::new(seed);
    let first = campaign.people.get_mut(&PersonId(1)).unwrap();
    first.class = PersonClass::Officer;
    first.birth_round = -235;
    first.service_start_round = 0;
    first
        .career
        .discipline_service_seasons
        .insert(kestrum::data::progression::TrainingDiscipline::Command, 4);
    first.career.course = Some(PersonCourse::Class {
        paid_gold: 20,
        target: PersonClass::Archer,
        site: SiteId(1),
        steps_completed: 0,
    });
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    let third = campaign.people.get_mut(&PersonId(3)).unwrap();
    third.class = PersonClass::Officer;
    third.birth_round = -275;
    third.service_start_round = 0;
    campaign.armies.get_mut(&ArmyId(3)).unwrap().commander = Some(PersonId(3));
    add_person(
        &mut campaign,
        5,
        1,
        PersonClass::Recruit,
        17,
        PersonAssignment::Site { site: SiteId(1) },
    );
    campaign.mentorships.insert(
        PersonId(5),
        Mentorship {
            mentor: PersonId(1),
            discipline: kestrum::data::progression::TrainingDiscipline::Command,
            started_round: 4,
            seasons_completed: 0,
            status: MentorshipStatus::Active,
        },
    );
    campaign.validate(&data).unwrap();

    let mut expected = SeededRng::new(seed);
    assert_eq!(expected.below(1000), first_roll);
    assert_eq!(expected.below(1000), second_roll);
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.rng.people.state(), expected.state());
    assert!(matches!(
        campaign.people[&PersonId(1)].status,
        PersonStatus::Dead { .. }
    ));
    assert_eq!(campaign.people[&PersonId(1)].career.course, None);
    assert_eq!(campaign.people[&PersonId(1)].career.site_role, None);
    assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
    assert!(campaign.mentorships.is_empty());
    assert!(campaign.people[&PersonId(3)].career.retired);
    assert_eq!(campaign.armies[&ArmyId(3)].commander, None);
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetCommander {
            army: ArmyId(1),
            person: Some(PersonId(1)),
        },
    )
    .is_err());
    assert!(campaign.validate(&data).is_ok());
}

#[test]
fn light_cavalry_person_allowance_is_nine_before_forty_one_and_eight_through_fifty_five() {
    let (data, mut campaign) = fixture();
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .service
        .specialization = Some(FormationSpecialization::LightCavalry);
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.class = PersonClass::Cavalry;
    person.birth_round = -160;
    assert_eq!(
        campaign.person_movement_allowance(&campaign.people[&PersonId(1)], &data),
        9
    );
    campaign.people.get_mut(&PersonId(1)).unwrap().birth_round = -164;
    assert_eq!(
        campaign.person_movement_allowance(&campaign.people[&PersonId(1)], &data),
        8
    );
    campaign.people.get_mut(&PersonId(1)).unwrap().class = PersonClass::Infantry;
    assert_eq!(
        campaign.person_movement_allowance(&campaign.people[&PersonId(1)], &data),
        6
    );
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .service
        .specialization = None;
    campaign.people.get_mut(&PersonId(1)).unwrap().class = PersonClass::Cavalry;
    assert_eq!(
        campaign.person_movement_allowance(&campaign.people[&PersonId(1)], &data),
        8
    );
}

#[test]
fn automatic_retirement_uses_the_nearest_owned_inhabited_route() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 4;
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.class = PersonClass::Officer;
    person.birth_round = -275;
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    let (seed, _, _) = seed_for_rolls(true, true);
    campaign.rng.people = SeededRng::new(seed);
    campaign.validate(&data).unwrap();
    finish_round(&mut campaign, &data);
    let retired = &campaign.people[&PersonId(1)];
    assert!(retired.career.retired);
    assert_eq!(retired.career.automatic_retirement_round, Some(5));
    assert_eq!(
        retired.assignment,
        PersonAssignment::Site { site: SiteId(1) }
    );
    assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
    assert_eq!(retired.status, PersonStatus::Fit);
}
