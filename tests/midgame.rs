//! The native review save is a progressed, resumable production campaign.

#[path = "../examples/prepare_midgame/campaign.rs"]
mod review_campaign;

use kestrum::{
    data::{economy::Habitation, world::DiplomaticState, GameData},
    engine::{apply, Actor, Command},
    state::{evidence::EvidenceKind, Campaign, CampaignPhase, StrategicCampaign},
};
use std::sync::OnceLock;

fn fixture() -> (GameData, StrategicCampaign) {
    static CAMPAIGN: OnceLock<StrategicCampaign> = OnceLock::new();
    let data = GameData::load().unwrap();
    let campaign = CAMPAIGN
        .get_or_init(|| review_campaign::generate(&data).unwrap())
        .clone();
    (data, campaign)
}

fn player_evidence_total(campaign: &StrategicCampaign, kind: EvidenceKind) -> u32 {
    campaign
        .people
        .values()
        .filter(|person| person.faction == campaign.player)
        .map(|person| {
            person
                .evidence
                .counts
                .get(&kind)
                .copied()
                .unwrap_or_default()
        })
        .sum()
}

#[test]
fn midgame_reveals_a_broad_connected_atlas_and_known_rivals() {
    let (_, campaign) = fixture();
    let visible = kestrum::engine::project_map(&campaign, campaign.player).unwrap();
    assert!(
        visible.world.markers.len() >= 30,
        "only {} markers",
        visible.world.markers.len()
    );
    assert!(
        visible.world.sites.len() >= 55,
        "only {} sites",
        visible.world.sites.len()
    );
    assert!(
        visible.factions.len() >= 3,
        "only {} known factions",
        visible.factions.len()
    );
    let mut navigation = kestrum::navigation::MapNavigation::default();
    let mut view = kestrum::navigation::MapView::default();
    navigation.frame_discovered(&visible.world, &mut view);
    assert_eq!(navigation.scope(), kestrum::navigation::MapScope::World);
    assert!(view.camera.zoom() < 1.5);
    assert_eq!(
        navigation.targets(&visible.world, &view).len(),
        visible.world.markers.len()
    );
}

#[test]
fn midgame_has_supplied_settlements_and_an_active_rival_border() {
    let (_, campaign) = fixture();
    let supplied = campaign.supplied_sites(campaign.player);
    let settlements: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| {
            site.controller == Some(campaign.player) && site.habitation >= Habitation::Outpost
        })
        .collect();
    assert!(
        settlements.len() >= 10,
        "only {} settlements",
        settlements.len()
    );
    assert!(
        settlements
            .iter()
            .filter(|site| supplied.contains(&site.id))
            .count()
            >= 10
    );
    let borders: std::collections::BTreeSet<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(campaign.player))
        .flat_map(|site| campaign.world.adjacent_sites(site.id))
        .filter_map(|id| campaign.world.site(id).unwrap().controller)
        .filter(|faction| *faction != campaign.player && campaign.is_independent(*faction))
        .collect();
    assert!(
        borders.len() >= 2,
        "only {} bordering factions",
        borders.len()
    );
    assert!(campaign
        .relations
        .iter()
        .any(|relation| relation.factions.contains(&campaign.player)
            && relation.state == DiplomaticState::War
            && relation
                .factions
                .iter()
                .any(|faction| borders.contains(faction))));
    assert!(
        campaign
            .armies
            .values()
            .filter(|army| army.faction == campaign.player)
            .count()
            >= 3
    );
}

#[test]
fn midgame_has_ten_distinct_living_named_people_with_service() {
    let (_, campaign) = fixture();
    let people: Vec<_> = campaign
        .people
        .values()
        .filter(|person| {
            person.faction == campaign.player
                && person.is_alive()
                && !person.career.retired
                && person.age_years(campaign.completed_rounds) >= 18
        })
        .collect();
    assert!(
        people.len() >= 10,
        "only {} adult named people",
        people.len()
    );
    let names: std::collections::BTreeSet<_> = people.iter().map(|person| &person.name).collect();
    assert_eq!(names.len(), people.len());
    let slots: std::collections::BTreeSet<_> = people
        .iter()
        .filter_map(|person| match person.assignment {
            kestrum::state::people::PersonAssignment::Formation { formation } => Some(formation),
            _ => None,
        })
        .collect();
    assert_eq!(
        slots.len(),
        people.len(),
        "every midgame hero has their own formation"
    );
    assert!(
        campaign
            .armies
            .values()
            .filter(|army| army.faction == campaign.player)
            .filter(|army| army.formation_ids().any(|id| slots.contains(&id)))
            .count()
            >= 3,
        "heroes are spread across the armies"
    );
    assert!(
        people
            .iter()
            .filter(|person| person.class != kestrum::data::world::PersonClass::Recruit)
            .count()
            >= 8,
        "only {} trained heroes",
        people
            .iter()
            .filter(|person| person.class != kestrum::data::world::PersonClass::Recruit)
            .count()
    );
    assert!(
        people
            .iter()
            .filter(|person| person.service_start_round < 40)
            .count()
            >= 8
    );
}

#[test]
fn review_save_has_real_progress_and_a_living_player_army() {
    let (data, campaign) = fixture();
    campaign.validate(&data).unwrap();
    assert_eq!(campaign.completed_rounds, 60);
    assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
    assert!(campaign.diplomacy.ending.is_none());
    assert!(
        player_evidence_total(&campaign, EvidenceKind::Battle) > 0,
        "the review campaign preserves player battle evidence"
    );
    assert!(
        player_evidence_total(&campaign, EvidenceKind::MeaningfulEncounter) > 0,
        "the review campaign preserves meaningful battle evidence"
    );
    assert!(campaign
        .battle_templates
        .iter()
        .any(|template| template.name == "Homeward Line"));
    assert!(campaign
        .formations
        .values()
        .any(|formation| formation.faction == campaign.player && formation.headcount > 0));
}

#[test]
fn progressed_save_roundtrips_with_immutable_battle_evidence() {
    let (data, campaign) = fixture();
    let battle_evidence = player_evidence_total(&campaign, EvidenceKind::Battle);
    let encounter_evidence = player_evidence_total(&campaign, EvidenceKind::MeaningfulEncounter);
    assert!(battle_evidence > 0);
    assert!(encounter_evidence > 0);
    let snapshot = Campaign::Strategic(Box::new(campaign.clone()));
    let raw = serde_json::to_string(&snapshot).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    assert_eq!(restored, snapshot);
    let mut restored = restored.strategic().unwrap().clone();
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::Battle),
        battle_evidence
    );
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::MeaningfulEncounter),
        encounter_evidence
    );
    let history = restored.battles.clone();
    assert!(apply(
        &mut restored,
        &data,
        Actor::Player,
        Command::StartPendingBattle
    )
    .is_err());
    assert_eq!(restored, campaign);
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::Battle),
        battle_evidence
    );
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::MeaningfulEncounter),
        encounter_evidence
    );
    apply(&mut restored, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(restored.battles, history);
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::Battle),
        battle_evidence
    );
    assert_eq!(
        player_evidence_total(&restored, EvidenceKind::MeaningfulEncounter),
        encounter_evidence
    );
    restored.validate(&data).unwrap();
}
