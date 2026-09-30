//! K18 integration of outpost work, a persisted live siege and relief.

use kestrum::{
    data::{
        economy::{Habitation, TroopKind},
        world::{FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{apply, Actor, Command, MoveOrder},
    state::{
        battle::{BattleContext, BattleOutcome},
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, OrderId},
        military::{Army, ArmyId, Formation, FormationId},
        persistence::{SaveLibrary, SaveReceipt},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::{IndexedSaveStore, RawSaveStore, WriterStatus};
use std::collections::BTreeMap;

#[test]
fn outpost_and_fortified_siege_survive_catalogue_reload_then_resolve_relief_once() {
    let mut data = GameData::load().expect("game data");
    data.threats.initial.clear();
    let mut campaign = StrategicCampaign::new(&data).expect("Rosemarch campaign");
    let rose = FactionId(1);
    let hawthorn = FactionId(3);

    // This authored deployment keeps the test focused on accepted orders and the
    // seasonal/save lifecycle instead of travel time to the selected front.
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(9);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(8);
    for id in campaign.armies[&ArmyId(1)]
        .formation_ids()
        .chain(campaign.armies[&ArmyId(3)].formation_ids())
    {
        let formation = campaign.formations.get_mut(&id).unwrap();
        formation.headcount = 1;
        formation.movement_spent = 0;
    }

    for site in [5, 6, 7, 14] {
        campaign
            .set_site_control(&data, SiteId(site), Some(rose), false)
            .expect("Rose supply corridor");
    }
    campaign
        .set_site_control(&data, SiteId(8), Some(hawthorn), false)
        .expect("Hawthorn staging site");
    campaign
        .set_site_control(&data, SiteId(9), Some(rose), false)
        .expect("Rose fort");
    let outpost = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(14))
        .unwrap();
    outpost.habitation = Habitation::Camp;
    outpost.military = MilitaryLayer::None;
    campaign.world.population.insert(SiteId(14), 20);

    let relief_army = ArmyId(campaign.next_ids.army.0);
    let relief_formation = FormationId(campaign.next_ids.formation.0);
    campaign.armies.insert(
        relief_army,
        Army {
            id: relief_army,
            faction: rose,
            site: SiteId(14),
            name: "Rose Relief Column".into(),
            slots: [Some(relief_formation), None, None, None, None, None],
            commander: None,
            battle_doctrine: None,
        },
    );
    campaign.formations.insert(
        relief_formation,
        Formation {
            battle_leader: None,
            tactics: None,
            tactics_override: Some(false),
            id: relief_formation,
            faction: rose,
            kind: TroopKind::Warriors,
            headcount: 100,
            capacity: data.economy.formations[&TroopKind::Warriors].capacity,
            movement_spent: 0,
            created_round: campaign.completed_rounds,
            service: Default::default(),
        },
    );
    campaign.next_ids.army = ArmyId(relief_army.0 + 1);
    campaign.next_ids.formation = FormationId(relief_formation.0 + 1);
    campaign
        .validate(&data)
        .expect("authored deployment is valid");

    let order = OrderId(campaign.next_ids.order.0);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(14)),
            kind: ConstructionKind::Outpost,
            builder: relief_army,
        },
    )
    .expect("place a real outpost order");
    assert_eq!(campaign.construction[&order].progress, 0);

    // Hawthorn opens the siege with a legal movement command during its turn.
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).expect("end Rose's turn");
    while campaign.active_faction() != hawthorn {
        pass_current_npc(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Npc(hawthorn),
        Command::Move(MoveOrder {
            armies: vec![ArmyId(3)],
            path: vec![SiteId(8), SiteId(9)],
        }),
    )
    .expect("Hawthorn enters Rose's fortified site");
    let initial_siege = campaign.sieges.get(&SiteId(9)).expect("live siege");
    assert_eq!(initial_siege.defender, rose);
    assert_eq!(initial_siege.besieger, hawthorn);
    assert_eq!(initial_siege.elapsed_steps, 0);
    assert_eq!(
        campaign.world.site(SiteId(9)).unwrap().military,
        MilitaryLayer::Fort
    );

    // Complete this season, then use the real indexed catalogue/load path at the
    // first player turn after siege weakening has been applied.
    finish_current_round(&mut campaign, &data);
    assert_eq!(campaign.completed_rounds, 1);
    assert_eq!(campaign.sieges[&SiteId(9)].elapsed_steps, 1);
    assert_eq!(
        campaign.world.fort_damage[&SiteId(9)],
        data.siege.fort_damage_per_step
    );
    assert_eq!(campaign.construction[&order].progress, 1);
    assert_eq!(campaign.construction[&order].last_progress_round, Some(0));

    let mut resumed = save_and_load(&campaign, &data);
    assert_eq!(
        resumed, campaign,
        "catalogue load must preserve the whole siege"
    );

    // Both uninterrupted and loaded states cross one more boundary. Exact equality
    // plus the single-step assertions catch replayed or doubled seasonal weakening.
    finish_current_round(&mut campaign, &data);
    finish_current_round(&mut resumed, &data);
    assert_eq!(resumed, campaign);
    assert_eq!(campaign.completed_rounds, 2);
    assert_eq!(campaign.sieges[&SiteId(9)].elapsed_steps, 2);
    assert_eq!(
        campaign.world.fort_damage[&SiteId(9)],
        data.siege.fort_damage_per_step * 2
    );
    assert_eq!(campaign.construction[&order].progress, 2);
    assert_eq!(
        campaign.construction[&order].status,
        ConstructionStatus::Active
    );

    // Reload the still-live siege again, then make the same real relief movement
    // in each branch. The outcome and persistent world state must remain identical.
    resumed = save_and_load(&campaign, &data);
    assert_eq!(resumed.sieges[&SiteId(9)].elapsed_steps, 2);
    let result = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![relief_army],
            path: vec![SiteId(14), SiteId(9)],
        }),
    )
    .expect("relief enters the besieged fort");
    assert!(result.battle_pending);
    let resolved = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .expect("player accepts the saved relief battle");
    let replayed_result = apply(
        &mut resumed,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![relief_army],
            path: vec![SiteId(14), SiteId(9)],
        }),
    )
    .expect("loaded relief enters the same fort");
    assert!(replayed_result.battle_pending);
    let replayed_resolved = apply(
        &mut resumed,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .expect("loaded player accepts the same relief battle");
    assert_eq!(resolved, replayed_resolved);
    let report = &campaign.battles[&resolved.battle.expect("relief battle")];
    assert!(matches!(report.context, BattleContext::Relief { .. }));
    assert_eq!(report.outcome, BattleOutcome::AttackerVictory);
    assert!(
        campaign.sieges.is_empty(),
        "victorious relief lifts the siege"
    );
    assert_eq!(
        campaign.world.fort_damage[&SiteId(9)],
        data.siege.fort_damage_per_step * 2
    );
    assert_eq!(resumed, campaign);
    assert_eq!(campaign.construction[&order].progress, 2);
    campaign
        .validate(&data)
        .expect("post-relief campaign remains valid");
}

