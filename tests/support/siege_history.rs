//! Real arrival/order fixtures; no fabricated battle or progression receipts.

use super::*;
use kestrum::{
    engine::ActionOutcome,
    state::military::{Army, Formation},
};

pub(super) fn assert_authored_target() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let first = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5), SiteId(6), SiteId(8)],
        }),
    )
    .unwrap();
    assert!(first.battle.is_none());
    finish(&mut campaign, &data);
    let second = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(8), SiteId(10), SiteId(11), SiteId(3)],
        }),
    )
    .unwrap();
    assert!(second.battle.is_none());
    assert_eq!(campaign.sieges[&SiteId(3)].defending, vec![ArmyId(3)]);
    assert_eq!(campaign.sieges[&SiteId(3)].besieging, vec![ArmyId(1)]);
    campaign.validate(&data).unwrap();
}

pub(super) fn fixture(defending: bool) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign
        .people
        .retain(|id, _| [PersonId(1), PersonId(3)].contains(id));
    campaign.legacy_items.clear();
    for (army_id, formation_id) in [(1, 1), (3, 7)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).unwrap();
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
        army.commander = None;
        army.site = if (army_id == 1) == defending {
            SiteId(9)
        } else {
            SiteId(8)
        };
    }
    let defender = if defending {
        FactionId(1)
    } else {
        FactionId(3)
    };
    let besieger = if defending {
        FactionId(3)
    } else {
        FactionId(1)
    };
    for site in [5, 6, 8] {
        campaign
            .set_site_control(&data, SiteId(site), Some(besieger), false)
            .unwrap();
    }
    campaign
        .set_site_control(&data, SiteId(9), Some(defender), false)
        .unwrap();
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn establish(campaign: &mut StrategicCampaign, data: &GameData, defending: bool) {
    if defending {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
        pass_npc(campaign, data).unwrap();
    }
    let (actor, army) = if defending {
        (Actor::Npc(FactionId(3)), ArmyId(3))
    } else {
        (Actor::Player, ArmyId(1))
    };
    let result = apply(
        campaign,
        data,
        actor,
        Command::Move(MoveOrder {
            armies: vec![army],
            path: vec![SiteId(8), SiteId(9)],
        }),
    )
    .unwrap();
    assert!(result.battle.is_none());
    assert!(campaign.sieges.contains_key(&SiteId(9)));
}

pub(super) fn order(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    action: SiegeAction,
    destination: Option<SiteId>,
) -> ActionOutcome {
    apply(
        campaign,
        data,
        Actor::Player,
        Command::Siege(SiegeOrder {
            site: SiteId(9),
            armies: vec![ArmyId(1)],
            action,
            destination,
        }),
    )
    .unwrap()
}

pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}

pub(super) fn records(campaign: &StrategicCampaign, observer: FactionId) -> Vec<HistoryRecord> {
    history_page(
        campaign,
        observer,
        &HistoryFilter {
            subject: Some(HistorySubject::Site(SiteId(9))),
            kind: Some(HistoryKindFilter::Siege),
            ..Default::default()
        },
    )
    .entries
}

pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let encoded = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    kestrum::state::persistence::load_legacy(&encoded, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

pub(super) fn count(campaign: &StrategicCampaign, formation: u32, kind: EvidenceKind) -> u32 {
    campaign.formations[&FormationId(formation)]
        .service
        .ledger
        .counts
        .get(&kind)
        .copied()
        .unwrap_or(0)
}

pub(super) fn add_relief(campaign: &mut StrategicCampaign) {
    campaign.armies.insert(
        ArmyId(5),
        Army {
            id: ArmyId(5),
            faction: FactionId(1),
            site: SiteId(10),
            name: "Incoming relief".into(),
            slots: [Some(FormationId(13)), None, None, None, None, None],
            commander: None,
            battle_doctrine: None,
        },
    );
    campaign.formations.insert(
        FormationId(13),
        Formation {
            battle_leader: None,
            tactics: None,
            tactics_override: Some(false),
            id: FormationId(13),
            faction: FactionId(1),
            kind: kestrum::data::economy::TroopKind::Warriors,
            headcount: 100,
            capacity: 100,
            movement_spent: 0,
            created_round: 0,
            service: Default::default(),
        },
    );
    campaign.next_ids.army = ArmyId(6);
    campaign.next_ids.formation = FormationId(14);
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(10))
        .unwrap()
        .controller = Some(FactionId(1));
    campaign.reconcile_region_control();
}
