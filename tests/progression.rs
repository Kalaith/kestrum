//! K13 emergence, recognitions, courses, relationships and conversions use real receipts.

use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        progression::FormationSpecialization,
        world::{Facility, FactionId, FounderClass, PersonClass, SiteId},
        GameData,
    },
    engine::{
        advance_npc, ai, apply, career_options, preview, specialization_options, Actor, Command,
        MoveOrder,
    },
    state::{
        evidence::{EvidenceKind, FormationCourse, Veterancy},
        military::{ArmyId, FormationId},
        people::{PersonAssignment, PersonId, PersonStatus, PersonTrait, Tendency},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::{persistence::encode_slot, rng::SeededRng};
use std::collections::{BTreeMap, BTreeSet};

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/evidence.rs"]
#[allow(dead_code)] // This test binary shares the receipt fixture with K08 regression cases.
mod support;
use support::*;

#[test]
fn roster_curve_still_allows_emergence_at_twenty_and_is_seed_reproducible() {
    fn prepared() -> (GameData, StrategicCampaign, u32) {
        let (data, mut campaign) = fixture();
        let template = campaign.people[&PersonId(1)].clone();
        campaign.people.clear();
        campaign.legacy_items.clear();
        for id in 1..=20 {
            let mut person = template.clone();
            person.id = PersonId(id);
            person.faction = FactionId(3);
            person.name = format!("Witness {id}");
            person.class = PersonClass::Officer;
            person.assignment = PersonAssignment::Formation {
                formation: FormationId(7),
            };
            person.career = Default::default();
            person.evidence = Default::default();
            campaign.people.insert(PersonId(id), person);
        }
        campaign.next_ids.person = PersonId(21);
        encounter(&mut campaign, &data, 5, 6, 100);
        let surviving_headcount = campaign.formations[&FormationId(7)].headcount;
        (data, campaign, surviving_headcount)
    }

    let seed = (0..100_000)
        .find(|seed| {
            let mut rng = SeededRng::new(*seed);
            let first = rng.below(1000);
            let second = rng.below(1000);
            first >= 500 && second < 20
        })
        .expect("find deterministic roster-sensitive rolls");
    let mut first = prepared();
    let mut second = prepared();
    first.1.rng.people = SeededRng::new(seed);
    second.1.rng.people = SeededRng::new(seed);
    finish(&mut first.1, &first.0);
    finish(&mut second.1, &second.0);
    assert_eq!(first.1, second.1);
    let recruits = first
        .1
        .people
        .values()
        .filter(|person| person.faction == FactionId(3) && person.career.emergence.is_some())
        .collect::<Vec<_>>();
    assert_eq!(recruits.len(), 1, "a twenty-person roster remains eligible");
    let recruit = recruits[0];
    let emergence = recruit.career.emergence.as_ref().unwrap();
    assert_eq!(emergence.source_formation, FormationId(7));
    assert_eq!(emergence.source_troop, TroopKind::Warriors);
    assert!(first.1.world.site(emergence.site).is_some());
    assert!((18..=30).contains(&recruit.age_years(first.1.completed_rounds)));
    assert_eq!(
        recruit.assignment,
        PersonAssignment::Formation {
            formation: FormationId(7)
        }
    );
    assert_eq!(first.1.formations[&FormationId(7)].headcount, first.2);
    let record = first.1.history.events.values().find(|record| matches!(record.kind,
        kestrum::state::history::HistoryKind::Life { person, event: kestrum::state::history::LifeEvent::Emerged { .. }, .. } if person == recruit.id)).unwrap();
    assert_eq!(record.completed_rounds, emergence.completed_rounds);
    assert_eq!(record.sites[0].id, emergence.site);
    assert_eq!(record.visible_to, BTreeSet::from([FactionId(3)]));
    assert_eq!(record.armies[0].id, ArmyId(3));
}

#[test]
fn command_deeds_belong_to_the_commander_not_companions_or_emerging_recruits() {
    use kestrum::data::progression::EpithetFact;
    let (data, mut campaign) = fixture();
    let mut companion = campaign.people[&PersonId(1)].clone();
    companion.id = PersonId(4);
    companion.class = PersonClass::Recruit;
    companion.name = "A serving companion".into();
    campaign.people.insert(companion.id, companion);
    campaign.next_ids.person = PersonId(5);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    encounter(&mut campaign, &data, 5, 6, 60);
    let seed = (0..100_000)
        .find(|seed| SeededRng::new(*seed).below(1000) == 0)
        .unwrap();
    campaign.rng.people = SeededRng::new(seed);
    finish(&mut campaign, &data);
    assert!(campaign.people[&PersonId(1)].evidence.counts[&EvidenceKind::CommandedVictory] > 0);
    assert!(
        campaign.formations[&FormationId(1)].service.recent[0].encounters[0]
            .tags
            .contains(&EvidenceKind::CommandedVictory)
    );
    let recruit = campaign
        .people
        .values()
        .find(|person| person.faction == FactionId(1) && person.career.emergence.is_some())
        .expect("seeded emergence");
    for person in [&campaign.people[&PersonId(4)], recruit] {
        assert!(person.evidence.counts[&EvidenceKind::MeaningfulEncounter] > 0);
        for tag in [
            EvidenceKind::CommandedVictory,
            EvidenceKind::AssumedCommand,
            EvidenceKind::CommanderWounded,
        ] {
            assert!(!person.evidence.counts.contains_key(&tag));
        }
        assert!(!person
            .career
            .traits
            .contains(&PersonTrait::NaturalCommander));
        assert!(!person
            .career
            .notable_sites
            .contains_key(&EpithetFact::CommandedVictory));
        assert!(!person
            .career
            .notable_sites
            .contains_key(&EpithetFact::AssumedCommand));
        assert!(!matches!(
            person.career.recognition.as_ref().map(|award| award.cause),
            Some(EpithetFact::CommandedVictory | EpithetFact::AssumedCommand)
        ));
    }
    assert!(!matches!(
        recruit
            .career
            .emergence
            .as_ref()
            .unwrap()
            .distinguishing_deed,
        Some(EpithetFact::CommandedVictory | EpithetFact::AssumedCommand)
    ));
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap(),
    )
    .unwrap();
    loaded.validate(&data).unwrap();
    assert_eq!(loaded.strategic().unwrap(), &campaign);
}

