//! Seasonal income followed by full-formation upkeep, with no debt or attrition.

use super::RuleError;
use crate::{
    data::{economy::Resources, world::FactionId, GameData},
    state::{military::EconomyStatement, StrategicCampaign},
};

pub(super) fn resolve(campaign: &mut StrategicCampaign, data: &GameData) -> Result<(), RuleError> {
    let completed_rounds = campaign
        .completed_rounds
        .checked_add(1)
        .ok_or(RuleError::Overflow {
            field: "completed rounds",
        })?;
    let statements = campaign
        .factions
        .keys()
        .map(|&faction| {
            statement(campaign, data, faction, completed_rounds)
                .map(|statement| (faction, statement))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Calculate every faction first. Applying income cannot affect another site's eligibility.
    for (id, statement) in statements {
        let faction = campaign.factions.get_mut(&id).expect("existing faction");
        faction.resources = statement.closing;
        faction.deficit = statement.shortfall > 0;
        faction.last_economy = Some(statement);
    }
    Ok(())
}

fn statement(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    completed_rounds: u32,
) -> Result<EconomyStatement, RuleError> {
    let owner = &campaign.factions[&faction];
    let mut income = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    if campaign.is_independent(faction) {
        for site in &campaign.world.sites {
            if !campaign.world.is_secure(site.id, faction) {
                continue;
            }
            let base = data.economy.settlement_income[&site.habitation];
            // P19 applies one final floor per resource; the role bonus stays unmodified.
            let percent = 100
                - campaign.world.structural_damage(site.id) / data.economy.income_damage_divisor;
            let amount = Resources {
                gold: scaled(base.gold, percent)?,
                wood: scaled(base.wood, percent)?,
                stone: scaled(base.stone, percent)?,
            };
            income = checked_add(income, amount)?;
        }
        if campaign.world.is_secure(owner.headquarters, faction) {
            income = checked_add(income, data.economy.headquarters_income_bonus)?;
        }
    }
    let upkeep_due = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction && formation.headcount > 0)
        .try_fold(0_i64, |total, formation| {
            total
                .checked_add(data.economy.formations[&formation.kind].upkeep_gold)
                .ok_or(RuleError::Overflow {
                    field: "formation upkeep",
                })
        })?;
    let mut closing = checked_add(owner.resources, income)?;
    let upkeep_paid = closing.gold.min(upkeep_due);
    closing.gold -= upkeep_paid;
    Ok(EconomyStatement {
        completed_rounds,
        income,
        upkeep_due,
        upkeep_paid,
        shortfall: upkeep_due - upkeep_paid,
        closing,
    })
}

fn scaled(amount: i64, percent: u32) -> Result<i64, RuleError> {
    i64::try_from(i128::from(amount) * i128::from(percent) / 100).map_err(|_| RuleError::Overflow {
        field: "settlement income",
    })
}

fn checked_add(left: Resources, right: Resources) -> Result<Resources, RuleError> {
    let add = |a: i64, b: i64| {
        a.checked_add(b).ok_or(RuleError::Overflow {
            field: "resource balance",
        })
    };
    Ok(Resources {
        gold: add(left.gold, right.gold)?,
        wood: add(left.wood, right.wood)?,
        stone: add(left.stone, right.stone)?,
    })
}
