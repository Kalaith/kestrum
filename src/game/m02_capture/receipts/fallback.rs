//! Completed/pruned work keeps a safe map destination in its dated receipt.

use super::super::super::*;
use crate::ui::NotificationAction;
use kestrum::{
    data::{economy::Resources, world::SiteId},
    engine::{self, Actor, Command},
    navigation::{MapScope, MapSelection},
    state::{
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, OrderId},
        notifications::{
            ConstructionNotificationSnapshot, NotificationDetail, NotificationEntity,
            NotificationId, NotificationKind, NotificationReceipt, NotificationSourceId,
            NotificationSubjectSnapshot,
        },
        CampaignPhase, StrategicCampaign,
    },
};
use macroquad::prelude::Vec2;

const DETAIL_SHOW_MAP: Vec2 = Vec2::new(147.0, 674.0);
const GROUP_PREVIOUS: Vec2 = Vec2::new(96.0, 617.0);
const GROUP_NEXT: Vec2 = Vec2::new(432.0, 617.0);

impl Game {
    pub(in crate::game::m02_capture) fn capture_m02_receipt_fallback(&mut self) -> bool {
        self.capture_campaign();
        self.fund_fallback_work();
        let first_order = self.start_fallback_order(ConstructionKind::Fort);
        self.finish_fallback_seasons(3);
        let first_receipt = self.completed_receipt_for(first_order);
        let first_site = self.fallback_work_snapshot(first_receipt);
        assert_eq!(first_site, SiteId(1));

        let second_order = self.start_fallback_order(ConstructionKind::Facility(
            kestrum::data::world::Facility::Stable,
        ));
        self.finish_fallback_seasons(2);
        let second_receipt = self.completed_receipt_for(second_order);
        {
            let campaign = self.live_fallback_campaign();
            assert!(!campaign.construction.contains_key(&first_order));
            assert!(campaign.construction.contains_key(&second_order));
            assert!(campaign
                .notifications
                .receipt(first_receipt)
                .is_some_and(|receipt| receipt.kind == NotificationKind::ConstructionCompleted));
            campaign
                .notifications
                .validate(&self.data.notifications)
                .expect("actual work receipts remain valid after terminal pruning");
        }

        self.invalidate_projection();
        self.refresh_projection();
        self.refresh_notification_projection();
        let world = &self.projection.as_ref().unwrap().world;
        let marker = world.site(first_site).unwrap().marker;
        let position = world.marker(marker).unwrap().position;
        self.navigation
            .focus_army_site(world, first_site, &mut self.view);
        self.navigation.clear_selection();
        let (minimum_zoom, maximum_zoom) = self.view.zoom_limits();
        self.view
            .focus(position, 2.6_f32.clamp(minimum_zoom, maximum_zoom));
        self.overview_ui.expanded = false;
        let (badge, _) = self.receipt_badge_for(first_receipt);
        let action = self
            .capture_sheet_pointer(badge, Some(badge), true)
            .expect("completed work badge accepts a pointer release");
        assert!(matches!(
            action,
            UiAction::Notification(NotificationAction::OpenMapGroup(_))
        ));
        self.apply(action);
        assert!(self.notifications.is_open);
        assert!(self.notifications.group_ids.contains(&first_receipt));
        assert!(self.notifications.group_ids.contains(&second_receipt));
        self.select_receipt_in_open_group(first_receipt);
        let before = self.view;
        let action = self.capture_sheet_tap(DETAIL_SHOW_MAP);
        assert!(matches!(
            action,
            UiAction::Notification(NotificationAction::ShowOnMap(id)) if id == first_receipt
        ));
        self.apply(action);
        assert!(!self.notifications.is_open);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.navigation.scope(), MapScope::World);
        assert_eq!(
            self.navigation.selection(),
            Some(MapSelection::Marker(
                self.projection
                    .as_ref()
                    .unwrap()
                    .world
                    .site(first_site)
                    .unwrap()
                    .marker
            ))
        );
        assert_ne!(self.view, before, "receipt focuses the saved known target");

