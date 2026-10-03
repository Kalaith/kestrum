//! K16 heirlooms, remembrance, private chronicles and additive save migration.

#[path = "support/appearance_frozen.rs"]
mod appearance_support;

use kestrum::{
    data::{
        progression::TrainingDiscipline,
        world::{DiplomaticState, FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{apply, history_page, person_site, project, Actor, Command, HistoryFilter},
    state::{
        campaign::FactId,
        diplomacy::DiplomacyReceipt,
        history::{AnniversarySubject, HistoryId, HistoryKind, HistoryRecord, HistorySubject},
        legacy::{LegacyItemCustody, LegacyItemId},
        people::{CompletedApprenticeship, PersonAssignment, PersonId, PersonStatus},
        relationships::{FamilyLink, FamilyOrigin, LegacyCategory, PersonFamily, SuccessorLink},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::{persistence::encode_slot, rng::SeededRng};

const PLAYER: FactionId = FactionId(1);
const OTHER: FactionId = FactionId(2);
const FOUNDER: PersonId = PersonId(1);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn add_person(
    campaign: &mut StrategicCampaign,
    faction: FactionId,
    assignment: PersonAssignment,
    age: u32,
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
    person.name = format!("K16 Person {}", id.0);
    person.birth_round = i64::from(campaign.completed_rounds) - i64::from(age) * 4;
    person.service_start_round = 0;
    person.class = PersonClass::Recruit;
    person.assignment = assignment;
    person.movement_spent = 0;
    person.status = PersonStatus::Fit;
    person.career = Default::default();
    person.evidence = Default::default();
    person.appearance = appearance_support::allocate_frozen(campaign, id);
    campaign.people.insert(id, person);
    id
}

fn item_held_by(campaign: &StrategicCampaign, person: PersonId) -> LegacyItemId {
    campaign
        .legacy_items
        .values()
        .find(|item| item.custody == LegacyItemCustody::Person(person))
        .expect("founder has a starting heirloom")
        .id
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

fn death_seed() -> u64 {
    (1..100_000)
        .find(|seed| {
            let mut rng = SeededRng::new(*seed);
            rng.below(1000) < 20 && rng.below(1000) >= 50
        })
        .expect("fixed birthday rolls for founder death and rival survival")
}

fn name_item_heir(data: &GameData, campaign: &mut StrategicCampaign, heir: PersonId) {
    let seasons = data.lifecycle.apprenticeship_seasons;
    campaign
        .people
        .get_mut(&heir)
        .unwrap()
        .career
        .mentorship_seasons
        .insert(TrainingDiscipline::Command, seasons);
    campaign
        .people
        .get_mut(&heir)
        .unwrap()
        .career
        .completed_mentors
        .insert(CompletedApprenticeship {
            mentor: FOUNDER,
            discipline: TrainingDiscipline::Command,
            started_round: campaign.completed_rounds - seasons,
            completed_round: campaign.completed_rounds,
        });
    apply(
        campaign,
        data,
        Actor::Player,
        Command::DesignateSuccessor {
            predecessor: FOUNDER,
            successor: heir,
            category: LegacyCategory::Item,
            link: SuccessorLink::Martial,
        },
    )
    .unwrap();
}

fn prepare_founder_death(
    data: &GameData,
    campaign: &mut StrategicCampaign,
) -> (LegacyItemId, PersonId) {
    campaign.completed_rounds = 4;
    let site = person_site(campaign, FOUNDER).unwrap();
    let heir = add_person(campaign, PLAYER, PersonAssignment::Site { site }, 24);
    campaign.people.get_mut(&FOUNDER).unwrap().class = PersonClass::Officer;
    campaign.people.get_mut(&FOUNDER).unwrap().birth_round = -235;
    campaign
        .people
        .get_mut(&FOUNDER)
        .unwrap()
        .service_start_round = 0;
    name_item_heir(data, campaign, heir);
    campaign.rng.people = SeededRng::new(death_seed());
    (item_held_by(campaign, FOUNDER), heir)
}

#[test]
fn transfers_are_local_and_the_same_private_deed_opens_from_each_subject() {
    let (data, mut campaign) = fixture();
    let site = person_site(&campaign, FOUNDER).unwrap();
    let item = item_held_by(&campaign, FOUNDER);
    let local = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 24);
    let remote = add_person(
        &mut campaign,
        PLAYER,
        PersonAssignment::Site { site: SiteId(2) },
        24,
    );
    let original_resources = campaign.factions[&PLAYER].resources;
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferLegacyItem { item, to: remote },
    )
    .is_err());
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Npc(OTHER),
        Command::TransferLegacyItem { item, to: local },
    )
    .is_err());
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferLegacyItem { item, to: local },
    )
    .unwrap();
    assert_eq!(outcome.legacy_items_changed, vec![item]);
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(local)
    );
    assert_eq!(campaign.factions[&PLAYER].resources, original_resources);

    let history = |subject| {
        history_page(
            &campaign,
            PLAYER,
            &HistoryFilter {
                subject: Some(subject),
                ..Default::default()
            },
        )
        .entries
    };
    let from_item = history(HistorySubject::Item(item));
    let from_person = history(HistorySubject::Person(FOUNDER));
    let from_site = history(HistorySubject::Site(site));
    assert_eq!(from_item.len(), 1);
    assert_eq!(from_item[0].id, from_person[0].id);
    assert_eq!(from_item[0].id, from_site[0].id);
    assert!(matches!(
        from_item[0].kind,
        HistoryKind::ItemCustodyChanged { .. }
    ));
    assert!(history_page(
        &campaign,
        OTHER,
        &HistoryFilter {
            subject: Some(HistorySubject::Item(item)),
            ..Default::default()
        }
    )
    .entries
    .is_empty());
    assert!(project(&campaign, OTHER)
        .unwrap()
        .legacy_items
        .iter()
        .all(|entry| entry.id != item));
    campaign.validate(&data).unwrap();
}

