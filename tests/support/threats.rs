use super::*;
use kestrum::state::{
    battle::BattleReport,
    persistence::{SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE},
};
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn fixture(person: bool) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots =
        [Some(FormationId(1)), None, None, None, None, None];
    campaign.formations.remove(&FormationId(2));
    campaign.formations.remove(&FormationId(3));
    if !person {
        campaign.people.remove(&PersonId(1));
        campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
    }
    campaign
        .set_site_control(&data, SiteId(12), Some(FactionId(1)), false)
        .unwrap();
    campaign.validate(&data).unwrap();
    (data, campaign)
}
pub(super) fn command(threat: u32) -> Command {
    Command::ClearThreat {
        armies: vec![ArmyId(1)],
        threat: ThreatId(threat),
    }
}
pub(super) fn clear(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    threat: u32,
) -> BattleReport {
    let outcome = apply(campaign, data, Actor::Player, command(threat)).unwrap();
    campaign.battles[&outcome.battle.unwrap()].clone()
}
pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}
pub(super) fn assert_invalid_content(data: &GameData) {
    let mut invalid = data.clone();
    invalid
        .threats
        .definitions
        .get_mut(&ThreatKind::Bandits)
        .unwrap()
        .attack = 0;
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    invalid
        .threats
        .initial
        .push(invalid.threats.initial[0].clone());
    assert!(invalid.validate().is_err());
    let mut invalid = data.clone();
    invalid
        .threats
        .definitions
        .get_mut(&ThreatKind::Wildlife)
        .unwrap()
        .reward
        .gold = -1;
    assert!(invalid.validate().is_err());
}
pub(super) fn assert_privacy_and_blockers(data: &GameData) {
    assert_hidden_route_supply(data);
    let mut campaign = StrategicCampaign::new(data).unwrap();
    let remote = project(&campaign, FactionId(1)).unwrap();
    assert!(remote.threats.is_empty());
    campaign.threats.get_mut(&ThreatId(1)).unwrap().headcount = 1;
    assert_eq!(remote, project(&campaign, FactionId(1)).unwrap());
    let before = campaign.clone();
    assert!(apply(&mut campaign, data, Actor::Player, command(1)).is_err());
    assert_eq!(campaign, before);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    let view = project(&campaign, FactionId(1)).unwrap();
    assert_eq!(view.threats.len(), 1);
    let preview = threat_preview(&campaign, data, FactionId(1), &[ArmyId(1)], ThreatId(1)).unwrap();
    campaign.threats.get_mut(&ThreatId(1)).unwrap().headcount = 60;
    assert_eq!(view, project(&campaign, FactionId(1)).unwrap());
    assert_eq!(
        preview,
        threat_preview(&campaign, data, FactionId(1), &[ArmyId(1)], ThreatId(1)).unwrap()
    );
    assert!(threat_preview(
        &campaign,
        data,
        FactionId(1),
        &[ArmyId(1), ArmyId(1)],
        ThreatId(1)
    )
    .is_err());
    let allowance =
        data.economy.formations[&campaign.formations[&FormationId(1)].kind].movement_allowance;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .movement_spent = allowance;
    let before = campaign.clone();
    assert!(apply(&mut campaign, data, Actor::Player, command(1)).is_err());
    assert_eq!(campaign, before);
}

fn assert_hidden_route_supply(data: &GameData) {
    let mut campaign = StrategicCampaign::new(data).unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(10);
    for site in [5, 6, 8, 10] {
        campaign
            .set_site_control(data, SiteId(site), Some(FactionId(1)), false)
            .unwrap();
    }
    assert!(project(&campaign, FactionId(1)).unwrap().threats.is_empty());
    let forecast =
        kestrum::engine::movement_preview(&campaign, data, FactionId(1), &[ArmyId(1)], SiteId(13))
            .unwrap();
    let mut empty = campaign.clone();
    empty.threats.remove(&ThreatId(1));
    let without =
        kestrum::engine::movement_preview(&empty, data, FactionId(1), &[ArmyId(1)], SiteId(13))
            .unwrap();
    assert_eq!(
        forecast, without,
        "a hidden future-route occupant cannot alter supply forecasts"
    );
    assert!(forecast.supplied_after);
    let outcome = apply(
        &mut campaign,
        data,
        Actor::Player,
        Command::Move(forecast.order),
    )
    .unwrap();
    let moved = outcome.movement.unwrap();
    assert_eq!(moved.path, vec![SiteId(10), SiteId(11)]);
    assert_eq!(moved.spent, 2);
    assert_eq!(
        moved.stop.unwrap().reason,
        kestrum::engine::MovementBlock::ThreatRequiresClear
    );
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(11));
    assert_eq!(campaign.threats[&ThreatId(1)].headcount, 60);
    assert_eq!(project(&campaign, FactionId(1)).unwrap().threats.len(), 1);
}

#[derive(Default)]
struct Store(BTreeMap<String, String>);
impl RawSaveStore for Store {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }
    fn write(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.0.insert(key.into(), value.into());
        Ok(())
    }
}
impl IndexedSaveStore for Store {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(WriterStatus::Ready)
    }
    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.0.remove(key);
        Ok(())
    }
}
pub(super) fn reload(data: &GameData, campaign: &StrategicCampaign) -> Campaign {
    let expected = Campaign::Strategic(Box::new(campaign.clone()));
    let raw = serde_json::to_string(&expected).unwrap();
    let mut store = Store::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let metadata = SaveMetadata {
        name: "Local threat".into(),
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
            "threat_checkpoint",
            None,
            metadata,
            &raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    let loaded = library.load(&mut store, data, saved.entry_id).unwrap();
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);
    loaded
}
