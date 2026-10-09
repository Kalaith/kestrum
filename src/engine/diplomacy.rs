//! Atomic agreements, defeat reconciliation and public diplomatic choices.

mod commands;
mod outcomes;
mod query;
mod withdrawal;
pub(crate) use commands::execute;
pub(crate) use outcomes::reconcile;
pub use query::{diplomacy_view, DiplomacyFactionView, DiplomacyView};

use super::{actions::record_fact, ActionOutcome, Command, RuleError};
use crate::{
    data::{
        world::{DiplomaticState, FactionId},
        GameData,
    },
    state::{
        campaign::{DomainFactKind, FactionStatus},
        diplomacy::*,
        StrategicCampaign,
    },
};

fn error(message: &str) -> RuleError {
    RuleError::Diplomacy(message.into())
}
fn record(
    campaign: &mut StrategicCampaign,
    outcome: &mut ActionOutcome,
    receipt: DiplomacyReceipt,
) -> Result<(), RuleError> {
    record_fact(
        campaign,
        outcome,
        DomainFactKind::DiplomacyChanged { receipt },
    )
}
fn relation(
    campaign: &StrategicCampaign,
    first: FactionId,
    second: FactionId,
) -> Option<DiplomaticState> {
    let pair = ordered_pair(first, second);
    campaign
        .relations
        .iter()
        .find(|entry| entry.factions == pair)
        .map(|entry| entry.state)
}

/// The one explicit P08 private sovereign evaluation. UI projections never call it.
pub fn peace_desired(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    opponent: FactionId,
) -> bool {
    if data.diplomacy.validate().is_err() || data.diplomacy.validate_economy(&data.economy).is_err()
    {
        return false;
    }
    if !campaign.is_independent(faction)
        || !campaign.is_independent(opponent)
        || relation(campaign, faction, opponent) != Some(DiplomaticState::War)
        || campaign
            .sieges
            .values()
            .any(|siege| siege.besieger == faction && siege.defender == opponent)
    {
        return false;
    }
    // A peace-seeking sovereign wants any war ended that it is not winning by siege.
    if campaign.factions.get(&faction).is_some_and(|sovereign| {
        data.ai.validate().is_ok() && data.ai.profile(sovereign.personality).seeks_peace
    }) {
        return true;
    }
    let loss = campaign.diplomacy.losses.iter().any(|loss| {
        loss.faction == faction
            && loss.victor == opponent
            && campaign
                .completed_rounds
                .saturating_sub(loss.completed_rounds)
                < data.diplomacy.recent_loss_rounds
    });
    // Recent site losses are retained for `recent_loss_rounds`; none taken
    // from this opponent in an old war means the war is no longer paying.
    let stalemate = campaign
        .diplomacy
        .pair(faction, opponent)
        .and_then(|pair| pair.war_started_round)
        .is_some_and(|started| {
            campaign.completed_rounds.saturating_sub(started) >= data.diplomacy.stalemate_rounds
        })
        && !campaign
            .diplomacy
            .losses
            .iter()
            .any(|loss| loss.faction == opponent && loss.victor == faction);
    let equivalents = |owner| military_equivalents(campaign, data, owner);
    loss || stalemate
        || equivalents(faction) * u128::from(data.diplomacy.peace_strength_denominator)
            < equivalents(opponent) * u128::from(data.diplomacy.peace_strength_numerator)
}

/// Surviving troops normalized by formation capacity; a private sovereign aggregate.
pub(crate) fn military_equivalents(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
) -> u128 {
    let common = data
        .economy
        .formations
        .values()
        .fold(1_u128, |value, formation| {
            let capacity = u128::from(formation.capacity);
            value / gcd(value, capacity) * capacity
        });
    campaign
        .formations
        .values()
        .filter(|formation| formation.faction == owner)
        .map(|formation| {
            u128::from(formation.headcount) * (common / u128::from(formation.capacity))
        })
        .sum::<u128>()
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let next = a % b;
        a = b;
        b = next;
    }
    a
}
