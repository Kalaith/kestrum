//! Seasonal income followed by full-formation upkeep, with no debt or attrition.

use super::RuleError;
use crate::{
    data::{
        economy::Resources,
        world::{FactionId, Site},
        GameData,
    },
    state::{
        military::{EconomyStatement, SettlementIncomeStatement, SiteIncomeStatement},
        StrategicCampaign,
    },
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
    let mut inputs = SettlementIncomeStatement {
        sites: Vec::new(),
        headquarters_bonus: Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
    };
    if campaign.is_independent(faction) {
        for site in &campaign.world.sites {
            if site.controller != Some(faction) {
                continue;
            }
            let entry = site_income(campaign, data, site)?;
            income = checked_add(income, entry.income)?;
            inputs.sites.push(entry);
        }
        if campaign.world.is_secure(owner.headquarters, faction)
            && !campaign.site_is_ruined(owner.headquarters)
            && campaign.active_threat(owner.headquarters).is_none()
        {
            inputs.headquarters_bonus = data.economy.headquarters_income_bonus;
            income = checked_add(income, inputs.headquarters_bonus)?;
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
        settlement_inputs: Some(inputs),
        completed_rounds,
        income,
        upkeep_due,
        upkeep_paid,
        shortfall: upkeep_due - upkeep_paid,
        closing,
    })
}

fn site_income(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: &Site,
) -> Result<SiteIncomeStatement, RuleError> {
    let occupation = campaign
        .world
        .occupation
        .get(&site.id)
        .copied()
        .unwrap_or(0);
    let damage = campaign.world.structural_damage(site.id);
    let mut entry = SiteIncomeStatement {
        site: site.id,
        habitation: site.habitation,
        base: data.economy.settlement_income[&site.habitation],
        structural_damage: damage,
        occupation,
        focus: campaign
            .world
            .focus
            .get(&site.id)
            .copied()
            .filter(|focus| super::development::focus_suitable(site, *focus)),
        damage_percent: 100 - damage / data.economy.income_damage_divisor,
        occupation_percent: if occupation >= data.development.conditions.occupation_threshold {
            data.development.occupation_income_percent
        } else {
            100
        },
        focus_bonus_percent: data.development.income_focus_bonus_percent,
        ruined: campaign.site_is_ruined(site.id),
        besieged: campaign.world.contested_sites.contains(&site.id),
        local_threat: campaign.active_threat(site.id).is_some(),
        income: Resources {
            gold: 0,
            wood: 0,
            stone: 0,
        },
    };
    entry.income = entry.calculated().ok_or(RuleError::Overflow {
        field: "settlement income",
    })?;
    Ok(entry)
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
