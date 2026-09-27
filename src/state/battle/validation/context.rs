//! Saved contexts explain exceptions without reading a changed live garrison.

use super::*;

pub(super) fn validate_context(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Result<(), String> {
    let same_site = matches!(
        report.context,
        BattleContext::Assault { .. } | BattleContext::Sortie { .. } | BattleContext::Escape { .. }
    );
    ensure(
        if same_site {
            report.origin == report.site
        } else {
            campaign
                .world
                .connected_route(report.origin, report.site)
                .is_some()
        },
        "origin disagrees with encounter context",
    )?;
    let siege = match &report.context {
        BattleContext::Field => None,
        BattleContext::Assault { siege } | BattleContext::Sortie { siege } => Some(*siege),
        BattleContext::Escape { siege, destination } => {
            ensure(
                campaign
                    .world
                    .connected_route(report.site, *destination)
                    .is_some(),
                "escape destination is not adjacent",
            )?;
            Some(*siege)
        }
        BattleContext::Relief { siege, garrison } => {
            ensure(
                garrison.windows(2).all(|pair| pair[0] < pair[1])
                    && garrison.len() < report.attacker.armies.len()
                    && garrison
                        .iter()
                        .all(|id| report.attacker.armies.iter().any(|army| army.id == *id)),
                "relief has invalid garrison or no incoming army",
            )?;
            Some(*siege)
        }
        BattleContext::BesiegerClash {
            siege,
            garrison_faction,
        } => {
            ensure(
                campaign.factions.contains_key(garrison_faction)
                    && *garrison_faction != report.attacker.faction
                    && *garrison_faction != report.defender.faction
                    && report.control_before == Some(*garrison_faction),
                "besieger clash has invalid garrison faction",
            )?;
            Some(*siege)
        }
    };
    ensure(
        siege.is_none_or(|id| id.0 > 0 && id < campaign.next_ids.siege),
        "invalid historical siege ID",
    )?;
    validate_damage(campaign, data, report)
}

fn validate_damage(
    campaign: &StrategicCampaign,
    data: &GameData,
    report: &BattleReport,
) -> Result<(), String> {
    let assault = matches!(report.context, BattleContext::Assault { .. });
    ensure(
        if assault {
            (data.siege.wall_minimum_permille..=data.siege.wall_base_permille)
                .contains(&report.wall_permille)
                && report.fort_damage_added <= data.siege.assault_fort_damage
        } else {
            report.wall_permille == 1000
                && report.fort_damage_added == 0
                && report.road_damage.is_none()
        },
        "wall or lasting damage disagrees with encounter context",
    )?;
    if let Some(damage) = &report.road_damage {
        ensure(
            campaign
                .world
                .route(damage.route)
                .is_some_and(|route| route.from == report.site || route.to == report.site)
                && damage.added <= data.siege.assault_road_damage,
            "invalid incident road damage",
        )?;
        let (start, lost) = report
            .defender
            .armies
            .iter()
            .flat_map(|army| &army.formations)
            .fold((0_u128, 0_u128), |(start, lost), formation| {
                let attack = u128::from(data.troops.formations[&formation.kind].attack);
                (
                    start + u128::from(formation.start) * attack,
                    lost + (u128::from(formation.combat_losses)
                        + u128::from(formation.encirclement_losses))
                        * attack,
                )
            });
        ensure(
            start > 0 && lost * 1000 >= start * u128::from(data.siege.assault_road_loss_permille),
            "road damage lacks sufficient destroyed defender power",
        )?;
    }
    Ok(())
}

pub(super) fn validate_wall(
    data: &GameData,
    report: &BattleReport,
    exchange: &BattleExchange,
    remaining: &BTreeMap<FormationId, u32>,
) -> Result<(), String> {
    let engines = report
        .attacker
        .armies
        .iter()
        .flat_map(|army| &army.formations)
        .any(|formation| {
            formation.kind == TroopKind::SiegeEngines
                && remaining.get(&formation.id).is_some_and(|count| *count > 0)
        });
    let expected = if matches!(report.context, BattleContext::Assault { .. }) && engines {
        report
            .wall_permille
            .saturating_sub(data.siege.engine_wall_reduction_permille)
            .max(data.siege.wall_minimum_permille)
    } else {
        report.wall_permille
    };
    ensure(
        exchange.wall_permille == expected,
        "exchange wall factor disagrees with surviving siege engines",
    )
}

pub(super) fn validate_outcome(
    campaign: &StrategicCampaign,
    report: &BattleReport,
) -> Result<(), String> {
    let survivors = |side: &BattleSideReport| {
        side.armies
            .iter()
            .flat_map(|army| &army.formations)
            .map(|formation| u64::from(formation.end))
            .sum::<u64>()
    };
    let valid = match report.outcome {
        BattleOutcome::AttackerVictory => survivors(&report.attacker) > 0,
        BattleOutcome::DefenderVictory | BattleOutcome::Stalemate => {
            survivors(&report.defender) > 0
        }
        BattleOutcome::MutualDestruction => {
            survivors(&report.attacker) == 0 && survivors(&report.defender) == 0
        }
    };
    ensure(
        valid && control_matches(report),
        "outcome disagrees with survivors/control",
    )?;
    for (attacking, side) in [(true, &report.attacker), (false, &report.defender)] {
        for army in &side.armies {
            if let Some(site) = army.final_site {
                ensure(
                    destination_matches(campaign, report, army.id, attacking, site),
                    "final army location disagrees with encounter context",
                )?;
            }
        }
    }
    Ok(())
}

fn control_matches(report: &BattleReport) -> bool {
    use BattleOutcome::*;
    let before = report.control_before;
    let after = report.control_after;
    let attacker = Some(report.attacker.faction);
    let defender = Some(report.defender.faction);
    match report.context {
        BattleContext::Field => match report.outcome {
            AttackerVictory => after == attacker,
            DefenderVictory | Stalemate => after == before,
            MutualDestruction => after.is_none(),
        },
        BattleContext::Assault { .. } => {
            before == defender
                && match report.outcome {
                    AttackerVictory => after == attacker,
                    DefenderVictory | Stalemate => after == before,
                    MutualDestruction => after.is_none() || after == attacker,
                }
        }
        BattleContext::BesiegerClash { .. } => after == before,
        BattleContext::Sortie { .. } | BattleContext::Relief { .. } => {
            before == attacker
                && match report.outcome {
                    AttackerVictory => after == before,
                    Stalemate => {
                        after == before
                            || (matches!(report.context, BattleContext::Relief { .. })
                                && after == defender)
                    }
                    DefenderVictory => after == attacker || after == defender,
                    MutualDestruction => after.is_none() || after == attacker,
                }
        }
        BattleContext::Escape { .. } => {
            before == attacker
                && match report.outcome {
                    AttackerVictory => after.is_none() || after == attacker || after == defender,
                    DefenderVictory => after == attacker || after == defender,
                    Stalemate => after == before,
                    MutualDestruction => after.is_none() || after == attacker,
                }
        }
    }
}

fn destination_matches(
    campaign: &StrategicCampaign,
    report: &BattleReport,
    army: ArmyId,
    attacking: bool,
    site: SiteId,
) -> bool {
    let won = report.outcome == BattleOutcome::AttackerVictory;
    let retreat = |exclude_origin: bool| {
        campaign.world.connected_route(report.site, site).is_some()
            && (!exclude_origin || site != report.origin)
    };
    match &report.context {
        BattleContext::Field | BattleContext::BesiegerClash { .. } => {
            if (won && !attacking) || (!won && attacking) {
                retreat(!attacking)
            } else {
                site == report.site
            }
        }
        BattleContext::Assault { .. } | BattleContext::Sortie { .. } => {
            if won && !attacking {
                retreat(false)
            } else {
                site == report.site
            }
        }
        BattleContext::Escape { destination, .. } => {
            if won && attacking {
                site == *destination
            } else {
                site == report.site
            }
        }
        BattleContext::Relief { garrison, .. } => {
            if won && !attacking {
                retreat(true)
            } else if !won && attacking && !garrison.contains(&army) {
                retreat(false)
            } else {
                site == report.site
            }
        }
    }
}
