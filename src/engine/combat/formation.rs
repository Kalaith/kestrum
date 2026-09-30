//! Campaign adapter for the deterministic formation resolver and its receipt.

use super::*;
use crate::{
    data::{
        battle_tactics::{
            TacticAction, TacticCondition, TacticRule, TacticTrigger, TargetFilter, TargetPriority,
        },
        economy::{Resources, TroopKind},
    },
    state::{
        battle::simulation::{
            BattleArmyInput, BattleEvent, BattleLeaderSnapshot, BattleSide, BattleUnitId,
            BattleUnitInput, FormationBattleInput,
        },
        campaign::DomainFactKind,
        evidence::MovementService,
        people::PersonAssignment,
        siege::SiegeChange,
        threat::{ThreatId, ThreatStatus},
    },
};

pub(in crate::engine) fn prepare_encounter(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    encounter: Encounter,
) -> Result<BattleId, RuleError> {
    let report = super::new_report(campaign, data, &encounter)?;
    if !retreat::hostile(
        campaign,
        report.attacker.faction,
        report.defender.faction().expect("faction encounter"),
    ) {
        return Err(RuleError::InvalidState(
            "Encounter participants are not hostile.".into(),
        ));
    }
    prepare(campaign, data, report, None)
}

pub(in crate::engine) fn prepare_threat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    attackers: &[ArmyId],
    origin: SiteId,
    id: ThreatId,
    movement: Option<MovementService>,
) -> Result<BattleId, RuleError> {
    let threat = campaign
        .threats
        .get(&id)
        .filter(|threat| threat.status == ThreatStatus::Active)
        .ok_or_else(|| RuleError::InvalidState("That threat is no longer active.".into()))?;
    let definition = &data.threats.definitions[&threat.kind];
    let site = campaign
        .world
        .site(threat.site)
        .expect("validated threat site");
    let report = BattleReport {
        simulation: None,
        context: BattleContext::Field,
        wall_permille: 1000,
        fort_damage_added: 0,
        road_damage: None,
        id: campaign.next_ids.battle,
        completed_rounds: campaign.completed_rounds,
        sequence: campaign.accepted_sequence,
        site: site.id,
        site_name: site.name.clone(),
        origin,
        outcome: BattleOutcome::Stalemate,
        reason: BattleEndReason::ExchangeLimit,
        exchanges: Vec::new(),
        attacker: super::snapshot(campaign, data, attackers)?,
        defender: BattleDefender::Threat(ThreatSideReport {
            id,
            kind: threat.kind,
            name: definition.name.clone(),
            start: threat.headcount,
            end: threat.headcount,
            combat_losses: 0,
            encirclement_losses: 0,
            attack: definition.attack,
            resistance: definition.resistance,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        }),
        terrain_permille: data.combat.terrain(site),
        counters: Vec::new(),
        control_before: site.controller,
        control_after: site.controller,
        structural_damage_added: 0,
        occupation_after: campaign
            .world
            .occupation
            .get(&site.id)
            .copied()
            .unwrap_or(0),
        person_events: Vec::new(),
    };
    prepare(campaign, data, report, movement)
}

fn prepare(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    mut report: BattleReport,
    movement: Option<MovementService>,
) -> Result<BattleId, RuleError> {
    if campaign.pending_battle.is_some() {
        return Err(RuleError::BattlePending);
    }
    let id = report.id;
    campaign.next_ids.battle = BattleId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "battle identifiers",
    })?);
    let applied_wall = assault_wall(campaign, data, &report);
    let input = battle_input(campaign, data, &report)?;
    let resolution = crate::engine::resolve_battle(&input, &data.battle_tactics)
        .map_err(RuleError::InvalidState)?;
    if resolution.outcome == BattleOutcome::AttackerVictory {
        if let BattleDefender::Threat(threat) = &report.defender {
            let mut projected_resources = campaign.factions[&report.attacker.faction].resources;
            add_resources(
                &mut projected_resources,
                data.threats.definitions[&threat.kind].reward,
            )?;
        }
    }
    report.outcome = resolution.outcome;
    report.reason = match resolution.reason {
        crate::state::battle::simulation::BattleResolutionReason::Annihilation => {
            BattleEndReason::Annihilation
        }
        crate::state::battle::simulation::BattleResolutionReason::Rout => BattleEndReason::Rout,
        crate::state::battle::simulation::BattleResolutionReason::RoundLimit => {
            BattleEndReason::ExchangeLimit
        }
    };
    report.exchanges = resolution_exchanges(&report, &resolution, applied_wall);
    report.simulation = Some(resolution);
    campaign.pending_battle = Some(crate::state::battle::PendingBattle {
        started_by: report.attacker.faction,
        report,
        movement,
    });
    Ok(id)
}

