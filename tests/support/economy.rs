//! Shared fixtures and focused edge cases for the five K05 acceptance tests.

use super::*;
use kestrum::state::persistence::{SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE};
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

pub(super) fn recruit(kind: TroopKind, army: Option<ArmyId>) -> Command {
    Command::Recruit {
        site: SiteId(1),
        army,
        kind,
    }
}

pub(super) fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn specialists(campaign: &mut StrategicCampaign) {
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(1))
        .unwrap();
    site.facilities
        .extend([Facility::Stable, Facility::Infirmary, Facility::Workshop]);
    site.military = MilitaryLayer::Fort;
}

pub(super) fn assert_legacy_military_migration(data: &GameData, mut campaign: StrategicCampaign) {
    campaign.completed_rounds = 7;
    let mut old = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    for key in ["armies", "formations", "people"] {
        old.as_object_mut().unwrap().remove(key);
    }
    for key in ["army", "formation", "person"] {
        old["next_ids"].as_object_mut().unwrap().remove(key);
    }
    old["world"].as_object_mut().unwrap().remove("site_damage");
    for faction in old["factions"].as_object_mut().unwrap().values_mut() {
        faction.as_object_mut().unwrap().remove("last_economy");
    }
    let restored: Campaign = serde_json::from_value(old.clone()).unwrap();
    restored.validate(data).unwrap();
    let strategic = restored.strategic().unwrap();
    assert!(
        strategic.armies.is_empty()
            && strategic.formations.is_empty()
            && strategic.people.is_empty()
    );
    assert_eq!(strategic.next_ids.army, ArmyId(1));
    assert_eq!(strategic.next_ids.formation, FormationId(1));
    assert_eq!(strategic.next_ids.person.0, 1);
    assert_eq!(strategic.completed_rounds, campaign.completed_rounds);
    assert_eq!(strategic.rng, campaign.rng);
    assert_eq!(
        strategic.factions[&campaign.player].resources,
        campaign.factions[&campaign.player].resources
    );
    let envelope = encode_slot("kestrum_strategic_v2", &old, "2").unwrap();
    assert_eq!(load_legacy(&envelope, data).unwrap(), restored);
    assert_catalogue_migration(data, &old, &restored);
    old["armies"] = serde_json::json!({});
    assert!(serde_json::from_value::<Campaign>(old).is_err());
    campaign.next_ids.formation = FormationId(1);
    assert!(campaign
        .validate(data)
        .unwrap_err()
        .contains("next_ids.formation"));
}

fn assert_catalogue_migration(data: &GameData, old: &serde_json::Value, expected: &Campaign) {
    let raw = serde_json::to_string(old).unwrap();
    let campaign = expected.strategic().unwrap();
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let metadata = SaveMetadata {
        name: "Before founding forces".into(),
        kind: SaveKind::Manual,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        schema_version: 2,
        content_version: 1,
        success_order,
    };
    let saved = catalogue
        .write(
            &mut store,
            "earlier_military",
            None,
            metadata.clone(),
            &raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    assert_eq!(library.entries()[0].metadata, metadata);
    let restored = library.load(&mut store, data, saved.entry_id).unwrap();
    assert_eq!(&restored, expected);
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);
    let mut resumed = restored.strategic().unwrap().clone();
    let result = apply(
        &mut resumed,
        data,
        Actor::Player,
        recruit(TroopKind::Warriors, None),
    )
    .unwrap()
    .recruited
    .unwrap();
    assert_eq!(result.army, ArmyId(1));
    assert_eq!(result.formation, FormationId(1));
    assert_eq!(
        resumed.formations[&result.formation].created_round,
        campaign.completed_rounds
    );
    assert_eq!(resumed.armies[&result.army].site, SiteId(1));
    assert!(resumed.people.is_empty());
    assert_eq!(resumed.rng, campaign.rng);
    resumed.validate(data).unwrap();
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);
}

#[derive(Default)]
struct MemoryStore(BTreeMap<String, String>);

impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        self.0.insert(key.to_owned(), content.to_owned());
        Ok(())
    }
}

impl IndexedSaveStore for MemoryStore {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(WriterStatus::Ready)
    }

    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.0.remove(key);
        Ok(())
    }
}

