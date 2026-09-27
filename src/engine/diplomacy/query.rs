//! Public relations, timers and pending decisions exclude private acceptance operands.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiplomacyFactionView {
    pub id: FactionId,
    pub name: String,
    pub status: FactionStatus,
    pub relation: DiplomaticState,
    pub truce_until: Option<u32>,
    pub declare_blocked: Option<String>,
    pub offer_blocked: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiplomacyView {
    pub factions: Vec<DiplomacyFactionView>,
    pub incoming_offers: Vec<PeaceOffer>,
    pub pending_defeats: Vec<PendingDefeat>,
    pub ending: Option<CampaignEnding>,
}
pub fn diplomacy_view(
    campaign: &StrategicCampaign,
    _data: &GameData,
    observer: FactionId,
) -> DiplomacyView {
    let factions = campaign
        .factions
        .values()
        .filter(|faction| faction.id != observer)
        .filter_map(|faction| {
            let relation = relation(campaign, observer, faction.id)?;
            let gate = if campaign.diplomacy.ending.is_some() {
                Some("This campaign has ended.")
            } else if campaign.diplomacy.has_pending_decision() {
                Some("Resolve the pending diplomatic decision first.")
            } else if campaign.active_faction() != observer {
                Some("Diplomacy requires your faction's turn.")
            } else {
                None
            };
            Some(DiplomacyFactionView {
                id: faction.id,
                name: faction.name.clone(),
                status: faction.status,
                relation,
                truce_until: campaign
                    .diplomacy
                    .pair(observer, faction.id)
                    .and_then(|pair| pair.truce_until),
                declare_blocked: gate.map(str::to_owned).or_else(|| {
                    commands::declare_check(campaign, observer, faction.id)
                        .err()
                        .map(|error| error.to_string())
                }),
                offer_blocked: gate.map(str::to_owned).or_else(|| {
                    commands::offer_check(campaign, observer, faction.id)
                        .err()
                        .map(|error| error.to_string())
                }),
            })
        })
        .collect();
    DiplomacyView {
        factions,
        incoming_offers: campaign
            .diplomacy
            .pending_offers
            .iter()
            .filter(|offer| offer.recipient == observer)
            .cloned()
            .collect(),
        pending_defeats: campaign
            .diplomacy
            .pending_defeats
            .iter()
            .filter(|entry| entry.victor == observer)
            .cloned()
            .collect(),
        ending: campaign.diplomacy.ending.clone(),
    }
}