#[test]
fn death_gives_the_same_item_to_a_designated_local_heir() {
    let (data, mut campaign) = fixture();
    let (item, heir) = prepare_founder_death(&data, &mut campaign);
    campaign.validate(&data).unwrap();
    finish_round(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&FOUNDER].status,
        PersonStatus::Dead { .. }
    ));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    assert!(campaign.history.events.values().any(|record| matches!(
        record.kind,
        HistoryKind::ItemCustodyChanged { item: recorded, .. } if recorded == item
    )));
    campaign.validate(&data).unwrap();
}

#[test]
fn a_distant_named_heir_leaves_the_item_at_the_death_site_and_retirement_keeps_custody() {
    let (data, mut campaign) = fixture();
    let (item, heir) = prepare_founder_death(&data, &mut campaign);
    campaign.people.get_mut(&heir).unwrap().assignment = PersonAssignment::Site { site: SiteId(2) };
    let death_site = person_site(&campaign, FOUNDER).unwrap();
    finish_round(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&FOUNDER].status,
        PersonStatus::Dead { .. }
    ));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::SiteEstate(death_site)
    );

    let (retired_data, mut retired) = fixture();
    let retained = item_held_by(&retired, FOUNDER);
    let site = person_site(&retired, FOUNDER).unwrap();
    apply(
        &mut retired,
        &retired_data,
        Actor::Player,
        Command::RetirePerson {
            person: FOUNDER,
            site,
        },
    )
    .unwrap();
    assert_eq!(
        retired.legacy_items[&retained].custody,
        LegacyItemCustody::Person(FOUNDER)
    );
    retired.validate(&retired_data).unwrap();
    campaign.validate(&data).unwrap();
}

