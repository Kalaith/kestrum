//! Hidden capture scenes audit notification actions against a valid campaign.

use super::*;
use crate::ui::{NotificationAction, NotificationTab};
use kestrum::{
    data::world::SiteId,
    state::{
        movement::MovementPlan,
        notifications::{
            NotificationDetail, NotificationId, NotificationInbox, NotificationKind,
            NotificationSubjectSnapshot, PlaceNotificationSnapshot,
        },
        people::PersonId,
    },
};
use macroquad::prelude::vec2;
use macroquad_toolkit::ui::Pointer;

mod fixtures;
mod settings;
use fixtures::{
    append_receipt, append_warning, facts_detail, person_snapshot, person_snapshot_for,
    place_snapshot,
};

#[derive(Clone, Copy)]
struct FixtureIds {
    hero: NotificationId,
    career: NotificationId,
    warning: NotificationId,
    missing: NotificationId,
    group_anchor: NotificationId,
    group_other: NotificationId,
}

impl Game {
    pub(super) fn capture_notifications(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "notifications"
                | "notification_details"
                | "notification_warning"
                | "notification_settings"
                | "notification_global_settings"
                | "notifications_dense"
        ) {
            return false;
        }

        self.setup_notification_capture();
        if scene == "notifications" {
            self.audit_notification_actions();
            self.setup_notification_capture();
        }
        let ids = self.populate_notification_fixture();
        self.validate_notification_capture();
        match scene {
            "notifications" => self.apply(UiAction::Notification(NotificationAction::Toggle)),
            "notification_details" => {
                self.apply(UiAction::Notification(NotificationAction::Open(ids.hero)))
            }
            "notification_warning" => self.apply(UiAction::Notification(NotificationAction::Open(
                ids.warning,
            ))),
            "notification_settings" => self.apply(UiAction::Notification(
                NotificationAction::SetTab(NotificationTab::Settings),
            )),
            "notification_global_settings" => {
                self.state.overlay = Overlay::Settings;
                self.apply(UiAction::Notification(
                    NotificationAction::OpenGlobalSettings,
                ));
            }
            "notifications_dense" => {}
            _ => unreachable!("matched notification capture scene"),
        }
        self.notice = None;
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("capture campaign")
            .notifications
            .validate(&self.data.notifications)
            .expect("notification capture actions preserve inbox invariants");
        true
    }

    fn setup_notification_capture(&mut self) {
        self.state = GameState::default();
        self.reset_notifications();
        self.invalidate_projection();
        self.navigation.reset(&mut self.view);
        self.movement = ui::MoveView::default();
        self.capture_campaign();
        self.refresh_projection();
    }

    fn populate_notification_fixture(&mut self) -> FixtureIds {
        let (ids, eligible_career) = {
            let Campaign::Strategic(campaign) = self
                .state
                .campaign
                .as_mut()
                .expect("capture campaign exists")
            else {
                unreachable!("capture campaign is strategic")
            };
            campaign.notifications = NotificationInbox {
                baseline_complete: true,
                ..Default::default()
            };
            let hero = person_snapshot(campaign);
            let home_id = campaign.factions[&campaign.player].headquarters;
            let home = place_snapshot(campaign, home_id);
            let preferences = &mut self.preferences.notifications;
            preferences.restore_defaults();
            for kind in [
                NotificationKind::NewHero,
                NotificationKind::PersonClassCompleted,
                NotificationKind::PersonRecognized,
                NotificationKind::PersonArrived,
                NotificationKind::PersonRetired,
                NotificationKind::PersonDied,
                NotificationKind::CareerOpportunity,
                NotificationKind::Succession,
                NotificationKind::VacantCommand,
                NotificationKind::HabitationChanged,
                NotificationKind::RuinRisk,
                NotificationKind::ControlLost,
                NotificationKind::Remembrance,
            ] {
                preferences.choices.insert(
                    kind,
                    kestrum::state::notifications::NotificationDelivery::TopBarAndHistory,
                );
            }
            campaign.notifications.preference_snapshot = preferences.clone();

            let hero_id = append_receipt(
                campaign,
                &self.data,
                NotificationKind::NewHero,
                1,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                NotificationDetail::Person {
                    person: Box::new(hero.clone()),
                    reason_key: Some("recognition".into()),
                    reason: Some("a winter patrol".into()),
                    opportunities: Vec::new(),
                },
                false,
            );
            let group_anchor = append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonClassCompleted,
                2,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail(&hero.name, &home.name),
                false,
            );
            let group_other = append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonClassCompleted,
                3,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail("Alden Vale", &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonRecognized,
                4,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail(&hero.name, &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonArrived,
                5,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail(&hero.name, &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonRetired,
                6,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail(&hero.name, &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonDied,
                7,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                facts_detail(&hero.name, &home.name),
                false,
            );
            let career_id = append_receipt(
                campaign,
                &self.data,
                NotificationKind::CareerOpportunity,
                8,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                NotificationDetail::Person {
                    person: Box::new(hero.clone()),
                    reason_key: Some("service".into()),
                    reason: Some("earned command experience".into()),
                    opportunities: vec![kestrum::data::world::PersonClass::Officer],
                },
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::Succession,
                9,
                None,
                facts_detail(&hero.name, &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::VacantCommand,
                10,
                None,
                facts_detail(&hero.name, &home.name),
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::HabitationChanged,
                11,
                Some(NotificationSubjectSnapshot::Place(home.clone())),
                NotificationDetail::Place {
                    place: home.clone(),
                    before: Some("village".into()),
                    after: Some("town".into()),
                    cause: Some("population_moved".into()),
                    forecast_round: None,
                    conditions: Vec::new(),
                },
                false,
            );
            let warning = append_warning(campaign, &self.data, &home);
            let missing_place = PlaceNotificationSnapshot {
                id: SiteId(u32::MAX),
                name: "Lost Settlement".into(),
                controller: None,
                habitation: home.habitation,
            };
            let missing = append_receipt(
                campaign,
                &self.data,
                NotificationKind::ControlLost,
                13,
                Some(NotificationSubjectSnapshot::Place(missing_place.clone())),
                NotificationDetail::Place {
                    place: missing_place,
                    before: Some("town".into()),
                    after: Some("unsettled".into()),
                    cause: Some("control_lost".into()),
                    forecast_round: None,
                    conditions: Vec::new(),
                },
                false,
            );
            append_receipt(
                campaign,
                &self.data,
                NotificationKind::Remembrance,
                14,
                Some(NotificationSubjectSnapshot::Person(Box::new(hero.clone()))),
                NotificationDetail::Remembrance {
                    subject: NotificationSubjectSnapshot::Person(Box::new(hero.clone())),
                    years: Some(12),
                    category: None,
                },
                false,
            );
            let eligible_career = campaign
                .people
                .values()
                .filter(|person| person.faction == campaign.player && person.is_alive())
                .find(|person| {
                    engine::person_site(campaign, person.id).is_some_and(|site| {
                        campaign
                            .world
                            .site(site)
                            .is_some_and(|site| site.controller == Some(campaign.player))
                    }) && engine::career_options(campaign, &self.data, person.id)
                        .is_ok_and(|options| options.iter().any(|option| option.eligible))
                })
                .map(|person| person.id);
            campaign
                .notifications
                .validate(&self.data.notifications)
                .expect("capture fixture inbox is structurally valid");
            (
                FixtureIds {
                    hero: hero_id,
                    career: career_id,
                    warning,
                    missing,
                    group_anchor,
                    group_other,
                },
                eligible_career,
            )
        };
        if let Some(career_person) = eligible_career {
            self.replace_career_subject(ids.career, career_person);
        }
        self.refresh_notification_projection();
        ids
    }

    fn replace_career_subject(&mut self, id: NotificationId, person_id: PersonId) {
        let Campaign::Strategic(campaign) = self.state.campaign.as_mut().expect("campaign") else {
            unreachable!()
        };
        let snapshot = person_snapshot_for(campaign, person_id);
        let receipt = campaign
            .notifications
            .receipt_mut(id)
            .expect("career receipt exists");
        receipt.subject = Some(NotificationSubjectSnapshot::Person(Box::new(
            snapshot.clone(),
        )));
        receipt.detail = NotificationDetail::Person {
            person: Box::new(snapshot),
            reason_key: Some("service".into()),
            reason: Some("earned command experience".into()),
            opportunities: vec![kestrum::data::world::PersonClass::Officer],
        };
    }

    fn validate_notification_capture(&mut self) {
        self.state
            .campaign
            .as_ref()
            .expect("campaign")
            .validate(&self.data)
            .expect("notification capture campaign and snapshots validate");
        let projection = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| {
                engine::notifications::project(
                    campaign,
                    &self.data,
                    &self.preferences.notifications,
                )
            })
            .expect("strategic campaign");
        self.notification_projection = Some(projection);
        self.refresh_projection();
    }

    fn audit_notification_actions(&mut self) {
        let mut ids = self.populate_notification_fixture();
        self.validate_notification_capture();
        self.audit_notification_global_settings();
        let (player_army, destination) = {
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().expect("campaign")
            else {
                unreachable!()
            };
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player)
                .expect("capture player army");
            let destination = campaign
                .world
                .adjacent_sites(army.site)
                .into_iter()
                .find(|site| campaign.world.connected_route(army.site, *site).is_some())
                .expect("capture army has a connected site");
            campaign.movement_plans.push(MovementPlan {
                armies: vec![army.id],
                path: vec![army.site, destination],
            });
            (army.id, destination)
        };
        self.validate_notification_capture();
        let queued = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .unwrap()
            .movement_plans
            .clone();
        self.apply(UiAction::BeginMove(player_army));
        assert_eq!(self.movement.stage, ui::MoveStage::Map);
        self.movement.planned_destination = Some(destination);
        let before_camera = self.view;
        self.apply(UiAction::Notification(NotificationAction::Toggle));
        assert!(self.notifications.is_open);
        assert_eq!(self.movement.stage, ui::MoveStage::Inactive);
        assert_eq!(
            self.view, before_camera,
            "opening notifications preserves camera"
        );
        assert_eq!(
            self.state
                .campaign
                .as_ref()
                .unwrap()
                .strategic()
                .unwrap()
                .movement_plans,
            queued,
            "opening notifications preserves queued routes"
        );
        self.apply(UiAction::Notification(NotificationAction::Close));
        assert!(!self.notifications.is_open);
        assert_eq!(
            self.view, before_camera,
            "closing notifications preserves camera"
        );
        assert_eq!(
            self.state
                .campaign
                .as_ref()
                .unwrap()
                .strategic()
                .unwrap()
                .movement_plans,
            queued,
            "closing notifications preserves queued routes"
        );

        self.view.pan(vec2(420.0, -190.0));
        let pre_open = self.view;
        self.apply(UiAction::Notification(NotificationAction::Open(ids.hero)));
        assert_eq!(
            self.view, pre_open,
            "opening a detail does not focus its subject"
        );
        let recent = self.capture_sheet_tap(vec2(92.0, 233.0));
        assert!(
            matches!(
                recent,
                UiAction::Notification(NotificationAction::SetTab(NotificationTab::Recent))
            ),
            "the visible Recent tab is an ordinary action"
        );
        self.apply(recent);
        assert!(
            self.notifications.selected.is_none(),
            "Recent returns to the list"
        );
        let first_row = self.capture_sheet_tap(vec2(136.0, 289.0));
        assert!(matches!(
            first_row,
            UiAction::Notification(NotificationAction::Open(_))
        ));
        self.apply(first_row);
        assert!(
            self.notifications.selected.is_some(),
            "the recent list opens a receipt"
        );
        self.apply(UiAction::Notification(NotificationAction::Open(ids.hero)));
        self.apply(UiAction::Notification(NotificationAction::FollowSubject(
            ids.hero,
        )));
        assert_eq!(self.state.overlay, Overlay::History);
        let history_view = self.view;
        self.apply(UiAction::HistoryBack);
        self.refresh_notifications(Pointer::default());
        assert_eq!(self.state.overlay, Overlay::None);
        assert!(
            self.notifications.is_open,
            "return from History restores the card"
        );
        assert_eq!(
            self.view, history_view,
            "History return retains map framing"
        );

        if let Some(career_id) = self.find_eligible_career_person() {
            self.apply(UiAction::Notification(NotificationAction::Close));
            self.replace_career_subject(ids.career, career_id);
            self.refresh_notification_projection();
            self.apply(UiAction::Notification(NotificationAction::Open(ids.career)));
            self.apply(UiAction::Notification(NotificationAction::FollowSubject(
                ids.career,
            )));
            assert_eq!(
                self.state.overlay,
                Overlay::Armies,
                "career link opens its live view"
            );
            assert_eq!(
                self.army.mode,
                ui::ArmyMode::ProgressionPerson(career_id),
                "career route targets the eligible person"
            );
            self.apply(UiAction::Back);
            assert_eq!(self.army.mode, ui::ArmyMode::Roster);
            self.apply(UiAction::Back);
            self.refresh_notifications(Pointer::default());
            assert_eq!(self.state.overlay, Overlay::None);
            assert!(
                self.notifications.is_open,
                "return from Career restores the card"
            );
        } else {
            self.apply(UiAction::Notification(NotificationAction::Close));
            self.apply(UiAction::Notification(NotificationAction::Open(ids.career)));
            self.apply(UiAction::Notification(NotificationAction::FollowSubject(
                ids.career,
            )));
            assert_eq!(self.state.overlay, Overlay::History);
            self.apply(UiAction::HistoryBack);
            self.refresh_notifications(Pointer::default());
            assert!(
                self.notifications.is_open,
                "ineligible career subject safely returns"
            );
        }

        self.apply(UiAction::Notification(NotificationAction::Close));
        self.apply(UiAction::Notification(NotificationAction::Open(ids.hero)));
        self.view.pan(vec2(650.0, 360.0));
        let camera_before_focus = self.view;
        self.apply(UiAction::Notification(NotificationAction::ShowOnMap(
            ids.hero,
        )));
        assert_ne!(
            self.view, camera_before_focus,
            "only Show on map changes focus"
        );
        assert!(!self.notifications.is_open);
        let focused = self.view;
        self.apply(UiAction::Notification(NotificationAction::Open(
            ids.missing,
        )));
        self.apply(UiAction::Notification(NotificationAction::ShowOnMap(
            ids.missing,
        )));
        assert_eq!(self.view, focused, "a missing map target is a safe no-op");
        self.apply(UiAction::Notification(NotificationAction::FollowSubject(
            ids.missing,
        )));
        assert_eq!(
            self.state.overlay,
            Overlay::None,
            "a missing subject is a safe no-op"
        );

        let inbox_before_failed_load = self
            .state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .notifications
            .clone();
        self.finish_load(Err("capture-only invalid save".into()));
        assert_eq!(
            self.state
                .campaign
                .as_ref()
                .unwrap()
                .strategic()
                .unwrap()
                .notifications,
            inbox_before_failed_load,
            "a failed load leaves the current inbox untouched"
        );
        self.error = None;

        self.apply(UiAction::Notification(NotificationAction::AllOff));
        assert!(self
            .notification_projection
            .as_ref()
            .is_some_and(|projection| projection.recent.is_empty()));
        self.apply(UiAction::Notification(NotificationAction::RestoreDefaults));
        assert!(
            self.notification_projection
                .as_ref()
                .is_some_and(|projection| projection.recent.is_empty()),
            "restoring defaults does not replay the muted backlog"
        );
        let (future, group_anchor, group_other) = {
            let Campaign::Strategic(campaign) = self.state.campaign.as_mut().expect("campaign")
            else {
                unreachable!()
            };
            let future = append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonRecognized,
                40,
                None,
                facts_detail("Mara Reed", "Rose Headquarters"),
                false,
            );
            let group_anchor = append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonClassCompleted,
                42,
                None,
                facts_detail("Alden Vale", "Rose Headquarters"),
                false,
            );
            let group_other = append_receipt(
                campaign,
                &self.data,
                NotificationKind::PersonClassCompleted,
                43,
                None,
                facts_detail("Mara Reed", "Rose Headquarters"),
                false,
            );
            (future, group_anchor, group_other)
        };
        ids.group_anchor = group_anchor;
        ids.group_other = group_other;
        self.refresh_notification_projection();
        assert!(self
            .notification_projection
            .as_ref()
            .is_some_and(|projection| projection.recent.iter().any(|item| item.id == future)));

        let group = self
            .notification_projection
            .as_ref()
            .unwrap()
            .rail
            .iter()
            .find(|group| group.kind == NotificationKind::PersonClassCompleted)
            .expect("group fixture is in the rail");
        let [anchor, other, ..] = group.receipt_ids.as_slice() else {
            panic!("group fixture contains both same-season receipts")
        };
        let (anchor, other) = (*anchor, *other);
        self.apply(UiAction::Notification(NotificationAction::OpenGroup(
            anchor,
        )));
        let inbox = &self
            .state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .notifications;
        assert!(inbox.receipt(anchor).unwrap().is_read);
        assert!(!inbox.receipt(other).unwrap().is_read);
        assert!(
            [ids.group_anchor, ids.group_other].contains(&anchor)
                && [ids.group_anchor, ids.group_other].contains(&other)
                && anchor != other,
            "only the group receipts are selected"
        );
        self.apply(UiAction::Notification(NotificationAction::SelectGroupItem(
            other,
        )));
        assert!(
            self.state
                .campaign
                .as_ref()
                .unwrap()
                .strategic()
                .unwrap()
                .notifications
                .receipt(other)
                .unwrap()
                .is_read
        );

        let old_projection = self.notification_projection.clone().unwrap();
        let old_anchor = old_projection
            .rail
            .first()
            .and_then(|group| group.receipt_ids.first())
            .copied()
            .expect("visible rail fixture");
        let Some(Campaign::Strategic(campaign)) = self.state.campaign.as_mut() else {
            unreachable!("capture campaign is strategic")
        };
        append_receipt(
            campaign,
            &self.data,
            NotificationKind::ControlLost,
            41,
            None,
            facts_detail("Mara Reed", "Rose Headquarters"),
            false,
        );
        let position = vec2(52.0, 128.0);
        self.origin = Some(position);
        self.was_down = true;
        self.map_gesture = true;
        self.refresh_notifications(Pointer {
            position,
            down: true,
            released: false,
            hovering: false,
        });
        assert!(self.notifications.pointer_captured);
        assert!(!self.map_gesture, "rail press is consumed before map input");
        assert_eq!(
            self.notifications.frozen_projection.as_ref(),
            Some(&old_projection)
        );
        self.was_down = false;
        self.refresh_notifications(Pointer {
            position,
            down: false,
            released: true,
            hovering: false,
        });
        let action = self.capture_sheet_tap(position);
        assert!(matches!(
            action,
            UiAction::Notification(NotificationAction::Open(id)
                | NotificationAction::OpenGroup(id)) if id == old_anchor
        ));
        self.apply(action);
        assert_eq!(self.notifications.selected_id, Some(old_anchor));
        self.refresh_notifications(Pointer::default());

        let Campaign::Strategic(campaign) = self.state.campaign.as_ref().unwrap() else {
            unreachable!()
        };
        let bytes = serde_json::to_vec(campaign).expect("serialize saved capture campaign");
        let loaded: kestrum::state::StrategicCampaign =
            serde_json::from_slice(&bytes).expect("deserialize saved capture campaign");
        assert_eq!(
            loaded.notifications.receipt(old_anchor).unwrap().is_read,
            campaign.notifications.receipt(old_anchor).unwrap().is_read,
            "notification acknowledgement survives campaign save roundtrip"
        );
        campaign
            .validate(&self.data)
            .expect("action audit leaves the capture campaign valid");
    }

    fn find_eligible_career_person(&self) -> Option<PersonId> {
        let campaign = self.state.campaign.as_ref()?.strategic()?;
        campaign
            .people
            .values()
            .filter(|person| person.faction == campaign.player && person.is_alive())
            .find(|person| {
                engine::person_site(campaign, person.id).is_some_and(|site| {
                    campaign
                        .world
                        .site(site)
                        .is_some_and(|site| site.controller == Some(campaign.player))
                }) && engine::career_options(campaign, &self.data, person.id)
                    .is_ok_and(|options| options.iter().any(|option| option.eligible))
            })
            .map(|person| person.id)
    }
}
