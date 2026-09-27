//! Saved income operands explain exactly one completed seasonal assessment.

use super::*;
use crate::{data::economy::Habitation, state::construction::Focus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettlementIncomeStatement {
    pub sites: Vec<SiteIncomeStatement>,
    pub headquarters_bonus: Resources,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteIncomeStatement {
    pub site: SiteId,
    pub habitation: Habitation,
    pub base: Resources,
    pub structural_damage: u32,
    pub occupation: u32,
    pub focus: Option<Focus>,
    pub damage_percent: u32,
    pub occupation_percent: u32,
    pub focus_bonus_percent: u32,
    pub ruined: bool,
    pub besieged: bool,
    pub local_threat: bool,
    pub income: Resources,
}

impl SiteIncomeStatement {
    pub(crate) fn calculated(&self) -> Option<Resources> {
        if self.ruined || self.besieged || self.local_threat {
            return Some(Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            });
        }
        let scale = |amount: i64, focus| {
            let multiplier = 100
                + if self.focus == Some(focus) {
                    self.focus_bonus_percent
                } else {
                    0
                };
            i64::try_from(
                i128::from(amount)
                    * i128::from(self.damage_percent)
                    * i128::from(self.occupation_percent)
                    * i128::from(multiplier)
                    / 1_000_000,
            )
            .ok()
        };
        Some(Resources {
            gold: scale(self.base.gold, Focus::Gold)?,
            wood: scale(self.base.wood, Focus::Wood)?,
            stone: scale(self.base.stone, Focus::Stone)?,
        })
    }
}

impl super::super::StrategicCampaign {
    pub(crate) fn validate_income_inputs(
        &self,
        statement: &EconomyStatement,
        data: &crate::data::GameData,
    ) -> Result<(), String> {
        let Some(inputs) = &statement.settlement_inputs else {
            return Ok(());
        };
        inputs
            .headquarters_bonus
            .validate("campaign", "income headquarters bonus")?;
        if inputs.headquarters_bonus
            != (Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            })
            && inputs.headquarters_bonus != data.economy.headquarters_income_bonus
        {
            return Err("campaign.income: invalid headquarters bonus".into());
        }
        let mut total = inputs.headquarters_bonus;
        let mut previous = None;
        for entry in &inputs.sites {
            if self.world.site(entry.site).is_none()
                || previous.is_some_and(|id| id >= entry.site)
                || entry.structural_damage > 100
                || entry.occupation > 100
                || entry.damage_percent > 100
                || entry.occupation_percent > 100
                || entry.focus_bonus_percent > 100
            {
                return Err("campaign.income: invalid saved site inputs".into());
            }
            if entry.base != data.economy.settlement_income[&entry.habitation]
                || entry.damage_percent
                    != 100 - entry.structural_damage / data.economy.income_damage_divisor
                || entry.occupation_percent
                    != if entry.occupation >= data.development.conditions.occupation_threshold {
                        data.development.occupation_income_percent
                    } else {
                        100
                    }
                || entry.focus_bonus_percent != data.development.income_focus_bonus_percent
            {
                return Err("campaign.income: saved modifiers do not match rules".into());
            }
            if entry.focus.is_some_and(|focus| {
                !crate::engine::development::focus_suitable(
                    self.world.site(entry.site).expect("known site"),
                    focus,
                )
            }) {
                return Err("campaign.income: focus unsuitable for fixed geography".into());
            }
            entry.base.validate("campaign", "income base")?;
            if entry.calculated() != Some(entry.income) {
                return Err("campaign.income: site formula does not match receipt".into());
            }
            total.gold = total
                .gold
                .checked_add(entry.income.gold)
                .ok_or("campaign.income: Gold overflow")?;
            total.wood = total
                .wood
                .checked_add(entry.income.wood)
                .ok_or("campaign.income: Wood overflow")?;
            total.stone = total
                .stone
                .checked_add(entry.income.stone)
                .ok_or("campaign.income: Stone overflow")?;
            previous = Some(entry.site);
        }
        if total != statement.income {
            return Err("campaign.income: site receipts do not sum to income".into());
        }
        Ok(())
    }
}
