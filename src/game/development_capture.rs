//! Four representative K11 captures use ordinary commands for observed outcomes.
use super::*;
use kestrum::{
    data::world::SiteId,
    engine::MoveOrder,
    state::{construction::Focus, military::ArmyId, CampaignPhase},
};
use macroquad_toolkit::ui::text_entry::{TextEdit, TextEntryAction};

impl Game {
    pub(super) fn capture_development_scene(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        if !scene.starts_with("development_") {
            return false;
        }
        self.capture_campaign();
        if scene == "development_threat_report" {
            let campaign = match self.state.campaign.as_mut().expect("capture campaign") {
                Campaign::Strategic(c) => c,
                Campaign::Shell(_) => unreachable!(),
            };
            // Author the expedition's initial adjacent position, then resolve real combat.
            campaign.armies.get_mut(&ArmyId(1)).expect("army").site = SiteId(7);
            campaign
                .validate(&self.data)
                .expect("valid adjacent expedition");
            let id = campaign.active_threat(SiteId(14)).expect("wildlife").id;
            self.invalidate_projection();
            self.open_threat(id);
            self.apply_threat_action(UiAction::ReviewThreat);
            self.apply_threat_action(UiAction::ConfirmThreat);
            assert_eq!(
                self.state.overlay,
                Overlay::Battle,
                "actual threat report: {}",
                self.threat.status
            );
            return true;
        }
        if scene == "development_resettle" {
            self.prepare_resettle_capture();
        }
        if scene == "development_overview" {
            self.state
                .command(
                    &self.data,
                    Command::SetFocus {
                        site: SiteId(1),
                        focus: Focus::Growth,
                    },
                )
                .expect("growth focus");
            for _ in 0..3 {
                complete_round(&mut self.state, &self.data);
            }
        }
        self.open_settlement(SiteId(1));
        match scene {
            "development_overview" => {}
            "development_rename" => {
                self.apply_settlement_action(UiAction::SelectLocalAction(ui::LocalAction::Rename));
                self.edit_place_name(TextEntryAction::Edit(TextEdit::Clear));
                for ch in "Rose Haven – Été Ω".chars() {
                    self.edit_place_name(TextEntryAction::Edit(TextEdit::Insert(ch)));
                }
            }
            "development_resettle" => {
                self.apply_settlement_action(UiAction::SelectLocalAction(
                    ui::LocalAction::Resettle,
                ));
                self.apply_settlement_action(UiAction::SelectResettleDestination(SiteId(6)));
                assert!(
                    self.settlement.blocked.is_none(),
                    "real displaced pool and route: {:?}",
                    self.settlement.blocked
                );
            }
            _ => panic!("unknown development capture {scene}"),
        }
        self.invalidate_projection();
        true
    }

    fn prepare_resettle_capture(&mut self) {
        let campaign = match self.state.campaign.as_mut().expect("campaign") {
            Campaign::Strategic(c) => c,
            Campaign::Shell(_) => unreachable!(),
        };
        campaign.world.site_damage.insert(SiteId(1), 80);
        campaign
            .validate(&self.data)
            .expect("valid damaged starting site");
        complete_round(&mut self.state, &self.data);
        self.state
            .command(
                &self.data,
                Command::Move(MoveOrder {
                    armies: vec![ArmyId(1)],
                    path: vec![SiteId(1), SiteId(5), SiteId(6)],
                }),
            )
            .expect("secure migration route");
    }
}

fn complete_round(state: &mut GameState, data: &GameData) {
    state.command(data, Command::EndTurn).expect("end turn");
    let Campaign::Strategic(campaign) = state.campaign.as_mut().expect("campaign") else {
        unreachable!()
    };
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        engine::advance_npc(campaign, data).expect("actual round");
    }
}
