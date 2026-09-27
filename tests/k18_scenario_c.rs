//! A controlled K18 lineage story followed through a long save/retention horizon.
//! This covers the non-kin apprentice, mentor, successor, transfer, death and 200-round
//! persistence slice; geography, refugees, capital moves, dependents and army
//! appointment remain covered at feature level rather than in this storyline.

use kestrum::{
    data::{
        progression::TrainingDiscipline,
        world::{DiplomaticState, PersonClass},
        GameData,
    },
    engine::{apply, history_page, person_site, Actor, Command, HistoryFilter},
    state::{
        history::{HistoryKind, HistorySubject},
        legacy::{LegacyItemCustody, LegacyItemId},
        people::{PersonId, PersonStatus},
        persistence::load_legacy,
        relationships::{FamilyOrigin, LegacyCategory, SuccessorLink},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::{persistence::encode_slot, rng::SeededRng};

const FOUNDER: PersonId = PersonId(1);
const ROUND_CAP: u32 = 200;
const FOUNDER_DEATH_ROUND: u32 = 13;

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

fn item_held_by(campaign: &StrategicCampaign, person: PersonId) -> LegacyItemId {
    campaign
        .legacy_items
        .values()
        .find(|item| item.custody == LegacyItemCustody::Person(person))
        .expect("founder's starting Muster Sword")
        .id
}

fn mortality_seed(
    campaign: &StrategicCampaign,
    data: &GameData,
    founder: PersonId,
    heir: PersonId,
) -> u64 {
    // The controlled route has no households or battle-earned emergence, so
    // birthdays are the only consumer of this RNG stream after the checkpoint.
    let birthday_rolls = (campaign.completed_rounds + 1..=ROUND_CAP)
        .flat_map(|round| {
            campaign.people.values().filter_map(move |person| {
                (person.is_alive()
                    && person.age_years(round) > person.age_years(round - 1)
                    && data
                        .lifecycle
                        .death_chance_permille(person.age_years(round))
                        > 0)
                .then_some((
                    round,
                    person.id,
                    data.lifecycle
                        .death_chance_permille(person.age_years(round)),
                    person_site(campaign, person.id).is_some(),
                ))
            })
        })
        .collect::<Vec<_>>();

    (1..100_000)
        .find(|seed| {
            let mut rng = SeededRng::new(*seed);
            let mut living = campaign
                .people
                .values()
                .filter(|person| person.is_alive())
                .map(|person| person.id)
                .collect::<std::collections::BTreeSet<_>>();
            let mut founder_died_at = None;
            for (round, person, chance, has_site) in &birthday_rolls {
                if !living.contains(person) {
                    continue;
                }
                if rng.below(1000) < *chance as usize && *has_site {
                    living.remove(person);
                    if *person == founder {
                        founder_died_at = Some(*round);
                    }
                }
            }
            founder_died_at == Some(FOUNDER_DEATH_ROUND) && living.contains(&heir)
        })
        .expect("a deterministic mortality stream for founder death and heir survival")
}

#[test]
fn unrelated_apprentice_inherits_an_item_and_survives_the_200_round_checkpoint() {
    let mut data = GameData::load().unwrap();
    // The authored Rosemarch campaign starts with Rose and Hawthorn at war;
    // this controlled continuity slice explicitly starts from peaceful relations.
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    assert!(campaign.households.is_empty());
    let faction = campaign.player;
    let founder_site = person_site(&campaign, FOUNDER).expect("founder begins in the field");
    let item = item_held_by(&campaign, FOUNDER);

    // Author only the minimum conditions for a real mentorship and scheduled
    // birthday; service, apprenticeship, death and custody still resolve in-game.
    let founder = campaign.people.get_mut(&FOUNDER).unwrap();
    founder.class = PersonClass::Infantry;
    founder.birth_round = -227;
    founder.service_start_round = 0;
    for _ in 0..4 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.completed_rounds, 4);
    assert_eq!(
        campaign.people[&FOUNDER].career.discipline_service_seasons[&TrainingDiscipline::Infantry],
        4
    );

    let heir = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::InviteApprentice { site: founder_site },
    )
    .unwrap()
    .new_people[0];
    assert_eq!(
        campaign.people[&heir].age_years(campaign.completed_rounds),
        17
    );
    assert_eq!(
        campaign.families[&heir].origin,
        FamilyOrigin::LocalApprentice
    );
    assert!(campaign.families[&heir].links.is_empty());
    assert!(!campaign.known_blood_relation(FOUNDER, heir));
    assert!(!campaign.shares_household(FOUNDER, heir));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor: FOUNDER,
            learner: heir,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    for _ in 0..4 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.completed_rounds, 8);
    assert!(campaign.is_pupil(FOUNDER, heir));
    assert!(campaign.people[&heir]
        .career
        .completed_mentors
        .iter()
        .any(|record| record.mentor == FOUNDER
            && record.discipline == TrainingDiscipline::Infantry
            && record.completed_round == 8));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DesignateSuccessor {
            predecessor: FOUNDER,
            successor: heir,
            category: LegacyCategory::Item,
            link: SuccessorLink::Martial,
        },
    )
    .unwrap();
    assert!(campaign.successors[&FOUNDER][&LegacyCategory::Item].link_witnessed);
    let transfer = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferLegacyItem { item, to: heir },
    )
    .unwrap();
    assert_eq!(transfer.legacy_items_changed, vec![item]);
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    let transfer_deed = campaign
        .history
        .events
        .values()
        .find(|record| matches!(record.kind, HistoryKind::ItemCustodyChanged { item: id, .. } if id == item))
        .expect("real transfer records a custody deed")
        .id;
    assert_eq!(
        history_page(
            &campaign,
            faction,
            &HistoryFilter {
                subject: Some(HistorySubject::Item(item)),
                ..Default::default()
            },
        )
        .entries
        .len(),
        1
    );

    let seed = mortality_seed(&campaign, &data, FOUNDER, heir);
    campaign.rng.people = SeededRng::new(seed);
    while campaign.completed_rounds < FOUNDER_DEATH_ROUND {
        finish_round(&mut campaign, &data);
    }
    assert!(matches!(
        campaign.people[&FOUNDER].status,
        PersonStatus::Dead {
            completed_rounds: FOUNDER_DEATH_ROUND,
            ..
        }
    ));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    assert!(campaign.successors[&FOUNDER].contains_key(&LegacyCategory::Item));
    assert!(campaign.people[&heir].is_alive());

    while campaign.completed_rounds < ROUND_CAP {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.completed_rounds, ROUND_CAP);
    assert_eq!(campaign.factions.len(), 4);
    assert!(campaign.people[&heir].is_alive());
    assert_eq!(campaign.people[&heir].age_years(ROUND_CAP), 66);
    assert_eq!(
        campaign.people[&heir]
            .career
            .mentorship_seasons
            .get(&TrainingDiscipline::Infantry),
        Some(&data.lifecycle.apprenticeship_seasons),
        "earned discipline training remains after the mentor's bounded record expires"
    );
    assert!(!campaign.people[&heir]
        .career
        .completed_mentors
        .iter()
        .any(|record| record.mentor == FOUNDER));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir),
        "current item custody outlives its short-lived deed and predecessor record"
    );
    assert!(ROUND_CAP - 8 > data.history.detail_max_age_rounds);
    assert!(!campaign.history.events.contains_key(&transfer_deed));
    assert!(!campaign.people.contains_key(&FOUNDER));
    assert!(!campaign.successors.contains_key(&FOUNDER));
    campaign.validate(&data).unwrap();

    let envelope = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    let restored = load_legacy(&envelope, &data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    assert_eq!(restored, campaign);
    assert_eq!(restored.completed_rounds, ROUND_CAP);
    assert_eq!(
        restored.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    assert!(restored.people[&heir].is_alive());
    assert_eq!(
        restored.people[&heir]
            .career
            .mentorship_seasons
            .get(&TrainingDiscipline::Infantry),
        Some(&data.lifecycle.apprenticeship_seasons)
    );

    println!(
        "K18_SCENARIO_C rounds={} seed={} mentor={} heir={} transfer_deed_pruned={} founder_record_pruned={} retained_people={} retained_history={}",
        campaign.completed_rounds,
        seed,
        FOUNDER.0,
        heir.0,
        !campaign.history.events.contains_key(&transfer_deed),
        !campaign.people.contains_key(&FOUNDER),
        campaign.people.len(),
        campaign.history.events.len(),
    );
}