#[test]
fn genuine_evidence_sets_traits_and_one_place_based_recognition() {
    let (data, mut campaign) = fixture();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.career.disposition = kestrum::state::people::Disposition {
        courage: Tendency::Positive,
        care: Tendency::Positive,
        curiosity: Tendency::Positive,
    };
    person.evidence.counts = BTreeMap::from([
        (EvidenceKind::Battle, 3),
        (EvidenceKind::MeaningfulEncounter, 3),
        (EvidenceKind::SurvivedOutnumbered, 2),
        (EvidenceKind::DefendedAnchor, 1),
        (EvidenceKind::TreatedWounded, 1),
        (EvidenceKind::AssumedCommand, 2),
    ]);
    person.evidence.service_by_troop = BTreeMap::from([(TroopKind::Warriors, 3)]);
    person.career.notable_sites = BTreeMap::from([
        (
            kestrum::data::progression::EpithetFact::SurvivedOutnumbered,
            SiteId(6),
        ),
        (
            kestrum::data::progression::EpithetFact::DefendedAnchor,
            SiteId(6),
        ),
    ]);
    campaign.validate(&data).unwrap();
    finish(&mut campaign, &data);
    let tracked = &campaign.people[&PersonId(1)];
    assert!(tracked.career.traits.contains(&PersonTrait::Bold));
    assert!(tracked.career.traits.contains(&PersonTrait::Protective));
    assert!(tracked
        .career
        .traits
        .contains(&PersonTrait::NaturalCommander));
    let recognition = tracked.career.recognition.as_ref().unwrap();
    assert_eq!(recognition.site, SiteId(6));
    assert_eq!(
        recognition.cause,
        kestrum::data::progression::EpithetFact::SurvivedOutnumbered
    );
    let first_award = recognition.clone();
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.people[&PersonId(1)].career.recognition,
        Some(first_award)
    );
    assert_eq!(campaign.people[&PersonId(1)].career.traits.len(), 3);
}

#[test]
fn all_six_ordinary_class_paths_show_valid_and_missing_evidence() {
    let (data, mut campaign) = fixture();
    install_complete_career_evidence(&mut campaign);
    campaign.validate(&data).unwrap();
    let options = career_options(&campaign, &data, PersonId(1)).unwrap();
    assert_eq!(options.len(), 6);
    assert!(options.iter().all(|option| option.eligible));

    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.evidence.counts.clear();
    person.evidence.service_by_troop.clear();
    person.evidence.traversed_routes.clear();
    let options = career_options(&campaign, &data, PersonId(1)).unwrap();
    assert!(options.iter().all(|option| !option.eligible));
    assert!(options.iter().all(|option| option
        .requirements
        .iter()
        .any(|fact| { fact.current < fact.required })));
}