pub(super) fn assert_army_panel_command_boundary(data: &GameData, campaign: StrategicCampaign) {
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(campaign)), data)
        .unwrap();
    state.overlay = Overlay::Armies;
    assert_eq!(
        state.command(data, Command::EndTurn),
        Err(RuleError::PlayObstructed)
    );
    state
        .command(data, recruit(TroopKind::Warriors, Some(ArmyId(1))))
        .unwrap();
    assert_eq!(state.overlay, Overlay::Armies);
}

pub(super) fn rejected(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
    reason: &str,
) {
    let before = campaign.clone();
    let expected = preview(campaign, data, actor, command.clone()).unwrap_err();
    assert!(expected.to_string().contains(reason), "{expected}");
    assert_eq!(apply(campaign, data, actor, command), Err(expected));
    assert_eq!(
        *campaign, before,
        "rejection preserves resources, IDs, RNG, and pending facts"
    );
}

pub(super) fn assert_invalid_facilities(data: &GameData, initial: &StrategicCampaign) {
    for (kind, facility) in [
        (TroopKind::Riders, Facility::Stable),
        (TroopKind::Medics, Facility::Infirmary),
        (TroopKind::SiegeEngines, Facility::Workshop),
    ] {
        let mut campaign = initial.clone();
        campaign.world.sites[1].facilities.push(facility);
        rejected(
            &mut campaign,
            data,
            Actor::Player,
            recruit(kind, None),
            "functional local",
        );
        campaign.world.sites[0].facilities.push(facility);
        campaign.world.sites[0].military = MilitaryLayer::Fort;
        campaign.world.site_damage.insert(SiteId(1), 75);
        rejected(
            &mut campaign,
            data,
            Actor::Player,
            recruit(kind, None),
            "Structural damage",
        );
        campaign.world.site_damage.insert(SiteId(1), 74);
        apply(&mut campaign, data, Actor::Player, recruit(kind, None)).unwrap();
    }
    let mut campaign = initial.clone();
    campaign.world.sites[0].facilities.push(Facility::Workshop);
    rejected(
        &mut campaign,
        data,
        Actor::Player,
        recruit(TroopKind::SiegeEngines, None),
        "Fort",
    );
    let mut campaign = initial.clone();
    for site in [SiteId(5), SiteId(6)] {
        campaign
            .set_site_control(data, site, Some(campaign.player), false)
            .unwrap();
    }
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(6))
        .unwrap();
    site.facilities.push(Facility::Stable);
    rejected(
        &mut campaign,
        data,
        Actor::Player,
        Command::Recruit {
            site: SiteId(6),
            army: None,
            kind: TroopKind::Riders,
        },
        "horse access",
    );
}

pub(super) fn assert_income_eligibility_and_overflow(data: &GameData) {
    for (damage, expected) in [
        (
            50,
            Resources {
                gold: 47,
                wood: 18,
                stone: 11,
            },
        ),
        (
            99,
            Resources {
                gold: 45,
                wood: 17,
                stone: 11,
            },
        ),
    ] {
        let mut campaign = StrategicCampaign::new(data).unwrap();
        campaign.world.site_damage.insert(SiteId(1), damage);
        finish_round(&mut campaign, data);
        assert_eq!(
            campaign.factions[&campaign.player]
                .last_economy
                .as_ref()
                .unwrap()
                .income,
            expected
        );
    }
    let mut campaign = StrategicCampaign::new(data).unwrap();
    campaign
        .set_site_control(data, SiteId(2), Some(campaign.player), false)
        .unwrap();
    finish_round(&mut campaign, data);
    assert_eq!(
        campaign.factions[&campaign.player]
            .last_economy
            .as_ref()
            .unwrap()
            .income
            .gold,
        60,
        "captured foreign HQ is only a settlement"
    );
    assert_eq!(
        campaign.factions[&FactionId(2)]
            .last_economy
            .as_ref()
            .unwrap()
            .income
            .gold,
        0
    );
    let mut campaign = StrategicCampaign::new(data).unwrap();
    campaign
        .set_site_control(data, SiteId(1), Some(campaign.player), true)
        .unwrap();
    finish_round(&mut campaign, data);
    assert_eq!(
        campaign.factions[&campaign.player]
            .last_economy
            .as_ref()
            .unwrap()
            .income
            .gold,
        0
    );
    let mut campaign = StrategicCampaign::new(data).unwrap();
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = i64::MAX;
    apply(&mut campaign, data, Actor::Player, Command::EndTurn).unwrap();
    advance_npc(&mut campaign, data).unwrap();
    advance_npc(&mut campaign, data).unwrap();
    rejected(
        &mut campaign,
        data,
        Actor::Npc(FactionId(4)),
        Command::EndTurn,
        "resource balance",
    );
}