        let unknown = self.append_unknown_work_receipt();
        self.invalidate_projection();
        self.refresh_projection();
        self.refresh_notification_projection();
        assert!(ui::map_receipt_group(&self.map_status_input(), unknown).is_empty());
        self.apply(UiAction::Notification(NotificationAction::Open(unknown)));
        assert_eq!(self.notifications.selected_id, Some(unknown));
        let view = self.view;
        let selection = self.navigation.selection();
        assert!(self
            .capture_sheet_pointer(DETAIL_SHOW_MAP, Some(DETAIL_SHOW_MAP), true)
            .is_none());
        self.apply(UiAction::Notification(NotificationAction::ShowOnMap(
            unknown,
        )));
        assert!(self.notifications.is_open);
        assert_eq!(self.view, view);
        assert_eq!(self.navigation.selection(), selection);
        self.apply(UiAction::Notification(NotificationAction::Close));
        self.notice = None;
        true
    }

    fn fund_fallback_work(&mut self) {
        let (data, campaign) = self.fallback_campaign_data_mut();
        campaign
            .factions
            .get_mut(&campaign.player)
            .unwrap()
            .resources = Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        };
        campaign
            .validate(data)
            .expect("funded headquarters fixture remains valid");
    }

    fn start_fallback_order(&mut self, kind: ConstructionKind) -> OrderId {
        let (data, campaign) = self.fallback_campaign_data_mut();
        engine::apply(
            campaign,
            data,
            Actor::Player,
            Command::StartConstruction {
                target: ConstructionTarget::Site(SiteId(1)),
                kind,
                builder: kestrum::state::military::ArmyId(1),
            },
        )
        .expect("ordinary headquarters construction order starts");
        *campaign
            .construction
            .last_key_value()
            .expect("started order is retained")
            .0
    }

    fn finish_fallback_seasons(&mut self, count: usize) {
        for _ in 0..count {
            let (data, campaign) = self.fallback_campaign_data_mut();
            engine::apply(campaign, data, Actor::Player, Command::EndTurn)
                .expect("work season advances through ordinary command");
            for _ in 0..campaign.factions.len() {
                if matches!(campaign.phase, CampaignPhase::PlayerTurn) {
                    break;
                }
                let actor = Actor::Npc(campaign.active_faction());
                engine::apply(campaign, data, actor, Command::EndTurn)
                    .expect("scripted rival pass resolves the phase");
            }
            assert!(matches!(campaign.phase, CampaignPhase::PlayerTurn));
        }
    }

    fn completed_receipt_for(&self, order: OrderId) -> NotificationId {
        self.live_fallback_campaign()
            .notifications
            .receipts
            .iter()
            .find_map(|receipt| {
                (receipt.kind == NotificationKind::ConstructionCompleted
                    && matches!(
                        &receipt.detail,
                        NotificationDetail::Work { work, .. } if work.id == order
                    ))
                .then_some(receipt.id)
            })
            .expect("engine recorded the completed construction outcome")
    }

    fn fallback_work_snapshot(&self, receipt: NotificationId) -> SiteId {
        let record = self
            .live_fallback_campaign()
            .notifications
            .receipt(receipt)
            .expect("completed receipt is saved");
        let NotificationSubjectSnapshot::Construction(work) =
            record.subject.as_ref().expect("work receipt is typed")
        else {
            panic!("construction completion keeps a typed snapshot")
        };
        assert_eq!(
            work.status,
            ConstructionStatus::Completed {
                completed_rounds: record.completed_rounds,
            }
        );
        assert_eq!(work.target, ConstructionTarget::Site(SiteId(1)));
        match work.target {
            ConstructionTarget::Site(site) => site,
            ConstructionTarget::Route(_) => unreachable!("fixture uses site work"),
        }
    }

    fn select_receipt_in_open_group(&mut self, wanted: NotificationId) {
        let ids = self.notifications.group_ids.clone();
        let target = ids
            .iter()
            .position(|id| *id == wanted)
            .expect("completed receipt belongs to its map-place group");
        while self.notifications.group_page < target {
            let action = self.capture_sheet_tap(GROUP_NEXT);
            assert!(matches!(
                action,
                UiAction::Notification(NotificationAction::SelectGroupItem(id))
                    if id == ids[self.notifications.group_page + 1]
            ));
            self.apply(action);
        }
        while self.notifications.group_page > target {
            let action = self.capture_sheet_tap(GROUP_PREVIOUS);
            assert!(matches!(
                action,
                UiAction::Notification(NotificationAction::SelectGroupItem(id))
                    if id == ids[self.notifications.group_page - 1]
            ));
            self.apply(action);
        }
        assert_eq!(self.notifications.selected_id, Some(wanted));
    }

    fn append_unknown_work_receipt(&mut self) -> NotificationId {
        let (data, campaign) = self.fallback_campaign_data_mut();
        let id = NotificationId(campaign.notifications.next_id);
        let order = OrderId(99_999);
        let sequence = campaign.accepted_sequence.max(1);
        let kind = NotificationKind::ConstructionCompleted;
        let rule = data.notifications.kind(kind).unwrap();
        let snapshot = ConstructionNotificationSnapshot {
            id: order,
            owner: campaign.player,
            kind: ConstructionKind::Fort,
            target: ConstructionTarget::Site(SiteId(99_999)),
            progress: 3,
            required_steps: 3,
            status: ConstructionStatus::Completed {
                completed_rounds: campaign.completed_rounds,
            },
            site: None,
            route: None,
        };
        campaign.notifications.receipts.push(NotificationReceipt {
            id,
            source: NotificationSourceId::Transition {
                accepted_sequence: sequence,
                kind,
                subject: NotificationEntity::Construction(order),
                ordinal: 99_999,
            },
            kind,
            priority: rule.default_priority,
            completed_rounds: campaign.completed_rounds,
            occurred_sequence: sequence,
            occurred_order: campaign.notifications.next_order,
            subject: Some(NotificationSubjectSnapshot::Construction(snapshot.clone())),
            detail: NotificationDetail::Work {
                work: snapshot,
                before: Some(ConstructionStatus::Active),
                cause: None,
                forecast_round: None,
            },
            is_read: false,
            is_dismissed: false,
            is_active: false,
            closed_round: None,
        });
        campaign.notifications.next_id += 1;
        campaign.notifications.next_order += 1;
        campaign
            .notifications
            .validate(&data.notifications)
            .expect("unknown typed snapshot remains a valid dated receipt");
        id
    }

    fn live_fallback_campaign(&self) -> &StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("strategic M02 capture")
    }

    fn fallback_campaign_data_mut(&mut self) -> (&GameData, &mut StrategicCampaign) {
        let data = &self.data;
        let campaign = match self.state.campaign.as_mut().expect("strategic M02 capture") {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M02 capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}
