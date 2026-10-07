//! Isolated armies regroup when no worthwhile wartime operation remains.

use kestrum::{
    data::{
        economy::Resources,
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{ai, apply, project, Actor, Command, MoveOrder},
    state::{
        ai::{AiFactionState, AiObjective, AiObjectiveKind},
        military::ArmyId,
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const OAK: FactionId = FactionId(2);
const ROSE: FactionId = FactionId(1);
const ARMY: ArmyId = ArmyId(2);

#[test]
fn healthy_unsupplied_army_returns_toward_supply_during_peace() {
    let (data, mut campaign) = fixture();
    set_peace_with_everyone(&mut campaign, OAK);
    let (isolated_site, next_site) = isolate_army_with_neutral_gap(&mut campaign, &data, OAK, ARMY);
    for formation_id in campaign.armies[&ARMY].formation_ids() {
        let formation = campaign.formations.get_mut(&formation_id).unwrap();
        formation.headcount = formation.capacity * 55 / 100;
        formation.movement_spent = 0;
    }
    no_resources(&mut campaign);

    assert_eq!(
        campaign.world.site(isolated_site).unwrap().controller,
        Some(OAK)
    );
    assert!(!project(&campaign, OAK)
        .unwrap()
        .supplied_armies
        .contains(&ARMY));

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        matches!(decision.command, Command::Move(MoveOrder { ref armies, ref path })
        if armies == &[ARMY] && path == &[isolated_site, next_site]),
        "{decision:?}"
    );
}

#[test]
fn an_unsupplied_army_keeps_a_viable_retained_attack_during_war() {
    let (data, mut campaign) = fixture();
    set_peace_with_everyone(&mut campaign, OAK);
    let (isolated_site, _) = isolate_army_with_neutral_gap(&mut campaign, &data, OAK, ARMY);
    let target = campaign
        .world
        .adjacent_sites(isolated_site)
        .into_iter()
        .find(|site| {
            *site != isolated_site
                && !campaign.threats.values().any(|threat| {
                    threat.site == *site
                        && threat.status == kestrum::state::threat::ThreatStatus::Active
                })
                && !campaign
                    .armies
                    .values()
                    .any(|army| army.faction != OAK && army.site == *site)
                && !campaign
                    .factions
                    .values()
                    .any(|faction| faction.headquarters == *site)
        })
        .expect("an unoccupied approach site");
    campaign
        .set_site_control(&data, target, Some(ROSE), false)
        .unwrap();
    set_relation(&mut campaign, OAK, ROSE, DiplomaticState::War);
    campaign.ai.factions.insert(
        OAK,
        AiFactionState {
            objective: Some(AiObjective {
                site: target,
                chosen_round: campaign.completed_rounds,
                kind: AiObjectiveKind::Attack,
            }),
            ..Default::default()
        },
    );
    no_resources(&mut campaign);
    campaign.validate(&data).unwrap();

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        matches!(decision.command, Command::Move(MoveOrder { ref armies, ref path })
        if armies == &[ARMY] && path == &[isolated_site, target]),
        "{decision:?}"
    );
    assert_eq!(decision.objective.as_ref().unwrap().site, target);
    assert_eq!(
        decision.objective.as_ref().unwrap().kind,
        AiObjectiveKind::Attack
    );
}

#[test]
fn the_last_wartime_garrison_stays_on_its_approach_when_outmatched() {
    let (data, mut campaign) = fixture();
    set_peace_with_everyone(&mut campaign, OAK);
    let (approach, _) = isolate_army_with_neutral_gap(&mut campaign, &data, OAK, ARMY);
    let enemy_site = campaign
        .world
        .adjacent_sites(approach)
        .into_iter()
        .find(|site| {
            !campaign.threats.values().any(|threat| {
                threat.site == *site
                    && threat.status == kestrum::state::threat::ThreatStatus::Active
            }) && !campaign
                .armies
                .values()
                .any(|army| army.faction != OAK && army.site == *site)
                && !campaign
                    .factions
                    .values()
                    .any(|faction| faction.headquarters == *site)
        })
        .expect("an unoccupied enemy approach");
    campaign
        .set_site_control(&data, enemy_site, Some(ROSE), false)
        .unwrap();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = enemy_site;
    set_relation(&mut campaign, OAK, ROSE, DiplomaticState::War);
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.headcount = 1;
        formation.movement_spent = 0;
    }
    no_resources(&mut campaign);
    campaign.validate(&data).unwrap();

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        !matches!(decision.command, Command::Move(ref order) if order.armies.contains(&ARMY)),
        "the only army posted on this wartime approach must remain: {decision:?}"
    );
}

