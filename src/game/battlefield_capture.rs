//! Stable battle showcases use the real resolver and never write campaign saves.

use super::*;
use kestrum::{
    data::{economy::TroopKind, world::PersonClass, GameData},
    engine,
    state::{
        battle::simulation::{
            BattleArmyInput, BattleEvent, BattleLeaderSnapshot, BattleSide, BattleUnitId,
            FormationBattleInput,
        },
        military::{ArmyId, FormationId},
        people::PersonId,
    },
};

impl Game {
    pub(super) fn capture_battlefield(&mut self, requested_scene: &str) -> bool {
        let scene = requested_scene.trim_end_matches("_minimum");
        if matches!(
            scene,
            "battle_campaign_pending"
                | "battle_campaign_leader"
                | "battle_campaign_unavailable"
                | "battle_campaign_doctrine"
                | "battle_campaign_editor_dense"
                | "battle_campaign_aftermath"
        ) {
            return self.capture_campaign_battle(scene);
        }
        if !matches!(
            scene,
            "battle_scene_demo"
                | "battle_scene_dense"
                | "battle_scene_gap"
                | "battle_scene_charge"
                | "battle_scene_brace"
                | "battle_scene_volley"
                | "battle_scene_impact"
                | "battle_scene_rout"
                | "battle_scene_aftermath"
                | "battle_scene_selected"
                | "battle_scene_leader"
        ) {
            return false;
        }

        let mut rules = self.data.battle_tactics.clone();
        if scene == "battle_scene_rout" {
            rules.rout_morale = 70;
            rules.morale_loss_per_casualty = 4;
        }
        let mut input = showcase_input(&self.data);
        if scene == "battle_scene_leader" {
            let rider = input.armies[0].slots[4].as_mut().expect("showcase rider");
            rider.leader = Some(BattleLeaderSnapshot {
                id: PersonId(5),
                name: "Mara of the Western Crossing".into(),
                class: PersonClass::Cavalry,
                active: true,
            });
            rider.capabilities = kestrum::data::battle_tactics::leader_capabilities(
                TroopKind::Riders,
                Some(PersonClass::Cavalry),
            );
            input.armies[1].slots[0] = None;
        }
        let resolution =
            engine::resolve_battle(&input, &rules).expect("battlefield capture fixture resolves");
        self.state.screen = kestrum::state::Screen::Campaign;
        self.state.overlay = Overlay::Battlefield;
        self.battlefield_view = ui::BattlefieldView {
            is_paused: true,
            ..Default::default()
        };
        match scene {
            "battle_scene_gap" => {
                self.battlefield_view.event_cursor = event_after(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::Damage {
                            target: BattleUnitId::Formation(FormationId(8)),
                            remaining: 0,
                            ..
                        }
                    )
                });
            }
            "battle_scene_charge" => {
                self.battlefield_view.event_cursor = event_at(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::Activation {
                            actor: BattleUnitId::Formation(FormationId(5)),
                            action: kestrum::data::battle_tactics::TacticAction::Breakthrough,
                            ..
                        }
                    )
                });
                self.battlefield_view.event_elapsed = 0.24;
            }
            "battle_scene_brace" => {
                self.battlefield_view.event_cursor = event_at(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::Reaction {
                            actor: BattleUnitId::Formation(FormationId(1)),
                            ..
                        }
                    )
                });
                self.battlefield_view.event_elapsed = 0.29;
            }
            "battle_scene_volley" => {
                self.battlefield_view.event_cursor = event_at(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::Activation {
                            actor: BattleUnitId::Formation(FormationId(4)),
                            action: kestrum::data::battle_tactics::TacticAction::Volley,
                            ..
                        }
                    )
                });
                self.battlefield_view.event_elapsed = 0.21;
            }
            "battle_scene_impact" => {
                self.battlefield_view.event_cursor = event_at(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::Damage {
                            target: BattleUnitId::Formation(FormationId(8)),
                            ..
                        }
                    )
                });
                self.battlefield_view.event_elapsed = 0.1;
            }
            "battle_scene_rout" => {
                self.battlefield_view.event_cursor = event_after(&resolution, |event| {
                    matches!(event, BattleEvent::Routed { .. })
                });
            }
            "battle_scene_aftermath" => {
                self.battlefield_view.event_cursor = resolution.events.len();
            }
            "battle_scene_selected" => {
                self.battlefield_view.selected = Some(BattleUnitId::Formation(FormationId(5)));
            }
            "battle_scene_leader" => {
                self.battlefield_view.event_cursor = event_at(&resolution, |event| {
                    matches!(
                        event,
                        BattleEvent::OpeningAction {
                            actor: BattleUnitId::Formation(FormationId(5)),
                            ..
                        }
                    )
                });
                self.battlefield_view.event_elapsed = 0.21;
            }
            _ => {}
        }
        self.battlefield = Some(resolution);
        true
    }

    fn capture_campaign_battle(&mut self, scene: &str) -> bool {
        let mut campaign =
            kestrum::state::StrategicCampaign::new(&self.data).expect("campaign capture fixture");
        campaign.armies.get_mut(&ArmyId(1)).unwrap().site = kestrum::data::world::SiteId(8);
        campaign.armies.get_mut(&ArmyId(3)).unwrap().site = kestrum::data::world::SiteId(10);
        if matches!(
            scene,
            "battle_campaign_editor_dense" | "battle_campaign_aftermath"
        ) {
            campaign.armies.get_mut(&ArmyId(1)).unwrap().name =
                "Rose Ward of the Northern Lantern March beyond the River".into();
            campaign.armies.get_mut(&ArmyId(3)).unwrap().name =
                "Hawthorn Host of the Fallen Pass and Eastern Meadowland".into();
        }
        if scene == "battle_campaign_unavailable" {
            let commander = campaign.armies[&ArmyId(1)]
                .commander
                .expect("founding commander is present");
            engine::apply(
                &mut campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SetBattleLeader {
                    formation: FormationId(1),
                    leader: Some(commander),
                },
            )
            .expect("capture selects the fit founding leader");
            campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = None;
            campaign.people.get_mut(&commander).unwrap().status =
                kestrum::state::people::PersonStatus::Wounded {
                    since_round: 0,
                    remaining_steps: 2,
                };
        }
        campaign.tutorial.dismiss();
        self.state
            .load_campaign(Campaign::Strategic(Box::new(campaign)), &self.data)
            .expect("live battle capture campaign validates");
        let contact = self
            .state
            .command(
                &self.data,
                engine::Command::Move(engine::MoveOrder {
                    armies: vec![ArmyId(1)],
                    path: vec![
                        kestrum::data::world::SiteId(8),
                        kestrum::data::world::SiteId(10),
                    ],
                }),
            )
            .expect("live battle capture reaches the opposing army");
        assert!(contact.battle_pending);
        self.battle_return = None;
        if scene == "battle_campaign_unavailable" {
            let mut tactics = self
                .data
                .battle_tactics
                .defaults_for(TroopKind::Warriors)
                .expect("warrior defaults")
                .clone();
            tactics.activation.insert(
                0,
                kestrum::data::battle_tactics::TacticRule {
                    id: "capture_unavailable_rally".into(),
                    trigger: kestrum::data::battle_tactics::TacticTrigger::Activation,
                    action: kestrum::data::battle_tactics::TacticAction::Rally,
                    condition: kestrum::data::battle_tactics::TacticCondition::Always,
                    target_filter: kestrum::data::battle_tactics::TargetFilter::None,
                    target_priority: kestrum::data::battle_tactics::TargetPriority::OwnColumnFirst,
                },
            );
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().unwrap() else {
                unreachable!("campaign capture is strategic")
            };
            engine::apply(
                campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SetFormationTactics {
                    formation: FormationId(1),
                    tactics,
                },
            )
            .expect("capture records the now-unavailable tactic");
        }
        if scene == "battle_campaign_leader" {
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().unwrap() else {
                unreachable!("campaign capture is strategic")
            };
            let leader = campaign.armies[&ArmyId(1)]
                .commander
                .expect("starting commander is eligible for the lead formation");
            engine::apply(
                campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SetBattleLeader {
                    formation: FormationId(1),
                    leader: Some(leader),
                },
            )
            .expect("leader capture edits the pending battle");
        }
        if scene == "battle_campaign_doctrine" {
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().unwrap() else {
                unreachable!("campaign capture is strategic")
            };
            engine::apply(
                campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SetBattleDoctrine {
                    army: ArmyId(1),
                    doctrine: kestrum::data::battle_tactics::BattleDoctrine::RangedSupport,
                },
            )
            .expect("doctrine capture applies legal snapshots");
            engine::apply(
                campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SaveBattleTemplate {
                    army: ArmyId(1),
                    name: "Northern Screen".into(),
                },
            )
            .expect("doctrine capture saves a personal template");
            self.battlefield_view.template_index = 0;
        }
        if scene == "battle_campaign_editor_dense" {
            let mut tactics = self
                .data
                .battle_tactics
                .defaults_for(TroopKind::Warriors)
                .expect("warrior defaults")
                .clone();
            let mut rows = Vec::new();
            for (index, action) in [
                kestrum::data::battle_tactics::TacticAction::Attack,
                kestrum::data::battle_tactics::TacticAction::Wait,
                kestrum::data::battle_tactics::TacticAction::Guard,
                kestrum::data::battle_tactics::TacticAction::Advance,
                kestrum::data::battle_tactics::TacticAction::Attack,
            ]
            .into_iter()
            .enumerate()
            {
                use kestrum::data::battle_tactics::{
                    TacticCondition, TacticRule, TacticTrigger, TargetFilter, TargetPriority,
                };
                let attack = matches!(action, kestrum::data::battle_tactics::TacticAction::Attack);
                rows.push(TacticRule {
                    id: format!("capture_activation_{index}"),
                    trigger: TacticTrigger::Activation,
                    action,
                    condition: TacticCondition::Always,
                    target_filter: if attack {
                        TargetFilter::AnyEnemy
                    } else {
                        TargetFilter::None
                    },
                    target_priority: TargetPriority::OwnColumnFirst,
                });
            }
            tactics.activation = rows;
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().unwrap() else {
                unreachable!("campaign capture is strategic")
            };
            engine::apply(
                campaign,
                &self.data,
                engine::Actor::Player,
                engine::Command::SetFormationTactics {
                    formation: FormationId(1),
                    tactics,
                },
            )
            .expect("dense tactics capture edits the pending battle");
        }
        if scene == "battle_campaign_aftermath" {
            let resolved = self
                .state
                .command(&self.data, engine::Command::StartPendingBattle)
                .expect("live battle capture accepts the saved encounter");
            self.open_committed_battlefield(resolved.battle.expect("committed capture battle"));
            self.battlefield_view.event_cursor = self
                .battlefield
                .as_ref()
                .map_or(0, |resolution| resolution.events.len());
        } else {
            self.open_pending_battlefield();
            self.battlefield_view.event_cursor = 0;
            self.battlefield_view.selected = Some(BattleUnitId::Formation(FormationId(1)));
        }
        self.battlefield_view.is_paused = true;
        true
    }
}

