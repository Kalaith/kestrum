use super::*;
use crate::data::economy::Habitation;
use crate::data::world::FactionId;

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
        let temperament = self.temperament();
        if self
            .view
            .armies
            .iter()
            .filter(|army| self.campaign.army_is_supplied(army.id))
            .count()
            < self.data.ai.target_armies
            || (!temperament.war_while_land_remains && self.nearby_neutral())
        {
            return None;
        }
        let adjacent = self.neighbors();
        // A distant war that no army can reach does not occupy a frontier.
        if adjacent.iter().filter(|other| self.at_war(**other)).count() >= temperament.maximum_wars
        {
            return None;
        }
        let mut rivals: Vec<_> = adjacent
            .into_iter()
            .filter(|other| {
                !self.at_war(*other) && !self.stronger(*other, temperament.war_strength_percent)
            })
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
                        < temperament.war_peace_rounds
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

    /// Neighboring independent kingdoms, from this sovereign's known border.
    fn neighbors(&self) -> std::collections::BTreeSet<FactionId> {
        self.view
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(self.owner))
            .flat_map(|site| self.view.world.adjacent_sites(site.id))
            .filter_map(|site| self.view.world.site(site)?.controller)
            .filter(|owner| *owner != self.owner && self.campaign.is_independent(*owner))
            .collect()
    }

    /// The private sovereign aggregate used for peace (P08) also keeps a
    /// kingdom from opening a war against a neighbor it cannot match by
    /// `percent` of that neighbor's strength.
    fn stronger(&self, other: FactionId, percent: u32) -> bool {
        let strength = |faction| {
            crate::engine::diplomacy::military_equivalents(self.campaign, self.data, faction)
        };
        strength(self.owner) * 100 < strength(other) * u128::from(percent)
    }

    /// Boxed in by a neighbor too strong to challenge: build toward parity.
    pub(super) fn outmatched(&self) -> bool {
        !self.nearby_neutral()
            && self
                .neighbors()
                .into_iter()
                .any(|other| self.stronger(other, 100))
    }

    /// Unclaimed land still joins the home border, so there is no need to fight.
    fn nearby_neutral(&self) -> bool {
        !self.frontier().is_empty()
    }
}
