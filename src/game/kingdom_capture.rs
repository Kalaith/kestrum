//! Four political scenes derive decisions and endings from ordinary campaign commands.

use super::*;
use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, FactionId, MilitaryLayer, SiteId},
    },
    engine::{Actor, MoveOrder},
    state::{diplomacy::DefeatResolution, military::ArmyId, CampaignPhase, StrategicCampaign},
};

impl Game {
    pub(super) fn capture_kingdom_scene(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        if !scene.starts_with("diplomacy_") {
            return false;
        }
        self.capture_campaign();
        if scene == "diplomacy_relations" {
            self.open_kingdom(Some(FactionId(2)));
            return true;
        }
        let Campaign::Strategic(campaign) = self.state.campaign.as_mut().expect("campaign") else {
            unreachable!()
        };
        if scene == "diplomacy_peace_response" {
            declare_if_peace(campaign, &self.data, FactionId(2));
            order(campaign, &self.data, Actor::Player, Command::EndTurn);
            order(
                campaign,
                &self.data,
                Actor::Npc(FactionId(2)),
                Command::OfferPeace {
                    faction: FactionId(1),
                },
            );
            self.invalidate_projection();
            self.open_kingdom(Some(FactionId(2)));
            return true;
        }
        prepare_conquest(campaign, &self.data);
        march(campaign, &self.data, &[1, 5, 2]);
        assert!(
            campaign
                .diplomacy
                .pending_defeats
                .iter()
                .any(|d| d.faction == FactionId(2)),
            "actual Oak defeat"
        );
        if scene == "diplomacy_defeat_choice" {
            self.invalidate_projection();
            self.open_kingdom(Some(FactionId(2)));
            return true;
        }
        assert_eq!(scene, "diplomacy_ending_reload");
        resolve(campaign, &self.data, FactionId(2), DefeatResolution::Annex);
        round(campaign, &self.data);
        march(campaign, &self.data, &[2, 5, 6, 8]);
        round(campaign, &self.data);
        march(campaign, &self.data, &[8, 9]);
        round(campaign, &self.data);
        march(campaign, &self.data, &[9, 10, 11]);
        round(campaign, &self.data);
        march(campaign, &self.data, &[11, 3]);
        resolve(campaign, &self.data, FactionId(3), DefeatResolution::Annex);
        round(campaign, &self.data);
        march(campaign, &self.data, &[3, 11, 4]);
        resolve(
            campaign,
            &self.data,
            FactionId(4),
            DefeatResolution::Submission,
        );
        assert!(campaign.diplomacy.ending.is_some(), "real conquest ending");
        let bytes = serde_json::to_string(self.state.campaign.as_ref().expect("campaign"))
            .expect("serialize terminal campaign");
        let restored = serde_json::from_str::<Campaign>(&bytes).expect("read terminal campaign");
        self.finish_load(Ok(restored));
        self.kingdom_events();
        assert_eq!(self.state.overlay, Overlay::CampaignEnd);
        self.notice = None;
        true
    }
}

fn prepare_conquest(campaign: &mut StrategicCampaign, data: &GameData) {
    // This field-conquest fixture keeps full-strength authored armies and ordinary recruitment.
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(3))
        .expect("Hawthorn")
        .military = MilitaryLayer::None;
    campaign
        .validate(data)
        .expect("valid conquest starting position");
    for id in [FactionId(2), FactionId(3), FactionId(4)] {
        declare_if_peace(campaign, data, id);
    }
    for _ in 0..3 {
        order(
            campaign,
            data,
            Actor::Player,
            Command::Recruit {
                site: SiteId(1),
                army: Some(ArmyId(1)),
                kind: TroopKind::Warriors,
            },
        );
    }
    round(campaign, data);
}

fn declare_if_peace(campaign: &mut StrategicCampaign, data: &GameData, faction: FactionId) {
    if campaign
        .relations
        .iter()
        .any(|r| r.factions == [campaign.player, faction] && r.state == DiplomaticState::Peace)
    {
        order(
            campaign,
            data,
            Actor::Player,
            Command::DeclareWar { faction },
        );
    }
}

fn march(campaign: &mut StrategicCampaign, data: &GameData, path: &[u32]) {
    order(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: path.iter().copied().map(SiteId).collect(),
        }),
    );
    assert_eq!(
        campaign.armies[&ArmyId(1)].site,
        SiteId(*path.last().expect("destination"))
    );
}

fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    resolution: DefeatResolution,
) {
    order(
        campaign,
        data,
        Actor::Player,
        Command::ResolveDefeat {
            faction,
            resolution,
        },
    );
}

fn round(campaign: &mut StrategicCampaign, data: &GameData) {
    order(campaign, data, Actor::Player, Command::EndTurn);
    while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
        // Stable legal passes isolate seasonal advancement from the evolving AI policy.
        order(campaign, data, Actor::Npc(faction), Command::EndTurn);
    }
}

fn order(campaign: &mut StrategicCampaign, data: &GameData, actor: Actor, command: Command) {
    let label = format!("{command:?}");
    engine::apply(campaign, data, actor, command)
        .unwrap_or_else(|error| panic!("ordinary capture command {label}: {error}"));
}
