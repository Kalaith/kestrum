//! Threat expeditions submit the same adjacent route command used by simulation.
use super::*;
use kestrum::state::{military::ArmyId, threat::ThreatId};

impl Game {
    pub(super) fn open_threat(&mut self, id: ThreatId) {
        self.threat = ui::ThreatPanel {
            id: Some(id),
            ..Default::default()
        };
        self.state.overlay = Overlay::Threat;
        self.refresh_threat();
        if let Some(army) = self
            .threat
            .view
            .as_ref()
            .and_then(|v| v.armies.iter().find(|a| a.blocked.is_none()))
        {
            self.threat.selected.push(army.id);
        }
        self.refresh_threat();
        self.error = None;
        self.notice = None;
    }

    pub(super) fn apply_threat_action(&mut self, action: UiAction) {
        self.threat.status.clear();
        match action {
            UiAction::OpenThreat(id) => {
                self.open_threat(id);
                return;
            }
            UiAction::ArmyThreats(army) => {
                self.open_army_threats(army);
                return;
            }
            UiAction::ToggleThreatArmy(army) => {
                if self.threat.selected.contains(&army) {
                    self.threat.selected.retain(|id| *id != army);
                } else {
                    self.threat.selected.push(army);
                    self.threat.selected.sort();
                }
            }
            UiAction::ThreatPage(delta) => {
                self.threat.page = self.threat.page.saturating_add_signed(delta as isize)
            }
            UiAction::ReviewThreat => self.threat.stage = ui::ThreatStage::Review,
            UiAction::ConfirmThreat => self.confirm_threat(),
            UiAction::ThreatBack => {
                self.threat_back();
                return;
            }
            _ => unreachable!("threat action dispatch"),
        }
        self.refresh_threat();
    }

    fn open_army_threats(&mut self, army: ArmyId) {
        self.refresh_projection();
        let targets = self
            .projection
            .as_ref()
            .map(|view| {
                let site = view.armies.iter().find(|a| a.id == army).map(|a| a.site);
                view.threats
                    .iter()
                    .filter(|threat| {
                        site.is_some_and(|site| {
                            view.world.adjacent_sites(site).contains(&threat.site)
                        })
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        self.threat = ui::ThreatPanel {
            stage: ui::ThreatStage::Targets,
            targets,
            ..Default::default()
        };
        self.state.overlay = Overlay::Threat;
        self.error = None;
        self.notice = None;
    }

    pub(super) fn refresh_threat(&mut self) {
        if self.state.overlay != Overlay::Threat {
            return;
        }
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(id) = self.threat.id else { return };
        self.threat.view = engine::threat_view(campaign, &self.data, campaign.player, id);
        if let Some(view) = &mut self.threat.view {
            let adjacent = campaign.world.adjacent_sites(view.threat.site);
            view.armies.retain(|army| adjacent.contains(&army.origin));
            self.threat
                .selected
                .retain(|id| view.armies.iter().any(|a| a.id == *id));
            self.threat.page = self.threat.page.min(
                view.armies
                    .len()
                    .div_ceil(ui::THREAT_PAGE_SIZE)
                    .saturating_sub(1),
            );
        }
        match engine::threat_preview(
            campaign,
            &self.data,
            campaign.player,
            &self.threat.selected,
            id,
        ) {
            Ok(preview) => {
                self.threat.preview = Some(preview);
                self.threat.blocked = None;
            }
            Err(error) => {
                self.threat.preview = None;
                self.threat.blocked = Some(error.to_string());
            }
        }
    }

    fn confirm_threat(&mut self) {
        let Some(id) = self.threat.id else { return };
        match self.state.command(
            &self.data,
            Command::ClearThreat {
                armies: self.threat.selected.clone(),
                threat: id,
            },
        ) {
            Ok(outcome) => {
                self.invalidate_projection();
                if outcome.battle_pending {
                    self.open_pending_battlefield();
                }
                if let Some(battle) = outcome.battle {
                    self.open_committed_battlefield(battle);
                }
            }
            Err(error) => self.threat.status = error.to_string(),
        }
    }

    pub(super) fn threat_back(&mut self) {
        if self.threat.stage == ui::ThreatStage::Review {
            self.threat.stage = ui::ThreatStage::Group;
        } else {
            self.state.overlay = Overlay::None;
            self.threat = ui::ThreatPanel::default();
        }
    }
}
