use kestrum::{
    data::{
        economy::TroopKind,
        progression::EpithetFact,
        world::{FactionId, FounderClass, SiteId},
        GameData,
    },
    engine::{apply, Actor, Command, MoveOrder},
    state::{
        battle::BattleReport,
        evidence::{EvidenceKind, FormationService, Veterancy},
        history::{HistoryKind, LifeEvent},
        military::{ArmyId, Formation, FormationId},
        people::{PersonAssignment, PersonId, PersonStatus},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;

#[path = "support/phases.rs"]
mod phases;
#[path = "support/evidence.rs"]
#[allow(dead_code)]
mod support;
use phases::pass_npc;

fn new_apprentice() -> (GameData, StrategicCampaign, PersonId) {
    new_apprentice_with_threshold(None)
}

fn new_apprentice_with_threshold(
    recognition_threshold: Option<u8>,
) -> (GameData, StrategicCampaign, PersonId) {
    let (mut data, mut campaign) = support::fixture();
    if let Some(threshold) = recognition_threshold {
        data.progression.recognition.personal_engagements = threshold;
    }
    campaign.rng.people = SeededRng::new(260_926);
    campaign.people.get_mut(&PersonId(1)).unwrap().assignment =
        PersonAssignment::Site { site: SiteId(1) };
    campaign.validate(&data).unwrap();

    for _ in 0..data.progression.emergence.vacant_slot_engagements {
        let report = support::encounter(&mut campaign, &data, 5, 6, 100);
        assert!(report_has_vacant_formation(&report, FormationId(1)));
        support::finish(&mut campaign, &data);
    }

    let recruit = campaign
        .people
        .values()
        .find(|person| person.career.emergence.is_some())
        .expect("vacant formation service must produce a named apprentice");
    assert_eq!(
        recruit.assignment,
        PersonAssignment::Formation {
            formation: FormationId(1)
        }
    );
    assert_eq!(recruit.career.hero_service_progress, 0);
    assert!(recruit.career.recognition.is_none());
    assert!(
        recruit
            .evidence
            .counts
            .get(&kestrum::state::evidence::EvidenceKind::MeaningfulEncounter)
            .copied()
            .unwrap_or(0)
            >= u32::from(data.progression.emergence.vacant_slot_engagements)
    );
    assert_eq!(
        life_event_count(&campaign, recruit.id, |event| matches!(
            event,
            LifeEvent::Emerged { .. }
        )),
        1
    );
    assert_eq!(
        life_event_count(&campaign, recruit.id, |event| matches!(
            event,
            LifeEvent::Recognized { .. }
        )),
        0
    );
    let id = recruit.id;
    (data, campaign, id)
}

fn report_has_vacant_formation(report: &BattleReport, id: FormationId) -> bool {
    report
        .attacker
        .armies
        .iter()
        .chain(report.defender.armies())
        .flat_map(|army| &army.formations)
        .any(|formation| formation.id == id && formation.named_slot_vacant == Some(true))
}

fn report_has_fit_person(report: &BattleReport, id: PersonId) -> bool {
    report
        .attacker
        .armies
        .iter()
        .chain(report.defender.armies())
        .flat_map(|army| &army.people)
        .any(|person| person.id == id && person.starting_status == Some(PersonStatus::Fit))
}

fn life_event_count(
    campaign: &StrategicCampaign,
    person: PersonId,
    matches: impl Fn(&LifeEvent) -> bool,
) -> usize {
    campaign
        .history
        .events
        .values()
        .filter(|record| {
            matches!(
                &record.kind,
                HistoryKind::Life {
                    person: subject,
                    event,
                    ..
                } if *subject == person && matches(event)
            )
        })
        .count()
}

fn qualifying_engagement(campaign: &mut StrategicCampaign, data: &GameData, id: PersonId) {
    let report = support::encounter(campaign, data, 5, 6, 100);
    assert!(report_has_fit_person(&report, id));
    support::finish(campaign, data);
}

#[test]
fn personal_progress_cannot_exceed_its_recorded_engagements() {
    let (data, mut campaign) = support::fixture();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.career.hero_service_progress = 2;
    person
        .career
        .hero_service_sites
        .insert(EpithetFact::BattleService, SiteId(1));
    person
        .evidence
        .counts
        .insert(EvidenceKind::MeaningfulEncounter, 1);

    assert!(campaign
        .validate(&data)
        .unwrap_err()
        .contains("invalid personal Hero service"));
}

#[test]
fn personal_service_starts_at_emergence_and_grants_one_hero_at_its_data_threshold() {
    let (data, mut campaign, id) = new_apprentice_with_threshold(Some(2));
    let apprentice = campaign.people[&id].clone();
    let threshold = data.progression.recognition.personal_engagements;

    qualifying_engagement(&mut campaign, &data, id);
    assert_eq!(campaign.people[&id].career.hero_service_progress, 1);
    assert_eq!(
        campaign.people[&id].career.recognition.is_some(),
        threshold == 1
    );
    assert_eq!(
        life_event_count(&campaign, id, |event| matches!(
            event,
            LifeEvent::Recognized { .. }
        )),
        usize::from(threshold == 1)
    );

    let mut reloaded = support::reload(&campaign, &data);
    assert_eq!(reloaded, campaign);

    let source = reloaded.formations[&FormationId(1)].clone();
    let target_id = reloaded.next_ids.formation;
    reloaded.next_ids.formation.0 += 1;
    reloaded.formations.insert(
        target_id,
        Formation {
            id: target_id,
            service: FormationService::default(),
            created_round: reloaded.completed_rounds,
            movement_spent: 0,
            ..source
        },
    );
    reloaded.armies.get_mut(&ArmyId(1)).unwrap().slots[1] = Some(target_id);
    apply(
        &mut reloaded,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: id,
            to_formation: target_id,
        },
    )
    .unwrap();
    assert_eq!(reloaded.people[&id].career.hero_service_progress, 1);
    assert_eq!(
        reloaded.people[&id].assignment,
        PersonAssignment::Formation {
            formation: target_id
        }
    );
    assert_eq!(
        reloaded.formations[&target_id]
            .service
            .ledger
            .counts
            .get(&EvidenceKind::MeaningfulEncounter),
        None,
        "transferring a person must not import their new formation's history"
    );
    reloaded.validate(&data).unwrap();

    for expected in 2..=threshold {
        qualifying_engagement(&mut reloaded, &data, id);
        assert_eq!(reloaded.people[&id].career.hero_service_progress, expected);
        assert_eq!(
            reloaded.people[&id].career.recognition.is_some(),
            expected == threshold
        );
        assert_eq!(
            life_event_count(&reloaded, id, |event| matches!(
                event,
                LifeEvent::Recognized { .. }
            )),
            usize::from(expected == threshold)
        );
    }
    let hero = &reloaded.people[&id];
    assert_eq!(hero.career.hero_service_progress, threshold);
    assert!(hero.career.recognition.is_some());
    assert_eq!(hero.id, apprentice.id);
    assert_eq!(hero.name, apprentice.name);
    assert_eq!(hero.appearance, apprentice.appearance);
    assert_eq!(hero.class, apprentice.class);
    assert_eq!(
        hero.assignment,
        PersonAssignment::Formation {
            formation: target_id
        }
    );
    assert_eq!(
        life_event_count(&reloaded, id, |event| matches!(
            event,
            LifeEvent::Emerged { .. }
        )),
        1
    );
    assert_eq!(
        life_event_count(&reloaded, id, |event| matches!(
            event,
            LifeEvent::Recognized { .. }
        )),
        1
    );
    assert_eq!(
        hero.career.recognition.as_ref().unwrap().cause,
        EpithetFact::BattleService
    );
    assert_eq!(reloaded.phase, CampaignPhase::PlayerTurn);
}

#[test]
fn retirement_before_battle_evidence_consumption_blocks_delayed_hero_recognition() {
    let (data, mut campaign, id) = new_apprentice();
    let threshold = data.progression.recognition.personal_engagements;
    for _ in 0..threshold.saturating_sub(1) {
        qualifying_engagement(&mut campaign, &data, id);
    }
    assert_eq!(
        campaign.people[&id].career.hero_service_progress,
        threshold - 1
    );

    let report = support::encounter(&mut campaign, &data, 5, 6, 100);
    assert!(report_has_fit_person(&report, id));
    campaign.people.get_mut(&id).unwrap().assignment = PersonAssignment::Site { site: SiteId(1) };
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RetirePerson {
            person: id,
            site: SiteId(1),
        },
    )
    .unwrap();
    support::finish(&mut campaign, &data);

    assert!(campaign.people[&id].career.retired);
    assert_eq!(
        campaign.people[&id].career.hero_service_progress,
        threshold - 1
    );
    assert!(campaign.people[&id].career.recognition.is_none());
    assert_eq!(
        life_event_count(&campaign, id, |event| matches!(
            event,
            LifeEvent::Recognized { .. }
        )),
        0
    );
}

