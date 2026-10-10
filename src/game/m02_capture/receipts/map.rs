//! Receipt grouping, delivery suppression and tap-driven map details.

use super::super::super::*;
use crate::ui::NotificationAction;
use kestrum::{
    data::world::SiteId,
    engine::AttentionKind,
    navigation::MapScope,
    state::notifications::{
        NotificationDelivery, NotificationDetail, NotificationEntity, NotificationId,
        NotificationInbox, NotificationKind, NotificationReceipt, NotificationSourceId,
        NotificationSubjectSnapshot, PlaceNotificationSnapshot,
    },
};
use macroquad::prelude::{vec2, Vec2};

const GROUP_PREVIOUS: Vec2 = Vec2::new(96.0, 617.0);
const GROUP_NEXT: Vec2 = Vec2::new(432.0, 617.0);
const DETAIL_DISMISS: Vec2 = Vec2::new(147.0, 732.0);

impl Game {
    pub(in crate::game::m02_capture) fn capture_m02_receipts_map(&mut self) -> bool {
        assert!(self.capture_overview("overview_urgent"));
        let (urgent, information, separate) = self.prepare_map_receipts();
        assert!(self.overview.as_ref().is_some_and(|overview| overview
            .attention
            .iter()
            .any(|entry| { entry.kind == AttentionKind::LocalThreat })));
        assert_eq!(self.navigation.scope(), MapScope::World);

        let (badge, badge_id) = self.receipt_badge_for(urgent);
        let group = ui::map_receipt_group(&self.map_status_input(), urgent);
        assert_eq!(group, vec![urgent, information]);
        assert_eq!(
            badge_id, urgent,
            "urgent receipt leads its shared-place group"
        );
        let input = self.map_status_input();
        assert!(ui::map_status_control_contains(badge, input));
        assert!(self.map_controls_block(badge));
        self.apply(UiAction::MapKey);
        assert!(ui::map_status_layout(self.map_status_input())
            .control_targets()
            .next()
            .is_none());
        assert!(!ui::map_status_control_contains(
            badge,
            self.map_status_input()
        ));
        assert!(!matches!(
            self.capture_sheet_pointer(badge, Some(badge), true),
            Some(UiAction::Notification(NotificationAction::OpenMapGroup(_)))
        ));
        self.apply(UiAction::Back);
        assert_eq!(self.state.overlay, Overlay::None);
        assert!(self.capture_sheet_pointer(badge, None, true).is_none());
        assert!(self
            .capture_sheet_pointer(badge + vec2(400.0, 0.0), Some(badge), true)
            .is_none());

        let action = self
            .capture_sheet_pointer(badge, Some(badge), true)
            .expect("receipt badge accepts its matching pointer release");
        assert!(matches!(
            action,
            UiAction::Notification(NotificationAction::OpenMapGroup(id)) if id == urgent
        ));
        self.apply(action);
        assert!(self.notifications.is_open);
        assert_eq!(self.notifications.selected_id, Some(urgent));
        assert_eq!(self.notifications.group_ids, group);
        assert!(self
            .live_receipt_campaign()
            .notifications
            .receipt(urgent)
            .is_some_and(|receipt| receipt.is_read));

        let next = self.capture_sheet_tap(GROUP_NEXT);
        assert!(matches!(
            next,
            UiAction::Notification(NotificationAction::SelectGroupItem(id))
                if id == information
        ));
        self.apply(next);
        assert_eq!(self.notifications.selected_id, Some(information));
        let previous = self.capture_sheet_tap(GROUP_PREVIOUS);
        assert!(matches!(
            previous,
            UiAction::Notification(NotificationAction::SelectGroupItem(id))
                if id == urgent
        ));
        self.apply(previous);
        assert_eq!(self.notifications.selected_id, Some(urgent));
        let dismiss = self.capture_sheet_tap(DETAIL_DISMISS);
        assert!(matches!(
            dismiss,
            UiAction::Notification(NotificationAction::Dismiss(id)) if id == urgent
        ));
        self.apply(dismiss);
        assert!(self
            .live_receipt_campaign()
            .notifications
            .receipt(urgent)
            .is_some_and(|receipt| receipt.is_dismissed));
        self.apply(UiAction::Notification(NotificationAction::Close));

        self.audit_hidden_receipt(separate);
        self.apply(UiAction::Notification(NotificationAction::AllOff));
        for id in [urgent, information, separate] {
            assert!(ui::map_receipt_group(&self.map_status_input(), id).is_empty());
        }
        assert!(self
            .notification_projection
            .as_ref()
            .unwrap()
            .map_receipt_ids
            .is_empty());
        assert!(self.overview.as_ref().is_some_and(|overview| overview
            .attention
            .iter()
            .any(|entry| { entry.kind == AttentionKind::LocalThreat })));
        assert!(self
            .projection
            .as_ref()
            .is_some_and(|projection| !projection.threats.is_empty()));
        self.apply(UiAction::Notification(NotificationAction::RestoreDefaults));
        self.prepare_map_receipts();
        self.focus_receipt_place(urgent);
        self.notifications.clear_card();
        self.navigation.clear_selection();
        self.overview_ui.expanded = true;
        self.notice = None;
        true
    }

