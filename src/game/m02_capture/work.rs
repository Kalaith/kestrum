//! Construction progress, loss of supply and map-target action checks.

use super::super::*;
use kestrum::{
    data::world::SiteId,
    engine::{self, Actor, Command},
    navigation::{MapScope, MapSelection},
    state::{
        construction::{
            ConstructionKind, ConstructionPause, ConstructionStatus, ConstructionTarget,
        },
        notifications::NotificationInbox,
        CampaignPhase, StrategicCampaign,
    },
};
use std::collections::BTreeMap;

impl Game {
    pub(super) fn capture_m02_construction_supply(&mut self) -> bool {
        assert!(self.capture_settlement_scene("settlement_dense"));
        self.apply(UiAction::SettlementBack);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.navigation.scope(), MapScope::World);

        let work_id = self
            .live_m02_campaign()
            .construction
            .values()
            .find(|order| {
                order.target == ConstructionTarget::Site(SiteId(10))
                    && order.kind
                        == ConstructionKind::Facility(kestrum::data::world::Facility::Infirmary)
            })
            .map(|order| order.id)
            .expect("dense fixture starts an Infirmary at Site10");
        let campaign = self.live_m02_campaign();
        assert!(campaign
            .supplied_sites(campaign.player)
            .contains(&SiteId(10)));
        self.advance_m02_season();
        {
            let order = &self.live_m02_campaign().construction[&work_id];
            assert_eq!(order.progress, 1);
            assert_eq!(order.status, ConstructionStatus::Active);
        }

        let cut = self.supply_cut_for(SiteId(10));
        assert!(
            !cut.is_empty(),
            "fixture can sever Site10 supply without losing it"
        );
        {
            let (data, campaign) = self.m02_campaign_data_mut();
            for gate in &cut {
                let player = campaign.player;
                campaign
                    .set_site_control(data, *gate, Some(player), true)
                    .expect("contested gate is a valid resolved-map state");
            }
            assert_eq!(
                campaign.world.site(SiteId(10)).unwrap().controller,
                Some(campaign.player)
            );
            assert!(!campaign
                .supplied_sites(campaign.player)
                .contains(&SiteId(10)));
        }
        self.advance_m02_season();
        {
            let campaign = self.live_m02_campaign();
            let order = &campaign.construction[&work_id];
            assert_eq!(order.progress, 1);
            assert_eq!(
                order.status,
                ConstructionStatus::Paused {
                    reason: ConstructionPause::SupplyLost,
                }
            );
            assert!(campaign.notifications.receipts.iter().any(|receipt| {
                receipt.kind == kestrum::state::notifications::NotificationKind::ConstructionBlocked
                    && matches!(
                        &receipt.detail,
                        kestrum::state::notifications::NotificationDetail::Work { work, .. }
                            if work.id == work_id
                                && work.target == ConstructionTarget::Site(SiteId(10))
                                && work.status == ConstructionStatus::Paused {
                                    reason: ConstructionPause::SupplyLost
                                }
                    )
            }));
        }

        self.keep_m02_place_status_fixture(work_id);
        self.refresh_projection();
        self.refresh_notification_projection();
        self.navigation.focus_army_site(
            &self.projection.as_ref().unwrap().world,
            SiteId(10),
            &mut self.view,
        );
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.navigation
            .select(
                &self.projection.as_ref().unwrap().world,
                MapSelection::Marker(
                    self.projection
                        .as_ref()
                        .unwrap()
                        .world
                        .site(SiteId(10))
                        .unwrap()
                        .marker,
                ),
            )
            .expect("paused site remains selectable on the world map");
        self.audit_paused_work_group_target(work_id);