#[test]
fn hero_command_retirement_and_replacement_resume_deterministically() {
    let (data, mut checkpoint, hero_id) = new_apprentice();
    let recognition_threshold = data.progression.recognition.personal_engagements;
    for _ in 0..recognition_threshold.saturating_sub(1) {
        qualifying_engagement(&mut checkpoint, &data, hero_id);
    }

    let mut uninterrupted = checkpoint.clone();
    let mut reloaded = support::reload(&checkpoint, &data);
    assert_eq!(uninterrupted, reloaded);

    for campaign in [&mut uninterrupted, &mut reloaded] {
        while campaign.people[&hero_id].career.hero_service_progress < recognition_threshold {
            qualifying_engagement(campaign, &data, hero_id);
        }
        assert!(campaign.people[&hero_id].career.recognition.is_some());

        // Retirement requires local presence at a friendly settlement.
        campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(1);
        assert_eq!(
            campaign.world.site(SiteId(1)).unwrap().controller,
            Some(campaign.people[&hero_id].faction)
        );

        apply(
            campaign,
            &data,
            Actor::Player,
            Command::SetCommander {
                army: ArmyId(1),
                person: Some(hero_id),
            },
        )
        .unwrap();
        assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(hero_id));

        let retirement_site = SiteId(1);
        apply(
            campaign,
            &data,
            Actor::Player,
            Command::RetirePerson {
                person: hero_id,
                site: retirement_site,
            },
        )
        .unwrap();
        assert!(campaign.people[&hero_id].career.retired);
        assert_eq!(campaign.armies[&ArmyId(1)].commander, None);
        assert_eq!(
            campaign.people[&hero_id].assignment,
            PersonAssignment::Site {
                site: retirement_site
            }
        );

        let report = support::encounter(campaign, &data, 5, 6, 100);
        assert!(report_has_vacant_formation(&report, FormationId(1)));
        support::finish(campaign, &data);

        let replacement = campaign
            .people
            .values()
            .find(|person| {
                person.career.emergence.as_ref().is_some_and(|record| {
                    record.source_formation == FormationId(1)
                        && record.completed_rounds == campaign.completed_rounds
                })
            })
            .expect("fresh service earns a new formation member");
        assert_ne!(replacement.id, hero_id);
        assert_eq!(
            replacement.assignment,
            PersonAssignment::Formation {
                formation: FormationId(1)
            }
        );
        assert!(campaign.people[&hero_id].career.retired);
    }

    assert_eq!(uninterrupted, reloaded);
}
