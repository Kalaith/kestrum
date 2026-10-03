//! Local actions keep stable place identity and use the atomic command preview.
use super::*;
use macroquad_toolkit::ui::text_entry::{apply_text_edit, TextEntryAction};

impl Game {
    pub(in crate::game) fn apply_local_action(&mut self, action: UiAction) -> bool {
        match action {
            UiAction::SelectLocalAction(action) => {
                self.settlement.local_action = Some(action);
                self.settlement.destination = None;
                self.settlement.page = 0;
                self.settlement.status.clear();
                self.settlement.mode = match action {
                    ui::LocalAction::Rename => {
                        self.settlement.name = self
                            .state
                            .campaign
                            .as_ref()
                            .and_then(Campaign::strategic)
                            .and_then(|c| c.world.site(self.settlement.site?))
                            .map(|s| s.name.clone())
                            .unwrap_or_default();
                        SettlementMode::Rename
                    }
                    ui::LocalAction::Resettle => SettlementMode::Resettle,
                    _ => SettlementMode::LocalReview,
                };
            }
            UiAction::SelectResettleDestination(site) => {
                self.settlement.destination = Some(site);
                self.settlement.mode = SettlementMode::LocalReview;
            }
            UiAction::EditPlaceName(action) => self.edit_place_name(action),
            UiAction::ConfirmLocalAction => {
                if let Some(command) = local_command(&self.settlement) {
                    let success =
                        if self.settlement.local_action == Some(ui::LocalAction::DevelopCity) {
                            "city_development_success"
                        } else {
                            "local_action_success"
                        };
                    self.settlement_command(command, success);
                }
            }
            _ => return false,
        }
        self.refresh_settlement();
        true
    }

    pub(in crate::game) fn edit_place_name(&mut self, action: TextEntryAction) {
        if self.state.overlay != Overlay::Settlement
            || self.settlement.mode != SettlementMode::Rename
        {
            return;
        }
        match action {
            TextEntryAction::SetPage(page) => self.settlement.keyboard_page = page,
            TextEntryAction::Edit(edit) => {
                self.settlement.status = apply_text_edit(&mut self.settlement.name, edit, 40)
                    .err()
                    .map_or_else(String::new, |error| error.to_string());
            }
        }
        self.refresh_development();
    }

    pub(in crate::game) fn refresh_development(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(site) = self.settlement.site else {
            return;
        };
        self.settlement.development =
            engine::development_view(campaign, &self.data, campaign.player, site);
        self.settlement.city_development =
            engine::city_development_option(campaign, &self.data, campaign.player, site);
        if matches!(
            self.settlement.mode,
            SettlementMode::Rename | SettlementMode::Resettle | SettlementMode::LocalReview
        ) {
            self.settlement.blocked = projected_blocker(&self.settlement).or_else(|| {
                local_command(&self.settlement).and_then(|command| {
                    engine::preview(campaign, &self.data, engine::Actor::Player, command)
                        .err()
                        .map(|e| e.to_string())
                })
            });
        }
        self.settlement.destinations = self
            .settlement
            .development
            .as_ref()
            .map(|view| {
                view.resettle_targets
                    .iter()
                    .filter_map(|id| {
                        Some(ui::LocalDestination {
                            site: *id,
                            name: campaign.world.site(*id)?.name.clone(),
                            blocked: engine::preview(
                                campaign,
                                &self.data,
                                engine::Actor::Player,
                                Command::Resettle {
                                    from: site,
                                    to: *id,
                                },
                            )
                            .err()
                            .map(|e| e.to_string()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
    }
}

fn projected_blocker(view: &ui::SettlementView) -> Option<String> {
    if view.local_action == Some(ui::LocalAction::DevelopCity) {
        return view
            .city_development
            .as_ref()
            .and_then(|option| option.blocked.clone());
    }
    let development = view.development.as_ref()?;
    match view.local_action? {
        ui::LocalAction::MoveCapital => development.capital_blocked.clone(),
        ui::LocalAction::RelocateHeadquarters => development.headquarters_blocked.clone(),
        _ => None,
    }
}

fn local_command(view: &ui::SettlementView) -> Option<Command> {
    let site = view.site?;
    Some(match view.local_action? {
        ui::LocalAction::Rename => Command::RenameSite {
            site,
            name: view.name.clone(),
        },
        ui::LocalAction::Resettle => Command::Resettle {
            from: site,
            to: view.destination?,
        },
        ui::LocalAction::DevelopCity => Command::DevelopCity { site },
        ui::LocalAction::MoveCapital => Command::MoveCapital { site },
        ui::LocalAction::RelocateHeadquarters => Command::RelocateHeadquarters { site },
    })
}
