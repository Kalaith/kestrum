//! Plan each site's natural change, then route displaced people from fixed budgets.

use super::*;
use conditions::LocalConditions;

pub(crate) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    snapshot: &DevelopmentSnapshot,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let ids: Vec<_> = campaign.world.sites.iter().map(|site| site.id).collect();
    let plans = ids
        .iter()
        .map(|id| plan(campaign, data, *id, &snapshot.conditions[id]))
        .collect::<Result<Vec<_>, _>>()?;
    for plan in plans {
        apply(campaign, outcome, plan)?;
    }
    migration::apply(campaign, data, snapshot, outcome)?;
    for id in ids {
        let state = &campaign.world.development[&id];
        if state.ruined
            && !state.threat_created
            && state.lawless_rounds >= data.threats.lawless_rounds
        {
            let transition = state.ruination;
            super::super::threats::spawn_ruin(campaign, data, id, transition)?;
            campaign
                .world
                .development
                .get_mut(&id)
                .expect("site")
                .threat_created = true;
        }
    }
    Ok(())
}

struct SitePlan {
    id: SiteId,
    state: SiteDevelopment,
    population: u32,
    habitation: Habitation,
    damage: u32,
    occupation: u32,
    fort_damage: u32,
    receipts: Vec<DevelopmentReceipt>,
}

fn plan(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: SiteId,
    c: &LocalConditions,
) -> Result<SitePlan, RuleError> {
    let site = campaign.world.site(id).expect("snapshot site");
    let mut plan = SitePlan {
        id,
        state: campaign.world.development[&id].clone(),
        population: campaign.world.population[&id],
        habitation: site.habitation,
        damage: campaign.world.structural_damage(id),
        occupation: campaign.world.occupation.get(&id).copied().unwrap_or(0),
        fort_damage: campaign.world.fort_damage.get(&id).copied().unwrap_or(0),
        receipts: Vec::new(),
    };
    if plan.state.last_resolved_round == Some(campaign.completed_rounds) {
        return Ok(plan);
    }
    plan.state.last_resolved_round = Some(campaign.completed_rounds);
    // Construction has already applied its conserved settlers and reclamation this boundary.
    if site.habitation != c.habitation || plan.state.ruined != c.ruined {
        return Ok(plan);
    }
    if !c.ruined {
        pressure(campaign, data, c, &mut plan)?;
    }
    population(data, site, c, &mut plan)?;
    repair(data, c, &mut plan);
    lawless(data, c, &mut plan);
    Ok(plan)
}

fn pressure(
    campaign: &StrategicCampaign,
    data: &GameData,
    c: &LocalConditions,
    plan: &mut SitePlan,
) -> Result<(), RuleError> {
    let rules = &data.development;
    plan.state.pressure = (plan.state.pressure + conditions::contribution(data, c))
        .clamp(rules.pressure.minimum, rules.pressure.maximum);
    let site = campaign.world.site(plan.id).expect("site");
    let next = if plan.state.pressure >= rules.pressure.upgrade_threshold {
        next_tier(c.habitation).filter(|next| {
            *next <= maximum_habitation(data, site)
                && plan.population >= data.construction.population.minimum[next]
        })
    } else if plan.state.pressure <= -rules.pressure.downgrade_threshold {
        previous_tier(c.habitation)
    } else {
        None
    };
    if let Some(to) = next {
        plan.state.pressure += if to > c.habitation {
            -rules.pressure.upgrade_threshold
        } else {
            rules.pressure.downgrade_threshold
        };
        plan.habitation = to;
        plan.receipts.push(DevelopmentReceipt::HabitationChanged {
            site: plan.id,
            owner: c.owner,
            from: c.habitation,
            to,
        });
    }
    plan.state.ruin_streak = if c.damage >= rules.conditions.ruin_damage
        && plan.population < rules.conditions.ruin_population
    {
        (plan.state.ruin_streak + 1).min(rules.conditions.ruin_steps)
    } else {
        0
    };
    if plan.state.ruin_streak == rules.conditions.ruin_steps {
        plan.state.ruination = plan
            .state
            .ruination
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "ruination identity",
            })?;
        plan.state.ruined = true;
        plan.state.ruined_round = Some(campaign.completed_rounds);
        plan.state.threat_created = false;
        plan.state.lawless_rounds = 0;
        plan.receipts.push(DevelopmentReceipt::Ruined {
            site: plan.id,
            owner: c.owner,
            ruination: plan.state.ruination,
        });
    }
    Ok(())
}

fn population(
    data: &GameData,
    site: &Site,
    c: &LocalConditions,
    plan: &mut SitePlan,
) -> Result<(), RuleError> {
    let rules = &data.development;
    if c.damage >= rules.conditions.displacement_damage
        || c.occupation >= rules.conditions.occupation_threshold
    {
        let newly = (plan.population / rules.population.displacement_divisor)
            .min(plan.population - plan.state.displaced);
        plan.state.displaced += newly;
    } else if c.habitation != Habitation::Unsettled && !plan.state.ruined && c.safe && c.supplied {
        let growth = (plan.population / rules.population.growth_divisor)
            .max(rules.population.minimum_growth);
        let room = population_capacity(data, site).saturating_sub(plan.population);
        plan.population =
            plan.population
                .checked_add(growth.min(room))
                .ok_or(RuleError::Overflow {
                    field: "natural population growth",
                })?;
    }
    Ok(())
}

fn repair(data: &GameData, c: &LocalConditions, plan: &mut SitePlan) {
    let rules = &data.development.conditions;
    if c.safe {
        plan.occupation = plan
            .occupation
            .saturating_sub(if c.garrison || c.functional_fort {
                rules.occupation_garrison_decay
            } else {
                rules.occupation_safe_decay
            });
    }
    if c.safe && c.supplied {
        plan.damage = plan.damage.saturating_sub(rules.structural_repair);
        let fort_repair = rules.fort_repair
            + if c.focus == Some(Focus::Fortification) {
                rules.fort_focus_repair
            } else {
                0
            };
        plan.fort_damage = plan.fort_damage.saturating_sub(fort_repair);
    }
}

fn lawless(data: &GameData, c: &LocalConditions, plan: &mut SitePlan) {
    if !plan.state.ruined || !c.ruined {
        plan.state.lawless_rounds = 0;
        return;
    }
    plan.state.lawless_rounds = if c.lawless {
        (plan.state.lawless_rounds + 1).min(data.threats.lawless_rounds)
    } else {
        0
    };
}

fn apply(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
    plan: SitePlan,
) -> Result<(), RuleError> {
    let was_unsettled = campaign
        .world
        .site(plan.id)
        .is_some_and(|site| site.habitation == crate::data::economy::Habitation::Unsettled);
    if was_unsettled && plan.habitation >= crate::data::economy::Habitation::Camp {
        campaign
            .world
            .founded_rounds
            .entry(plan.id)
            .or_insert(campaign.completed_rounds);
    }
    campaign.world.development.insert(plan.id, plan.state);
    campaign.world.population.insert(plan.id, plan.population);
    campaign.world.site_damage.insert(plan.id, plan.damage);
    campaign.world.occupation.insert(plan.id, plan.occupation);
    if campaign
        .world
        .site(plan.id)
        .is_some_and(|site| site.military == crate::data::world::MilitaryLayer::Fort)
    {
        campaign.world.fort_damage.insert(plan.id, plan.fort_damage);
    }
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == plan.id)
        .expect("site")
        .habitation = plan.habitation;
    for receipt in plan.receipts {
        record(campaign, outcome, receipt)?;
    }
    Ok(())
}