fn showcase_input(data: &GameData) -> FormationBattleInput {
    let attacker = army(
        1,
        1,
        "Rosemarch Guard",
        BattleSide::Attacker,
        vec![
            unit(data, 1, TroopKind::Spearmen, 100, 18, None, None),
            unit(data, 2, TroopKind::Warriors, 100, 30, Some(24), None),
            unit(data, 3, TroopKind::Warriors, 85, 12, None, None),
            unit(data, 4, TroopKind::Archers, 68, 17, None, None),
            unit(data, 5, TroopKind::Riders, 35, 16, None, None),
            unit(data, 6, TroopKind::Medics, 34, 9, None, None),
        ],
    );
    let defender = army(
        2,
        2,
        "Ironcrest Legion",
        BattleSide::Defender,
        vec![
            unit(data, 7, TroopKind::Riders, 32, 26, None, None),
            unit(data, 8, TroopKind::Warriors, 20, 13, None, None),
            unit(data, 9, TroopKind::Spearmen, 90, 10, None, None),
            unit(data, 10, TroopKind::Archers, 64, 15, None, None),
            unit(data, 11, TroopKind::Archers, 80, 14, None, None),
            unit(data, 12, TroopKind::Medics, 40, 8, None, None),
        ],
    );
    FormationBattleInput {
        terrain_permille: 1100,
        armies: vec![attacker, defender],
    }
}

