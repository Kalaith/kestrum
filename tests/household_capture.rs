//! Household seats remain valid across capture and seasonal settlement changes.

use kestrum::{
    data::{
        economy::Habitation,
        world::{DiplomaticState, FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{self, Actor, Command, MoveOrder},
    state::{
        history::{HistoryKind, LifeEvent},
        military::ArmyId,
        people::{PersonAssignment, PersonId, PersonRelationship, PersonStatus},
        relationships::{HouseholdEndReason, HouseholdId, HouseholdStatus},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence;

const PLAYER: FactionId = FactionId(1);
const RIVAL: FactionId = FactionId(2);
const HOME: SiteId = SiteId(1);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn peaceful_fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    for relation in &mut data.scenario.relations {
        if relation.factions == [PLAYER, RIVAL] || relation.factions == [RIVAL, PLAYER] {
            relation.state = DiplomaticState::Peace;
        }
    }
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

fn add_partner(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    site: SiteId,
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
    person.name = format!("Household fixture {id:?}");
    person.birth_round = i64::from(campaign.completed_rounds) - i64::from(age) * 4;
    person.service_start_round = 0;
    person.class = PersonClass::Recruit;
    person.assignment = PersonAssignment::Site { site };
    person.movement_spent = 0;
    person.status = PersonStatus::Fit;
    person.career = Default::default();
    person.evidence = Default::default();
    person.appearance =
        engine::portraits::allocate_for_person(campaign, &data.portraits, id).unwrap();
    campaign.people.insert(id, person);
    id
}

fn form_household(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    faction: FactionId,
    site: SiteId,
) -> HouseholdId {
    let household = campaign.next_ids.household;
    let first = add_partner(campaign, data, faction, site, 25);
    let second = add_partner(campaign, data, faction, site, 27);
    let relationship = PersonRelationship {
        shared_service_seasons: 4,
        last_shared_service_round: Some(campaign.completed_rounds),
        mutual_combat_rounds: 0,
        last_mutual_combat_round: None,
    };
    campaign
        .people
        .get_mut(&first)
        .unwrap()
        .career
        .relationships
        .insert(second, relationship.clone());
    campaign
        .people
        .get_mut(&second)
        .unwrap()
        .career
        .relationships
        .insert(first, relationship);
    engine::apply(
        campaign,
        data,
        actor,
        Command::FormHousehold {
            first,
            second,
            site,
        },
    )
    .unwrap();
    household
}

fn complete_round(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<engine::ActionOutcome, engine::RuleError> {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        let outcome = engine::apply(campaign, data, actor, Command::EndTurn)?;
        if campaign.completed_rounds != round {
            return Ok(outcome);
        }
    }
    unreachable!("the completed round returns its final action outcome")
}

fn advance_to_player(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), engine::RuleError> {
    while !matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        engine::apply(campaign, data, actor, Command::EndTurn)?;
    }
    Ok(())
}

#[test]
fn a_household_ends_when_its_friendly_camp_becomes_unsettled() {
    let (data, mut campaign) = fixture();
    let home = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == HOME)
        .unwrap();
    home.habitation = Habitation::Camp;
    campaign.world.population.insert(
        HOME,
        data.construction.population.minimum[&Habitation::Camp],
    );
    campaign.world.development.get_mut(&HOME).unwrap().pressure = data.development.pressure.minimum;
    let abandoned = form_household(&mut campaign, &data, Actor::Player, PLAYER, HOME);
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetHouseholdChildraising {
            household: abandoned,
            enabled: true,
        },
    )
    .unwrap();
    engine::apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let unaffected = form_household(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        FactionId(2),
        SiteId(2),
    );
    campaign.validate(&data).unwrap();

    let outcome = complete_round(&mut campaign, &data).unwrap();

    assert_eq!(campaign.world.site(HOME).unwrap().controller, Some(PLAYER));
    assert_eq!(
        campaign.world.site(HOME).unwrap().habitation,
        Habitation::Unsettled
    );
    assert!(matches!(
        campaign.households[&abandoned].status,
        HouseholdStatus::Ended {
            completed_rounds,
            reason: HouseholdEndReason::SiteAbandoned,
        } if completed_rounds == campaign.completed_rounds
    ));
    assert!(!campaign.households[&abandoned].raising_children);
    assert_eq!(
        campaign.households[&unaffected].status,
        HouseholdStatus::Active
    );
    assert!(outcome.life_events.iter().any(|id| matches!(
        campaign.history.events[id].kind,
        HistoryKind::Life {
            event: LifeEvent::HouseholdEnded {
                reason: HouseholdEndReason::SiteAbandoned,
                ..
            },
            ..
        }
    )));
    assert_eq!(
        data.game_text.life_event_text(&LifeEvent::HouseholdEnded {
            household: abandoned,
            reason: HouseholdEndReason::SiteAbandoned,
        }),
        "Household ended: the home became uninhabited"
    );
    campaign.validate(&data).unwrap();

    let encoded = persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    let restored = kestrum::state::persistence::load_legacy(&encoded, &data).unwrap();
    assert_eq!(restored.strategic(), Some(&campaign));
}

#[test]
fn ordinary_hostile_capture_ends_the_household_as_captured() {
    let (data, mut campaign) = peaceful_fixture();
    let rival_home = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap();
    rival_home.habitation = Habitation::Camp;
    campaign.world.population.insert(
        SiteId(5),
        data.construction.population.minimum[&Habitation::Camp],
    );
    campaign.world.founded_rounds.insert(SiteId(5), 0);
    campaign
        .set_site_control(&data, SiteId(5), Some(RIVAL), false)
        .unwrap();
    engine::apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let household = form_household(&mut campaign, &data, Actor::Npc(RIVAL), RIVAL, SiteId(5));
    advance_to_player(&mut campaign, &data).unwrap();
    engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar { faction: RIVAL },
    )
    .unwrap();
    campaign.validate(&data).unwrap();

    let outcome = engine::apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();

    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().controller,
        Some(PLAYER)
    );
    assert_eq!(
        campaign.households[&household].status,
        HouseholdStatus::Ended {
            completed_rounds: campaign.completed_rounds,
            reason: HouseholdEndReason::SiteCaptured,
        }
    );
    assert!(outcome.life_events.iter().any(|id| matches!(
        campaign.history.events[id].kind,
        HistoryKind::Life {
            event: LifeEvent::HouseholdEnded {
                reason: HouseholdEndReason::SiteCaptured,
                ..
            },
            ..
        }
    )));
    campaign.validate(&data).unwrap();
}
