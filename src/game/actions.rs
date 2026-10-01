//! Route explicit UI intents to their campaign, military and save handlers.

use super::*;

impl Game {
    pub(super) fn apply(&mut self, action: UiAction) {
        if self.apply_tutorial_action(action) {
            return;
        }
        self.dispatch(action);
        self.observe_tutorial(action);
    }

    fn dispatch(&mut self, action: UiAction) {
        self.army_refresh_pending = true;
        if let UiAction::Battlefield(action) = action {
            self.apply_battlefield_action(action);
            return;
        }
        if action.is_kingdom() {
            self.apply_kingdom_action(action);
            return;
        }
        if action.is_threat() {
            self.apply_threat_action(action);
            return;
        }
        if action.is_siege() {
            self.apply_siege_action(action);
            return;
        }
        if action.is_history() {
            self.apply_history_action(action);
            return;
        }
        if action.is_settlement() {
            self.apply_settlement_action(action);
            return;
        }
        match action {
            UiAction::OpenBattleReports
            | UiAction::BattleReport(_)
            | UiAction::BattlePage(_)
            | UiAction::SetBattleTab(_) => self.apply_battle_action(action),
            UiAction::ArmyOrders
            | UiAction::ArmyPeople
            | UiAction::ArmyPeoplePage(_)
            | UiAction::ArmyHouseholds
            | UiAction::ArmyLegacy
            | UiAction::HouseholdPage(_)
            | UiAction::SelectHouseholdPerson(_, _)
            | UiAction::SelectLegacyCategory(_)
            | UiAction::SelectLegacyLink(_)
            | UiAction::MentorshipPage(_)
            | UiAction::OpenPersonProgression(_)
            | UiAction::OpenMentorship(_)
            | UiAction::StartMentorship(_, _, _)
            | UiAction::ReviewHousehold(_)
            | UiAction::ConfirmHousehold
            | UiAction::CancelHouseholdReview
            | UiAction::EndMentorship(_)
            | UiAction::RecoverPersonAtSite(_, _)
            | UiAction::RetirePerson(_, _)
            | UiAction::AppointGovernor(_, _)
            | UiAction::OpenFormationProgression(_)
            | UiAction::TrainPerson(_, _, _)
            | UiAction::PracticeRiding(_, _)
            | UiAction::CancelPersonCourse(_)
            | UiAction::SpecializeFormation(_, _, _)
            | UiAction::CancelFormationCourse(_)
            | UiAction::SetCommander(_, _)
            | UiAction::BeginMove(_)
            | UiAction::EditMoveGroup
            | UiAction::ToggleMoveArmy(_)
            | UiAction::MoveGroupPage(_)
            | UiAction::ChooseMoveDestination
            | UiAction::ReviewMove
            | UiAction::MoveRoutePage(_)
            | UiAction::ConfirmMove
            | UiAction::CancelMovementPlan(_)
            | UiAction::CancelMove
            | UiAction::BeginTransferFormation(_)
            | UiAction::BeginTransferPerson(_)
            | UiAction::SelectTransferArmy(_)
            | UiAction::ClearTransferArmy
            | UiAction::SelectTransferSlot(_)
            | UiAction::SelectTransferFormation(_)
            | UiAction::TransferPage(_)
            | UiAction::ConfirmTransfer
            | UiAction::SplitArmy(_)
            | UiAction::OpenArmies(_)
            | UiAction::ArmyPage(_)
            | UiAction::SelectFormation(_)
            | UiAction::BeginRecruit(_)
            | UiAction::SelectRecruit(_)
            | UiAction::ConfirmRecruit
            | UiAction::AskDisband(_)
            | UiAction::ConfirmDisband(_)
            | UiAction::CancelArmyAction => self.apply_army_action(action),
            UiAction::HelpPage(_)
            | UiAction::NewGame
            | UiAction::ConfirmNew
            | UiAction::OpenSetupName
            | UiAction::EditSetupName(_)
            | UiAction::SetupNameDone
            | UiAction::SelectSetupEmblem(_)
            | UiAction::ChangeSetupFactionCount(_)
            | UiAction::RandomizeSetupSeed
            | UiAction::StartProductionCampaign
            | UiAction::Continue
            | UiAction::Open(_)
            | UiAction::Back
            | UiAction::MainMenu
            | UiAction::SelectMap(_)
            | UiAction::EnterRegion(_)
            | UiAction::WorldMap
            | UiAction::CloseSelection => self.apply_navigation_action(action),
            UiAction::Save
            | UiAction::Load
            | UiAction::LoadLegacy
            | UiAction::ImportCampaign
            | UiAction::SelectSave(_)
            | UiAction::SavePage(_)
            | UiAction::NameSave(_)
            | UiAction::LoadSelectedSave
            | UiAction::OverwriteSave
            | UiAction::AskDeleteSave
            | UiAction::ConfirmDeleteSave(_)
            | UiAction::CommitNamedSave
            | UiAction::CancelSaveEdit
            | UiAction::RetryStorage
            | UiAction::RetrySave
            | UiAction::ContinueUnsaved
            | UiAction::EditSaveName(_) => self.apply_save_action(action),
            UiAction::EndTurn => self.apply_campaign_command(Command::EndTurn),
            UiAction::PauseNpcs(paused) => {
                self.apply_campaign_command(Command::SetNpcPaused(paused))
            }
            UiAction::StepNpc => self.apply_campaign_command(Command::StepNpc),
            UiAction::Zoom(_)
            | UiAction::Recenter
            | UiAction::ToggleLabels
            | UiAction::ToggleContrast
            | UiAction::DismissFeedback => self.apply_view_action(action),
            #[cfg(not(target_arch = "wasm32"))]
            UiAction::Fullscreen | UiAction::Quit => self.apply_view_action(action),
            _ => unreachable!("history actions handled above"),
        }
    }