fn army(
    id: u32,
    faction: u32,
    name: &str,
    side: BattleSide,
    groups: Vec<kestrum::state::battle::simulation::BattleUnitInput>,
) -> BattleArmyInput {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, group) in groups.into_iter().enumerate() {
        slots[slot] = Some(group);
    }
    BattleArmyInput {
        id: ArmyId(id),
        faction: kestrum::data::world::FactionId(faction),
        name: name.into(),
        side,
        slots,
    }
}

fn unit(
    data: &GameData,
    id: u32,
    kind: TroopKind,
    headcount: u32,
    initiative: u32,
    attack: Option<u32>,
    resistance: Option<u32>,
) -> kestrum::state::battle::simulation::BattleUnitInput {
    let stats = &data.troops.formations[&kind];
    let tactics = data
        .battle_tactics
        .defaults_for(kind)
        .expect("all troop defaults");
    kestrum::state::battle::simulation::BattleUnitInput {
        id: BattleUnitId::Formation(FormationId(id)),
        kind: Some(kind),
        headcount,
        capacity: data.economy.formations[&kind].capacity,
        attack: attack.unwrap_or(stats.attack),
        resistance: resistance.unwrap_or(stats.resistance),
        initiative,
        leader: None,
        capabilities: kestrum::data::battle_tactics::leader_capabilities(kind, None),
        activation_tactics: tactics.activation.clone(),
        reaction_tactics: tactics.reaction.clone(),
    }
}

fn event_at(
    resolution: &kestrum::state::battle::simulation::BattleResolution,
    predicate: impl Fn(&BattleEvent) -> bool,
) -> usize {
    resolution
        .events
        .iter()
        .position(predicate)
        .expect("showcase event exists")
}

fn event_after(
    resolution: &kestrum::state::battle::simulation::BattleResolution,
    predicate: impl Fn(&BattleEvent) -> bool,
) -> usize {
    event_at(resolution, predicate) + 1
}
