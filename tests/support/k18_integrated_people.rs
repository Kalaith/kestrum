//! Reusable physical-route and receipt assertions for the K18 people chain.

use kestrum::{
    data::{
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{apply, career_options, movement_preview, Actor, Command, MoveOrder, MovementBlock},
    state::{
        battle::{BattleReport, BattleSideReport},
        history::HistoryKind,
        military::{ArmyId, FormationId},
        people::PersonId,
        persistence::load_legacy,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
use macroquad_toolkit::persistence::encode_slot;
use std::collections::{BTreeMap, VecDeque};
pub(super) const ROSE: FactionId = FactionId(1);
pub(super) const HAWTHORN: FactionId = FactionId(3);
pub(super) const HQ: SiteId = SiteId(1);
pub(super) const WEST_GATE: SiteId = SiteId(5);
pub(super) const MILLTOWN: SiteId = SiteId(6);
pub(super) const BRIDGE: SiteId = SiteId(8);
pub(super) const HIGH_FORT: SiteId = SiteId(9);
pub(super) const ROSE_ARMY: ArmyId = ArmyId(1);
pub(super) const HAWTHORN_ARMY: ArmyId = ArmyId(3);
pub(super) const AVELINE: PersonId = PersonId(1);
pub(super) const FIRST_WARRIORS: FormationId = FormationId(1);
pub(super) const SPEARMEN: FormationId = FormationId(2);
pub(super) const HAWTHORN_WARRIORS: FormationId = FormationId(7);
pub(super) fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        apply(campaign, data, actor, Command::EndTurn).expect("pass a legal faction turn");
    }
}

pub(super) fn finish_hawthorn_approach(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        if faction == HAWTHORN
            && campaign.armies[&HAWTHORN_ARMY].site == SiteId(3)
            && campaign.armies[&HAWTHORN_ARMY]
                .formation_ids()
                .any(|formation| formation == HAWTHORN_WARRIORS)
        {
            let detached = apply(
                campaign,
                data,
                Actor::Npc(faction),
                Command::SplitArmy {
                    formation: HAWTHORN_WARRIORS,
                },
            )
            .unwrap()
            .split_army
            .unwrap();
            assert_eq!(detached, ArmyId(6));
            let approach = move_army(
                campaign,
                data,
                Actor::Npc(faction),
                detached,
                &[SiteId(3), SiteId(11), SiteId(10), BRIDGE],
            );
            assert!(approach.battle.is_none());
        } else if faction == HAWTHORN && campaign.armies[&HAWTHORN_ARMY].site == SiteId(3) {
            let approach = move_army(
                campaign,
                data,
                Actor::Npc(faction),
                HAWTHORN_ARMY,
                &[SiteId(3), SiteId(11), SiteId(10), BRIDGE],
            );
            assert!(approach.battle.is_none());
        }
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, round + 1);
}

pub(super) fn finish_hawthorn_counterattack(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> kestrum::state::battle::BattleId {
    let round = campaign.completed_rounds;
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    let mut battle = None;
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        if faction == HAWTHORN {
            let result = move_army(
                campaign,
                data,
                Actor::Npc(faction),
                ArmyId(6),
                &[BRIDGE, MILLTOWN],
            );
            battle = result.battle;
            assert!(battle.is_some(), "Hawthorn must attack the occupied town");
        }
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, round + 1);
    battle.unwrap()
}

pub(super) fn finish_hawthorn_main_counterattack(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> kestrum::state::battle::BattleId {
    let round = campaign.completed_rounds;
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    let mut battle = None;
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        if faction == HAWTHORN {
            let result = move_army(
                campaign,
                data,
                Actor::Npc(faction),
                HAWTHORN_ARMY,
                &[BRIDGE, MILLTOWN],
            );
            battle = result.battle;
            assert!(
                battle.is_some(),
                "Hawthorn's main army must counterattack legally"
            );
        }
        apply(campaign, data, Actor::Npc(faction), Command::EndTurn).unwrap();
    }
    assert_eq!(campaign.completed_rounds, round + 1);
    battle.unwrap()
}

pub(super) fn move_army(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    army: ArmyId,
    path: &[SiteId],
) -> kestrum::engine::ActionOutcome {
    apply(
        campaign,
        data,
        actor,
        Command::Move(MoveOrder {
            armies: vec![army],
            path: path.to_vec(),
        }),
    )
    .unwrap()
}

