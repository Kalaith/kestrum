//! Campaign-owned notification actions and live-target validation.

use super::*;
use crate::ui::{NotificationAction, NotificationTab};
use kestrum::{
    data::world::{RouteId, SiteId},
    engine::VisibleCampaign,
    state::{
        construction::ConstructionTarget,
        history::HistorySubject,
        notifications::{
            NotificationDelivery, NotificationId, NotificationKind, NotificationSubjectSnapshot,
            RenderedNotification,
        },
        people::{PersonAssignment, PersonId},
    },
};
use macroquad_toolkit::ui::Pointer;

impl Game {
    pub(super) fn reset_notifications(&mut self) {
        self.notifications.reset();
        self.notification_projection = None;
    }

    /// Start a fresh UI session after the caller has accepted a campaign.
    /// Baseline collection is deliberately done against the candidate before
    /// replacement, so this method only synchronizes local delivery choices.
    pub(super) fn initialize_notification_campaign(&mut self) {
        self.reset_notifications();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        if campaign.observer_mode {
            return;
        }
        campaign
            .notifications
            .sync_preferences(&self.preferences.notifications, &self.data.notifications);
        self.refresh_notification_projection();
    }

    pub(super) fn notification_map_visible(&self) -> bool {
        self.state.screen == Screen::Campaign
            && self.state.overlay == Overlay::None
            && self
                .state
                .campaign
                .as_ref()
                .and_then(Campaign::strategic)
                .is_some_and(|campaign| !campaign.observer_mode)
    }

    pub(super) fn refresh_notifications(&mut self, pointer: Pointer) {
        self.notifications.clear_finished_gesture();

        // The NPC step runs before input and may have changed this rail since
        // it was displayed, so press capture uses the exact visible snapshot.
        let displayed_projection = self.notification_projection.clone();
        let displayed_rail =
            ui::notification_reserved_rects(&self.notifications, displayed_projection.as_ref())
                .first()
                .copied();

        if self.notification_map_visible() && self.notifications.return_context.is_some() {
            self.notifications.restore_return_context();
        }

        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            self.notification_projection = None;
            return;
        };
        if campaign.observer_mode {
            self.reset_notifications();
            return;
        }

        self.notification_projection = Some(engine::notifications::project(
            campaign,
            &self.data,
            &self.preferences.notifications,
        ));

        if !self.notification_map_visible() {
            return;
        }

