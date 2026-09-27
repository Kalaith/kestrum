//! Strategic command pacing and stable save boundaries for the application.

use super::*;

pub use kestrum::state::STRATEGIC_SAVE_SLOT as STRATEGIC_SLOT;

impl Game {
    pub(super) fn capture_campaign(&mut self) {
        if let Err(error) = self.state.new_game(&self.data) {
            self.error = Some(error);
        }
    }

    pub(super) fn start_game(&mut self) {
        let result = (|| {
            let mut campaign = kestrum::state::StrategicCampaign::new_production(
                &self.data,
                &self.setup.campaign_setup(),
            )?;
            if !self.capture {
                let library = self
                    .library
                    .as_mut()
                    .ok_or_else(|| self.saves.status.clone())?;
                let store = self
                    .storage
                    .as_mut()
                    .ok_or_else(|| self.saves.status.clone())?;
                library.refresh(store, &self.data)?;
                campaign.campaign_id = library.allocate_campaign_id(store)?;
            }
            self.state
                .load_campaign(Campaign::Strategic(Box::new(campaign)), &self.data)
        })();
        match result {
            Ok(()) => {
                self.navigation.reset(&mut self.view);
                self.army = ui::ArmyView::default();
                self.movement = ui::MoveView::default();
                self.battle = ui::BattleView::default();
                self.reset_history();
                self.kingdom = ui::KingdomView::default();
                self.diplomacy_seen.clear();
                self.ending_saved = false;
                self.invalidate_projection();
                self.error = None;
                self.npc_delay = 0.0;
                self.save_checkpoint();
            }
            Err(error) => {
                self.error = Some(error);
                self.open_saves(false);
            }
        }
    }

    pub(super) fn apply_campaign_command(&mut self, command: Command) {
        let result = self.state.command(&self.data, command);
        self.handle_campaign_result(result);
        self.npc_delay = 0.0;
    }

    pub(super) fn handle_campaign_result(
        &mut self,
        result: Result<engine::ActionOutcome, engine::RuleError>,
    ) {
        match result {
            Ok(outcome) => {
                let mut messages = Vec::new();
                if !outcome.automatic_retirements.is_empty() {
                    let names = self
                        .state
                        .campaign
                        .as_ref()
                        .and_then(Campaign::strategic)
                        .map(|campaign| {
                            outcome
                                .automatic_retirements
                                .iter()
                                .filter_map(|id| {
                                    campaign.people.get(id).map(|person| person.name.as_str())
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    let message = self
                        .data
                        .presentation
                        .text("automatic_retirement_notice")
                        .replace("{names}", &names);
                    messages.push(message);
                }
                if !outcome.new_people.is_empty() {
                    let names = self
                        .state
                        .campaign
                        .as_ref()
                        .and_then(Campaign::strategic)
                        .map(|campaign| {
                            outcome
                                .new_people
                                .iter()
                                .filter_map(|id| {
                                    campaign.people.get(id).map(|person| person.name.as_str())
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    messages.push(
                        self.data
                            .presentation
                            .text("new_people_notice")
                            .replace("{names}", &names),
                    );
                }
                if !outcome.succession.is_empty() {
                    if let Some(campaign) =
                        self.state.campaign.as_ref().and_then(Campaign::strategic)
                    {
                        for succession in &outcome.succession {
                            let predecessor = campaign
                                .people
                                .get(&succession.predecessor)
                                .map_or_else(|| "".to_owned(), |person| person.name.clone());
                            let army = campaign
                                .armies
                                .get(&succession.army)
                                .map_or_else(|| "".to_owned(), |entry| entry.name.clone());
                            if let Some(successor) =
                                succession.successor.and_then(|id| campaign.people.get(&id))
                            {
                                messages.push(
                                    self.data
                                        .presentation
                                        .text("succession_notice")
                                        .replace("{predecessor}", &predecessor)
                                        .replace("{successor}", &successor.name)
                                        .replace("{army}", &army),
                                );
                            } else {
                                messages.push(
                                    self.data
                                        .presentation
                                        .text("vacant_succession_notice")
                                        .replace("{predecessor}", &predecessor)
                                        .replace("{army}", &army),
                                );
                            }
                        }
                    }
                }
                if let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) {
                    if !outcome.legacy_items_changed.is_empty() {
                        let names = outcome
                            .legacy_items_changed
                            .iter()
                            .filter_map(|id| campaign.legacy_items.get(id))
                            .map(|item| item.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if !names.is_empty() {
                            messages.push(
                                self.data
                                    .presentation
                                    .text("legacy_item_transfer_notice")
                                    .replace("{names}", &names),
                            );
                        }
                    }
                    for subject in &outcome.anniversary_reminders {
                        let (name, years, kind) = match subject {
                            kestrum::state::history::AnniversarySubject::Person(id) => {
                                let Some(person) = campaign.people.get(id) else {
                                    continue;
                                };
                                (
                                    person.name.clone(),
                                    campaign.history.person_last_reminded[id],
                                    "history_service_anniversary",
                                )
                            }
                            kestrum::state::history::AnniversarySubject::Site(id) => {
                                let Some(site) = campaign.world.site(*id) else {
                                    continue;
                                };
                                (
                                    site.name.clone(),
                                    campaign.history.site_last_reminded[id],
                                    "history_foundation_anniversary",
                                )
                            }
                        };
                        messages.push(
                            self.data
                                .presentation
                                .text("history_anniversary_notice")
                                .replace("{kind}", self.data.presentation.text(kind))
                                .replace("{name}", &name)
                                .replace("{years}", &years.to_string()),
                        );
                    }
                }
                if !messages.is_empty() {
                    self.notice = Some((messages.join(" "), 5.0));
                }
                if outcome.round_completed
                    && self
                        .state
                        .campaign
                        .as_ref()
                        .and_then(Campaign::strategic)
                        .is_some_and(|campaign| campaign.diplomacy.ending.is_none())
                {
                    self.save_checkpoint();
                }
                if outcome
                    .battle
                    .is_some_and(|battle| self.witnessed_battle(battle))
                    && self.state.overlay != Overlay::SaveRecovery
                {
                    self.open_battle_reports();
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn progress_npcs(&mut self, dt: f32) {
        if self.capture
            || self.error.is_some()
            || self.state.screen != Screen::Campaign
            || self.state.overlay != Overlay::None
        {
            self.npc_delay = 0.0;
            return;
        }
        let ready = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(|campaign| {
                !campaign.diplomacy.is_blocked()
                    && matches!(
                        campaign.phase,
                        kestrum::state::campaign::CampaignPhase::NpcTurn { paused: false, .. }
                    )
            });
        if !ready {
            self.npc_delay = 0.0;
            return;
        }
        self.npc_delay += dt;
        // Presentation pacing only; each engine command remains an atomic, deterministic step.
        if self.npc_delay >= self.data.presentation.npc_action_delay_seconds {
            self.npc_delay = 0.0;
            let result = self.state.advance_npc(&self.data);
            self.handle_campaign_result(result);
        }
    }
}