#[test]
fn recent_observed_losses_are_recovery_targets_only_during_war() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    set_relation(&mut campaign, OAK, ROSE, DiplomaticState::War);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign.armies.get_mut(&ARMY).unwrap().site = SiteId(6);
    campaign
        .set_site_control(&data, SiteId(5), Some(ROSE), false)
        .unwrap();
    campaign
        .set_site_control(&data, SiteId(6), Some(OAK), false)
        .unwrap();

    let movement = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(5), SiteId(6)],
        }),
    )
    .unwrap();
    assert!(movement.battle_pending);
    let battle = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap()
    .battle
    .unwrap();
    let lost_site = campaign.battles[&battle].site;
    let report = &campaign.battles[&battle];
    assert_eq!(report.control_before, Some(OAK));
    assert_eq!(report.control_after, Some(ROSE));
    assert_eq!(
        campaign.world.site(lost_site).unwrap().controller,
        Some(ROSE)
    );
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();

    let safe_rose_site = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(ROSE) && site.id != lost_site)
        .filter(|site| {
            !campaign
                .world
                .adjacent_sites(site.id)
                .contains(&campaign.factions[&OAK].headquarters)
                && site.id != campaign.factions[&OAK].headquarters
        })
        .max_by_key(|site| site.id)
        .expect("a distant Rose site")
        .id;
    for army in campaign
        .armies
        .values_mut()
        .filter(|army| army.faction == ROSE)
    {
        army.site = safe_rose_site;
    }
    let oak_headquarters = campaign.factions[&OAK].headquarters;
    campaign.armies.get_mut(&ARMY).unwrap().site = oak_headquarters;
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.headcount = formation.capacity;
        formation.movement_spent = 0;
    }
    for person in campaign
        .people
        .values_mut()
        .filter(|person| person.faction == OAK)
    {
        person.movement_spent = 0;
    }
    no_resources(&mut campaign);
    campaign.validate(&data).unwrap();

    let observed = project(&campaign, OAK).unwrap();
    assert!(observed
        .battles
        .iter()
        .any(|report| report.site == lost_site));
    let recovery = ai::propose(&campaign, &data, OAK).unwrap();
    assert_eq!(
        recovery.objective,
        Some(AiObjective {
            site: lost_site,
            chosen_round: campaign.completed_rounds,
            kind: AiObjectiveKind::Attack,
        }),
        "the witnessed loss is the recovery objective: {recovery:?}"
    );

    set_relation(&mut campaign, OAK, ROSE, DiplomaticState::Peace);
    let peace = ai::propose(&campaign, &data, OAK).unwrap();
    assert_ne!(
        peace.objective.as_ref().map(|objective| objective.kind),
        Some(AiObjectiveKind::Attack),
        "the AI must not attack the former enemy site during peace: {peace:?}"
    );
}

