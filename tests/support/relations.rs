//! Keep explicit historical relation fixtures consistent with their timers.
use kestrum::{data::world::DiplomaticState, state::StrategicCampaign};
pub fn sync_relations(campaign: &mut StrategicCampaign) {
    campaign.relations.sort_by_key(|relation| relation.factions);
    campaign.diplomacy.pairs = campaign
        .relations
        .iter()
        .map(|relation| kestrum::state::diplomacy::PairDiplomacy {
            factions: relation.factions,
            peace_since: (relation.state == DiplomaticState::Peace)
                .then_some(campaign.completed_rounds),
            truce_until: None,
            last_offer_round: None,
        })
        .collect();
}
