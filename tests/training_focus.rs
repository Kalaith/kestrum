//! Course previews, charges and historical receipts agree at local training sites.

use kestrum::{
    data::{
        economy::TroopKind,
        world::{PersonClass, SiteId},
        GameData,
    },
    engine::{apply, career_options, Actor, Command},
    state::{
        construction::Focus,
        evidence::EvidenceKind,
        people::{PersonCourse, PersonId},
        Campaign, StrategicCampaign,
    },
};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let person = campaign.people.get_mut(&PersonId(1)).unwrap();
    person.class = PersonClass::Recruit;
    person.evidence.counts.extend([
        (EvidenceKind::Battle, 2),
        (EvidenceKind::MeaningfulEncounter, 2),
    ]);
    person
        .evidence
        .service_by_troop
        .insert(TroopKind::Warriors, 2);
    (data, campaign)
}

fn train(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(
        campaign,
        data,
        Actor::Player,
        Command::TrainPerson {
            person: PersonId(1),
            class: PersonClass::Infantry,
            site: SiteId(1),
        },
    )
    .unwrap();
}

fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

fn focus(campaign: &mut StrategicCampaign, data: &GameData, focus: Focus) {
    apply(
        campaign,
        data,
        Actor::Player,
        Command::SetFocus {
            site: SiteId(1),
            focus,
        },
    )
    .unwrap();
}

#[test]
fn displayed_local_price_uses_ceil_and_is_the_actual_affordability_threshold() {
    let data = GameData::load().unwrap();
    assert_eq!(
        kestrum::engine::course_gold_cost(
            21,
            &data.progression.careers,
            Some(Focus::TroopTraining)
        ),
        16
    );
    for (selected, expected) in [(Focus::TroopTraining, 15), (Focus::Gold, 20)] {
        let (data, mut campaign) = fixture();
        focus(&mut campaign, &data, selected);
        let option = career_options(&campaign, &data, PersonId(1))
            .unwrap()
            .into_iter()
            .find(|option| option.class == PersonClass::Infantry)
            .unwrap();
        assert_eq!(option.course.gold_cost, expected);
        campaign
            .factions
            .get_mut(&campaign.player)
            .unwrap()
            .resources
            .gold = expected - 1;
        let before = campaign.clone();
        assert!(apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::TrainPerson {
                person: PersonId(1),
                class: PersonClass::Infantry,
                site: SiteId(1),
            }
        )
        .is_err());
        assert_eq!(campaign, before);
        campaign
            .factions
            .get_mut(&campaign.player)
            .unwrap()
            .resources
            .gold = expected;
        train(&mut campaign, &data);
        assert_eq!(campaign.factions[&campaign.player].resources.gold, 0);
        assert!(matches!(campaign.people[&PersonId(1)].career.course,
            Some(PersonCourse::Class { paid_gold, .. }) if paid_gold == expected));
    }
}

#[test]
fn focus_changes_and_reload_cannot_change_a_refund_or_double_course_progress() {
    let (data, mut campaign) = fixture();
    focus(&mut campaign, &data, Focus::TroopTraining);
    train(&mut campaign, &data);
    focus(&mut campaign, &data, Focus::Gold);
    let restored: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign))).unwrap(),
    )
    .unwrap();
    let mut campaign = restored.strategic().unwrap().clone();
    let gold = campaign.factions[&campaign.player].resources.gold;
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
        campaign.factions[&campaign.player].resources.gold,
        gold + 15
    );
    focus(&mut campaign, &data, Focus::TroopTraining);
    train(&mut campaign, &data);
    finish(&mut campaign, &data);
    assert!(matches!(
        campaign.people[&PersonId(1)].career.course,
        Some(PersonCourse::Class {
            steps_completed: 1,
            ..
        })
    ));
    let gold = campaign.factions[&campaign.player].resources.gold;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelPersonCourse {
            person: PersonId(1),
        },
    )
    .unwrap();
    assert_eq!(campaign.factions[&campaign.player].resources.gold, gold);
}

