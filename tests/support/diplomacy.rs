//! Validated setup variation and ordinary commands, with no alternate simulator.

use super::*;

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}
pub(super) fn state(campaign: &StrategicCampaign, a: u32, b: u32) -> DiplomaticState {
    campaign
        .relations
        .iter()
        .find(|pair| pair.factions == [FactionId(a), FactionId(b)])
        .unwrap()
        .state
}
pub(super) fn weak(campaign: &mut StrategicCampaign, owner: u32) {
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == FactionId(owner))
    {
        formation.headcount = 1;
    }
}
pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
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
pub(super) fn reject(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    command: Command,
) {
    let before = campaign.clone();
    assert!(apply(campaign, data, actor, command).is_err());
    assert_eq!(*campaign, before);
}
pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let raw = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    kestrum::state::persistence::load_legacy(&raw, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}
pub(super) fn move_order(path: &[u32]) -> Command {
    Command::Move(MoveOrder {
        armies: vec![ArmyId(1)],
        path: path.iter().map(|id| SiteId(*id)).collect(),
    })
}

pub(super) fn conquest_fixture() -> (GameData, StrategicCampaign) {
    let (mut data, _) = fixture();
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::War;
    }
    for site in &mut data.scenario.sites {
        site.military = MilitaryLayer::None;
    }
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for _ in 0..3 {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::Recruit {
                site: SiteId(1),
                army: Some(ArmyId(1)),
                kind: TroopKind::Warriors,
            },
        )
        .unwrap();
    }

    (data, campaign)
}

pub(super) fn travel(campaign: &mut StrategicCampaign, data: &GameData, path: &[u32]) {
    let destination = SiteId(*path.last().unwrap());
    let mut attempts = 0;
    while campaign.armies[&ArmyId(1)].site != destination {
        attempts += 1;
        assert!(attempts < 30);
        let current = campaign.armies[&ArmyId(1)].site;
        let position = path.iter().position(|id| SiteId(*id) == current).unwrap();
        let command = move_order(&path[position..]);
        if preview(campaign, data, Actor::Player, command.clone()).is_err() {
            finish(campaign, data);
            continue;
        }
        let result = apply(campaign, data, Actor::Player, command).unwrap();
        if result
            .movement
            .as_ref()
            .is_some_and(|moved| moved.planned_destination.is_some())
        {
            finish(campaign, data);
        }
        if campaign.pending_battle.is_some() {
            apply(campaign, data, Actor::Player, Command::StartPendingBattle).unwrap();
        }
        if campaign.diplomacy.has_pending_decision() {
            break;
        }
    }
}

pub(super) fn pending_defeat() -> (GameData, StrategicCampaign) {
    let (data, mut campaign) = conquest_fixture();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    let formations: Vec<_> = campaign.armies[&ArmyId(2)].formation_ids().collect();
    for formation in formations {
        apply(
            &mut campaign,
            &data,
            Actor::Npc(FactionId(2)),
            Command::Disband { formation },
        )
        .unwrap();
    }
    assert!(
        campaign.is_independent(FactionId(2)),
        "controlled HQ still prevents defeat"
    );
    finish(&mut campaign, &data);
    travel(&mut campaign, &data, &[1, 5, 2]);
    assert_eq!(campaign.diplomacy.pending_defeats[0].faction, FactionId(2));
    (data, campaign)
}

pub(super) fn assert_incoming_offer() {
    let (data, mut campaign) = fixture();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::OfferPeace {
            faction: FactionId(1),
        },
    )
    .unwrap();
    assert_eq!(state(&campaign, 1, 2), DiplomaticState::War);
    assert_eq!(campaign.diplomacy.pending_offers.len(), 1);
    reject(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::EndTurn,
    );
    let before = campaign.clone();
    let mut restored = reload(&campaign, &data);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RespondPeace {
            proposer: FactionId(2),
            accept: true,
        },
    )
    .unwrap();
    apply(
        &mut restored,
        &data,
        Actor::Player,
        Command::RespondPeace {
            proposer: FactionId(2),
            accept: true,
        },
    )
    .unwrap();
    assert_eq!(campaign, restored);
    assert_eq!(campaign.completed_rounds, before.completed_rounds);
    assert_eq!(state(&campaign, 1, 2), DiplomaticState::Peace);
}

