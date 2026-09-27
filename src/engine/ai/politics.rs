use super::*;
use crate::data::economy::Habitation;

impl Planner<'_> {
    pub(super) fn peace(&self) -> Option<AiDecision> {
        for other in self
            .view
            .factions
            .iter()
            .filter(|faction| self.at_war(faction.id))
        {
            let pair = self.campaign.diplomacy.pair(self.owner, other.id)?;
            if pair.last_offer_round == Some(self.campaign.completed_rounds) {
                continue;
            }
            // P08 explicitly permits this private aggregate sovereign evaluation.
            if crate::engine::diplomacy::peace_desired(
                self.campaign,
                self.data,
                self.owner,
                other.id,
            ) {
                if let Some(decision) = self.choose(
                    Command::OfferPeace { faction: other.id },
                    self.objective.clone(),
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    pub(super) fn declare_war(&self) -> Option<AiDecision> {
        if self
            .view
            .armies
            .iter()
            .filter(|army| self.campaign.army_is_supplied(army.id))
            .count()
            < self.data.ai.target_armies
            || self.nearby_neutral()
        {
            return None;
        }
        let adjacent: std::collections::BTreeSet<_> = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(self.owner))
            .flat_map(|site| self.view.world.adjacent_sites(site.id))
            .filter_map(|site| self.view.world.site(site)?.controller)
            .filter(|owner| *owner != self.owner)
            .collect();
        let mut rivals: Vec<_> = adjacent
            .into_iter()
            .filter(|other| !self.at_war(*other) && self.campaign.is_independent(*other))
            .collect();
        rivals.sort_by_key(|other| {
            (
                self.view
                    .world
                    .sites
                    .iter()
                    .filter(|site| {
                        site.controller == Some(*other) && site.habitation != Habitation::Unsettled
                    })
                    .count(),
                *other,
            )
        });
        for rival in rivals {
            let pair = self.campaign.diplomacy.pair(self.owner, rival)?;
            if pair
                .truce_until
                .is_some_and(|until| self.campaign.completed_rounds < until)
                || pair.peace_since.is_none_or(|round| {
                    self.campaign.completed_rounds.saturating_sub(round)
                        < self.data.ai.war_peace_rounds
                })
            {
                continue;
            }
            if let Some(decision) = self.choose(
                Command::DeclareWar { faction: rival },
                self.objective.clone(),
            ) {
                return Some(decision);
            }
        }
        None
    }

    fn nearby_neutral(&self) -> bool {
        self.view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller.is_none() && !self.view.hostile_presence.contains(&site.id)
            })
            .any(|site| {
                self.view.armies.iter().any(|army| {
                    self.path(army.site, site.id, false)
                        .is_some_and(|(_, path)| {
                            path.len().saturating_sub(1) <= self.data.ai.neutral_search_edges
                        })
                })
            })
    }
}
