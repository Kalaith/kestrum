//! Application-owned roster navigation and explicit military command submission.

mod households;
use super::*;
use kestrum::{
    data::world::SiteId,
    state::military::{ArmyId, FormationId},
};

impl Game {
    pub(super) fn open_armies(&mut self, site: SiteId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let owned_site = campaign
            .world
            .site(site)
            .is_some_and(|site| site.controller == Some(campaign.player));
        let own_army = campaign
            .armies
            .values()
            .any(|army| army.faction == campaign.player && army.site == site);
        if !owned_site && !own_army {
            self.error = Some(
                "Your forces and recruiting sites are available from their map markers.".into(),
            );
            return;
        }
        if matches!(
            campaign.phase,
            kestrum::state::CampaignPhase::NpcTurn { paused: false, .. }
        ) {
            self.apply_campaign_command(Command::SetNpcPaused(true));
        }
        self.army = ui::ArmyView {
            site: Some(site),
            ..Default::default()
        };
        self.army.page = self
            .local_armies()
            .iter()
            .position(|id| self.movement.armies.first() == Some(id))
            .unwrap_or(0);
        self.state.overlay = Overlay::Armies;
        self.error = None;
        self.notice = None;
        self.refresh_army();
    }

    pub(super) fn local_armies(&self) -> Vec<ArmyId> {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| {
                campaign
                    .armies
                    .values()
                    .filter(|army| {
                        army.faction == campaign.player && Some(army.site) == self.army.site
                    })
                    .map(|army| army.id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(super) fn refresh_army(&mut self) {
        if self.state.overlay != Overlay::Armies {
            return;
        }
        let armies = self.local_armies();
        self.army.page = self.army.page.min(armies.len().saturating_sub(1));
        let current = armies.get(self.army.page).copied();
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let formation_ids = current
            .and_then(|id| campaign.armies.get(&id))
            .map(|army| army.slots.into_iter().flatten().collect::<Vec<_>>())
            .unwrap_or_default();
        if self
            .army
            .selected
            .is_none_or(|selected| !formation_ids.contains(&selected))
        {
            self.army.selected = formation_ids.first().copied();
        }
        self.army.leadership_permille = current
            .and_then(|id| campaign.army_leadership_permille(id, &self.data))
            .unwrap_or(0);
        self.army.remaining = current
            .and_then(|id| engine::army_remaining(campaign, &self.data, id).ok())
            .unwrap_or(0);
        self.army.supplied = current.is_some_and(|id| campaign.army_is_supplied(id));
        self.army.member_remaining = formation_ids
            .iter()
            .filter_map(|id| {
                engine::formation_remaining(campaign, &self.data, *id)
                    .ok()
                    .map(|remaining| (*id, remaining))
            })
            .collect();
        self.army.person_remaining = campaign
            .people
            .values()
            .filter(|person| person.faction == campaign.player)
            .filter_map(|person| {
                engine::person_remaining(campaign, &self.data, person.id)
                    .ok()
                    .map(|remaining| (person.id, remaining))
            })
            .collect();
        self.army.recovery = engine::recovery_preview(campaign, &self.data, campaign.player)
            .ok()
            .and_then(|entries| {
                entries
                    .into_iter()
                    .find(|entry| Some(entry.formation) == self.army.selected)
            });
        if let (Some(site), ui::ArmyMode::Recruit { army, .. }) = (self.army.site, self.army.mode) {
            self.army.options =
                engine::recruit_options(campaign, &self.data, campaign.player, site, army);
        } else {
            self.army.options.clear();
        }
        self.refresh_transfer();
        self.refresh_household();
    }

    pub(super) fn army_page(&mut self, delta: i32) {
        let count = self.local_armies().len();
        self.army.page = self
            .army
            .page
            .saturating_add_signed(delta as isize)
            .min(count.saturating_sub(1));
        self.army.selected = None;
        self.army.status.clear();
        self.refresh_army();
    }

    pub(super) fn begin_recruit(&mut self, army: Option<ArmyId>) {
        self.army.mode = ui::ArmyMode::Recruit { army, kind: None };
        self.army.status.clear();
        self.refresh_army();
    }

    pub(super) fn confirm_recruit(&mut self) {
        let (
            Some(site),
            ui::ArmyMode::Recruit {
                army,
                kind: Some(kind),
            },
        ) = (self.army.site, self.army.mode)
        else {
            return;
        };
        match self
            .state
            .command(&self.data, Command::Recruit { site, army, kind })
        {
            Ok(outcome) => {
                self.army.mode = ui::ArmyMode::Roster;
                if let Some(recruited) = outcome.recruited {
                    self.army.page = self
                        .local_armies()
                        .iter()
                        .position(|id| *id == recruited.army)
                        .unwrap_or(0);
                    self.army.selected = Some(recruited.formation);
                }
                self.army.status = self.data.game_text.text("recruit_success").into();
                self.refresh_army();
            }
            Err(error) => self.army.status = error.to_string(),
        }
    }

    pub(super) fn confirm_disband(&mut self, formation: FormationId) {
        match self
            .state
            .command(&self.data, Command::Disband { formation })
        {
            Ok(_) => {
                self.army.mode = ui::ArmyMode::Roster;
                self.army.selected = None;
                self.army.status = self.data.game_text.text("disband_success").into();
                self.refresh_army();
            }
            Err(error) => self.army.status = error.to_string(),
        }
    }
}