fn battle_input(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Result<FormationBattleInput, RuleError> {
    let assault_wall = assault_wall(campaign, data, report);
    let mut armies = Vec::new();
    for side in std::iter::once((&report.attacker, BattleSide::Attacker)).chain(
        report
            .defender
            .faction_side()
            .map(|side| (side, BattleSide::Defender)),
    ) {
        for army in &side.0.armies {
            let source = campaign
                .armies
                .get(&army.id)
                .ok_or(RuleError::UnknownArmy { army: army.id })?;
            let mut slots = std::array::from_fn(|_| None);
            for formation in &army.formations {
                let actual =
                    campaign
                        .formations
                        .get(&formation.id)
                        .ok_or(RuleError::UnknownFormation {
                            formation: formation.id,
                        })?;
                let tactics = actual
                    .tactics
                    .as_ref()
                    .or_else(|| data.battle_tactics.defaults_for(formation.kind))
                    .ok_or_else(|| RuleError::InvalidState("Missing troop tactics.".into()))?;
                let stats = &data.troops.formations[&formation.kind];
                let attack = u128::from(stats.attack)
                    * u128::from(army.leadership_permille)
                    * u128::from(formation.veterancy_permille)
                    / 1_000_000;
                let mut resistance = super::arithmetic::guarded_resistance(
                    campaign,
                    data,
                    formation.id,
                    formation.kind,
                    report.site,
                    side.1 == BattleSide::Defender,
                );
                if side.1 == BattleSide::Defender
                    && matches!(report.context, BattleContext::Assault { .. })
                {
                    resistance = (u128::from(resistance) * u128::from(assault_wall) / 1000)
                        .clamp(1, u128::from(u32::MAX)) as u32;
                }
                let leader = actual.battle_leader.and_then(|id| {
                    campaign.people.get(&id).map(|person| BattleLeaderSnapshot {
                        id,
                        name: person.name.clone(),
                        class: person.class,
                        active: person.faction == actual.faction
                            && person.assignment
                                == (PersonAssignment::Formation {
                                    formation: actual.id,
                                })
                            && !person.career.retired
                            && person.is_fit_for_field(
                                campaign.completed_rounds,
                                data.rules.leadership.field_min_age_years,
                            )
                            && (person.age_years(campaign.completed_rounds)
                                < data.lifecycle.elder_age_years
                                || source.commander == Some(id)),
                    })
                });
                slots[formation.slot] = Some(BattleUnitInput {
                    id: BattleUnitId::Formation(formation.id),
                    kind: Some(formation.kind),
                    headcount: actual.headcount,
                    capacity: actual.capacity,
                    attack: attack.clamp(1, u128::from(u32::MAX)) as u32,
                    resistance,
                    initiative: initiative(formation.kind),
                    capabilities: crate::data::battle_tactics::leader_capabilities(
                        formation.kind,
                        leader
                            .as_ref()
                            .filter(|leader| leader.active)
                            .map(|leader| leader.class),
                    ),
                    leader,
                    activation_tactics: tactics.activation.clone(),
                    reaction_tactics: tactics.reaction.clone(),
                });
            }
            armies.push(BattleArmyInput {
                id: source.id,
                faction: side.0.faction,
                name: source.name.clone(),
                side: side.1,
                slots,
            });
        }
    }
    if let BattleDefender::Threat(threat) = &report.defender {
        let virtual_army = ArmyId(u32::MAX);
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Some(BattleUnitInput {
            id: BattleUnitId::Threat(threat.id),
            kind: None,
            headcount: threat.start,
            capacity: threat.start,
            attack: threat.attack,
            resistance: threat.resistance,
            initiative: initiative_for_threat(),
            leader: None,
            capabilities: Vec::new(),
            activation_tactics: vec![TacticRule {
                id: "threat_attack".into(),
                trigger: TacticTrigger::Activation,
                action: TacticAction::Attack,
                condition: TacticCondition::Always,
                target_filter: TargetFilter::AnyEnemy,
                target_priority: TargetPriority::OwnColumnFirst,
            }],
            reaction_tactics: vec![TacticRule {
                id: "threat_guard".into(),
                trigger: TacticTrigger::IncomingAttack,
                action: TacticAction::Guard,
                condition: TacticCondition::Always,
                target_filter: TargetFilter::None,
                target_priority: TargetPriority::OwnColumnFirst,
            }],
        });
        armies.push(BattleArmyInput {
            id: virtual_army,
            faction: FactionId(u32::MAX),
            name: threat.name.clone(),
            side: BattleSide::Defender,
            slots,
        });
    }
    Ok(FormationBattleInput {
        terrain_permille: report.terrain_permille,
        armies,
    })
}

pub(in crate::engine) fn refresh_pending(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), RuleError> {
    let mut report = campaign
        .pending_battle
        .as_ref()
        .ok_or(RuleError::NoPendingBattle)?
        .report
        .clone();
    let applied_wall = assault_wall(campaign, data, &report);
    let input = battle_input(campaign, data, &report)?;
    let resolution = crate::engine::resolve_battle(&input, &data.battle_tactics)
        .map_err(RuleError::InvalidState)?;
    report.outcome = resolution.outcome;
    report.reason = match resolution.reason {
        crate::state::battle::simulation::BattleResolutionReason::Annihilation => {
            BattleEndReason::Annihilation
        }
        crate::state::battle::simulation::BattleResolutionReason::Rout => BattleEndReason::Rout,
        crate::state::battle::simulation::BattleResolutionReason::RoundLimit => {
            BattleEndReason::ExchangeLimit
        }
    };
    report.exchanges = resolution_exchanges(&report, &resolution, applied_wall);
    report.simulation = Some(resolution);
    campaign
        .pending_battle
        .as_mut()
        .expect("pending report was just read")
        .report = report;
    Ok(())
}

fn assault_wall(campaign: &StrategicCampaign, data: &GameData, report: &BattleReport) -> u32 {
    if !matches!(report.context, BattleContext::Assault { .. }) {
        return 1000;
    }
    let engines = report
        .attacker
        .armies
        .iter()
        .flat_map(|army| &army.formations)
        .any(|unit| {
            campaign.formations[&unit.id].headcount > 0 && unit.kind == TroopKind::SiegeEngines
        });
    if engines {
        report
            .wall_permille
            .saturating_sub(data.siege.engine_wall_reduction_permille)
            .max(data.siege.wall_minimum_permille)
    } else {
        report.wall_permille
    }
}

fn initiative(kind: TroopKind) -> u32 {
    match kind {
        TroopKind::Riders => 60,
        TroopKind::Archers => 50,
        TroopKind::Warriors => 40,
        TroopKind::Spearmen => 35,
        TroopKind::Medics => 30,
        TroopKind::SiegeEngines => 20,
    }
}

fn initiative_for_threat() -> u32 {
    25
}

fn resolution_exchanges(
    report: &BattleReport,
    resolution: &crate::state::battle::simulation::BattleResolution,
    applied_wall: u32,
) -> Vec<BattleExchange> {
    let mut exchanges = BTreeMap::<u32, BTreeMap<FormationId, u32>>::new();
    let mut threat_losses = BTreeMap::<u32, u32>::new();
    let mut round = 1;
    for event in &resolution.events {
        match event {
            BattleEvent::RoundStarted { round: next } => round = *next,
            BattleEvent::Damage {
                target: BattleUnitId::Formation(id),
                amount,
                ..
            } => *exchanges.entry(round).or_default().entry(*id).or_default() += amount,
            BattleEvent::Damage {
                target: BattleUnitId::Threat(_),
                amount,
                ..
            } => *threat_losses.entry(round).or_default() += amount,
            _ => {}
        }
    }
    let rounds: BTreeSet<_> = exchanges
        .keys()
        .chain(threat_losses.keys())
        .copied()
        .collect();
    rounds
        .into_iter()
        .map(|number| BattleExchange {
            threat_losses: threat_losses.get(&number).copied().unwrap_or_default(),
            wall_permille: applied_wall,
            number,
            losses: exchanges
                .remove(&number)
                .unwrap_or_default()
                .into_iter()
                .map(|(formation, amount)| FormationLoss { formation, amount })
                .collect(),
            leadership: {
                let mut leadership: Vec<_> = report
                    .faction_sides()
                    .flat_map(|side| &side.armies)
                    .map(|army| ArmyLeadership {
                        army: army.id,
                        permille: army.leadership_permille,
                    })
                    .collect();
                leadership.sort_by_key(|entry| entry.army);
                leadership
            },
        })
        .collect()
}

pub(in crate::engine) fn commit_pending(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut super::super::ActionOutcome,
) -> Result<(), RuleError> {
    let pending = campaign
        .pending_battle
        .take()
        .ok_or(RuleError::NoPendingBattle)?;
    let mut report = pending.report;
    report.sequence = campaign.accepted_sequence;
    let besieger_clash = if matches!(report.context, BattleContext::BesiegerClash { .. }) {
        campaign.sieges.get(&report.site).cloned()
    } else {
        None
    };
    let resolution = report
        .simulation
        .as_ref()
        .ok_or_else(|| RuleError::InvalidState("Pending battle lost its resolution.".into()))?;
    let mut people = PersonCombatContext {
        site: report.site,
        sides: vec![person_side(campaign, data, &report.attacker)],
    };
    if let Some(defenders) = report.defender.faction_side() {
        people.sides.push(person_side(campaign, data, defenders));
    }
    apply_resolution(campaign, &report, resolution)?;
    record_losses(campaign, data, &mut report);
    let battle_outcome = report.outcome;
    let attacker_faction = report.attacker.faction;
    let completed_rounds = report.completed_rounds;
    if let BattleDefender::Threat(threat) = &mut report.defender {
        finish_threat(
            campaign,
            data,
            battle_outcome,
            attacker_faction,
            completed_rounds,
            threat,
        )?;
    }
    if matches!(report.defender, BattleDefender::Threat(_)) {
        if matches!(
            report.outcome,
            BattleOutcome::DefenderVictory | BattleOutcome::Stalemate
        ) {
            super::withdraw(
                campaign,
                &mut report.attacker,
                report.site,
                Some(report.origin),
                None,
            );
        }
    } else {
        context::withdrawals(campaign, &mut report);
    }
    record_survivors(campaign, &mut report);
    people.sides[0].refuges =
        retreat::refuges(campaign, report.attacker.faction, report.site, None);
    if let Some(defender) = report.defender.faction_side() {
        people.sides[1].refuges =
            retreat::refuges(campaign, defender.faction, report.site, Some(report.origin));
    }
    report.person_events = resolve_person_combat(campaign, data, &people)?;
    cleanup(campaign, data);
    finish_side(campaign, &mut report.attacker);
    if let Some(defender) = report.defender.faction_side_mut() {
        finish_side(campaign, defender);
    }
    context::site_result(campaign, data, &mut report);
    super::super::knowledge::observe_battle(&mut campaign.knowledge, &report);
    let id = report.id;
    campaign.battles.insert(id, report);
    super::super::actions::record_fact(
        campaign,
        outcome,
        DomainFactKind::BattleResolved {
            battle: id,
            movement: pending.movement,
        },
    )?;
    if let Some(siege) = besieger_clash.filter(|_| battle_outcome == BattleOutcome::AttackerVictory)
    {
        campaign.sieges.remove(&siege.site);
        campaign.world.contested_sites.remove(&siege.site);
        super::super::actions::record_fact(
            campaign,
            outcome,
            DomainFactKind::SiegeChanged {
                siege: siege.clone(),
                change: SiegeChange::Lifted,
            },
        )?;
        super::super::siege::establish(
            campaign,
            data,
            siege.site,
            siege.defender,
            attacker_faction,
            outcome,
        )?;
    }
    outcome.battle = Some(id);
    Ok(())
}

fn apply_resolution(
    campaign: &mut StrategicCampaign,
    report: &BattleReport,
    resolution: &crate::state::battle::simulation::BattleResolution,
) -> Result<(), RuleError> {
    for unit in &resolution.units {
        match unit.id {
            BattleUnitId::Formation(id) => {
                let formation = campaign
                    .formations
                    .get_mut(&id)
                    .ok_or(RuleError::UnknownFormation { formation: id })?;
                if formation.headcount != opening_count(report, id) {
                    return Err(RuleError::InvalidState(
                        "A pending participant changed before battle acceptance.".into(),
                    ));
                }
                formation.headcount = unit.headcount;
            }
            BattleUnitId::Threat(id) => {
                let threat = campaign
                    .threats
                    .get_mut(&id)
                    .ok_or_else(|| RuleError::InvalidState("Unknown pending threat.".into()))?;
                threat.headcount = unit.headcount;
            }
        }
    }
    Ok(())
}

fn opening_count(report: &BattleReport, id: FormationId) -> u32 {
    report
        .faction_sides()
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.formations)
        .find(|formation| formation.id == id)
        .map_or(0, |formation| formation.start)
}

