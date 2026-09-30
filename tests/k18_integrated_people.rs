//! K18's Rosemarch people chain uses accepted actions and battle receipts.

#[path = "support/k18_integrated_people.rs"]
mod support;
use support::*;

use kestrum::{
    data::{
        economy::TroopKind,
        progression::EpithetFact,
        world::{Facility, PersonClass, SiteId},
        GameData,
    },
    engine::{apply, movement_preview, person_site, Actor, Command, MoveOrder},
    state::{
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, OrderId},
        evidence::EvidenceKind,
        military::ArmyId,
        people::{PersonAssignment, PersonCourse, PersonId, PersonStatus, WoundCause},
        StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;
#[test]
fn medic_career_witnesses_casualties_then_recovers_a_wounded_commander_once() {
    let data = GameData::load().expect("Rosemarch data");
    let mut campaign = StrategicCampaign::new(&data).expect("Rosemarch campaign");
    // A fixed stream makes the limited commander-casualty rule reproducible;
    // campaign entities, evidence, and all movement still come from legal actions.
    campaign.rng.combat = SeededRng::new(9);

    assert_eq!(campaign.seed, data.scenario.seed);
    assert_eq!(campaign.world.site(WEST_GATE).unwrap().key, "west_gate");
    assert_eq!(campaign.world.site(MILLTOWN).unwrap().key, "milltown");
    assert_eq!(campaign.world.site(HIGH_FORT).unwrap().key, "high_fort");
    assert_eq!(campaign.armies[&ROSE_ARMY].formation_ids().count(), 3);
    assert_eq!(campaign.formations[&FIRST_WARRIORS].headcount, 100);
    assert_eq!(campaign.people[&AVELINE].class, PersonClass::Officer);

    let infirmary_order = campaign.next_ids.order;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(HQ),
            kind: ConstructionKind::Facility(Facility::Infirmary),
            builder: ROSE_ARMY,
        },
    )
    .expect("place the supplied Rose HQ Infirmary order");
    let apprentice = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: HQ },
    )
    .expect("invite a local apprentice through the yearly rule")
    .new_people[0];
    assert_eq!(apprentice, PersonId(5));
    assert_eq!(campaign.people[&apprentice].age_years(0), 17);
    assert_eq!(
        campaign.people[&apprentice].assignment,
        PersonAssignment::Site { site: HQ }
    );

    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.construction[&OrderId(infirmary_order.0)].progress,
        1
    );
    finish_round(&mut campaign, &data);
    assert!(matches!(
        campaign.construction[&OrderId(infirmary_order.0)].status,
        ConstructionStatus::Completed { .. }
    ));
    assert!(campaign
        .world
        .site(HQ)
        .unwrap()
        .facilities
        .contains(&Facility::Infirmary));

    let medic_formation = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Recruit {
            site: HQ,
            army: Some(ROSE_ARMY),
            kind: TroopKind::Medics,
        },
    )
    .expect("recruit Medics only after the Infirmary is complete")
    .recruited
    .unwrap()
    .formation;
    assert_eq!(campaign.formations[&medic_formation].headcount, 40);
    assert_eq!(
        campaign.formations[&medic_formation].kind,
        TroopKind::Medics
    );
    assert_ne!(medic_formation, SPEARMEN);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: apprentice,
            to_formation: medic_formation,
        },
    )
    .expect("attach the apprentice to the real Medics formation");
    // A newly recruited formation starts with its movement spent. Let one
    // ordinary boundary restore it before asking the mixed army to march.
    finish_round(&mut campaign, &data);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: AVELINE,
            to_formation: SPEARMEN,
        },
    )
    .expect("keep AVELINE with the Rose holding force when Warriors split off");
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetCommander {
            army: ROSE_ARMY,
            person: None,
        },
    )
    .expect("no appointed commander during the evidence-only battles");
    let high_fort_army = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FIRST_WARRIORS,
        },
    )
    .expect("split the Warriors into a second legal front")
    .split_army
    .unwrap();
    assert_eq!(high_fort_army, ArmyId(5));
    assert_eq!(campaign.armies[&ROSE_ARMY].slots[1], Some(SPEARMEN));
    assert!(campaign.armies[&ROSE_ARMY]
        .slots
        .contains(&Some(medic_formation)));
    assert_eq!(
        campaign.armies[&high_fort_army].slots[0],
        Some(FIRST_WARRIORS)
    );

    let approach = movement_preview(&campaign, &data, ROSE, &[ROSE_ARMY], MILLTOWN)
        .expect("preview the physical West Gate route");
    assert_eq!(approach.order.path, vec![HQ, WEST_GATE, MILLTOWN]);
    assert!(approach.total_cost <= 6, "route costs {:?}", approach.steps);
    assert_eq!(approach.reachable_steps, approach.steps.len());
    let moved = move_army(
        &mut campaign,
        &data,
        Actor::Player,
        ROSE_ARMY,
        &[HQ, WEST_GATE, MILLTOWN],
    );
    assert_eq!(moved.movement.unwrap().spent, approach.total_cost);
    assert_eq!(
        campaign.world.site(WEST_GATE).unwrap().controller,
        Some(ROSE)
    );
    assert_eq!(
        campaign.world.site(MILLTOWN).unwrap().controller,
        Some(ROSE)
    );
    assert!(campaign.supplied_sites(ROSE).contains(&MILLTOWN));
    assert_eq!(campaign.armies[&ROSE_ARMY].site, MILLTOWN);

    move_to_site(&mut campaign, &data, ROSE, high_fort_army, BRIDGE);
    move_to_site(&mut campaign, &data, ROSE, high_fort_army, HIGH_FORT);
    assert_eq!(campaign.armies[&high_fort_army].site, HIGH_FORT);
    assert_eq!(
        campaign.world.site(HIGH_FORT).unwrap().controller,
        Some(ROSE)
    );
    finish_hawthorn_approach(&mut campaign, &data);
    assert_eq!(campaign.armies[&HAWTHORN_ARMY].site, SiteId(3));
    assert_eq!(campaign.armies[&ArmyId(6)].site, BRIDGE);

    // A same-faction person stays at home through the second encounter. They
    // must not inherit another person's service from a shared side or report.
    let remote_person = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: HQ },
    )
    .expect("the next-year invitation is legal at the owned, supplied Village")
    .new_people[0];
    assert_eq!(remote_person, PersonId(6));
    let first_battle = finish_hawthorn_counterattack(&mut campaign, &data);
    let first = campaign.battles[&first_battle].clone();
    assert_eq!(first.site, MILLTOWN);
    assert_side_casualties_and_medic(&first, ROSE, ROSE_ARMY, medic_formation, apprentice);
    assert_eq!(
        campaign.people[&apprentice]
            .evidence
            .counts
            .get(&EvidenceKind::TreatedWounded)
            .copied()
            .unwrap_or_default(),
        1,
        "a living medic attached to the casualty-taking army earns treatment service"
    );
    assert!(!campaign.people[&remote_person]
        .evidence
        .counts
        .contains_key(&EvidenceKind::MeaningfulEncounter));
    let rejected_course = campaign.clone();
    assert!(!medic_option(&campaign, &data, apprentice).eligible);
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: apprentice,
            class: PersonClass::Medic,
            site: HQ,
        },
    )
    .is_err());
    assert_eq!(
        campaign, rejected_course,
        "one casualty encounter cannot bypass two occasions"
    );
    // Hawthorn moves its other army by the same legal roads, then counterattacks
    // the Milltown garrison again in a distinct season with real losses.
    finish_hawthorn_approach(&mut campaign, &data);
    assert_eq!(campaign.armies[&HAWTHORN_ARMY].site, BRIDGE);
    let second_battle = finish_hawthorn_main_counterattack(&mut campaign, &data);
    let second = campaign.battles[&second_battle].clone();
    assert_eq!(second.site, MILLTOWN);
    assert_side_casualties_and_medic(&second, ROSE, ROSE_ARMY, medic_formation, apprentice);
    assert_eq!(
        campaign.people[&apprentice].evidence.counts[&EvidenceKind::TreatedWounded],
        2
    );
    assert_eq!(campaign.people[&remote_person].evidence.counts.len(), 0);
    assert!(campaign.people[&remote_person]
        .evidence
        .service_by_troop
        .is_empty());
    assert!(campaign.people[&remote_person]
        .evidence
        .traversed_routes
        .is_empty());
    assert_eq!(
        campaign.people[&AVELINE].evidence.counts[&EvidenceKind::MeaningfulEncounter],
        2,
        "only the named people in the combat army gain personal encounter credit"
    );
    assert!(medic_option(&campaign, &data, apprentice).eligible);
    assert_eq!(
        campaign.people[&apprentice].career.notable_sites[&EpithetFact::TreatedWounded],
        MILLTOWN
    );
    assert_eq!(campaign.armies[&ROSE_ARMY].site, MILLTOWN);
    assert!(campaign.supplied_sites(ROSE).contains(&MILLTOWN));

    return_to_site(&mut campaign, &data, ROSE_ARMY, HQ);
    assert_eq!(campaign.armies[&ROSE_ARMY].site, HQ);
    // Consume the return-trip movement receipt before starting a course that
    // requires an uninterrupted fit season at the Infirmary.
    finish_round(&mut campaign, &data);
    let blocked_location = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: apprentice,
            class: PersonClass::Medic,
            site: MILLTOWN,
        },
    )
    .is_err());
    assert_eq!(
        campaign, blocked_location,
        "the Infirmary remains a real course prerequisite"
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TrainPerson {
            person: apprentice,
            class: PersonClass::Medic,
            site: HQ,
        },
    )
    .expect("the two real treatment occasions unlock the ordinary Medic course");
    finish_round(&mut campaign, &data);
    assert!(
        matches!(
            campaign.people[&apprentice].career.course,
            Some(PersonCourse::Class {
                target: PersonClass::Medic,
                steps_completed: 1,
                ..
            })
        ),
        "unexpected course after one season: {:?}",
        campaign.people[&apprentice].career.course
    );
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.people[&apprentice].class, PersonClass::Medic);
    assert!(campaign.people[&apprentice].career.course.is_none());
    assert_eq!(campaign.people[&remote_person].evidence.counts.len(), 0);

    // Keep the parties in their actual sites while the course completes. The
    // surviving Hawthorn main army can legally take Milltown and be attacked by
    // the now-trained Medic on the return route.
    let commander_formation = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Recruit {
            site: HQ,
            army: Some(ROSE_ARMY),
            kind: TroopKind::Warriors,
        },
    )
    .expect("replace the depleted front with a legal full-strength formation")
    .recruited
    .unwrap()
    .formation;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: AVELINE,
            to_formation: commander_formation,
        },
    )
    .expect("Aveline joins the fresh frontline formation at the Rose HQ");
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.formations[&commander_formation].kind,
        TroopKind::Warriors
    );
    let enemy_site = campaign.armies[&HAWTHORN_ARMY].site;
    assert!(campaign.armies[&HAWTHORN_ARMY]
        .formation_ids()
        .any(|id| campaign.formations[&id].headcount > 0));
    assert!(matches!(
        campaign.people[&AVELINE].status,
        PersonStatus::Fit
    ));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetCommander {
            army: ROSE_ARMY,
            person: Some(AVELINE),
        },
    )
    .expect("AVELINE is fit, adult and attached to this army");
    assert_eq!(
        campaign.people[&apprentice].assignment,
        PersonAssignment::Formation {
            formation: medic_formation
        }
    );
    assert_eq!(
        campaign.people[&AVELINE].assignment,
        PersonAssignment::Formation {
            formation: commander_formation
        }
    );
    let final_path = shortest_path(&campaign, HQ, enemy_site);
    let final_preview = movement_preview(&campaign, &data, ROSE, &[ROSE_ARMY], enemy_site)
        .expect("the tracked hostile army is physically reachable");
    assert_eq!(final_preview.order.path, final_path);
    assert!(final_preview.total_cost <= final_preview.remaining);
    let final_move = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ROSE_ARMY],
            path: final_path,
        }),
    )
    .expect("attack the real surviving Hawthorn force");
    assert!(final_move.battle_pending);
    let final_battle = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap()
    .battle
    .expect("the hostile army triggers combat");
    let final_report = campaign.battles[&final_battle].clone();
    assert!(
        final_report
            .person_events
            .iter()
            .any(|event| event.person == AVELINE
                && matches!(
                    event.outcome,
                    kestrum::state::people::PersonCombatOutcome::Wounded {
                        cause: WoundCause::CommandCasualty,
                        ..
                    }
                )),
        "final battle report: {final_report:#?}"
    );
    assert!(final_report.person_events.iter().any(|event| {
        event.person == apprentice
            && matches!(
                event.outcome,
                kestrum::state::people::PersonCombatOutcome::AssumedCommand {
                    army: ROSE_ARMY,
                    previous: AVELINE,
                }
            )
    }));
    assert!(matches!(
        campaign.people[&AVELINE].status,
        PersonStatus::Wounded {
            remaining_steps: 2,
            ..
        }
    ));
    assert_eq!(campaign.armies[&ROSE_ARMY].commander, Some(apprentice));
    let wound_site = person_site(&campaign, AVELINE).unwrap();
    assert_eq!(person_site(&campaign, apprentice), Some(wound_site));
    assert!(campaign.supply_path(ROSE, wound_site).is_some());
    assert!(!campaign.sieges.contains_key(&wound_site));
    assert_side_casualties_and_medic(&final_report, ROSE, ROSE_ARMY, medic_formation, apprentice);
    assert_eq!(campaign.people[&remote_person].evidence.counts.len(), 0);

    let wound_recovery_before = campaign.people[&apprentice]
        .evidence
        .counts
        .get(&EvidenceKind::TreatedWounded)
        .copied()
        .unwrap_or(0);
    let mut resumed = reload(&campaign, &data);
    assert_eq!(resumed, campaign);
    finish_round(&mut campaign, &data);
    finish_round(&mut resumed, &data);
    assert_eq!(campaign, resumed);
    assert!(matches!(
        campaign.people[&AVELINE].status,
        PersonStatus::Wounded {
            remaining_steps: 1,
            ..
        }
    ));
    assert!(
        campaign.people[&apprentice]
            .evidence
            .counts
            .get(&EvidenceKind::AssumedCommand)
            .copied()
            .unwrap_or(0)
            > 0
    );
    let first_treatment =
        campaign.people[&apprentice].evidence.counts[&EvidenceKind::TreatedWounded];
    assert!(first_treatment > wound_recovery_before);
    finish_round(&mut campaign, &data);
    finish_round(&mut resumed, &data);
    assert_eq!(campaign, resumed);
    assert_eq!(campaign.people[&AVELINE].status, PersonStatus::Fit);
    assert!(
        campaign.people[&apprentice].evidence.counts[&EvidenceKind::TreatedWounded]
            > first_treatment
    );
    assert_eq!(campaign.people[&remote_person].evidence.counts.len(), 0);

    let recognition = campaign.people[&apprentice]
        .career
        .recognition
        .as_ref()
        .expect("three witnessed encounters and earned facts award recognition");
    let recognition_evidence = match recognition.cause {
        EpithetFact::TreatedWounded => EvidenceKind::TreatedWounded,
        EpithetFact::AssumedCommand => EvidenceKind::AssumedCommand,
        EpithetFact::CommandedVictory => EvidenceKind::CommandedVictory,
        EpithetFact::DefendedAnchor => EvidenceKind::DefendedAnchor,
        EpithetFact::CapturedAnchor => EvidenceKind::CapturedAnchor,
        EpithetFact::SurvivedOutnumbered => EvidenceKind::SurvivedOutnumbered,
    };
    assert!(
        campaign.people[&apprentice]
            .evidence
            .counts
            .get(&recognition_evidence)
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert_eq!(
        campaign.people[&apprentice].career.notable_sites[&recognition.cause],
        recognition.site
    );
    assert_eq!(battle_history_count(&campaign, final_battle), 1);
    assert_eq!(battle_history_count(&resumed, final_battle), 1);
    campaign.validate(&data).unwrap();
}
