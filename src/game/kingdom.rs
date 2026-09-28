//! Diplomacy sends ordinary commands; alerts and ending navigation remain presentation state.

use super::*;
use kestrum::{data::world::FactionId, state::diplomacy::DefeatResolution};

impl Game {
    pub(super) fn refresh_kingdom(&mut self) {
        self.kingdom.data = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| {
                let mut view = engine::diplomacy_view(campaign, &self.data, campaign.player);
                let known = engine::known_factions(campaign, campaign.player);
                view.factions.retain(|faction| known.contains(&faction.id));
                view
            });
        let Some(view) = &self.kingdom.data else {
            return;
        };
        if !view
            .factions
            .iter()
            .any(|f| Some(f.id) == self.kingdom.selected)
        {
            self.kingdom.selected = view.factions.first().map(|f| f.id);
        }
        self.kingdom.page = self.kingdom.page.min(
            view.factions
                .len()
                .div_ceil(ui::KINGDOM_PAGE_SIZE)
                .saturating_sub(1),
        );
        self.kingdom.blocked = self.diplomacy_command().and_then(|command| {
            let campaign = self.state.campaign.as_ref()?.strategic()?;
            engine::preview(campaign, &self.data, engine::Actor::Player, command)
                .err()
                .map(|error| error.to_string())
        });
    }

    pub(super) fn kingdom_ended(&self) -> bool {
        self.kingdom
            .data
            .as_ref()
            .is_some_and(|view| view.ending.is_some())
    }

    pub(super) fn kingdom_events(&mut self) {
        if self.state.screen != Screen::Campaign {
            return;
        }
        if self.kingdom_ended() {
            self.movement = ui::MoveView::default();
            if !self.ending_saved {
                self.ending_saved = true;
                self.save_checkpoint();
            }
            if self.state.overlay == Overlay::None {
                self.state.overlay = Overlay::CampaignEnd;
            }
            return;
        }
        self.ending_saved = false;
        let Some(view) = &self.kingdom.data else {
            return;
        };
        let alerts: Vec<_> = view
            .incoming_offers
            .iter()
            .map(|offer| (offer.completed_rounds, offer.proposer, false))
            .chain(
                view.pending_defeats
                    .iter()
                    .map(|defeat| (defeat.completed_rounds, defeat.faction, true)),
            )
            .collect();
        if self.state.overlay == Overlay::None {
            if let Some(alert) = alerts
                .into_iter()
                .find(|alert| !self.diplomacy_seen.contains(alert))
            {
                self.diplomacy_seen.insert(alert);
                self.open_kingdom(Some(alert.1));
            }
        }
    }

    pub(super) fn open_kingdom(&mut self, faction: Option<FactionId>) {
        self.kingdom.intent = None;
        self.kingdom.status.clear();
        if let Some(faction) = faction {
            self.kingdom.selected = Some(faction);
        }
        self.refresh_kingdom();
        if faction.is_none() {
            if let Some(view) = &self.kingdom.data {
                self.kingdom.selected = view
                    .incoming_offers
                    .first()
                    .map(|o| o.proposer)
                    .or_else(|| view.pending_defeats.first().map(|d| d.faction))
                    .or(self.kingdom.selected);
            }
        }
        if let Some(view) = &self.kingdom.data {
            self.diplomacy_seen.extend(
                view.incoming_offers
                    .iter()
                    .filter(|o| Some(o.proposer) == self.kingdom.selected)
                    .map(|o| (o.completed_rounds, o.proposer, false)),
            );
            self.diplomacy_seen.extend(
                view.pending_defeats
                    .iter()
                    .filter(|d| Some(d.faction) == self.kingdom.selected)
                    .map(|d| (d.completed_rounds, d.faction, true)),
            );
        }
        if let Some(index) = self.kingdom.data.as_ref().and_then(|view| {
            view.factions
                .iter()
                .position(|f| Some(f.id) == self.kingdom.selected)
        }) {
            self.kingdom.page = index / ui::KINGDOM_PAGE_SIZE;
        }
        self.state.overlay = Overlay::Kingdom;
        self.error = None;
        self.notice = None;
    }

    pub(super) fn apply_kingdom_action(&mut self, action: UiAction) {
        match action {
            UiAction::OpenKingdom(faction) => self.open_kingdom(faction),
            UiAction::SelectKingdom(faction) => {
                self.kingdom.selected = Some(faction);
                self.kingdom.status.clear();
            }
            UiAction::KingdomPage(delta) => {
                self.kingdom.page = self.kingdom.page.saturating_add_signed(delta as isize)
            }
            UiAction::ReviewDiplomacy(intent) => {
                self.kingdom.intent = Some(intent);
                self.kingdom.status.clear();
            }
            UiAction::ConfirmDiplomacy => self.confirm_diplomacy(),
            UiAction::KingdomBack => self.kingdom_back(),
            _ => unreachable!("kingdom action routing"),
        }
        self.refresh_kingdom();
    }

    pub(super) fn kingdom_back(&mut self) {
        if self.kingdom.intent.take().is_some() {
            self.kingdom.blocked = None;
            self.kingdom.status.clear();
        } else {
            self.state.overlay = if self.kingdom_ended() {
                Overlay::CampaignEnd
            } else {
                Overlay::None
            };
        }
    }

    fn diplomacy_command(&self) -> Option<Command> {
        let faction = self.kingdom.selected?;
        Some(match self.kingdom.intent? {
            ui::KingdomIntent::DeclareWar => Command::DeclareWar { faction },
            ui::KingdomIntent::OfferPeace => Command::OfferPeace { faction },
            ui::KingdomIntent::AcceptPeace => Command::RespondPeace {
                proposer: faction,
                accept: true,
            },
            ui::KingdomIntent::DeclinePeace => Command::RespondPeace {
                proposer: faction,
                accept: false,
            },
            ui::KingdomIntent::Annex => Command::ResolveDefeat {
                faction,
                resolution: DefeatResolution::Annex,
            },
            ui::KingdomIntent::Submission => Command::ResolveDefeat {
                faction,
                resolution: DefeatResolution::Submission,
            },
        })
    }

    fn confirm_diplomacy(&mut self) {
        let Some(command) = self.diplomacy_command() else {
            return;
        };
        match self.state.command(&self.data, command) {
            Ok(outcome) => {
                self.kingdom.status = self.diplomacy_result(&outcome);
                self.handle_campaign_result(Ok(outcome));
                self.kingdom.intent = None;
                self.invalidate_projection();
                self.refresh_kingdom();
                if self.state.overlay != Overlay::SaveRecovery {
                    self.state.overlay = if self.kingdom_ended() {
                        Overlay::CampaignEnd
                    } else {
                        Overlay::Kingdom
                    };
                }
            }
            Err(error) => self.kingdom.status = error.to_string(),
        }
    }

    fn diplomacy_result(&self, outcome: &engine::ActionOutcome) -> String {
        use kestrum::state::{campaign::DomainFactKind, diplomacy::DiplomacyReceipt};
        let key = outcome
            .facts
            .iter()
            .find_map(|fact| match &fact.kind {
                DomainFactKind::DiplomacyChanged {
                    receipt: DiplomacyReceipt::PeaceAgreed { .. },
                } => Some("peace_agreed"),
                DomainFactKind::DiplomacyChanged {
                    receipt: DiplomacyReceipt::PeaceRejected { .. },
                } => Some("peace_rejected"),
                _ => None,
            })
            .unwrap_or("diplomacy_success");
        self.data.presentation.text(key).into()
    }
}
