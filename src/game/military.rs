//! Application-owned roster navigation and explicit military command submission.

use super::*;
use kestrum::{
    data::world::SiteId,
    state::military::{ArmyId, FormationId},
};

impl Game {
    pub(super) fn capture_army(&mut self, scene: &str) {
        use kestrum::{data::economy::TroopKind, engine::Actor};
        self.capture_campaign();
        let scene = scene.trim_end_matches("_minimum");
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player)
                .expect("capture starts with the founding army")
                .id;
            if matches!(scene, "army_full" | "recruit_full") {
                for _ in 0..3 {
                    engine::apply(
                        campaign,
                        &self.data,
                        Actor::Player,
                        Command::Recruit {
                            site: SiteId(1),
                            army: Some(army),
                            kind: TroopKind::Warriors,
                        },
                    )
                    .expect("three affordable founding army slots");
                }
            }
            if matches!(scene, "army_empty" | "disband_last") {
                let ids: Vec<_> = campaign.armies[&army].formation_ids().collect();
                for (index, formation) in ids.into_iter().enumerate() {
                    if scene == "disband_last" && index == 0 {
                        continue;
                    }
                    engine::apply(
                        campaign,
                        &self.data,
                        Actor::Player,
                        Command::Disband { formation },
                    )
                    .expect("owned formation can be disbanded");
                }
            }
            if matches!(scene, "army_long_name" | "army_dense") {
                campaign.armies.get_mut(&army).expect("founding army").name =
                    "The Riverward Silver Hawthorn Regiment of the Northern Marches I".into();
                campaign
                    .people
                    .values_mut()
                    .find(|person| person.faction == campaign.player)
                    .expect("founder")
                    .name =
                    "Alexandria of the Silver Hawthorns and Northern River Marches II".into();
            }
            if matches!(scene, "army_economy" | "army_deficit" | "army_dense") {
                if matches!(scene, "army_deficit" | "army_dense") {
                    campaign
                        .factions
                        .get_mut(&campaign.player)
                        .expect("player")
                        .resources
                        .gold = 0;
                    campaign
                        .set_site_control(&self.data, SiteId(1), Some(campaign.player), true)
                        .expect("contested headquarters fixture");
                }
                engine::apply(campaign, &self.data, Actor::Player, Command::EndTurn)
                    .expect("player finishes capture round");
                while !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
                    engine::advance_npc(campaign, &self.data).expect("rivals finish capture round");
                }
            }
        }
        self.open_armies(SiteId(1));
        if matches!(scene, "recruit" | "recruit_blocked" | "recruit_full") {
            let army = self.local_armies().first().copied();
            self.begin_recruit(army);
            self.army.mode = ui::ArmyMode::Recruit {
                army,
                kind: Some(if scene == "recruit_blocked" {
                    TroopKind::Riders
                } else {
                    TroopKind::Warriors
                }),
            };
        }
        if matches!(scene, "disband" | "disband_last") {
            if let Some(formation) = self.army.selected {
                self.army.mode = ui::ArmyMode::Disband(formation);
            }
        }
    }

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
        self.army = ui::ArmyView {
            site: Some(site),
            ..Default::default()
        };
        self.state.overlay = Overlay::Armies;
        self.error = None;
        self.notice = None;
        self.refresh_army();
    }

    fn local_armies(&self) -> Vec<ArmyId> {
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
        if let (Some(site), ui::ArmyMode::Recruit { army, .. }) = (self.army.site, self.army.mode) {
            self.army.options =
                engine::recruit_options(campaign, &self.data, campaign.player, site, army);
        } else {
            self.army.options.clear();
        }
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
                self.army.status = self.data.presentation.text("recruit_success").into();
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
                self.army.status = self.data.presentation.text("disband_success").into();
                self.refresh_army();
            }
            Err(error) => self.army.status = error.to_string(),
        }
    }
}