pub(super) fn assert_side_casualties_and_medic(
    report: &BattleReport,
    faction: FactionId,
    army_id: ArmyId,
    medic_formation: FormationId,
    medic_person: PersonId,
) {
    let side = side_for(report, faction);
    assert!(
        side.armies
            .iter()
            .flat_map(|army| &army.formations)
            .any(|formation| { formation.combat_losses + formation.encirclement_losses > 0 }),
        "the treatment receipt must come from real friendly casualties"
    );
    let army = side.armies.iter().find(|army| army.id == army_id).unwrap();
    let medic = army
        .formations
        .iter()
        .find(|formation| formation.id == medic_formation)
        .expect("the real Medics formation participates");
    assert!(medic.end > 0, "the Medics formation must survive to treat");
    assert!(army.people.iter().any(|person| {
        person.id == medic_person && person.starting_formation == medic_formation
    }));
}

pub(super) fn side_for(report: &BattleReport, faction: FactionId) -> &BattleSideReport {
    if report.attacker.faction == faction {
        &report.attacker
    } else {
        report.defender.faction_side().unwrap()
    }
}

pub(super) fn medic_option(
    campaign: &StrategicCampaign,
    data: &GameData,
    person: PersonId,
) -> kestrum::engine::CareerOption {
    career_options(campaign, data, person)
        .unwrap()
        .into_iter()
        .find(|option| option.class == PersonClass::Medic)
        .unwrap()
}

pub(super) fn return_to_site(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    army: ArmyId,
    destination: SiteId,
) {
    while campaign.armies[&army].site != destination {
        let origin = campaign.armies[&army].site;
        let preview = movement_preview(campaign, data, campaign.player, &[army], destination)
            .expect("return path remains legal");
        assert!(
            preview.reachable_steps > 0,
            "return to {destination:?} from {origin:?}"
        );
        let path = std::iter::once(origin)
            .chain(
                preview
                    .steps
                    .iter()
                    .take(preview.reachable_steps)
                    .map(|step| step.to),
            )
            .collect::<Vec<_>>();
        let progress = move_army(campaign, data, Actor::Player, army, &path);
        assert!(progress.movement.unwrap().spent > 0);
        if campaign.armies[&army].site != destination {
            finish_round(campaign, data);
        }
    }
}

pub(super) fn move_to_site(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    army: ArmyId,
    destination: SiteId,
) {
    while campaign.armies[&army].site != destination {
        let origin = campaign.armies[&army].site;
        let preview = movement_preview(campaign, data, owner, &[army], destination)
            .expect("the authored physical route remains available");
        if preview.reachable_steps == 0
            && matches!(
                preview.stop.as_ref().map(|stop| &stop.reason),
                Some(MovementBlock::InsufficientMovement { .. })
            )
        {
            finish_round(campaign, data);
            continue;
        }
        assert!(
            preview.reachable_steps > 0,
            "reach {destination:?} from {origin:?}: {:?}",
            preview.stop
        );
        let path = std::iter::once(origin)
            .chain(
                preview
                    .steps
                    .iter()
                    .take(preview.reachable_steps)
                    .map(|step| step.to),
            )
            .collect::<Vec<_>>();
        let progress = move_army(campaign, data, Actor::Player, army, &path);
        assert!(progress.battle.is_none());
        assert!(progress.movement.unwrap().spent > 0);
        if campaign.armies[&army].site != destination {
            finish_round(campaign, data);
        }
    }
}

pub(super) fn shortest_path(
    campaign: &StrategicCampaign,
    origin: SiteId,
    destination: SiteId,
) -> Vec<SiteId> {
    let mut previous = BTreeMap::from([(origin, None)]);
    let mut pending = VecDeque::from([origin]);
    while let Some(site) = pending.pop_front() {
        if site == destination {
            break;
        }
        for adjacent in campaign.world.adjacent_sites(site) {
            if let std::collections::btree_map::Entry::Vacant(entry) = previous.entry(adjacent) {
                entry.insert(Some(site));
                pending.push_back(adjacent);
            }
        }
    }
    assert!(previous.contains_key(&destination), "physical route exists");
    let mut path = vec![destination];
    let mut cursor = destination;
    while cursor != origin {
        cursor = previous[&cursor].expect("non-origin predecessor");
        path.push(cursor);
    }
    path.reverse();
    path
}

pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let encoded = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    load_legacy(&encoded, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

pub(super) fn battle_history_count(
    campaign: &StrategicCampaign,
    battle: kestrum::state::battle::BattleId,
) -> usize {
    campaign
        .history
        .events
        .values()
        .filter(
            |record| matches!(record.kind, HistoryKind::Battle { battle: id, .. } if id == battle),
        )
        .count()
}