    fn prepare_map_receipts(&mut self) -> (NotificationId, NotificationId, NotificationId) {
        self.preferences.notifications.restore_defaults();
        for kind in [
            NotificationKind::ControlLost,
            NotificationKind::HabitationChanged,
            NotificationKind::SiteReclaimed,
        ] {
            assert_eq!(
                self.data.notifications.kind(kind).unwrap().default_delivery,
                NotificationDelivery::TopBarAndHistory
            );
            self.preferences
                .notifications
                .choices
                .insert(kind, NotificationDelivery::TopBarAndHistory);
        }
        let preference_snapshot = self.preferences.notifications.clone();
        let (urgent, information, separate, home, other) = {
            let (data, campaign) = self.live_receipt_campaign_data_mut();
            let home = campaign.factions[&campaign.player].headquarters;
            let home_marker = campaign.world.site(home).unwrap().marker;
            let home_position = campaign.world.marker(home_marker).unwrap().position;
            let known_sites = engine::explored_sites(campaign, campaign.player);
            let other = known_sites
                .into_iter()
                .filter_map(|site| {
                    let place = campaign.world.site(site)?;
                    let marker = campaign.world.marker(place.marker)?;
                    (place.marker != home_marker).then_some((site, marker.position))
                })
                .max_by(|(_, left), (_, right)| {
                    let distance = |point: [f32; 2]| {
                        (point[0] - home_position[0]).powi(2)
                            + (point[1] - home_position[1]).powi(2)
                    };
                    distance(*left).total_cmp(&distance(*right))
                })
                .map(|(site, _)| site)
                .expect("urgent map fixture has a second known place");
            campaign.notifications = NotificationInbox {
                baseline_complete: true,
                ..Default::default()
            };
            campaign.notifications.preference_snapshot = preference_snapshot;
            let urgent = append_place_receipt(
                campaign,
                data,
                home,
                MapReceiptSample {
                    kind: NotificationKind::ControlLost,
                    ordinal: 1,
                    before: "village",
                    after: "foreign",
                    cause: "control_lost",
                },
            );
            let information = append_place_receipt(
                campaign,
                data,
                home,
                MapReceiptSample {
                    kind: NotificationKind::HabitationChanged,
                    ordinal: 2,
                    before: "village",
                    after: "town",
                    cause: "population_moved",
                },
            );
            let separate = append_place_receipt(
                campaign,
                data,
                other,
                MapReceiptSample {
                    kind: NotificationKind::SiteReclaimed,
                    ordinal: 3,
                    before: "ruined",
                    after: "reclaimed",
                    cause: "reclaimed",
                },
            );
            campaign
                .notifications
                .validate(&data.notifications)
                .expect("map receipt fixture satisfies inbox rules");
            let saved = serde_json::to_vec(campaign).expect("campaign saves map receipt snapshots");
            let restored: kestrum::state::StrategicCampaign =
                serde_json::from_slice(&saved).expect("saved receipts reload without replay");
            assert_eq!(restored.accepted_sequence, campaign.accepted_sequence);
            assert_eq!(restored.notifications, campaign.notifications);
            (urgent, information, separate, home, other)
        };
        self.invalidate_projection();
        self.refresh_projection();
        self.refresh_notification_projection();
        self.navigation.focus_army_site(
            &self.projection.as_ref().unwrap().world,
            home,
            &mut self.view,
        );
        self.navigation.clear_selection();
        self.overview_ui.expanded = false;
        assert_ne!(
            self.projection
                .as_ref()
                .unwrap()
                .world
                .site(home)
                .unwrap()
                .marker,
            self.projection
                .as_ref()
                .unwrap()
                .world
                .site(other)
                .unwrap()
                .marker
        );
        let projection = self.notification_projection.as_ref().unwrap();
        assert!(projection.map_receipt_ids.contains(&urgent));
        assert!(projection.map_receipt_ids.contains(&information));
        assert!(projection.map_receipt_ids.contains(&separate));
        (urgent, information, separate)
    }

