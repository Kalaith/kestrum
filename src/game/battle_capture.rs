//! Durable encounter captures run the same movement and battle commands as play.

use super::*;
use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, FactionId, SiteId},
    },
    engine::{Actor, MoveOrder},
    state::{
        military::{ArmyId, FormationId},
        people::PersonId,
        StrategicCampaign,
    },
};

impl Game {
    pub(super) fn capture_battle(&mut self, scene: &str) {
        self.capture_campaign();
        let scene = scene.trim_end_matches("_minimum");
        if scene == "battle_empty" {
            self.open_battle_reports();
            return;
        }
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        // These existing scenes document field combat rather than defended-fort arrival.
        campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == SiteId(3))
            .expect("field fixture site")
            .military = kestrum::data::world::MilitaryLayer::None;
        if matches!(scene, "battle_wounded" | "battle_succession") {
            capture_commander_wound(campaign, &self.data);
            self.open_battle_reports();
            self.battle.tab = ui::BattleTab::People;
            self.battle.page = usize::from(scene == "battle_succession");
            return;
        }
        if matches!(scene, "battle_victory" | "battle_dense") {
            recruit_capture_roster(campaign, &self.data, Actor::Player, ArmyId(1), SiteId(1));
        }
        if scene == "battle_dense" {
            campaign.armies.get_mut(&ArmyId(1)).expect("army").name =
                "The Rose Ward of the Long Western March and Winter Crossing".into();
            campaign.people.values_mut().for_each(|person| {
                person.name = format!(
                    "{} of the Silver Hawthorns and Western Crossing",
                    person.name
                );
            });
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::SplitArmy {
                    formation: FormationId(2),
                },
            )
            .expect("capture split");
        }
        if scene == "battle_defeat" {
            for formation in [FormationId(2), FormationId(3)] {
                engine::apply(
                    campaign,
                    &self.data,
                    Actor::Player,
                    Command::Disband { formation },
                )
                .expect("capture reduced force");
            }
        }
        // Ready recruits before the first march; capture storage never writes saves.
        if matches!(scene, "battle_victory" | "battle_dense") {
            complete_capture_round(campaign, &self.data, scene == "battle_dense");
        }
        march_capture(campaign, &self.data, &[1, 5, 6, 8]);
        complete_capture_round(campaign, &self.data, false);
        march_capture(campaign, &self.data, &[8, 10, 11]);
        if scene == "battle_destroyed" {
            for formation in campaign
                .formations
                .values_mut()
                .filter(|formation| formation.faction == campaign.player)
            {
                formation.headcount = 1;
            }
        }
        march_capture(campaign, &self.data, &[11, 3]);
        assert!(
            !campaign.battles.is_empty(),
            "capture resolves a real battle"
        );
        self.open_battle_reports();
        self.battle.tab = match scene {
            "battle_forces" | "battle_dense" => ui::BattleTab::Forces,
            "battle_people" => ui::BattleTab::People,
            "battle_factors" => ui::BattleTab::Factors,
            _ => ui::BattleTab::Outcome,
        };
    }
}

fn capture_commander_wound(campaign: &mut StrategicCampaign, data: &GameData) {
    let mut successor = campaign.people[&PersonId(1)].clone();
    successor.id = PersonId(5);
    successor.name = "Elian of the Western Crossing".into();
    campaign.people.insert(successor.id, successor);
    campaign.next_ids.person = PersonId(6);
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .expect("capture relation")
        .state = DiplomaticState::War;
    campaign.armies.get_mut(&ArmyId(2)).expect("defender").site = SiteId(5);
    campaign
        .set_site_control(data, SiteId(5), Some(FactionId(2)), false)
        .expect("capture occupied gate");
    campaign.rng.combat = macroquad_toolkit::rng::SeededRng::new(4);
    march_capture(campaign, data, &[1, 5]);
    assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(PersonId(5)));
}

fn recruit_capture_roster(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    actor: Actor,
    army: ArmyId,
    site: SiteId,
) {
    for _ in 0..3 {
        engine::apply(
            campaign,
            data,
            actor,
            Command::Recruit {
                site,
                army: Some(army),
                kind: TroopKind::Warriors,
            },
        )
        .expect("capture six-slot roster");
    }
}

fn complete_capture_round(campaign: &mut StrategicCampaign, data: &GameData, dense: bool) {
    let mut prepare_defenders = dense;
    engine::apply(campaign, data, Actor::Player, Command::EndTurn).expect("capture ends turn");
    while !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
        if prepare_defenders && campaign.active_faction() == FactionId(3) {
            recruit_capture_roster(
                campaign,
                data,
                Actor::Npc(FactionId(3)),
                ArmyId(3),
                SiteId(3),
            );
            engine::apply(
                campaign,
                data,
                Actor::Npc(FactionId(3)),
                Command::SplitArmy {
                    formation: FormationId(8),
                },
            )
            .expect("capture defender split");
            prepare_defenders = false;
        }
        engine::advance_npc(campaign, data).expect("capture rival phase");
    }
}

fn march_capture(campaign: &mut StrategicCampaign, data: &GameData, path: &[u32]) {
    let armies = campaign
        .armies
        .values()
        .filter(|army| army.faction == campaign.player && army.site == SiteId(path[0]))
        .map(|army| army.id)
        .collect();
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies,
            path: path.iter().copied().map(SiteId).collect(),
        }),
    )
    .expect("capture follows physical route");
}
