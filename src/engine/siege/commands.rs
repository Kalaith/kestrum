//! Legal decisions do not inspect undisclosed enemy strength or roll combat previews.

use super::*;
use crate::state::{
    battle::BattleContext,
    siege::{SiegeAction, SiegeOrder},
};

pub(in crate::engine) fn validate_order(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    order: &SiegeOrder,
) -> Result<(), RuleError> {
    let siege = campaign
        .sieges
        .get(&order.site)
        .ok_or_else(|| blocked("There is no siege at this site."))?;
    let besieger = owner == siege.besieger;
    let defender = owner == siege.defender;
    let allowed = match order.action {
        SiegeAction::Maintain | SiegeAction::Assault | SiegeAction::Withdraw => besieger,
        SiegeAction::Sortie | SiegeAction::Escape => defender,
    };
    if !allowed {
        return Err(blocked("Your faction cannot issue that siege order."));
    }
    if order.armies.is_empty()
        || order.armies.iter().collect::<BTreeSet<_>>().len() != order.armies.len()
    {
        return Err(RuleError::InvalidArmyGroup);
    }
    let roster = if besieger {
        &siege.besieging
    } else {
        &siege.defending
    };
    if order.armies.iter().any(|id| !roster.contains(id)) {
        return Err(blocked(
            "Choose only your own armies on this side of the siege.",
        ));
    }
    if matches!(
        order.action,
        SiegeAction::Assault | SiegeAction::Sortie | SiegeAction::Escape
    ) && !order.armies.iter().any(|id| {
        campaign
            .army_movement_remaining(*id, data)
            .is_some_and(|left| left > 0)
    }) {
        return Err(blocked(
            "At least one selected army needs unused movement this season.",
        ));
    }
    if matches!(order.action, SiegeAction::Withdraw | SiegeAction::Escape) {
        if !order
            .destination
            .is_some_and(|site| destinations(campaign, owner, order.site).contains(&site))
        {
            return Err(blocked("Choose a legal adjacent neutral or friendly exit free of siege and hostile armies."));
        }
    } else if order.destination.is_some() {
        return Err(blocked(
            "This siege action does not use an exit destination.",
        ));
    }
    Ok(())
}

fn blocked(message: &str) -> RuleError {
    RuleError::Siege(message.to_owned())
}

pub(in crate::engine) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    order: SiegeOrder,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    validate_order(campaign, data, owner, &order)?;
    let siege = campaign.sieges[&order.site].clone();
    if order.action == SiegeAction::Maintain {
        return Ok(());
    }
    if order.action == SiegeAction::Withdraw {
        let destination = order.destination.expect("validated exit");
        for id in &order.armies {
            campaign.armies.get_mut(id).expect("own army").site = destination;
        }
        exhaust(campaign, data, &order.armies);
        return reconcile(campaign, data, outcome);
    }
    let (defenders, context) = match order.action {
        SiegeAction::Assault => (
            siege.defending.clone(),
            BattleContext::Assault { siege: siege.id },
        ),
        SiegeAction::Sortie => (
            siege.besieging.clone(),
            BattleContext::Sortie { siege: siege.id },
        ),
        SiegeAction::Escape => (
            siege.besieging.clone(),
            BattleContext::Escape {
                siege: siege.id,
                destination: order.destination.expect("exit"),
            },
        ),
        _ => unreachable!("noncombat actions returned"),
    };
    combat::prepare_encounter(
        campaign,
        data,
        combat::Encounter {
            attackers: order.armies,
            defenders,
            origin: order.site,
            site: order.site,
            context,
        },
    )?;
    reconcile(campaign, data, outcome)
}
