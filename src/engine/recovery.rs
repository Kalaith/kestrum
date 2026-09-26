//! Supply-snapshot recovery, using post-upkeep Gold and one exact cost rounding.

use super::{economy, RuleError};
use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        military::{ArmyId, FormationId, RecoveryEntry, RecoveryStatement},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPreview {
    pub army: ArmyId,
    pub formation: FormationId,
    pub site: SiteId,
    pub headcount_before: u32,
    pub maximum: u32,
    pub restored: u32,
    pub gold_cost: i64,
    pub blocked: Option<String>,
}

/// Forecast the next boundary if present control, troops and balances remain unchanged.
/// Only observer-owned entries leave the engine; preview consumes no RNG or currency.
pub fn recovery_preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
) -> Result<Vec<RecoveryPreview>, RuleError> {
    data.economy.validate().map_err(RuleError::InvalidState)?;
    campaign.validate(data).map_err(RuleError::InvalidState)?;
    if !campaign.factions.contains_key(&observer) {
        return Err(RuleError::UnknownActor);
    }
    let supply = snapshot(campaign);
    let mut candidate = campaign.clone();
    economy::resolve(&mut candidate, data)?;
    let (previews, _) = plan_faction(&candidate, data, observer, &supply)?;
    Ok(previews)
}

/// Capture physical connectivity before income, construction or other boundary work.
/// K10 will add actual siege exclusions to the shared supply query when sieges exist.
pub(super) fn snapshot(campaign: &StrategicCampaign) -> SupplySnapshot {
    SupplySnapshot {
        supplied: campaign
            .factions
            .values()
            .map(|faction| {
                (
                    faction.id,
                    campaign
                        .world
                        .supplied_sites(faction.id, faction.headquarters),
                )
            })
            .collect(),
    }
}

pub(super) struct SupplySnapshot {
    supplied: BTreeMap<FactionId, BTreeSet<SiteId>>,
}

pub(super) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    supply: &SupplySnapshot,
) -> Result<(), RuleError> {
    let statements = campaign
        .factions
        .keys()
        .map(|&faction| {
            plan_faction(campaign, data, faction, supply).map(|(_, statement)| (faction, statement))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (id, statement) in statements {
        for entry in &statement.entries {
            let formation = campaign
                .formations
                .get_mut(&entry.formation)
                .ok_or_else(|| RuleError::InvalidState("Recovery formation disappeared.".into()))?;
            formation.headcount =
                formation
                    .headcount
                    .checked_add(entry.restored)
                    .ok_or(RuleError::Overflow {
                        field: "recovered headcount",
                    })?;
        }
        let faction = campaign
            .factions
            .get_mut(&id)
            .expect("existing recovery faction");
        faction.resources.gold = statement.closing_gold;
        faction.last_recovery = Some(statement);
    }
    Ok(())
}

fn plan_faction(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    supply: &SupplySnapshot,
) -> Result<(Vec<RecoveryPreview>, RecoveryStatement), RuleError> {
    let owner = &campaign.factions[&faction];
    let completed_rounds = campaign
        .completed_rounds
        .checked_add(1)
        .ok_or(RuleError::Overflow {
            field: "completed rounds",
        })?;
    let mut statement = RecoveryStatement {
        completed_rounds,
        opening_gold: owner.resources.gold,
        closing_gold: owner.resources.gold,
        gold_spent: 0,
        restored: 0,
        entries: Vec::new(),
    };
    let mut previews = Vec::new();
    for army in campaign
        .armies
        .values()
        .filter(|army| army.faction == faction)
    {
        for id in army.formation_ids() {
            let formation = &campaign.formations[&id];
            let definition = &data.economy.formations[&formation.kind];
            let maximum = (u64::from(formation.capacity)
                * u64::from(data.economy.recovery.capacity_percent_per_round)
                / 100) as u32;
            let maximum = maximum.min(formation.capacity.saturating_sub(formation.headcount));
            let blocked = if !campaign.is_independent(faction) {
                Some("Inactive factions cannot recover formations.")
            } else if formation.headcount == 0 {
                Some("Destroyed formations cannot recover.")
            } else if maximum == 0 {
                Some("No headcount is eligible for recovery this season.")
            } else if !supply.supplied[&faction].contains(&army.site) {
                Some("Cut off: no friendly supply path to headquarters.")
            } else if owner.deficit {
                Some("Upkeep shortfall blocks recovery.")
            } else {
                None
            };
            let (restored, gold_cost) = if blocked.is_none() {
                affordable_recovery(
                    definition.recruit_cost.gold,
                    formation.capacity,
                    maximum,
                    statement.closing_gold,
                    data.economy.recovery.full_replacement_recruit_gold_percent,
                )?
            } else {
                (0, 0)
            };
            let blocked = blocked.or_else(|| {
                (restored == 0)
                    .then_some("No Gold remains for recovery after upkeep and earlier formations.")
            });
            previews.push(RecoveryPreview {
                army: army.id,
                formation: id,
                site: army.site,
                headcount_before: formation.headcount,
                maximum,
                restored,
                gold_cost,
                blocked: blocked.map(str::to_owned),
            });
            if restored > 0 {
                record_recovery(
                    &mut statement,
                    RecoveryEntry {
                        army: army.id,
                        formation: id,
                        site: army.site,
                        kind: formation.kind,
                        capacity: formation.capacity,
                        headcount_before: formation.headcount,
                        restored,
                        gold_cost,
                    },
                )?;
            }
        }
    }
    Ok((previews, statement))
}

fn record_recovery(
    statement: &mut RecoveryStatement,
    entry: RecoveryEntry,
) -> Result<(), RuleError> {
    statement.closing_gold = statement
        .closing_gold
        .checked_sub(entry.gold_cost)
        .filter(|balance| *balance >= 0)
        .ok_or(RuleError::Overflow {
            field: "recovery Gold balance",
        })?;
    statement.gold_spent =
        statement
            .gold_spent
            .checked_add(entry.gold_cost)
            .ok_or(RuleError::Overflow {
                field: "recovery Gold",
            })?;
    statement.restored = statement
        .restored
        .checked_add(u64::from(entry.restored))
        .ok_or(RuleError::Overflow {
            field: "recovery total headcount",
        })?;
    statement.entries.push(entry);
    Ok(())
}

fn affordable_recovery(
    recruit_gold: i64,
    capacity: u32,
    maximum: u32,
    gold: i64,
    replacement_percent: u32,
) -> Result<(u32, i64), RuleError> {
    // Inputs are validated i64/u32 values: these products fit comfortably in i128.
    let numerator_per_member = i128::from(recruit_gold) * i128::from(replacement_percent);
    let denominator = 100 * i128::from(capacity);
    // ceil(rate*n/denominator) <= Gold iff rate*n <= Gold*denominator.
    let affordable = i128::from(gold) * denominator / numerator_per_member;
    let restored = affordable.min(i128::from(maximum)) as u32;
    let cost = (numerator_per_member * i128::from(restored) + denominator - 1) / denominator;
    let cost = i64::try_from(cost).map_err(|_| RuleError::Overflow {
        field: "recovery cost",
    })?;
    Ok((restored, cost))
}