fn isolate_army_with_neutral_gap(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    army: ArmyId,
) -> (SiteId, SiteId) {
    let headquarters = campaign.factions[&faction].headquarters;
    let capital = campaign.factions[&faction].capital;
    let protected_sites: BTreeSet<_> = campaign
        .factions
        .values()
        .filter(|other| other.id != faction)
        .flat_map(|other| [other.headquarters, other.capital])
        .collect();
    let occupied: BTreeSet<_> = campaign
        .armies
        .values()
        .filter(|other| other.id != army)
        .map(|other| other.site)
        .collect();
    let blocked: BTreeSet<_> = protected_sites
        .union(&occupied)
        .copied()
        .chain(campaign.world.contested_sites.iter().copied())
        .chain(
            campaign
                .threats
                .values()
                .filter(|threat| threat.status == kestrum::state::threat::ThreatStatus::Active)
                .map(|threat| threat.site),
        )
        .collect();
    let mut candidates: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.id != headquarters && site.id != capital)
        .filter_map(|site| {
            topology_path(&campaign.world, headquarters, site.id, &blocked)
                .filter(|path| path.len() >= 3)
                .map(|path| (site.id, path))
        })
        .collect();
    candidates.sort_by_key(|(site, path)| (std::cmp::Reverse(path.len()), *site));

    for (site, path) in candidates {
        let mut trial = campaign.clone();
        let owner_sites: Vec<_> = trial
            .world
            .sites
            .iter()
            .filter(|candidate| {
                candidate.controller == Some(faction)
                    && candidate.id != headquarters
                    && candidate.id != capital
                    && candidate.id != site
            })
            .map(|candidate| candidate.id)
            .collect();
        for old_site in owner_sites {
            trial.set_site_control(data, old_site, None, false).unwrap();
        }
        for connector in path.iter().skip(1).take(path.len() - 2) {
            trial
                .set_site_control(data, *connector, None, false)
                .unwrap();
        }
        trial
            .set_site_control(data, site, Some(faction), false)
            .unwrap();
        for other in trial
            .armies
            .values_mut()
            .filter(|other| other.faction == faction && other.id != army)
        {
            other.site = headquarters;
        }
        trial.armies.get_mut(&army).unwrap().site = site;
        if trial.supplied_sites(faction).contains(&site) {
            continue;
        }
        let view = project(&trial, faction).unwrap();
        let routes = kestrum::engine::ai::ObservedRoutes::new(&view, data, &BTreeSet::new());
        let mut destinations: Vec<_> = view
            .supplied_sites
            .iter()
            .filter_map(|destination| {
                routes
                    .path(site, *destination, false)
                    .filter(|(_, path)| path.len() > 1)
                    .map(|(cost, path)| (cost, *destination, path))
            })
            .collect();
        destinations.sort();
        if let Some((_, _, path)) = destinations.first() {
            trial.validate(data).unwrap();
            *campaign = trial;
            return (site, path[1]);
        }
    }
    panic!("the authored map needs an owned site isolated by a neutral gap");
}

fn topology_path(
    world: &kestrum::state::world::CampaignWorld,
    origin: SiteId,
    destination: SiteId,
    blocked: &BTreeSet<SiteId>,
) -> Option<Vec<SiteId>> {
    let mut parents = BTreeMap::from([(origin, None)]);
    let mut pending = VecDeque::from([origin]);
    while let Some(site) = pending.pop_front() {
        if site == destination {
            let mut path = vec![site];
            let mut cursor = site;
            while let Some(Some(parent)) = parents.get(&cursor) {
                path.push(*parent);
                cursor = *parent;
            }
            path.reverse();
            return Some(path);
        }
        for next in world.adjacent_sites(site) {
            if !blocked.contains(&next) && !parents.contains_key(&next) {
                parents.insert(next, Some(site));
                pending.push_back(next);
            }
        }
    }
    None
}

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), OAK);
    (data, campaign)
}

fn no_resources(campaign: &mut StrategicCampaign) {
    campaign.factions.get_mut(&OAK).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    for site in campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(OAK))
    {
        campaign
            .world
            .focus
            .insert(site.id, kestrum::state::construction::Focus::Gold);
    }
}

fn set_peace_with_everyone(campaign: &mut StrategicCampaign, faction: FactionId) {
    let others: Vec<_> = campaign
        .relations
        .iter()
        .filter(|relation| relation.factions.contains(&faction))
        .flat_map(|relation| relation.factions)
        .filter(|other| *other != faction)
        .collect();
    for other in others {
        set_relation(campaign, faction, other, DiplomaticState::Peace);
    }
}

fn set_relation(
    campaign: &mut StrategicCampaign,
    left: FactionId,
    right: FactionId,
    state: DiplomaticState,
) {
    let factions = if left.0 < right.0 {
        [left, right]
    } else {
        [right, left]
    };
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == factions)
        .unwrap()
        .state = state;
    let round = campaign.completed_rounds;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .unwrap();
    pair.peace_since = (state == DiplomaticState::Peace).then_some(round);
    pair.truce_until = None;
}