#[test]
fn dependents_and_trainees_begin_service_anniversaries_only_after_enlistment() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 40;
    let site = campaign.factions[&PLAYER].headquarters;
    let child = add_person(
        &mut campaign,
        PLAYER,
        PersonAssignment::Dependent { site },
        10,
    );
    let trainee = add_person(
        &mut campaign,
        PLAYER,
        PersonAssignment::Trainee { site },
        14,
    );
    for (id, origin, link) in [
        (child, FamilyOrigin::Birth, FamilyLink::BiologicalParent),
        (
            trainee,
            FamilyOrigin::AdoptedWard,
            FamilyLink::AdoptiveGuardian,
        ),
    ] {
        campaign.families.insert(
            id,
            PersonFamily {
                origin,
                origin_site: site,
                household: None,
                links: [(FOUNDER, link)].into(),
            },
        );
    }
    let parent = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 35);
    campaign
        .families
        .get_mut(&child)
        .unwrap()
        .links
        .insert(parent, FamilyLink::BiologicalParent);
    for _ in 0..3 {
        finish_round(&mut campaign, &data);
    }
    for id in [child, trainee] {
        assert!(!campaign.history.person_last_reminded.contains_key(&id));
        assert!(!campaign.history.events.values().any(|record| matches!(record.kind,
            HistoryKind::Anniversary { subject: AnniversarySubject::Person(person), .. } if person == id)));
    }
    // An adult dependent is still not in service until the real entry command.
    campaign.people.get_mut(&child).unwrap().birth_round =
        i64::from(campaign.completed_rounds) - 68;
    finish_round(&mut campaign, &data);
    assert!(!campaign.history.person_last_reminded.contains_key(&child));
    let entered = campaign.completed_rounds;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::EnterService {
            person: child,
            formation: None,
        },
    )
    .unwrap();
    assert_eq!(campaign.people[&child].service_start_round, entered);
    finish_round(&mut campaign, &data);
    assert!(!campaign.history.person_last_reminded.contains_key(&child));
    // A time jump must also represent the trainee's normal aging-out transition.
    campaign.people.get_mut(&trainee).unwrap().assignment = PersonAssignment::Dependent { site };
    campaign.completed_rounds = entered + 38;
    finish_round(&mut campaign, &data);
    assert!(!campaign.history.person_last_reminded.contains_key(&child));
    for _ in 0..3 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.history.person_last_reminded.get(&child), Some(&10));
    let reminders = campaign.history.events.values().filter(|record| matches!(record.kind,
        HistoryKind::Anniversary { subject: AnniversarySubject::Person(person), years: 10 } if person == child)).count();
    assert_eq!(reminders, 1);
    assert!(!campaign.history.person_last_reminded.contains_key(&trainee));
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap(),
    )
    .unwrap();
    loaded.validate(&data).unwrap();
    campaign = loaded.strategic().unwrap().clone();
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.history.events.values().filter(|record| matches!(record.kind,
        HistoryKind::Anniversary { subject: AnniversarySubject::Person(person), years: 10 } if person == child)).count(), 1);
}

#[test]
fn each_faction_gets_one_stable_anniversary_per_completed_round() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 40;
    for person in campaign.people.values_mut() {
        person.service_start_round = 0;
    }
    for relation in &mut campaign.relations {
        relation.state = DiplomaticState::Peace;
    }
    for pair in &mut campaign.diplomacy.pairs {
        pair.peace_since = Some(40);
        pair.war_started_round = None;
        pair.war_ended_round = None;
        pair.truce_until = None;
        pair.last_offer_round = None;
    }
    campaign.diplomacy.era_history_complete = true;
    campaign.validate(&data).unwrap();
    assert_eq!(project(&campaign, PLAYER).unwrap().era_label, "Founding");

    finish_round(&mut campaign, &data);
    let reminders = campaign
        .history
        .events
        .values()
        .filter(|record| matches!(record.kind, HistoryKind::Anniversary { .. }))
        .collect::<Vec<_>>();
    assert_eq!(reminders.len(), campaign.factions.len());
    let mut owners = std::collections::BTreeSet::new();
    for record in &reminders {
        assert_eq!(record.visible_to.len(), 1);
        let owner = *record.visible_to.iter().next().unwrap();
        assert!(owners.insert(owner));
        assert!(matches!(
            record.kind,
            HistoryKind::Anniversary { years: 10, .. }
        ));
    }
    assert!(reminders.iter().any(|record| matches!(
        record.kind,
        HistoryKind::Anniversary {
            subject: AnniversarySubject::Person(_),
            ..
        }
    )));
    let before = campaign.completed_rounds;
    finish_round(&mut campaign, &data);
    let next_round = campaign
        .history
        .events
        .values()
        .filter(|record| {
            record.completed_rounds > before
                && matches!(record.kind, HistoryKind::Anniversary { .. })
        })
        .collect::<Vec<_>>();
    assert_eq!(next_round.len(), campaign.factions.len());
    let mut seen = std::collections::BTreeSet::new();
    for record in campaign
        .history
        .events
        .values()
        .filter(|record| matches!(record.kind, HistoryKind::Anniversary { .. }))
    {
        let faction = *record.visible_to.iter().next().unwrap();
        let HistoryKind::Anniversary { subject, years } = record.kind else {
            unreachable!()
        };
        assert!(seen.insert((faction, subject, years)));
    }
    campaign.validate(&data).unwrap();
}