    fn audit_hidden_receipt(&mut self, id: NotificationId) {
        self.focus_receipt_place(id);
        let (center, _) = self.receipt_badge_for(id);
        let visible = self.map_status_input();
        assert!(ui::map_status_control_contains(center, visible));
        self.apply(UiAction::Notification(NotificationAction::SetDelivery(
            NotificationKind::SiteReclaimed,
            NotificationDelivery::HistoryOnly,
        )));
        let input = self.map_status_input();
        assert!(!ui::map_status_control_contains(center, input));
        assert!(self
            .notification_projection
            .as_ref()
            .unwrap()
            .recent
            .iter()
            .any(|item| item.id == id));
        assert!(!self
            .notification_projection
            .as_ref()
            .unwrap()
            .map_receipt_ids
            .contains(&id));
        assert!(self
            .capture_sheet_pointer(center, Some(center), true)
            .is_none());
        self.apply(UiAction::Notification(NotificationAction::OpenMapGroup(id)));
        assert!(!self.notifications.is_open);
        assert!(ui::map_receipt_group(&self.map_status_input(), id).is_empty());

        self.apply(UiAction::Notification(NotificationAction::SetDelivery(
            NotificationKind::SiteReclaimed,
            NotificationDelivery::Off,
        )));
        let input = self.map_status_input();
        assert!(!ui::map_status_control_contains(center, input));
        assert!(!self
            .notification_projection
            .as_ref()
            .unwrap()
            .recent
            .iter()
            .any(|item| item.id == id));
        assert!(self
            .capture_sheet_pointer(center, Some(center), true)
            .is_none());
        self.apply(UiAction::Notification(NotificationAction::OpenMapGroup(id)));
        assert!(!self.notifications.is_open);
        assert!(ui::map_receipt_group(&self.map_status_input(), id).is_empty());
    }

    fn focus_receipt_place(&mut self, id: NotificationId) {
        let site = self
            .live_receipt_campaign()
            .notifications
            .receipt(id)
            .and_then(|receipt| receipt.subject.as_ref())
            .and_then(|subject| match subject {
                NotificationSubjectSnapshot::Place(place) => Some(place.id),
                _ => None,
            })
            .expect("map capture receipt keeps a typed place subject");
        let world = &self.projection.as_ref().unwrap().world;
        let position = world
            .site(site)
            .and_then(|place| world.marker(place.marker))
            .expect("receipt place remains known on the map")
            .position;
        self.navigation.focus_army_site(world, site, &mut self.view);
        self.navigation.clear_selection();
        let (minimum_zoom, maximum_zoom) = self.view.zoom_limits();
        self.view
            .focus(position, 2.6_f32.clamp(minimum_zoom, maximum_zoom));
        self.overview_ui.expanded = false;
    }

