//! Entry spends the physical edge before establishment, relief or a camp challenge.

use super::*;
use crate::state::battle::{BattleContext, BattleId, BattleOutcome};

pub(in crate::engine) fn admission(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: SiteId,
) -> Result<(), super::super::MovementBlock> {
    let blocked = super::super::MovementBlock::EncounterUnavailable;
    if let Some(siege) = campaign.sieges.get(&site) {
        return if owner == siege.defender
            || owner == siege.besieger
            || (retreat::hostile(campaign, owner, siege.defender)
                && retreat::hostile(campaign, owner, siege.besieger))
        {
            Ok(())
        } else {
            Err(blocked)
        };
    }
    let foreign: BTreeSet<_> = campaign
        .armies
        .values()
        .filter(|army| army.site == site && army.faction != owner)
        .map(|army| army.faction)
        .collect();
    if foreign.len() > 1 {
        return Err(blocked);
    }
    if let Some(foreign) = foreign.first() {
        let controller = campaign.world.site(site).and_then(|site| site.controller);
        if retreat::hostile(campaign, owner, *foreign) {
            if controller != Some(*foreign) {
                return Err(blocked);
            }
        } else if controller.is_some() {
            return Err(super::super::MovementBlock::PeaceBoundary);
        }
    }
    Ok(())
}

pub(in crate::engine) fn arrive(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    armies: &[ArmyId],
    origin: SiteId,
    site: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<Option<BattleId>, RuleError> {
    let owner = campaign.armies[&armies[0]].faction;
    if let Some(siege) = campaign.sieges.get(&site).cloned() {
        if owner == siege.besieger {
            exhaust(campaign, data, armies);
            reconcile(campaign, data, outcome)?;
            return Ok(None);
        }
        let mut attackers = armies.to_vec();
        let context = if owner == siege.defender {
            attackers.extend(&siege.defending);
            attackers.sort();
            BattleContext::Relief {
                siege: siege.id,
                garrison: siege.defending.clone(),
            }
        } else {
            if !retreat::hostile(campaign, owner, siege.besieger)
                || !retreat::hostile(campaign, owner, siege.defender)
            {
                return Err(RuleError::Siege(
                    "A third faction must be hostile to both siege sides before entry.".into(),
                ));
            }
            BattleContext::BesiegerClash {
                siege: siege.id,
                garrison_faction: siege.defender,
            }
        };
        let battle = combat::resolve_encounter(
            campaign,
            data,
            combat::Encounter {
                attackers,
                defenders: siege.besieging.clone(),
                origin,
                site,
                context,
            },
        )?;
        if owner != siege.defender
            && campaign.battles[&battle].outcome == BattleOutcome::AttackerVictory
        {
            campaign.sieges.remove(&site);
            record_fact(
                campaign,
                outcome,
                DomainFactKind::SiegeChanged {
                    siege: siege.clone(),
                    change: SiegeChange::Lifted,
                },
            )?;
            establish(campaign, data, site, siege.defender, owner, outcome)?;
        } else {
            reconcile(campaign, data, outcome)?;
        }
        return Ok(Some(battle));
    }
    establish_or_capture(campaign, data, owner, site, outcome)
}

fn establish_or_capture(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
    outcome: &mut ActionOutcome,
) -> Result<Option<BattleId>, RuleError> {
    let defenders: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.site == site && retreat::hostile(campaign, owner, army.faction))
        .map(|army| army.id)
        .collect();
    if defenders.is_empty() {
        if !campaign
            .armies
            .values()
            .any(|army| army.site == site && army.faction != owner)
        {
            combat::capture(campaign, data, site, owner);
        }
        return Ok(None);
    }
    let defender = campaign.armies[&defenders[0]].faction;
    if defenders
        .iter()
        .any(|id| campaign.armies[id].faction != defender)
        || campaign
            .armies
            .values()
            .any(|army| army.site == site && army.faction != owner && army.faction != defender)
    {
        return Err(RuleError::Siege(
            "This entry involves incompatible military sides.".into(),
        ));
    }
    if campaign.world.site(site).and_then(|site| site.controller) != Some(defender) {
        return Err(RuleError::Siege(
            "A defended fort must be controlled by its garrison before siege entry.".into(),
        ));
    }
    establish(campaign, data, site, defender, owner, outcome)?;
    Ok(None)
}