#[test]
fn npc_trains_from_the_same_evidence_options_and_command_rules() {
    let (data, mut campaign) = fixture();
    install_complete_career_evidence(&mut campaign);
    let evidence = campaign.people[&PersonId(1)].evidence.clone();
    let npc = campaign.people.get_mut(&PersonId(3)).unwrap();
    npc.class = PersonClass::Recruit;
    npc.evidence = evidence;
    let site = campaign
        .armies
        .values()
        .find(|army| army.faction == FactionId(3))
        .unwrap()
        .site;
    let location = campaign
        .world
        .sites
        .iter_mut()
        .find(|location| location.id == site)
        .unwrap();
    if !location.facilities.contains(&Facility::TrainingGround) {
        location.facilities.push(Facility::TrainingGround);
    }
    campaign.factions.get_mut(&FactionId(3)).unwrap().resources = Resources {
        gold: 20,
        wood: 0,
        stone: 0,
    };
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == FactionId(3))
    {
        formation.movement_spent = formation.movement_allowance(&data);
    }
    campaign.validate(&data).unwrap();
    let options = career_options(&campaign, &data, PersonId(3)).unwrap();
    assert!(
        options
            .iter()
            .find(|option| option.class == PersonClass::Infantry)
            .unwrap()
            .eligible
    );

    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    pass_npc(&mut campaign, &data).unwrap();
    let decision = ai::propose(&campaign, &data, FactionId(3)).unwrap();
    assert_eq!(
        decision.command,
        Command::TrainPerson {
            person: PersonId(3),
            class: PersonClass::Infantry,
            site,
        }
    );
    preview(&campaign, &data, Actor::Npc(FactionId(3)), decision.command).unwrap();
    advance_npc(&mut campaign, &data).unwrap();
    assert!(matches!(
        campaign.people[&PersonId(3)].career.course,
        Some(kestrum::state::people::PersonCourse::Class {
            target: PersonClass::Infantry,
            steps_completed: 0,
            ..
        })
    ));
    assert_eq!(campaign.factions[&FactionId(3)].resources.gold, 0);
}

#[test]
fn person_course_pauses_for_wounds_and_refunds_only_before_progress() {
    let (data, mut campaign) = fixture();
    install_complete_career_evidence(&mut campaign);
    campaign.people.get_mut(&PersonId(1)).unwrap().class = PersonClass::Recruit;
    let opening_gold = campaign.factions[&FactionId(1)].resources.gold;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: PersonId(1),
            class: PersonClass::Infantry,
            site: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.factions[&FactionId(1)].resources.gold,
        opening_gold - 20
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelPersonCourse {
            person: PersonId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.factions[&FactionId(1)].resources.gold,
        opening_gold
    );

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: PersonId(1),
            class: PersonClass::Infantry,
            site: SiteId(1),
        },
    )
    .unwrap();
    campaign.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Wounded {
        since_round: 0,
        remaining_steps: 2,
    };
    finish(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&PersonId(1)].career.course,
        Some(kestrum::state::people::PersonCourse::Class {
            steps_completed: 0,
            ..
        })
    ));
    campaign.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Fit;
    finish(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&PersonId(1)].career.course,
        Some(kestrum::state::people::PersonCourse::Class {
            steps_completed: 1,
            ..
        })
    ));
    finish(&mut campaign, &data);
    assert_eq!(campaign.people[&PersonId(1)].class, PersonClass::Infantry);
    assert!(campaign.people[&PersonId(1)].career.course.is_none());
}

#[test]
fn a_known_rival_requires_two_actual_mutual_combats() {
    let (data, mut campaign) = fixture();
    let mut companion = campaign.people[&PersonId(1)].clone();
    companion.id = PersonId(4);
    companion.name = "Della Rose".into();
    companion.career = Default::default();
    companion.evidence = Default::default();
    campaign.people.insert(companion.id, companion);
    campaign.next_ids.person = PersonId(5);
    encounter(&mut campaign, &data, 5, 6, 100);
    encounter(&mut campaign, &data, 8, 10, 100);
    finish(&mut campaign, &data);
    let first = &campaign.people[&PersonId(1)].career.relationships[&PersonId(3)];
    let second = &campaign.people[&PersonId(3)].career.relationships[&PersonId(1)];
    assert_eq!(first.mutual_combat_rounds, 2);
    assert_eq!(first.shared_service_seasons, 0);
    assert_eq!(first, second);
    let service_link = &campaign.people[&PersonId(1)].career.relationships[&PersonId(4)];
    assert_eq!(service_link.shared_service_seasons, 1);
    assert_eq!(service_link.mutual_combat_rounds, 0);
    campaign.validate(&data).unwrap();
}

