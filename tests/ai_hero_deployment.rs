//! NPCs redeploy recognized Heroes when a colocated army lacks one.

use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        progression::EpithetFact,
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{ai, apply, Actor, Command},
    state::{
        evidence::EvidenceKind,
        military::ArmyId,
        people::{Person, PersonAssignment, PersonId, Recognition},
        StrategicCampaign,
    },
};

const NPC: FactionId = FactionId(2);

#[test]
fn npc_spreads_recognized_heroes_across_armies_with_vacant_slots() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    campaign.factions.get_mut(&NPC).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    let first_army = ArmyId(2);
    let site = campaign.armies[&first_army].site;
    let split_formation = campaign.armies[&first_army]
        .formation_ids()
        .find(|formation| campaign.formation_person(*formation).is_none())
        .unwrap();
    let second_army = apply(
        &mut campaign,
        &data,
        Actor::Npc(NPC),
        Command::SplitArmy {
            formation: split_formation,
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    let first_hero =
        add_recognized_hero(&mut campaign, &data, site, PersonAssignment::Site { site });
    let second_hero =
        add_recognized_hero(&mut campaign, &data, site, PersonAssignment::Site { site });
    campaign.validate(&data).unwrap();

    for (person, expected_army) in [(first_hero, first_army), (second_hero, second_army)] {
        let decision = ai::propose(&campaign, &data, NPC).unwrap();
        assert!(
            matches!(decision.command, Command::TransferPerson { person: id, .. } if id == person),
            "the AI should deploy the recognized Hero into the next uncovered army: {decision:?}"
        );
        apply(&mut campaign, &data, Actor::Npc(NPC), decision.command).unwrap();
        let formation = match &campaign.people[&person].assignment {
            PersonAssignment::Formation { formation } => *formation,
            _ => panic!("the Hero should enter a vacant formation"),
        };
        let army = campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .unwrap();
        assert_eq!(army.id, expected_army);
    }
    campaign.validate(&data).unwrap();
}

#[test]
fn npc_moves_a_surplus_hero_to_a_colocated_army_without_one() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    campaign.factions.get_mut(&NPC).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    let source = ArmyId(2);
    let site = campaign.armies[&source].site;
    let empty_formations = campaign.armies[&source]
        .formation_ids()
        .filter(|formation| campaign.formation_person(*formation).is_none())
        .collect::<Vec<_>>();
    let target_army = apply(
        &mut campaign,
        &data,
        Actor::Npc(NPC),
        Command::SplitArmy {
            formation: empty_formations[0],
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    let source_hero = campaign.armies[&source].commander.unwrap();
    mark_recognized_hero(campaign.people.get_mut(&source_hero).unwrap(), &data, site);
    let extra_hero = add_recognized_hero(
        &mut campaign,
        &data,
        site,
        PersonAssignment::Formation {
            formation: empty_formations[1],
        },
    );
    campaign.validate(&data).unwrap();

    let decision = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(
        matches!(decision.command, Command::TransferPerson { person, .. } if person == extra_hero),
        "the extra Hero should move to the colocated army without one: {decision:?}"
    );
    apply(&mut campaign, &data, Actor::Npc(NPC), decision.command).unwrap();
    let formation = match &campaign.people[&extra_hero].assignment {
        PersonAssignment::Formation { formation } => *formation,
        _ => panic!("the transferred Hero should remain in field service"),
    };
    assert!(campaign.armies[&target_army]
        .formation_ids()
        .any(|member| member == formation));
    assert_eq!(campaign.armies[&source].commander, Some(source_hero));
    campaign.validate(&data).unwrap();
}

fn add_recognized_hero(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    assignment: PersonAssignment,
) -> PersonId {
    let template = campaign
        .people
        .values()
        .find(|person| person.faction == NPC)
        .unwrap()
        .clone();
    let id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(id.0.checked_add(1).unwrap());
    let appearance =
        kestrum::engine::portraits::allocate_for_person(campaign, &data.portraits, id).unwrap();
    let mut hero = template;
    hero.id = id;
    hero.name = format!("Recognized Hero {}", id.0);
    hero.appearance = appearance;
    hero.class = PersonClass::Recruit;
    hero.assignment = assignment;
    hero.birth_round = -80;
    hero.service_start_round = campaign.completed_rounds;
    hero.movement_spent = 0;
    hero.career = Default::default();
    mark_recognized_hero(&mut hero, data, site);
    campaign.people.insert(id, hero);
    id
}

fn mark_recognized_hero(person: &mut Person, data: &GameData, site: SiteId) {
    let cause = EpithetFact::BattleService;
    let kind = TroopKind::Warriors;
    person.career.hero_service_progress = data.progression.recognition.personal_engagements;
    person.career.hero_service_sites.insert(cause, site);
    person.career.notable_sites.insert(cause, site);
    person.career.recognition = Some(Recognition {
        completed_rounds: 0,
        epithet: data.human_names.epithets[&cause].clone(),
        cause,
        site,
    });
    person.evidence.counts.insert(EvidenceKind::Battle, 1);
    person
        .evidence
        .counts
        .insert(EvidenceKind::MeaningfulEncounter, 1);
    person.evidence.encountered_troops.insert(kind);
    person.evidence.service_by_troop.insert(kind, 1);
}