#[test]
fn detailed_deeds_expire_without_erasing_the_item_or_its_current_custody() {
    let (data, mut campaign) = fixture();
    let site = person_site(&campaign, FOUNDER).unwrap();
    let item = item_held_by(&campaign, FOUNDER);
    let heir = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 24);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferLegacyItem { item, to: heir },
    )
    .unwrap();
    let deed = campaign
        .history
        .events
        .values()
        .find(|record| {
            matches!(
                record.kind,
                HistoryKind::ItemCustodyChanged { item: id, .. } if id == item
            )
        })
        .unwrap()
        .id;
    campaign.completed_rounds = 41;
    finish_round(&mut campaign, &data);
    assert!(!campaign.history.events.contains_key(&deed));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    assert!(history_page(
        &campaign,
        PLAYER,
        &HistoryFilter {
            subject: Some(HistorySubject::Item(item)),
            ..Default::default()
        }
    )
    .entries
    .is_empty());
    campaign.validate(&data).unwrap();
}

#[test]
fn detail_count_limit_prunes_deeds_without_pruning_current_custody() {
    let (mut data, mut campaign) = fixture();
    data.history.detail_max_entries = 2;
    let site = person_site(&campaign, FOUNDER).unwrap();
    let item = item_held_by(&campaign, FOUNDER);
    let first = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 24);
    let second = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 24);
    let third = add_person(&mut campaign, PLAYER, PersonAssignment::Site { site }, 24);
    for to in [first, second, third] {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::TransferLegacyItem { item, to },
        )
        .unwrap();
    }
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.history.events.len(), 2);
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(third)
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn departed_pruning_cleans_designations_and_mentor_links_but_keeps_heir_progress() {
    let (data, mut campaign) = fixture();
    let (item, heir) = prepare_founder_death(&data, &mut campaign);
    let seasons = data.lifecycle.apprenticeship_seasons;
    finish_round(&mut campaign, &data);
    let death_round = match campaign.people[&FOUNDER].status {
        PersonStatus::Dead {
            completed_rounds, ..
        } => completed_rounds,
        _ => panic!("founder died at the round boundary"),
    };
    assert!(campaign.successors.contains_key(&FOUNDER));
    campaign.completed_rounds = death_round + data.history.departed_max_age_rounds + 1;
    finish_round(&mut campaign, &data);

    assert!(!campaign.people.contains_key(&FOUNDER));
    assert!(!campaign.successors.contains_key(&FOUNDER));
    assert!(campaign
        .successors
        .values()
        .all(|entries| entries.values().all(|entry| entry.successor != FOUNDER)));
    assert_eq!(
        campaign.people[&heir].career.mentorship_seasons[&TrainingDiscipline::Command],
        seasons
    );
    assert!(campaign.people[&heir]
        .career
        .completed_mentors
        .iter()
        .all(|record| record.mentor != FOUNDER));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn k15_saves_gain_current_round_founders_without_fabricated_dates_or_deeds() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 8;
    let factions = campaign
        .factions
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for (id, round, related_events) in [
        (HistoryId(1), 1, Vec::new()),
        (HistoryId(2), 5, vec![HistoryId(1)]),
    ] {
        campaign.history.events.insert(
            id,
            HistoryRecord {
                id,
                completed_rounds: round,
                source_fact: Some(FactId(id.0)),
                kind: HistoryKind::Diplomacy {
                    receipt: DiplomacyReceipt::WarDeclared {
                        factions: [PLAYER, OTHER],
                    },
                },
                sites: Vec::new(),
                armies: Vec::new(),
                people: Vec::new(),
                formations: Vec::new(),
                items: Vec::new(),
                related_events,
                visible_to: factions.clone(),
            },
        );
    }
    campaign.next_ids.history = HistoryId(3);
    campaign.next_ids.fact = FactId(3);
    campaign.validate(&data).unwrap();
    let expected = campaign
        .people
        .values()
        .filter(|person| {
            person.class == PersonClass::Officer
                && person.is_alive()
                && person.career.emergence.is_none()
        })
        .map(|person| (person.id, person.faction, person.name.clone()))
        .collect::<Vec<_>>();
    let expected_count = expected.len();
    let mut old = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    let payload = old.as_object_mut().unwrap();
    payload.remove("legacy_items");
    payload
        .get_mut("next_ids")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("legacy_item");
    payload
        .get_mut("world")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("founded_rounds");
    let diplomacy = payload
        .get_mut("diplomacy")
        .unwrap()
        .as_object_mut()
        .unwrap();
    diplomacy.remove("era_history_complete");
    for pair in diplomacy["pairs"].as_array_mut().unwrap() {
        pair.as_object_mut().unwrap().remove("war_started_round");
        pair.as_object_mut().unwrap().remove("war_ended_round");
    }
    for record in payload["history"]["events"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        record.as_object_mut().unwrap().remove("related_events");
    }
    let encoded = encode_slot("strategic_v2", &old, "2").unwrap();
    let migrated = kestrum::state::persistence::load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(migrated.legacy_items.len(), expected_count);
    for (person, faction, name) in expected {
        let item = migrated
            .legacy_items
            .values()
            .find(|item| item.custody == LegacyItemCustody::Person(person))
            .unwrap();
        assert_eq!(item.faction, faction);
        assert_eq!(item.name, format!("{name}'s Muster Sword"));
        assert_eq!(item.created_round, 8);
    }
    assert_eq!(migrated.history.events.len(), 2);
    assert!(migrated.history.events[&HistoryId(1)]
        .related_events
        .is_empty());
    assert_eq!(
        migrated.history.events[&HistoryId(2)].related_events,
        vec![HistoryId(1)]
    );
    assert!(migrated.world.founded_rounds.is_empty());
    assert!(!migrated.diplomacy.era_history_complete);
    assert_eq!(
        migrated.next_ids.legacy_item,
        LegacyItemId(expected_count as u32 + 1)
    );
    migrated.validate(&data).unwrap();
}