#[test]
fn legacy_courses_recover_the_original_full_price_without_inventing_a_discount() {
    let (data, mut campaign) = fixture();
    train(&mut campaign, &data);
    focus(&mut campaign, &data, Focus::TroopTraining);
    let mut encoded = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    encoded["people"]["1"]["career"]["course"]
        .as_object_mut()
        .unwrap()
        .remove("paid_gold");
    let restored: Campaign = serde_json::from_value(encoded).unwrap();
    let mut campaign = restored.strategic().unwrap().clone();
    campaign.validate(&data).unwrap();
    let gold = campaign.factions[&campaign.player].resources.gold;
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
        campaign.factions[&campaign.player].resources.gold,
        gold + 20
    );
}

#[test]
fn retirement_returns_an_unstarted_paid_receipt_only_once() {
    let (data, mut campaign) = fixture();
    focus(&mut campaign, &data, Focus::TroopTraining);
    train(&mut campaign, &data);
    let gold = campaign.factions[&campaign.player].resources.gold;
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
    assert_eq!(
        campaign.factions[&campaign.player].resources.gold,
        gold + 15
    );
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelPersonCourse {
            person: PersonId(1)
        }
    )
    .is_err());
    assert_eq!(campaign, before);
}

#[test]
fn formation_specialization_uses_the_same_receipt_and_rounding() {
    use kestrum::{data::progression::FormationSpecialization, state::military::FormationId};
    let (data, mut campaign) = fixture();
    let formation = FormationId(1);
    let ledger = &mut campaign
        .formations
        .get_mut(&formation)
        .unwrap()
        .service
        .ledger;
    ledger.counts.extend([
        (EvidenceKind::Battle, 3),
        (EvidenceKind::MeaningfulEncounter, 3),
        (EvidenceKind::DefendedAnchor, 3),
    ]);
    ledger.service_by_troop.insert(TroopKind::Warriors, 3);
    focus(&mut campaign, &data, Focus::TroopTraining);
    let gold = campaign.factions[&campaign.player].resources.gold;
    let option = kestrum::engine::specialization_options(&campaign, &data, formation)
        .unwrap()
        .into_iter()
        .find(|option| option.specialization == FormationSpecialization::ShieldGuard)
        .unwrap();
    assert_eq!(option.gold_cost, 23);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SpecializeFormation {
            formation,
            specialization: FormationSpecialization::ShieldGuard,
            site: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.factions[&campaign.player].resources.gold,
        gold - 23
    );
    focus(&mut campaign, &data, Focus::Gold);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelFormationCourse { formation },
    )
    .unwrap();
    assert_eq!(campaign.factions[&campaign.player].resources.gold, gold);
}

#[test]
fn death_and_loss_of_the_course_site_refund_the_saved_payment() {
    use macroquad_toolkit::rng::SeededRng;
    for death in [false, true] {
        let (data, mut campaign) = fixture();
        focus(&mut campaign, &data, Focus::TroopTraining);
        train(&mut campaign, &data);
        if death {
            campaign.people.get_mut(&PersonId(1)).unwrap().birth_round = -239;
            let seed = (1..100_000)
                .find(|seed| SeededRng::new(*seed).below(1000) < 20)
                .unwrap();
            campaign.rng.people = SeededRng::new(seed);
        } else {
            campaign
                .set_site_control(
                    &data,
                    SiteId(1),
                    Some(kestrum::data::world::FactionId(2)),
                    false,
                )
                .unwrap();
        }
        let mut without_receipt = campaign.clone();
        without_receipt
            .people
            .get_mut(&PersonId(1))
            .unwrap()
            .career
            .course = None;
        finish(&mut campaign, &data);
        finish(&mut without_receipt, &data);
        assert!(campaign.people[&PersonId(1)].career.course.is_none());
        assert_eq!(campaign.people[&PersonId(1)].is_alive(), !death);
        assert_eq!(
            campaign.factions[&campaign.player].resources.gold,
            without_receipt.factions[&campaign.player].resources.gold + 15
        );
    }
}