pub(super) fn assert_siege_peace() {
    let (mut data, _) = fixture();
    data.scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(2))
        .unwrap()
        .military = MilitaryLayer::Fort;
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    apply(&mut campaign, &data, Actor::Player, move_order(&[1, 5])).unwrap();
    finish(&mut campaign, &data);
    apply(&mut campaign, &data, Actor::Player, move_order(&[5, 2])).unwrap();
    assert!(campaign.sieges.contains_key(&SiteId(2)));
    weak(&mut campaign, 2);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert!(campaign.sieges.is_empty());
    assert!(!campaign.world.contested_sites.contains(&SiteId(2)));
    assert_eq!(
        campaign.world.site(SiteId(2)).unwrap().controller,
        Some(FactionId(2))
    );
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(5));
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_inherited_vassal() {
    let (data, mut campaign) = conquest_fixture();
    let ids: Vec<_> = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == FactionId(3))
        .map(|formation| formation.id)
        .collect();
    for id in ids {
        campaign.remove_formation(id).unwrap();
    }
    campaign.factions.get_mut(&FactionId(3)).unwrap().status = FactionStatus::Vassal {
        sovereign: FactionId(2),
    };
    for person in campaign
        .people
        .values_mut()
        .filter(|person| person.faction == FactionId(3))
    {
        let kestrum::state::people::PersonAssignment::Site { site } = person.assignment else {
            panic!("released survivor")
        };
        person.status = PersonStatus::Displaced {
            completed_rounds: 0,
            site,
        };
        person.movement_spent = 0;
    }
    campaign.round_order.retain(|id| *id != FactionId(3));
    travel(&mut campaign, &data, &[1, 5, 2]);
    assert_eq!(
        campaign.factions[&FactionId(3)].status,
        FactionStatus::Vassal {
            sovereign: FactionId(1)
        }
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::ResolveDefeat {
            faction: FactionId(2),
            resolution: DefeatResolution::Annex,
        },
    )
    .unwrap();
    assert!(!campaign.is_independent(FactionId(3)));
    assert!(campaign
        .armies
        .values()
        .all(|army| army.faction != FactionId(3)));
    assert_eq!(reload(&campaign, &data), campaign);
}

pub(super) fn assert_braced_defeat() {
    let (data, mut campaign) = conquest_fixture();
    let first = campaign.armies[&ArmyId(1)].formation_ids().next().unwrap();
    let second = campaign.armies[&ArmyId(2)].formation_ids().next().unwrap();
    let remove: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| [FactionId(1), FactionId(2)].contains(&army.faction))
        .flat_map(|army| army.formation_ids())
        .filter(|id| *id != first && *id != second)
        .collect();
    for id in remove {
        campaign.remove_formation(id).unwrap();
    }
    campaign.people.clear();
    campaign.legacy_items.clear();
    for (id, kind) in [(first, TroopKind::Riders), (second, TroopKind::Spearmen)] {
        let formation = campaign.formations.get_mut(&id).unwrap();
        formation.kind = kind;
        formation.capacity = data.economy.formations[&kind].capacity;
        formation.headcount = 1;
    }
    for army in campaign.armies.values_mut() {
        army.commander = None;
    }
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(5);
    for site in &mut campaign.world.sites {
        if site
            .controller
            .is_some_and(|owner| [FactionId(1), FactionId(2)].contains(&owner))
        {
            site.habitation = Habitation::Camp;
        }
    }
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap()
        .controller = Some(FactionId(2));
    campaign.reconcile_region_control();
    let contact = apply(&mut campaign, &data, Actor::Player, move_order(&[1, 5])).unwrap();
    assert!(contact.battle_pending);
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    assert_eq!(
        campaign.battles[&outcome.battle.unwrap()].outcome,
        kestrum::state::battle::BattleOutcome::DefenderVictory
    );
    assert_eq!(campaign.formations[&second].headcount, 1);
    assert!(!campaign.formations.contains_key(&first));
    assert_eq!(
        campaign.diplomacy.ending.as_ref().unwrap().kind,
        EndingKind::Defeat
    );
    assert_eq!(campaign.completed_rounds, 0);
    assert!(campaign.pending_facts.is_empty());
    assert_eq!(campaign.consumed_sequence, campaign.accepted_sequence);
    reject(&mut campaign, &data, Actor::Player, Command::EndTurn);
    assert_eq!(reload(&campaign, &data), campaign);
}