    fn apply_army_action(&mut self, action: UiAction) {
        match action {
            UiAction::ArmyOrders => self.army.mode = ui::ArmyMode::Orders,
            UiAction::ArmyPeople => {
                self.army.mode = ui::ArmyMode::People;
                self.army.people_page = 0;
            }
            UiAction::ArmyHouseholds => {
                self.army.mode = ui::ArmyMode::Households;
                self.army.household_review = None;
                self.army.status.clear();
                self.army.household_page = 0;
                self.army.household_first = None;
                self.army.household_second = None;
            }
            UiAction::ArmyLegacy => {
                self.army.mode = ui::ArmyMode::Legacy;
                self.army.household_review = None;
                self.army.status.clear();
            }
            UiAction::HouseholdPage(delta) => {
                self.army.household_page = self
                    .army
                    .household_page
                    .saturating_add_signed(delta as isize)
            }
            UiAction::SelectHouseholdPerson(person, slot) => match slot {
                0 => self.army.household_first = Some(person),
                1 => self.army.household_second = Some(person),
                _ => unreachable!("two household selection slots"),
            },
            UiAction::SelectLegacyCategory(category) => self.army.legacy_category = category,
            UiAction::SelectLegacyLink(link) => self.army.legacy_link = link,
            UiAction::ArmyPeoplePage(delta) => {
                self.army.people_page = self.army.people_page.saturating_add_signed(delta as isize)
            }
            UiAction::MentorshipPage(delta) => {
                self.army.mentorship_page = self
                    .army
                    .mentorship_page
                    .saturating_add_signed(delta as isize)
            }
            UiAction::OpenPersonProgression(person) => {
                self.army.mode = ui::ArmyMode::ProgressionPerson(person)
            }
            UiAction::OpenMentorship(person) => {
                self.army.mentorship_page = 0;
                self.army.mode = ui::ArmyMode::Mentorship(person);
            }
            UiAction::StartMentorship(mentor, learner, discipline) => {
                self.apply_campaign_command(Command::StartMentorship {
                    mentor,
                    learner,
                    discipline,
                })
            }
            UiAction::EndMentorship(learner) => {
                self.apply_campaign_command(Command::EndMentorship { learner })
            }
            UiAction::ReviewHousehold(action) => self.army.household_review = Some(action),
            UiAction::ConfirmHousehold => self.confirm_household(),
            UiAction::CancelHouseholdReview => self.army.household_review = None,
            UiAction::RecoverPersonAtSite(person, site) => {
                self.apply_campaign_command(Command::RecoverPersonAtSite { person, site })
            }
            UiAction::RetirePerson(person, site) => {
                self.apply_campaign_command(Command::RetirePerson { person, site })
            }
            UiAction::AppointGovernor(person, site) => {
                self.apply_campaign_command(Command::AppointGovernor { person, site })
            }
            UiAction::OpenFormationProgression(formation) => {
                self.army.mode = ui::ArmyMode::ProgressionFormation(formation)
            }
            UiAction::TrainPerson(person, class, site) => {
                self.apply_campaign_command(Command::TrainPerson {
                    person,
                    class,
                    site,
                })
            }
            UiAction::PracticeRiding(person, site) => {
                self.apply_campaign_command(Command::PracticeRiding { person, site })
            }
            UiAction::CancelPersonCourse(person) => {
                self.apply_campaign_command(Command::CancelPersonCourse { person })
            }
            UiAction::SpecializeFormation(formation, specialization, site) => self
                .apply_campaign_command(Command::SpecializeFormation {
                    formation,
                    specialization,
                    site,
                }),
            UiAction::CancelFormationCourse(formation) => {
                self.apply_campaign_command(Command::CancelFormationCourse { formation })
            }
            UiAction::SetCommander(army, person) => {
                self.apply_campaign_command(Command::SetCommander { army, person })
            }
            UiAction::BeginMove(army) => self.begin_move(army),
            UiAction::EditMoveGroup => {
                self.movement.stage = ui::MoveStage::Group;
                self.state.overlay = Overlay::MoveGroup;
            }
            UiAction::ToggleMoveArmy(army) => self.toggle_move_army(army),
            UiAction::MoveGroupPage(delta) => {
                self.movement.page = self.movement.page.saturating_add_signed(delta as isize)
            }
            UiAction::ChooseMoveDestination => self.choose_move_destination(),
            UiAction::ReviewMove => self.review_move(),
            UiAction::MoveRoutePage(delta) => {
                self.movement.route_page = self
                    .movement
                    .route_page
                    .saturating_add_signed(delta as isize)
            }
            UiAction::ConfirmMove => self.confirm_move(),
            UiAction::CancelMovementPlan(army) => {
                self.notice = None;
                self.apply_campaign_command(Command::CancelMovementPlan { army });
                self.refresh_movement();
            }
            UiAction::CancelMove => self.cancel_move(),
            UiAction::BeginTransferFormation(formation) => {
                self.begin_transfer(ui::TransferSubject::Formation(formation))
            }
            UiAction::BeginTransferPerson(person) => {
                self.begin_transfer(ui::TransferSubject::Person(person))
            }
            UiAction::SelectTransferArmy(army) => {
                self.army.transfer.army = Some(army);
                self.army.transfer.slot = None;
                self.army.transfer.formation = None;
            }
            UiAction::ClearTransferArmy => {
                self.army.transfer.army = None;
                self.army.transfer.slot = None;
                self.army.transfer.formation = None;
            }
            UiAction::SelectTransferSlot(slot) => self.army.transfer.slot = Some(slot),
            UiAction::SelectTransferFormation(formation) => {
                self.army.transfer.formation = Some(formation)
            }
            UiAction::TransferPage(delta) => {
                self.army.transfer.page = self
                    .army
                    .transfer
                    .page
                    .saturating_add_signed(delta as isize)
            }
            UiAction::ConfirmTransfer => self.confirm_transfer(),
            UiAction::SplitArmy(formation) => self.split_army(formation),
            UiAction::OpenArmies(site) => self.open_armies(site),
            UiAction::ArmyPage(delta) => self.army_page(delta),
            UiAction::SelectFormation(id) => self.army.selected = Some(id),
            UiAction::BeginRecruit(army) => self.begin_recruit(army),
            UiAction::SelectRecruit(kind) => {
                if let ui::ArmyMode::Recruit { kind: selected, .. } = &mut self.army.mode {
                    *selected = Some(kind);
                }
            }
            UiAction::ConfirmRecruit => self.confirm_recruit(),
            UiAction::AskDisband(id) => {
                self.army.mode = ui::ArmyMode::Disband(id);
                self.army.status.clear();
            }
            UiAction::ConfirmDisband(id) => self.confirm_disband(id),
            UiAction::CancelArmyAction => {
                self.army.mode = ui::ArmyMode::Roster;
                self.army.status.clear();
            }
            _ => unreachable!("military action dispatch"),
        }
    }