#[test]
fn specialization_preserves_formation_identity_service_and_capacity() {
    let (data, mut campaign) = fixture();
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.service.ledger.counts = BTreeMap::from([
        (EvidenceKind::Battle, 3),
        (EvidenceKind::MeaningfulEncounter, 3),
        (EvidenceKind::DefendedAnchor, 3),
    ]);
    formation.service.ledger.service_by_troop = BTreeMap::from([(TroopKind::Warriors, 3)]);
    formation.service.xp = 12;
    formation.service.tier = kestrum::state::evidence::Veterancy::Seasoned;
    campaign.validate(&data).unwrap();
    assert!(specialization_options(&campaign, &data, FormationId(1))
        .unwrap()
        .iter()
        .any(
            |option| option.specialization == FormationSpecialization::ShieldGuard
                && option.eligible
        ));
    let original = campaign.formations[&FormationId(1)].clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SpecializeFormation {
            formation: FormationId(1),
            specialization: FormationSpecialization::ShieldGuard,
            site: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.formations[&FormationId(1)]
            .service
            .course
            .as_ref()
            .unwrap()
            .steps_completed,
        0
    );
    finish(&mut campaign, &data);
    finish(&mut campaign, &data);
    let converted = &campaign.formations[&FormationId(1)];
    assert_eq!(converted.id, original.id);
    assert_eq!(converted.capacity, original.capacity);
    assert_eq!(converted.headcount, original.headcount);
    assert_eq!(converted.service.xp, original.service.xp);
    assert_eq!(converted.service.tier, original.service.tier);
    assert_eq!(
        converted.service.specialization,
        Some(FormationSpecialization::ShieldGuard)
    );
    assert_eq!(converted.service.course, None::<FormationCourse>);
}

#[test]
fn light_cavalry_keeps_its_nine_point_allowance_through_combat_validation() {
    let (data, mut campaign) = fixture();
    let formation = campaign.formations.get_mut(&FormationId(1)).unwrap();
    formation.kind = TroopKind::Riders;
    formation.capacity = data.economy.formations[&TroopKind::Riders].capacity;
    formation.headcount = formation.capacity;
    formation.service.specialization = Some(FormationSpecialization::LightCavalry);
    assert_eq!(formation.movement_allowance(&data), 9);
    encounter(&mut campaign, &data, 5, 6, 100);
    assert_eq!(campaign.formations[&FormationId(1)].movement_spent, 9);
    campaign.validate(&data).unwrap();
}

#[test]
fn legacy_k12_people_receive_a_deterministic_k13_default() {
    let (data, campaign) = fixture();
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    for person in value["people"].as_object_mut().unwrap().values_mut() {
        person.as_object_mut().unwrap().remove("career");
    }
    let encoded = encode_slot("strategic_v2", &value, "2").unwrap();
    let load = || {
        kestrum::state::persistence::load_legacy(&encoded, &data)
            .unwrap()
            .strategic()
            .unwrap()
            .clone()
    };
    let first = load();
    let second = load();
    assert_eq!(first, second);
    assert!(first.people.values().all(|person| {
        person.career.emergence.is_none()
            && person.career.recognition.is_none()
            && person.career.relationships.is_empty()
    }));
}

fn install_complete_career_evidence(campaign: &mut StrategicCampaign) {
    let routes = campaign
        .world
        .routes
        .iter()
        .map(|route| route.id)
        .take(6)
        .collect::<BTreeSet<_>>();
    assert_eq!(routes.len(), 6);
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.evidence.counts = BTreeMap::from([
        (EvidenceKind::Battle, 5),
        (EvidenceKind::MeaningfulEncounter, 5),
        (EvidenceKind::CommandedVictory, 1),
        (EvidenceKind::TreatedWounded, 2),
    ]);
    person.evidence.service_by_troop = BTreeMap::from([
        (TroopKind::Warriors, 2),
        (TroopKind::Archers, 2),
        (TroopKind::Riders, 1),
    ]);
    person.evidence.traversed_routes = routes;
    person.career.riding_practice_seasons = 2;
}