        let input = self.map_status_input();
        let mut expected = vec![SiteId(10)];
        expected.extend(supply_representatives(input.campaign));
        expected.sort_unstable();
        let targets = ui::map_status_layout(input)
            .control_targets()
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), expected.len());
        let mut observed = targets
            .iter()
            .map(|target| {
                let action = self
                    .capture_sheet_pointer(target.center(), Some(target.center()), true)
                    .expect("map work or supply marker emits an app action");
                let UiAction::OpenSettlement(site) = action else {
                    panic!("work and supply markers open their affected settlement");
                };
                self.apply(action);
                assert_eq!(self.state.overlay, Overlay::Settlement);
                assert_eq!(self.settlement.site, Some(site));
                self.apply(UiAction::SettlementBack);
                assert_eq!(self.state.overlay, Overlay::None);
                site
            })
            .collect::<Vec<_>>();
        observed.sort_unstable();
        assert_eq!(observed, expected);
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.notice = None;
        true
    }

    fn advance_m02_season(&mut self) {
        let (data, campaign) = self.m02_campaign_data_mut();
        engine::apply(campaign, data, Actor::Player, Command::EndTurn)
            .expect("season advances through the real player command");
        for _ in 0..campaign.factions.len() + 1 {
            if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
                break;
            }
            let actor = Actor::Npc(campaign.active_faction());
            engine::apply(campaign, data, actor, Command::EndTurn)
                .expect("scripted rival pass resolves the phase");
        }
        assert!(matches!(campaign.phase, CampaignPhase::PlayerTurn));
        self.invalidate_projection();
        self.refresh_projection();
        self.refresh_notification_projection();
    }

    fn supply_cut_for(&self, target: SiteId) -> Vec<SiteId> {
        let campaign = self.live_m02_campaign();
        let candidates = (5..=14)
            .map(SiteId)
            .filter(|site| {
                *site != target
                    && campaign
                        .world
                        .site(*site)
                        .is_some_and(|place| place.controller == Some(campaign.player))
            })
            .collect::<Vec<_>>();
        for gate in &candidates {
            let mut candidate = campaign.clone();
            if candidate
                .set_site_control(&self.data, *gate, Some(candidate.player), true)
                .is_ok()
                && !candidate.supplied_sites(candidate.player).contains(&target)
            {
                return vec![*gate];
            }
        }
        for first in 0..candidates.len() {
            for second in first + 1..candidates.len() {
                let mut candidate = campaign.clone();
                let player = candidate.player;
                if candidate
                    .set_site_control(&self.data, candidates[first], Some(player), true)
                    .and_then(|()| {
                        candidate.set_site_control(
                            &self.data,
                            candidates[second],
                            Some(player),
                            true,
                        )
                    })
                    .is_ok()
                    && !candidate.supplied_sites(player).contains(&target)
                {
                    return vec![candidates[first], candidates[second]];
                }
            }
        }
        Vec::new()
    }

    fn keep_m02_place_status_fixture(&mut self, work_id: kestrum::state::construction::OrderId) {
        let (data, campaign) = self.m02_campaign_data_mut();
        campaign.construction.retain(|id, _| *id == work_id);
        campaign.notifications = NotificationInbox {
            baseline_complete: true,
            ..Default::default()
        };
        campaign
            .validate(data)
            .expect("focused work-map fixture remains valid");
    }

    fn audit_paused_work_group_target(&mut self, work_id: kestrum::state::construction::OrderId) {
        let original_view = self.view;
        let original_projection = self
            .projection
            .clone()
            .expect("work marker audit has a current map projection");
        let original_notification_projection = self.notification_projection.clone();
        let mut projection = original_projection.clone();
        let paused = projection
            .construction
            .iter()
            .find(|order| order.id == work_id)
            .cloned()
            .expect("paused work remains visible");
        assert!(matches!(
            paused.target,
            ConstructionTarget::Site(SiteId(10))
        ));
        assert!(matches!(
            paused.status,
            ConstructionStatus::Paused {
                reason: ConstructionPause::SupplyLost
            }
        ));
        let mut earlier_active = paused.clone();
        earlier_active.id = kestrum::state::construction::OrderId(0);
        earlier_active.target = ConstructionTarget::Site(SiteId(5));
        earlier_active.kind = ConstructionKind::Outpost;
        earlier_active.progress = 0;
        earlier_active.status = ConstructionStatus::Active;
        assert_eq!(
            projection.world.site(SiteId(5)).unwrap().marker,
            projection.world.site(SiteId(10)).unwrap().marker,
            "synthetic map-only work shares Rosemarch's marker"
        );

        // This is a presentation-only twin: earlier active work at Site5
        // precedes real paused work at Site10 in the same map status group.
        projection.construction = vec![earlier_active, paused];
        projection.supplied_sites.extend(
            projection
                .world
                .sites
                .iter()
                .filter(|site| site.controller == Some(projection.player))
                .map(|site| site.id),
        );
        self.projection = Some(projection);
        self.notification_projection = None;
        if self.view.band() != kestrum::navigation::MapScaleBand::Overview {
            self.apply(UiAction::Overview);
        }
        assert_eq!(
            self.view.band(),
            kestrum::navigation::MapScaleBand::Overview,
            "map-only group audit uses overview aggregation"
        );
        let targets = ui::map_status_layout(self.map_status_input())
            .control_targets()
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), 1, "isolated work group has one map target");
        let target = targets[0].center();
        let action = self
            .capture_sheet_pointer(target, Some(target), true)
            .expect("visible grouped work badge emits an action");
        assert!(matches!(action, UiAction::OpenSettlement(SiteId(10))));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::Settlement);
        assert_eq!(self.settlement.site, Some(SiteId(10)));
        self.apply(UiAction::SettlementBack);
        assert_eq!(self.state.overlay, Overlay::None);

        self.view = original_view;
        self.projection = Some(original_projection);
        self.notification_projection = original_notification_projection;
    }

    fn live_m02_campaign(&self) -> &StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("strategic M02 capture")
    }

    fn m02_campaign_data_mut(&mut self) -> (&GameData, &mut StrategicCampaign) {
        let data = &self.data;
        let campaign = match self.state.campaign.as_mut().expect("strategic M02 capture") {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M02 capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}

fn supply_representatives(campaign: &engine::VisibleCampaign) -> Vec<SiteId> {
    let mut sites = BTreeMap::new();
    for place in &campaign.world.sites {
        if place.controller == Some(campaign.player)
            && campaign.known_sites.contains(&place.id)
            && !campaign.supplied_sites.contains(&place.id)
        {
            sites.entry(place.marker).or_insert(place.id);
        }
    }
    sites.into_values().collect()
}