pub(super) fn assert_unaffordable(data: &GameData, initial: &StrategicCampaign) {
    for resource in ["gold", "wood", "stone"] {
        let mut campaign = initial.clone();
        specialists(&mut campaign);
        let balance = &mut campaign
            .factions
            .get_mut(&campaign.player)
            .unwrap()
            .resources;
        match resource {
            "gold" => balance.gold = 0,
            "wood" => balance.wood = 0,
            _ => balance.stone = 0,
        }
        rejected(
            &mut campaign,
            data,
            Actor::Player,
            recruit(TroopKind::SiegeEngines, None),
            "available",
        );
    }
}

pub(super) fn prepare_shortfall(campaign: &mut StrategicCampaign, data: &GameData) {
    for _ in 0..3 {
        apply(
            campaign,
            data,
            Actor::Player,
            recruit(TroopKind::Warriors, Some(ArmyId(1))),
        )
        .unwrap();
    }
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 0;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 1;
    for formation in campaign.formations.values_mut() {
        formation.movement_spent = data.economy.formations[&formation.kind].movement_allowance;
    }
    for person in campaign.people.values_mut() {
        person.movement_spent = 6;
    }
}

pub(super) fn assert_zero_headcount_cleanup(
    data: &GameData,
    campaign: &mut StrategicCampaign,
    founder: kestrum::state::people::PersonId,
) {
    campaign
        .formations
        .get_mut(&FormationId(2))
        .unwrap()
        .headcount = 0;
    assert!(campaign.validate(data).is_err());
    campaign.remove_formation(FormationId(2)).unwrap();
    campaign.validate(data).unwrap();
    assert_eq!(
        campaign.people[&founder].assignment,
        PersonAssignment::Formation {
            formation: FormationId(3)
        }
    );
    let before = campaign.clone();
    assert!(campaign.remove_formation(FormationId(2)).is_err());
    assert_eq!(*campaign, before);
}

pub(super) fn assert_fresh_after_removal(
    data: &GameData,
    campaign: &mut StrategicCampaign,
    replacement: kestrum::engine::RecruitmentResult,
) {
    let fresh = apply(
        campaign,
        data,
        Actor::Player,
        recruit(TroopKind::Warriors, None),
    )
    .unwrap()
    .recruited
    .unwrap();
    assert!(fresh.formation > replacement.formation && fresh.army > replacement.army);
    assert_eq!(campaign.formations[&fresh.formation].headcount, 100);
    rejected(
        campaign,
        data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(4),
        },
        "own formations",
    );
    rejected(
        campaign,
        data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(1),
        },
        "unavailable",
    );
    campaign.validate(data).unwrap();
}

pub(super) fn assert_deficit_statement_contract(data: &GameData, campaign: &StrategicCampaign) {
    let encoded = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let mut missing_deficit = encoded.clone();
    missing_deficit["factions"]["1"]["deficit"] = serde_json::json!(false);
    let restored: Campaign = serde_json::from_value(missing_deficit).unwrap();
    assert!(restored.validate(data).is_err());

    let mut invented_deficit = encoded.clone();
    let statement = &mut invented_deficit["factions"]["1"]["last_economy"];
    statement["shortfall"] = serde_json::json!(0);
    statement["upkeep_paid"] = statement["upkeep_due"].clone();
    let restored: Campaign = serde_json::from_value(invented_deficit).unwrap();
    assert!(restored.validate(data).is_err());

    let mut legacy_deficit = encoded;
    legacy_deficit["factions"]["1"]["last_economy"] = serde_json::Value::Null;
    legacy_deficit["factions"]["1"]["last_recovery"] = serde_json::Value::Null;
    let restored: Campaign = serde_json::from_value(legacy_deficit).unwrap();
    restored.validate(data).unwrap();
    assert!(restored.strategic().unwrap().factions[&campaign.player].deficit);
}