        let began_on_rail = self
            .origin
            .is_some_and(|origin| displayed_rail.is_some_and(|rect| rect.contains(origin)));
        if began_on_rail
            && !self.notifications.pointer_captured
            && (self.was_down || pointer.down || pointer.released)
        {
            self.notifications
                .capture_rail_gesture(displayed_projection.as_ref());
        }
        if self.notifications.pointer_captured && !pointer.down {
            self.notifications.finish_pointer_gesture();
        }
        if self.notifications.pointer_captured || self.notifications.release_pending {
            self.map_gesture = false;
        }
    }

    pub(super) fn apply_notification_action(&mut self, action: NotificationAction) {
        match action {
            NotificationAction::Toggle => {
                if self.notifications.is_open {
                    self.notifications.is_open = false;
                } else {
                    self.prepare_notification_open();
                    self.notifications.is_open = true;
                    self.notifications.tab = NotificationTab::Recent;
                }
            }
            NotificationAction::Close => {
                self.notifications.is_open = false;
                self.notifications.global_settings_open = false;
            }
            NotificationAction::SetTab(tab) => {
                if !self.notifications.is_open {
                    self.prepare_notification_open();
                }
                self.notifications.is_open = true;
                self.notifications.tab = tab;
                self.notifications.settings_page = 0;
                if tab == NotificationTab::Recent {
                    self.notifications.selected_id = None;
                    self.notifications.selected = None;
                    self.notifications.group_ids.clear();
                    self.notifications.group_snapshots.clear();
                    self.notifications.group_page = 0;
                    self.notifications.detail_page = 0;
                }
            }
            NotificationAction::Open(id) => self.open_notification(id, false),
            NotificationAction::OpenGroup(id) => self.open_notification(id, true),
            NotificationAction::SelectGroupItem(id) => {
                if self.notifications.select_group_item(id) {
                    self.mark_notification_read(id);
                }
            }
            NotificationAction::Page(delta) => self.change_notification_page(delta),
            NotificationAction::OpenRecords => self.open_notification_records(),
            NotificationAction::OpenGlobalSettings => {
                self.notifications.global_settings_open = true;
            }
            NotificationAction::CloseGlobalSettings => {
                self.notifications.global_settings_open = false;
            }
            NotificationAction::MarkAllRead => {
                self.mutate_notification_inbox(|inbox| {
                    inbox.mark_all_read();
                });
            }
            NotificationAction::Dismiss(id) => {
                self.mutate_notification_inbox(|inbox| {
                    inbox.dismiss(id);
                });
            }
            NotificationAction::DismissRead => {
                self.mutate_notification_inbox(|inbox| {
                    inbox.dismiss_read();
                });
            }
            NotificationAction::ShowOnMap(id) => self.show_notification_on_map(id),
            NotificationAction::FollowSubject(id) => self.follow_notification_subject(id),
            NotificationAction::SetDelivery(kind, delivery) => {
                self.set_notification_delivery(kind, delivery);
            }
            NotificationAction::ToggleSettingsCategory(category) => {
                if self.notifications.settings_category == category {
                    self.notifications.settings_category =
                        kestrum::state::notifications::NotificationSettingsCategory::All;
                } else {
                    self.notifications.settings_category = category;
                }
                self.notifications.settings_page = 0;
            }
            NotificationAction::AllOff => {
                self.preferences.notifications.all_off();
                self.sync_notification_preferences();
                self.save_preferences();
            }
            NotificationAction::RestoreDefaults => {
                self.preferences.notifications.restore_defaults();
                self.sync_notification_preferences();
                self.save_preferences();
            }
        }
        self.refresh_notification_projection();
    }

    fn open_notification(&mut self, id: NotificationId, grouped: bool) {
        let projection = self
            .notifications
            .visible_projection(self.notification_projection.as_ref())
            .cloned()
            .or_else(|| self.notification_projection.clone());
        let Some(projection) = projection else {
            return;
        };
        let Some(item) = projection.recent.iter().find(|item| item.id == id).cloned() else {
            return;
        };
        if !self.notifications.is_open {
            self.prepare_notification_open();
        }
        self.notifications.is_open = true;
        self.notifications.tab = NotificationTab::Recent;
        self.notifications.selected_id = Some(id);
        self.notifications.selected = Some(item);
        self.notifications.detail_page = 0;
        if grouped {
            if let Some(group) = projection
                .rail
                .iter()
                .find(|group| group.receipt_ids.first() == Some(&id))
            {
                self.notifications.group_ids = group.receipt_ids.clone();
                self.notifications.group_snapshots = self
                    .notifications
                    .snapshots_for_group(group, Some(&projection));
                self.notifications.group_page = 0;
            }
        } else {
            self.notifications.group_ids.clear();
            self.notifications.group_snapshots.clear();
            self.notifications.group_page = 0;
        }
        self.mark_notification_read(id);
    }

    fn mark_notification_read(&mut self, id: NotificationId) {
        self.mutate_notification_inbox(|inbox| {
            inbox.mark_read(id);
        });
    }

    fn mutate_notification_inbox(
        &mut self,
        mutate: impl FnOnce(&mut kestrum::state::notifications::NotificationInbox),
    ) {
        match &mut self.state.campaign {
            Some(Campaign::Strategic(campaign)) if !campaign.observer_mode => {
                mutate(&mut campaign.notifications)
            }
            _ => {}
        }
    }

    fn set_notification_delivery(
        &mut self,
        kind: NotificationKind,
        delivery: NotificationDelivery,
    ) {
        self.preferences
            .notifications
            .choices
            .insert(kind, delivery);
        self.sync_notification_preferences();
        self.save_preferences();
    }

    fn sync_notification_preferences(&mut self) {
        match &mut self.state.campaign {
            Some(Campaign::Strategic(campaign)) if !campaign.observer_mode => {
                campaign
                    .notifications
                    .sync_preferences(&self.preferences.notifications, &self.data.notifications);
            }
            _ => {}
        }
    }

    pub(super) fn refresh_notification_projection(&mut self) {
        self.notification_projection = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .filter(|campaign| !campaign.observer_mode)
            .map(|campaign| {
                engine::notifications::project(
                    campaign,
                    &self.data,
                    &self.preferences.notifications,
                )
            });
    }

    fn change_notification_page(&mut self, delta: i32) {
        if self.notifications.global_settings_open {
            let last_page = self.notification_settings_last_page();
            self.notifications.settings_page = self
                .notifications
                .settings_page
                .saturating_add_signed(delta as isize)
                .min(last_page);
            return;
        }
        match self.notifications.tab {
            NotificationTab::Recent if self.notifications.selected.is_some() => {
                self.notifications.detail_page = self
                    .notifications
                    .detail_page
                    .saturating_add_signed(delta as isize);
            }
            NotificationTab::Recent if self.notifications.group_ids.len() > 1 => {
                let last = self.notifications.group_ids.len() - 1;
                self.notifications.group_page = self
                    .notifications
                    .group_page
                    .saturating_add_signed(delta as isize)
                    .min(last);
                let id = self
                    .notifications
                    .group_ids
                    .get(self.notifications.group_page)
                    .copied();
                if let Some(id) = id {
                    self.notifications.select_group_item(id);
                    self.mark_notification_read(id);
                }
            }
            NotificationTab::Recent => {
                let last = self
                    .notification_projection
                    .as_ref()
                    .map_or(0, |projection| {
                        projection.recent.len().div_ceil(5).saturating_sub(1)
                    });
                self.notifications.recent_page = self
                    .notifications
                    .recent_page
                    .saturating_add_signed(delta as isize)
                    .min(last);
            }
            NotificationTab::Settings => {
                self.notifications.settings_page = self
                    .notifications
                    .settings_page
                    .saturating_add_signed(delta as isize)
                    .min(self.notification_settings_last_page());
            }
        }
    }

    fn selected_notification(&self, id: NotificationId) -> Option<RenderedNotification> {
        if self.notifications.selected_id == Some(id) {
            if let Some(item) = &self.notifications.selected {
                return Some(item.clone());
            }
        }
        self.notification_projection
            .as_ref()?
            .recent
            .iter()
            .find(|item| item.id == id)
            .cloned()
    }

    fn current_visible_campaign(&self) -> Option<VisibleCampaign> {
        let campaign = self.state.campaign.as_ref()?.strategic()?;
        if campaign.observer_mode {
            return None;
        }
        engine::project_map(campaign, campaign.player).ok()
    }

    fn notification_settings_last_page(&self) -> usize {
        use kestrum::state::notifications::{
            NotificationCategory, NotificationSettingsCategory as Category,
        };
        let category = match self.notifications.settings_category {
            Category::All => None,
            Category::People => Some(NotificationCategory::People),
            Category::Places => Some(NotificationCategory::Places),
            Category::Security => Some(NotificationCategory::Security),
            Category::Orders => Some(NotificationCategory::Orders),
            Category::Military => Some(NotificationCategory::Military),
            Category::EconomyDiplomacy => Some(NotificationCategory::EconomyDiplomacy),
            Category::Remembrance => Some(NotificationCategory::Remembrance),
        };
        let count = self
            .data
            .notifications
            .kinds
            .keys()
            .filter(|kind| category.is_none_or(|category| kind.category() == category))
            .count();
        let page_size = if self.notifications.global_settings_open {
            2
        } else {
            3
        };
        count.div_ceil(page_size).saturating_sub(1)
    }

    fn show_notification_on_map(&mut self, id: NotificationId) {
        let Some(item) = self.selected_notification(id) else {
            return;
        };
        let Some(view) = self.current_visible_campaign() else {
            return;
        };
        let Some(site) = notification_site(&item, &view) else {
            return;
        };
        if view.world.site(site).is_none() {
            return;
        }
        self.movement = ui::MoveView::default();
        self.notifications.clear_card();
        self.state.overlay = Overlay::None;
        self.navigation
            .focus_site(&view.world, site, &mut self.view);
    }

    fn follow_notification_subject(&mut self, id: NotificationId) {
        let Some(item) = self.selected_notification(id) else {
            return;
        };
        let Some(view) = self.current_visible_campaign() else {
            return;
        };
        match item.subject.as_ref() {
            Some(NotificationSubjectSnapshot::Person(person))
                if view
                    .people
                    .iter()
                    .any(|current| current.id == person.id && current.faction == view.player) =>
            {
                if item.kind == NotificationKind::CareerOpportunity
                    && self.open_career_notification(person.id, &view)
                {
                    return;
                }
                self.route_from_notification();
                self.apply(UiAction::OpenHistory(HistorySubject::Person(person.id)));
            }
            Some(NotificationSubjectSnapshot::Place(place))
                if view
                    .world
                    .site(place.id)
                    .is_some_and(|site| site.controller == Some(view.player)) =>
            {
                self.route_from_notification();
                self.open_settlement(place.id);
            }
            Some(NotificationSubjectSnapshot::Army(army)) => {
                if let Some(current) = view
                    .armies
                    .iter()
                    .find(|current| current.id == army.id && current.faction == view.player)
                {
                    self.route_from_notification();
                    self.open_armies(current.site);
                }
            }
            Some(NotificationSubjectSnapshot::Construction(work)) => {
                let target = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .and_then(|campaign| campaign.construction.get(&work.id))
                    .filter(|order| order.owner == view.player)
                    .map(|order| order.target);
                if let Some(ConstructionTarget::Site(site)) = target {
                    if view
                        .world
                        .site(site)
                        .is_some_and(|site| site.controller == Some(view.player))
                    {
                        self.route_from_notification();
                        self.open_settlement(site);
                    }
                }
            }
            Some(NotificationSubjectSnapshot::Battle(battle)) => {
                let retained = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .is_some_and(|campaign| {
                        engine::battle_reports(campaign, campaign.player)
                            .iter()
                            .any(|report| report.id == battle.id)
                    });
                self.route_from_notification();
                self.apply(UiAction::OpenRecords);
                if retained {
                    self.apply(UiAction::OpenRecordedBattle(battle.id));
                }
            }
            Some(NotificationSubjectSnapshot::History { id, .. }) => {
                let retained = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .is_some_and(|campaign| campaign.history.events.contains_key(id));
                self.route_from_notification();
                if retained {
                    self.apply(UiAction::OpenRecords);
                    self.apply(UiAction::OpenRelatedHistoryEvent(*id));
                } else {
                    self.apply(UiAction::OpenRecords);
                }
            }
            Some(NotificationSubjectSnapshot::Person(_))
            | Some(NotificationSubjectSnapshot::Place(_)) => {}
            Some(NotificationSubjectSnapshot::Route(_))
            | Some(NotificationSubjectSnapshot::Faction { .. })
            | None => {
                self.route_from_notification();
                self.apply(UiAction::OpenRecords);
            }
        }
    }

    fn open_career_notification(&mut self, person: PersonId, view: &VisibleCampaign) -> bool {
        let eligible = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| engine::career_options(campaign, &self.data, person).ok())
            .is_some_and(|options| options.iter().any(|option| option.eligible));
        if !eligible {
            return false;
        }
        let Some(site) = live_person_site(view, person) else {
            return false;
        };
        let manageable = view
            .world
            .site(site)
            .is_some_and(|site| site.controller == Some(view.player))
            || view
                .armies
                .iter()
                .any(|army| army.faction == view.player && army.site == site);
        if !manageable {
            return false;
        }
        self.route_from_notification();
        self.open_armies(site);
        if self.state.overlay == Overlay::Armies {
            self.army.mode = ui::ArmyMode::ProgressionPerson(person);
            true
        } else {
            false
        }
    }

    fn route_from_notification(&mut self) {
        self.notifications.remember_return_context();
        self.notifications.suspend_for_route();
    }

    fn prepare_notification_open(&mut self) {
        self.movement = ui::MoveView::default();
        self.overview_ui.expanded = false;
    }

    fn open_notification_records(&mut self) {
        self.route_from_notification();
        self.apply(UiAction::OpenRecords);
    }
}

