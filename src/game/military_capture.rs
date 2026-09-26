//! Durable army and logistics scenes, using real orders in isolated capture state.

use super::*;
use kestrum::{data::world::SiteId, state::military::ArmyId};

impl Game {
    pub(super) fn capture_logistics(&mut self, scene: &str) {
        use kestrum::{
            data::{economy::TroopKind, world::MarkerId},
            engine::Actor,
        };
        self.capture_campaign();
        let mut army = ArmyId(1);
        let mut added = None;
        let scene = scene.trim_end_matches("_minimum");
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            let recruits = if scene == "move_group_dense" {
                7
            } else if scene.starts_with("transfer") || scene == "move_exhausted" {
                1
            } else {
                0
            };
            for _ in 0..recruits {
                added = engine::apply(
                    campaign,
                    &self.data,
                    Actor::Player,
                    Command::Recruit {
                        site: SiteId(1),
                        army: None,
                        kind: TroopKind::Warriors,
                    },
                )
                .expect("affordable separate capture army")
                .recruited
                .map(|result| result.army);
            }
            if scene == "move_exhausted" {
                army = added.expect("new army");
            }
            if matches!(scene, "army_recovery" | "army_recovered" | "army_cutoff") {
                let formation = campaign.armies[&army]
                    .formation_ids()
                    .next()
                    .expect("founding force");
                campaign
                    .formations
                    .get_mut(&formation)
                    .expect("formation")
                    .headcount = 21;
                if scene == "army_cutoff" {
                    campaign
                        .set_site_control(&self.data, SiteId(1), Some(campaign.player), true)
                        .expect("contested supply root");
                }
            }
            if matches!(scene, "army_recovered" | "army_paused") {
                engine::apply(campaign, &self.data, Actor::Player, Command::EndTurn)
                    .expect("capture ends player turn");
                if scene == "army_recovered" {
                    while !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
                        engine::advance_npc(campaign, &self.data).expect("capture completes round");
                    }
                }
            }
        }
        self.open_armies(SiteId(1));
        if scene == "army_people" {
            self.army.mode = ui::ArmyMode::People;
        }
        if matches!(
            scene,
            "army_recovery" | "army_recovered" | "army_cutoff" | "army_paused"
        ) {
            self.army.mode = ui::ArmyMode::Orders;
        }
        if scene.starts_with("move_") {
            self.begin_move(army);
            if scene == "move_group_dense" {
                self.movement.armies = self.local_armies();
            } else if scene != "move_group" {
                self.choose_move_destination();
                self.enter_region(MarkerId(5));
                if scene != "move_map" {
                    self.select_map(kestrum::navigation::MapSelection::Site(
                        if scene == "move_blocked" {
                            SiteId(9)
                        } else {
                            SiteId(6)
                        },
                    ));
                    if scene == "move_review" {
                        self.review_move();
                    }
                }
            }
        } else if scene.starts_with("transfer") {
            let subject = if scene == "transfer_person" {
                ui::TransferSubject::Person(kestrum::state::people::PersonId(1))
            } else {
                ui::TransferSubject::Formation(self.army.selected.expect("founding formation"))
            };
            self.begin_transfer(subject);
            if matches!(scene, "transfer_slots" | "transfer_person") {
                self.army.transfer.army = added;
                if scene == "transfer_slots" {
                    self.army.transfer.slot = Some(1);
                } else if let Some(Campaign::Strategic(campaign)) = &self.state.campaign {
                    self.army.transfer.formation =
                        added.and_then(|army| campaign.armies[&army].formation_ids().next());
                }
            }
            self.refresh_transfer();
        }
    }

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
}
