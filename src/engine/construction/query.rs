//! Read-only choices use the command's public constraints and own builder roster.

use super::*;
use crate::engine::{actions::validate_command, Actor, Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionOption {
    pub kind: ConstructionKind,
    pub cost: Resources,
    pub steps: u32,
    pub builders: Vec<ArmyId>,
    pub blocked: Option<String>,
}

pub fn construction_options(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    target: ConstructionTarget,
) -> Vec<ConstructionOption> {
    let kinds = match target {
        ConstructionTarget::Site(_) => [ConstructionKind::Outpost, ConstructionKind::Fort]
            .into_iter()
            .chain(
                data.construction
                    .facilities
                    .keys()
                    .copied()
                    .map(ConstructionKind::Facility),
            )
            .collect(),
        ConstructionTarget::Route(_) => vec![ConstructionKind::Road, ConstructionKind::RoadRepair],
    };
    let actor = if observer == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(observer)
    };
    let local: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| {
            army.faction == observer
                && sites(campaign, target).is_ok_and(|sites| sites.contains(&army.site))
        })
        .map(|army| army.id)
        .collect();
    kinds
        .into_iter()
        .map(|kind| {
            let (cost, steps) = terms(data, kind);
            let attempts: Vec<_> = local
                .iter()
                .map(|builder| {
                    (
                        *builder,
                        validate_command(
                            campaign,
                            actor,
                            &Command::StartConstruction {
                                target,
                                kind,
                                builder: *builder,
                            },
                        )
                        .and_then(|()| {
                            commands::validate_start(
                                campaign, data, observer, target, kind, *builder,
                            )
                        }),
                    )
                })
                .collect();
            let builders = attempts
                .iter()
                .filter_map(|(id, result)| result.is_ok().then_some(*id))
                .collect::<Vec<_>>();
            let blocked = if builders.is_empty() {
                Some(
                    attempts
                        .iter()
                        .find_map(|(_, result)| result.as_ref().err())
                        .map(ToString::to_string)
                        .unwrap_or_else(|| ConstructionBlock::BuilderMissing.to_string()),
                )
            } else {
                None
            };
            ConstructionOption {
                kind,
                cost,
                steps,
                builders,
                blocked,
            }
        })
        .collect()
}

pub fn construction_refund(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    id: OrderId,
) -> Result<Resources, RuleError> {
    let order = owned_order(campaign, observer, id)?;
    let percent = if order.progress == 0 {
        data.economy.refunds.unstarted_construction_percent
    } else {
        data.economy.refunds.started_construction_percent
    };
    let portion = |amount: i64| -> Result<i64, RuleError> {
        (i128::from(amount) * i128::from(percent) / 100)
            .try_into()
            .map_err(|_| RuleError::Overflow {
                field: "construction refund",
            })
    };
    Ok(Resources {
        gold: portion(order.paid.gold)?,
        wood: portion(order.paid.wood)?,
        stone: portion(order.paid.stone)?,
    })
}