    fn apply_save_action(&mut self, action: UiAction) {
        match action {
            UiAction::Save => self.open_saves(true),
            UiAction::Load => self.open_saves(false),
            UiAction::LoadLegacy => self.load_slot(SAVE_SLOT),
            UiAction::ImportCampaign => self.import_campaign(),
            UiAction::SelectSave(id) => self.saves.selected = Some(id),
            UiAction::SavePage(delta) => self.saves.change_page(delta),
            UiAction::NameSave(target) => self.name_save(target),
            UiAction::LoadSelectedSave => self.load_selected(),
            UiAction::OverwriteSave => self.name_save(self.saves.selected),
            UiAction::AskDeleteSave => {
                if let Some(id) = self.saves.selected {
                    self.saves.mode = ui::SaveMode::ConfirmDelete(id);
                }
            }
            UiAction::ConfirmDeleteSave(id) => self.delete_save(id),
            UiAction::CommitNamedSave => self.commit_named_save(),
            UiAction::CancelSaveEdit => self.saves.mode = ui::SaveMode::Browse,
            UiAction::RetryStorage => self.retry_storage(),
            UiAction::RetrySave => self.retry_save(),
            UiAction::ContinueUnsaved => self.continue_unsaved(),
            UiAction::EditSaveName(action) => self.edit_save_name(action),
            _ => unreachable!("save action dispatch"),
        }
    }
    fn apply_navigation_action(&mut self, action: UiAction) {
        match action {
            UiAction::HelpPage(delta) => {
                self.help_page = self
                    .help_page
                    .saturating_add_signed(delta as isize)
                    .min(ui::HELP_PAGE_COUNT - 1);
            }
            UiAction::NewGame => {
                if self.save_exists || self.state.campaign.is_some() {
                    self.state.overlay = Overlay::ConfirmNew;
                } else {
                    self.state.overlay = Overlay::Setup;
                }
            }
            UiAction::ConfirmNew => self.state.overlay = Overlay::Setup,
            UiAction::OpenSetupName => {
                self.setup.editing_name = true;
                self.setup.name_error = None;
            }
            UiAction::EditSetupName(action) => self.edit_setup_name(action),
            UiAction::SetupNameDone => self.setup.editing_name = false,
            UiAction::SelectSetupEmblem(emblem) => self.setup.emblem = emblem,
            UiAction::ChangeSetupFactionCount(delta) => {
                self.setup.factions = self
                    .setup
                    .factions
                    .saturating_add_signed(delta as isize)
                    .clamp(self.data.rules.min_factions, self.data.rules.max_factions);
            }
            UiAction::RandomizeSetupSeed => {
                self.setup.seed = u64::from(macroquad::rand::gen_range(1u32, u32::MAX));
            }
            UiAction::StartProductionCampaign => self.start_game(),
            UiAction::Continue => {
                if self.state.campaign.is_some() {
                    self.state.screen = Screen::Campaign;
                } else {
                    self.load();
                }
            }
            UiAction::Open(overlay) => {
                self.state.overlay = overlay;
                if overlay == Overlay::Help {
                    self.help_page = 0;
                }
            }
            UiAction::Back => self.go_back(),
            UiAction::MainMenu => {
                self.state.main_menu();
                self.navigation.reset(&mut self.view);
                self.movement = ui::MoveView::default();
            }
            UiAction::SelectMap(selection) => self.select_map(selection),
            UiAction::EnterRegion(region) => self.enter_region(region),
            UiAction::WorldMap => {
                self.navigation.show_world(&mut self.view);
                self.refresh_move_options();
            }
            UiAction::CloseSelection => self.navigation.clear_selection(),
            _ => unreachable!("navigation action dispatch"),
        }
    }

    fn apply_view_action(&mut self, action: UiAction) {
        match action {
            UiAction::Zoom(factor) => self.view.zoom(vec2(WIDTH / 2.0, HEIGHT / 2.0), factor),
            UiAction::Recenter => self.focus_home(),
            UiAction::ToggleLabels => {
                self.preferences.hide_labels = !self.preferences.hide_labels;
                self.save_preferences();
            }
            UiAction::ToggleContrast => {
                self.preferences.high_contrast = !self.preferences.high_contrast;
                self.save_preferences();
            }
            UiAction::DismissFeedback => {
                self.error = None;
                self.notice = None;
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiAction::Fullscreen => {
                self.fullscreen = !self.fullscreen;
                set_fullscreen(self.fullscreen);
            }
            #[cfg(not(target_arch = "wasm32"))]
            UiAction::Quit => self.running = false,
            _ => unreachable!("view action dispatch"),
        }
    }
}