fn notification_site(item: &RenderedNotification, view: &VisibleCampaign) -> Option<SiteId> {
    match item.subject.as_ref()? {
        NotificationSubjectSnapshot::Person(person) => live_person_site(view, person.id),
        NotificationSubjectSnapshot::Place(place) => view.world.site(place.id).map(|site| site.id),
        NotificationSubjectSnapshot::Army(army) => view
            .armies
            .iter()
            .find(|current| current.id == army.id && current.faction == view.player)
            .map(|current| current.site)
            .filter(|site| view.world.site(*site).is_some()),
        NotificationSubjectSnapshot::Route(route) => [route.from.id, route.to.id]
            .into_iter()
            .find(|site| view.world.site(*site).is_some()),
        NotificationSubjectSnapshot::Construction(work) => {
            let order = view
                .construction
                .iter()
                .find(|order| order.id == work.id && order.owner == view.player)?;
            match order.target {
                ConstructionTarget::Site(site) => view.world.site(site).map(|site| site.id),
                ConstructionTarget::Route(route) => route_site(&view.world, route),
            }
        }
        NotificationSubjectSnapshot::Battle(battle) => view
            .battles
            .iter()
            .find(|current| current.id == battle.id)
            .and_then(|current| view.world.site(current.site).map(|site| site.id)),
        NotificationSubjectSnapshot::History { .. }
        | NotificationSubjectSnapshot::Faction { .. } => None,
    }
}

fn route_site(world: &kestrum::state::world::CampaignWorld, id: RouteId) -> Option<SiteId> {
    let route = world.routes.iter().find(|route| route.id == id)?;
    [route.from, route.to]
        .into_iter()
        .find(|site| world.site(*site).is_some())
}

fn live_person_site(view: &VisibleCampaign, id: PersonId) -> Option<SiteId> {
    let person = view
        .people
        .iter()
        .find(|person| person.id == id && person.faction == view.player)?;
    let site = match person.assignment {
        PersonAssignment::Site { site }
        | PersonAssignment::Dependent { site }
        | PersonAssignment::Trainee { site } => site,
        PersonAssignment::Formation { formation } => {
            view.armies
                .iter()
                .find(|army| {
                    army.faction == view.player && army.formation_ids().any(|id| id == formation)
                })?
                .site
        }
        PersonAssignment::Dead => return None,
    };
    view.world.site(site).map(|site| site.id)
}