fn finish_threat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: BattleOutcome,
    attacker: FactionId,
    completed_rounds: u32,
    side: &mut ThreatSideReport,
) -> Result<(), RuleError> {
    let threat = campaign
        .threats
        .get_mut(&side.id)
        .ok_or_else(|| RuleError::InvalidState("Pending threat disappeared.".into()))?;
    side.combat_losses = side.start - threat.headcount;
    let no_reward = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    if outcome == BattleOutcome::AttackerVictory {
        side.encirclement_losses = threat.headcount;
        threat.headcount = 0;
        side.payout = data.threats.definitions[&side.kind].reward;
        add_resources(
            &mut campaign
                .factions
                .get_mut(&attacker)
                .expect("attacker")
                .resources,
            side.payout,
        )?;
    } else if outcome == BattleOutcome::MutualDestruction {
        side.encirclement_losses = 0;
        side.payout = no_reward;
    } else {
        side.payout = no_reward;
    }
    side.end = threat.headcount;
    if threat.headcount == 0 {
        threat.status = ThreatStatus::Cleared {
            round: completed_rounds,
            by: attacker,
            payout: side.payout,
        };
    }
    Ok(())
}

fn add_resources(resources: &mut Resources, payout: Resources) -> Result<(), RuleError> {
    resources.gold = resources
        .gold
        .checked_add(payout.gold)
        .ok_or(RuleError::Overflow {
            field: "threat Gold reward",
        })?;
    resources.wood = resources
        .wood
        .checked_add(payout.wood)
        .ok_or(RuleError::Overflow {
            field: "threat Wood reward",
        })?;
    resources.stone = resources
        .stone
        .checked_add(payout.stone)
        .ok_or(RuleError::Overflow {
            field: "threat Stone reward",
        })?;
    Ok(())
}

fn record_survivors(campaign: &StrategicCampaign, report: &mut BattleReport) {
    for side in std::iter::once(&mut report.attacker).chain(report.defender.faction_side_mut()) {
        for army in &mut side.armies {
            for formation in &mut army.formations {
                let actual = &campaign.formations[&formation.id];
                formation.encirclement_losses = formation
                    .start
                    .saturating_sub(formation.combat_losses)
                    .saturating_sub(actual.headcount);
                formation.end = actual.headcount;
            }
        }
    }
}