#[test]
fn an_old_active_war_reports_its_unknown_start_honestly() {
    let (data, mut campaign) = fixture();
    let relation = campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [PLAYER, OTHER])
        .unwrap();
    relation.state = DiplomaticState::War;
    campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == [PLAYER, OTHER])
        .unwrap()
        .war_started_round = None;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == [PLAYER, OTHER])
        .unwrap();
    pair.peace_since = None;
    pair.war_ended_round = None;
    pair.truce_until = None;
    pair.last_offer_round = None;
    campaign.diplomacy.era_history_complete = false;
    assert!(project(&campaign, PLAYER)
        .unwrap()
        .era_label
        .contains("start not recorded"));
    campaign.validate(&data).unwrap();
}

#[test]
fn a_pre_diplomacy_save_does_not_fabricate_the_start_of_an_active_war() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 8;
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [PLAYER, OTHER])
        .unwrap()
        .state = DiplomaticState::War;
    let mut old = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    let payload = old.as_object_mut().unwrap();
    payload.remove("diplomacy");
    payload.remove("ai");
    let encoded = encode_slot("strategic_v2", &old, "2").unwrap();
    let migrated = kestrum::state::persistence::load_legacy(&encoded, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert!(!migrated.diplomacy.era_history_complete);
    assert!(project(&migrated, PLAYER)
        .unwrap()
        .era_label
        .contains("start not recorded"));
    migrated.validate(&data).unwrap();
}