    pub(super) fn receipt_badge_for(&self, wanted: NotificationId) -> (Vec2, NotificationId) {
        let group = ui::map_receipt_group(&self.map_status_input(), wanted);
        assert!(!group.is_empty(), "receipt has a live map badge");
        let layout = ui::map_status_layout(self.map_status_input());
        let targets = layout
            .control_targets()
            .zip(layout.control_bounds())
            .collect::<Vec<_>>();
        let mut attempts = Vec::new();
        for (target, bounds) in targets {
            for point in [target.center(), bounds.center()] {
                let action = self.capture_sheet_pointer(point, Some(point), true);
                attempts.push((point, action));
                let Some(UiAction::Notification(NotificationAction::OpenMapGroup(id))) = action
                else {
                    continue;
                };
                if ui::map_receipt_group(&self.map_status_input(), id) == group {
                    let label = bounds.center();
                    assert!(ui::map_status_control_contains(
                        label,
                        self.map_status_input()
                    ));
                    assert!(self.map_controls_block(label));
                    assert!(matches!(
                        self.capture_sheet_pointer(label, Some(label), true),
                        Some(UiAction::Notification(NotificationAction::OpenMapGroup(label_id)))
                            if label_id == id
                    ));
                    return (label, id);
                }
            }
        }
        let all_groups = self
            .live_receipt_campaign()
            .notifications
            .receipts
            .iter()
            .map(|receipt| {
                (
                    receipt.id,
                    receipt.kind,
                    ui::map_receipt_group(&self.map_status_input(), receipt.id),
                )
            })
            .collect::<Vec<_>>();
        panic!(
            "receipt group {wanted:?} ({group:?}) has no visible tap target; known groups {all_groups:?}; attempts {attempts:?}"
        );
    }

    fn live_receipt_campaign(&self) -> &kestrum::state::StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("strategic M02 capture")
    }

    fn live_receipt_campaign_data_mut(
        &mut self,
    ) -> (&GameData, &mut kestrum::state::StrategicCampaign) {
        let data = &self.data;
        let campaign = match self.state.campaign.as_mut().expect("strategic M02 capture") {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M02 capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}

struct MapReceiptSample<'a> {
    kind: NotificationKind,
    ordinal: u32,
    before: &'a str,
    after: &'a str,
    cause: &'a str,
}

fn append_place_receipt(
    campaign: &mut kestrum::state::StrategicCampaign,
    data: &GameData,
    site: SiteId,
    sample: MapReceiptSample<'_>,
) -> NotificationId {
    // These three map-routing samples are synthetic typed place receipts. Work
    // and supply states in the other scenes come from ordinary engine actions.
    let place = campaign.world.site(site).expect("receipt place exists");
    let snapshot = PlaceNotificationSnapshot {
        id: place.id,
        name: place.name.clone(),
        controller: place.controller,
        habitation: place.habitation,
    };
    let id = NotificationId(campaign.notifications.next_id);
    let rule = data
        .notifications
        .kind(sample.kind)
        .expect("receipt rule exists");
    let sequence = campaign.accepted_sequence.max(1);
    campaign.notifications.receipts.push(NotificationReceipt {
        id,
        source: NotificationSourceId::Transition {
            accepted_sequence: sequence,
            kind: sample.kind,
            subject: NotificationEntity::Site(site),
            ordinal: sample.ordinal,
        },
        kind: sample.kind,
        priority: rule.default_priority,
        completed_rounds: campaign.completed_rounds,
        occurred_sequence: sequence,
        occurred_order: campaign.notifications.next_order,
        subject: Some(NotificationSubjectSnapshot::Place(snapshot.clone())),
        detail: NotificationDetail::Place {
            place: snapshot,
            before: Some(sample.before.into()),
            after: Some(sample.after.into()),
            cause: Some(sample.cause.into()),
            forecast_round: None,
            conditions: Vec::new(),
        },
        is_read: false,
        is_dismissed: false,
        is_active: false,
        closed_round: None,
    });
    campaign.notifications.next_id += 1;
    campaign.notifications.next_order += 1;
    id
}