fn pass_current_npc(campaign: &mut StrategicCampaign, data: &GameData) {
    let CampaignPhase::NpcTurn { faction, .. } = campaign.phase else {
        panic!("expected an NPC phase")
    };
    apply(campaign, data, Actor::Npc(faction), Command::EndTurn).expect("pass NPC phase");
}

fn finish_current_round(campaign: &mut StrategicCampaign, data: &GameData) {
    if campaign.phase == CampaignPhase::PlayerTurn {
        apply(campaign, data, Actor::Player, Command::EndTurn).expect("end player turn");
    }
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_current_npc(campaign, data);
    }
    assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
}

fn save_and_load(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let mut store = MemoryStore::default();
    let mut saves = SaveLibrary::open(&mut store, data).expect("open save catalogue");
    let payload = Campaign::Strategic(Box::new(campaign.clone()));
    let prepared = saves
        .prepare_manual(&mut store, data, &payload, "K18 siege checkpoint", None)
        .expect("prepare manual save at player turn");
    let SaveReceipt { id, .. } = saves
        .write(&mut store, data, &prepared)
        .expect("write campaign save");
    saves
        .load(&mut store, data, id)
        .expect("load campaign through catalogue")
        .strategic()
        .expect("strategic campaign")
        .clone()
}

#[derive(Default)]
struct MemoryStore(BTreeMap<String, String>);

impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.0.insert(key.into(), value.into());
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
