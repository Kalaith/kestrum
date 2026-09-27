//! Offers reveal a decision, never the private sovereign comparison behind it.

use super::*;

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: Command,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    match command {
        Command::DeclareWar { faction } => declare(campaign, owner, faction, outcome),
        Command::OfferPeace { faction } => offer(campaign, data, owner, faction, outcome),
        Command::RespondPeace { proposer, accept } => {
            respond(campaign, data, owner, proposer, accept, outcome)
        }
        Command::ResolveDefeat {
            faction,
            resolution,
        } => outcomes::resolve_choice(campaign, owner, faction, resolution, outcome),
        _ => Err(error("Choose a diplomatic action.")),
    }
}

pub(super) fn check_pair(
    campaign: &StrategicCampaign,
    owner: FactionId,
    target: FactionId,
) -> Result<(), RuleError> {
    if owner == target || !campaign.is_independent(owner) || !campaign.is_independent(target) {
        return Err(error("Choose another independent kingdom."));
    }
    if relation(campaign, owner, target).is_none() {
        return Err(error("This diplomatic relation is unavailable."));
    }
    Ok(())
}

pub(super) fn declare_check(
    campaign: &StrategicCampaign,
    owner: FactionId,
    target: FactionId,
) -> Result<(), RuleError> {
    check_pair(campaign, owner, target)?;
    if relation(campaign, owner, target) != Some(DiplomaticState::Peace) {
        return Err(error("These kingdoms are already at war."));
    }
    if campaign
        .diplomacy
        .pair(owner, target)
        .and_then(|pair| pair.truce_until)
        .is_some_and(|until| campaign.completed_rounds < until)
    {
        return Err(error("The non-aggression agreement is still in force."));
    }
    Ok(())
}
fn declare(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    target: FactionId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    declare_check(campaign, owner, target)?;
    let factions = ordered_pair(owner, target);
    campaign
        .relations
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .expect("pair")
        .state = DiplomaticState::War;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .expect("pair");
    pair.peace_since = None;
    pair.truce_until = None;
    pair.war_started_round = Some(campaign.completed_rounds);
    record(
        campaign,
        outcome,
        DiplomacyReceipt::WarDeclared { factions },
    )
}

pub(super) fn offer_check(
    campaign: &StrategicCampaign,
    owner: FactionId,
    target: FactionId,
) -> Result<(), RuleError> {
    check_pair(campaign, owner, target)?;
    if relation(campaign, owner, target) != Some(DiplomaticState::War) {
        return Err(error("These kingdoms are already at peace."));
    }
    if campaign
        .diplomacy
        .pair(owner, target)
        .and_then(|pair| pair.last_offer_round)
        == Some(campaign.completed_rounds)
    {
        return Err(error(
            "This pair of kingdoms has already exchanged a peace offer this season.",
        ));
    }
    Ok(())
}
fn offer(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    target: FactionId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    offer_check(campaign, owner, target)?;
    let pair = ordered_pair(owner, target);
    campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|entry| entry.factions == pair)
        .expect("pair")
        .last_offer_round = Some(campaign.completed_rounds);
    record(
        campaign,
        outcome,
        DiplomacyReceipt::PeaceOffered {
            proposer: owner,
            recipient: target,
        },
    )?;
    if target == campaign.player {
        campaign.diplomacy.pending_offers.push(PeaceOffer {
            proposer: owner,
            recipient: target,
            completed_rounds: campaign.completed_rounds,
        });
        campaign
            .diplomacy
            .pending_offers
            .sort_by_key(|offer| offer.proposer);
        Ok(())
    } else if peace_desired(campaign, data, target, owner) {
        agree(campaign, data, owner, target, outcome)
    } else {
        record(
            campaign,
            outcome,
            DiplomacyReceipt::PeaceRejected {
                proposer: owner,
                recipient: target,
            },
        )
    }
}
fn respond(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    proposer: FactionId,
    accept: bool,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    if owner != campaign.player
        || !campaign
            .diplomacy
            .pending_offers
            .iter()
            .any(|offer| offer.proposer == proposer && offer.recipient == owner)
    {
        return Err(error(
            "This peace offer is no longer awaiting your response.",
        ));
    }
    if accept {
        agree(campaign, data, proposer, owner, outcome)?;
    } else {
        record(
            campaign,
            outcome,
            DiplomacyReceipt::PeaceRejected {
                proposer,
                recipient: owner,
            },
        )?;
    }
    campaign
        .diplomacy
        .pending_offers
        .retain(|offer| offer.proposer != proposer);
    Ok(())
}
fn agree(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    first: FactionId,
    second: FactionId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    check_pair(campaign, first, second)?;
    let factions = ordered_pair(first, second);
    let until = campaign
        .completed_rounds
        .checked_add(data.diplomacy.truce_rounds)
        .ok_or(RuleError::Overflow {
            field: "non-aggression date",
        })?;
    campaign
        .relations
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .expect("pair")
        .state = DiplomaticState::Peace;
    withdrawal::resolve(campaign, data, factions, outcome)?;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .expect("pair");
    pair.peace_since = Some(campaign.completed_rounds);
    pair.truce_until = Some(until);
    pair.war_ended_round = Some(campaign.completed_rounds);
    record(
        campaign,
        outcome,
        DiplomacyReceipt::PeaceAgreed {
            factions,
            truce_until: until,
        },
    )
}
