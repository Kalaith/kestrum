//! Every illegal foreign occupant gets one adjacent lawful exit before peace commits.

use super::*;

pub(super) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    factions: [FactionId; 2],
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let before: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| factions.contains(&army.faction))
        .map(|army| (army.id, army.faction, army.site))
        .collect();
    // Plan against one post-agreement board, before any group has moved. Mutual
    // siege sites are being lifted and cannot become civilian ownership transfers.
    let mut board = campaign.clone();
    for siege in campaign
        .sieges
        .values()
        .filter(|siege| ordered_pair(siege.defender, siege.besieger) == factions)
    {
        board.sieges.remove(&siege.site);
        board.world.contested_sites.remove(&siege.site);
    }
    let mut moves = Vec::new();
    for (army, faction, from) in before {
        let owner = campaign.world.site(from).and_then(|site| site.controller);
        if owner.is_some_and(|owner| owner != faction && factions.contains(&owner)) {
            let to = super::super::retreat::destination(&board, faction, from, None, None)
                .ok_or_else(|| error("Forces must withdraw first"))?;
            moves.push((army, faction, from, to));
        }
    }
    for (army, faction, from, to) in moves {
        campaign.armies.get_mut(&army).expect("army").site = to;
        super::super::siege::exhaust(campaign, data, &[army]);
        record(
            campaign,
            outcome,
            DiplomacyReceipt::ArmyWithdrawn {
                faction,
                army,
                from,
                to,
            },
        )?;
    }
    super::super::siege::reconcile(campaign, data, outcome)
}
