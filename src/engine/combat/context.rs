//! Siege exceptions change survivor destinations, never restore destroyed troops.

use super::*;

pub(super) fn withdrawals(campaign: &mut StrategicCampaign, report: &mut BattleReport) {
    let won = report.outcome == BattleOutcome::AttackerVictory;
    match &report.context {
        BattleContext::Field => match report.outcome {
            BattleOutcome::AttackerVictory => withdraw(
                campaign,
                &mut report.defender,
                report.site,
                None,
                Some(report.origin),
            ),
            BattleOutcome::DefenderVictory | BattleOutcome::Stalemate => withdraw(
                campaign,
                &mut report.attacker,
                report.site,
                Some(report.origin),
                None,
            ),
            BattleOutcome::MutualDestruction => {}
        },
        BattleContext::Assault { .. } | BattleContext::Sortie { .. } if won => {
            withdraw(campaign, &mut report.defender, report.site, None, None);
        }
        BattleContext::Escape { destination, .. } if won => {
            for army in &report.attacker.armies {
                campaign.armies.get_mut(&army.id).expect("participant").site = *destination;
            }
        }
        BattleContext::Relief { garrison, .. } => {
            if won {
                withdraw(
                    campaign,
                    &mut report.defender,
                    report.site,
                    None,
                    Some(report.origin),
                );
            } else {
                let mut incoming = report.attacker.clone();
                incoming.armies.retain(|army| !garrison.contains(&army.id));
                withdraw(
                    campaign,
                    &mut incoming,
                    report.site,
                    Some(report.origin),
                    None,
                );
                for army in incoming.armies {
                    let id = army.id;
                    let target = report
                        .attacker
                        .armies
                        .iter_mut()
                        .find(|entry| entry.id == id)
                        .expect("incoming");
                    *target = army;
                }
            }
        }
        BattleContext::BesiegerClash { .. } => {
            if won {
                withdraw(
                    campaign,
                    &mut report.defender,
                    report.site,
                    None,
                    Some(report.origin),
                );
            } else {
                withdraw(
                    campaign,
                    &mut report.attacker,
                    report.site,
                    Some(report.origin),
                    None,
                );
            }
        }
        _ => {}
    }
}

pub(super) fn site_result(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    report: &mut BattleReport,
) {
    if report.context == BattleContext::Field {
        return apply_site_result(campaign, data, report);
    }
    let before = campaign.world.structural_damage(report.site);
    let assault = matches!(report.context, BattleContext::Assault { .. });
    let added = if assault {
        data.siege.assault_structural_damage
    } else {
        data.combat.field_damage
    };
    if campaign.world.site(report.site).expect("site").habitation != Habitation::Unsettled {
        damage(campaign, report.site, added);
    }
    if assault {
        assault_damage(campaign, data, report);
    }
    let siege = campaign
        .sieges
        .get(&report.site)
        .cloned()
        .expect("siege context");
    let garrison = super::super::siege::occupants(campaign, report.site, siege.defender);
    let outside = super::super::siege::occupants(campaign, report.site, siege.besieger);
    let controller = if matches!(report.context, BattleContext::Assault { .. })
        && report.outcome == BattleOutcome::AttackerVictory
    {
        Some(siege.besieger)
    } else if garrison.is_empty() {
        if !outside.is_empty() {
            Some(siege.besieger)
        } else if matches!(report.context, BattleContext::BesiegerClash { .. }) {
            report.control_before
        } else {
            None
        }
    } else {
        Some(siege.defender)
    };
    if let Some(owner) = controller {
        capture(campaign, data, report.site, owner);
    } else {
        campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == report.site)
            .expect("site")
            .controller = None;
        campaign.reconcile_region_control();
    }
    report.control_after = controller;
    report.structural_damage_added = campaign.world.structural_damage(report.site) - before;
    report.occupation_after = campaign
        .world
        .occupation
        .get(&report.site)
        .copied()
        .unwrap_or(0);
}

fn assault_damage(campaign: &mut StrategicCampaign, data: &GameData, report: &mut BattleReport) {
    let damage = campaign.world.fort_damage.entry(report.site).or_default();
    let before = *damage;
    *damage = damage
        .saturating_add(data.siege.assault_fort_damage)
        .min(100);
    report.fort_damage_added = *damage - before;
    let (start, lost) = report
        .defender
        .armies
        .iter()
        .flat_map(|army| &army.formations)
        .fold((0_u128, 0_u128), |(start, lost), formation| {
            let attack = u128::from(data.troops.formations[&formation.kind].attack);
            (
                start + u128::from(formation.start) * attack,
                lost + u128::from(formation.combat_losses + formation.encirclement_losses) * attack,
            )
        });
    if lost * 1000 < start * u128::from(data.siege.assault_road_loss_permille) {
        return;
    }
    if let Some(route) = campaign
        .world
        .routes
        .iter_mut()
        .filter(|route| {
            (route.from == report.site || route.to == report.site) && route.road.improved
        })
        .min_by_key(|route| route.id)
    {
        let before = route.road.damage;
        route.road.damage = route
            .road
            .damage
            .saturating_add(data.siege.assault_road_damage)
            .min(100);
        report.road_damage = Some(BattleRoadDamage {
            route: route.id,
            added: route.road.damage - before,
        });
    }
}
