//! Application coordination, toolkit persistence, and routed atlas gestures.

use crate::ui::{self, UiAction};
mod campaign;
mod saves;
mod storage;
mod world;
use kestrum::{
    data::GameData,
    engine::{self, Command},
    navigation::{MapNavigation, MapView, HEIGHT, WIDTH},
    state::{Campaign, GameState, Overlay, Preferences, Screen, SAVE_SLOT},
};
use macroquad::prelude::*;
use macroquad_toolkit::{
    assets::AssetManager,
    input::gestures::{GestureTouch, TouchGesture},
    persistence::{json_key_exists, load_json_key, save_json_key, slot_exists},
    ui::{begin_virtual_ui_frame, end_virtual_ui_frame, Pointer, VirtualUi},
};

pub struct Game {
    data: GameData,
    state: GameState,
    preferences: Preferences,
    assets: AssetManager,
    view: MapView,
    navigation: MapNavigation,
    gesture: TouchGesture,
    origin: Option<Vec2>,
    was_down: bool,
    map_gesture: bool,
    save_exists: bool,
    legacy_save_exists: bool,
    import_save_exists: bool,
    storage: Option<macroquad_toolkit::persistence::IndexedKeyStore>,
    library: Option<kestrum::state::persistence::SaveLibrary>,
    storage_checked: bool,
    saves: ui::SaveView,
    pending_save: Option<storage::PendingWrite>,
    retry_save_when_ready: bool,
    save_error: String,
    npc_delay: f32,
    capture: bool,
    notice: Option<(String, f32)>,
    error: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    fullscreen: bool,
    pub running: bool,
}

impl Game {
    pub async fn new() -> Result<Self, String> {
        let data = GameData::load()?;
        let capture = macroquad_toolkit::capture::CaptureConfig::all_from_env("KESTRUM").is_some();
        let mut assets = AssetManager::new();
        assets
            .load_texture_with_filter("atlas", &data.presentation.map_path, FilterMode::Linear)
            .await?;
        assets
            .load_font("cinzel", &data.presentation.font_path)
            .await?;
        assets
            .load_font("body", &data.presentation.body_font_path)
            .await?;
        // Prepare all fixed UI sizes together before the first visible frame.
        // Runtime names and messages are prepared separately before UI drawing.
        let mut characters: Vec<char> = (b' '..=b'~').map(char::from).collect();
        characters.extend("…—–×·".chars());
        for text in data.presentation.text.values().chain([
            &data.presentation.title,
            &data.presentation.subtitle,
            &data.presentation.edition,
        ]) {
            characters.extend(text.chars());
        }
        characters.sort_unstable();
        characters.dedup();
        let common_text: String = characters.into_iter().collect();
        for (key, sizes) in [
            ("cinzel", &[15, 17, 18, 19, 20, 21, 24, 28, 30][..]),
            ("body", &[16, 18, 19, 20, 21, 23, 25][..]),
        ] {
            let font = assets
                .get_font(key)
                .ok_or_else(|| format!("Loaded UI font {key} is unavailable"))?;
            let mut samples: Vec<_> = sizes
                .iter()
                .map(|size| (*size, common_text.as_str()))
                .collect();
            if key == "cinzel" {
                samples.push((76, &data.presentation.title));
                samples.extend(
                    data.presentation
                        .geography
                        .iter()
                        .map(|label| (label.size as u16, label.name.as_str())),
                );
            }
            macroquad_toolkit::ui::prepare_font_text(font, &samples);
        }
        next_frame().await;
        let mut game = Self {
            save_exists: false,
            legacy_save_exists: !capture && slot_exists(&data.presentation.game_id, SAVE_SLOT),
            import_save_exists: !capture
                && slot_exists(&data.presentation.game_id, campaign::STRATEGIC_SLOT),
            storage: None,
            library: None,
            storage_checked: false,
            saves: ui::SaveView::default(),
            pending_save: None,
            retry_save_when_ready: false,
            save_error: String::new(),
            npc_delay: 0.0,
            data,
            assets,
            state: GameState::default(),
            preferences: Preferences::default(),
            view: MapView::default(),
            navigation: MapNavigation::default(),
            gesture: TouchGesture::new(),
            origin: None,
            was_down: false,
            map_gesture: false,
            capture,
            notice: None,
            error: None,
            #[cfg(not(target_arch = "wasm32"))]
            fullscreen: false,
            running: true,
        };
        if !capture && json_key_exists("kestrum", "preferences") {
            match load_json_key("kestrum", "preferences") {
                Ok(preferences) => game.preferences = preferences,
                Err(error) => game.error = Some(error),
            }
        }
        if !capture {
            game.retry_storage();
        }
        Ok(game)
    }

    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture = true;
        self.state = GameState::default();
        self.preferences = Preferences::default();
        self.navigation.reset(&mut self.view);
        self.notice = None;
        self.error = None;
        self.save_exists = false;
        self.legacy_save_exists = false;
        self.import_save_exists = false;
        self.saves = ui::SaveView::default();
        self.save_error.clear();
        match scene.trim_end_matches("_minimum") {
            "title" => {}
            "gameplay" => self.capture_campaign(),
            "npc_paused" | "npc_menu" | "npc_long_name" => {
                self.capture_campaign();
                self.apply_campaign_command(Command::EndTurn);
                self.apply_campaign_command(Command::SetNpcPaused(true));
                if scene.starts_with("npc_menu") {
                    self.state.overlay = Overlay::Menu;
                }
                if scene.starts_with("npc_long_name") {
                    if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                        let active = campaign.active_faction();
                        if let Some(faction) = campaign.factions.get_mut(&active) {
                            faction.name = "The Kingdom of Silver Hawthorns".into();
                        }
                    }
                }
            }
            "legacy" => {
                self.state.campaign = Some(Campaign::Shell(Default::default()));
                self.state.screen = Screen::Campaign;
            }
            "zoomed" => {
                self.capture_campaign();
                self.view.zoom(vec2(720.0, 330.0), 2.0);
            }
            "world_selected" | "region" | "region_selected" | "region_partial"
            | "region_contested" | "region_long_name" => self.capture_world(scene),
            "menu" => {
                self.capture_campaign();
                self.state.overlay = Overlay::Menu;
            }
            "settings" => self.state.overlay = Overlay::Settings,
            "help" => self.state.overlay = Overlay::Help,
            "confirm_new" => self.state.overlay = Overlay::ConfirmNew,
            "save_list" | "save_name" | "save_symbols" | "save_busy" | "save_invalid"
            | "save_delete" | "save_recovery" => self.capture_saves(scene),
            "save_error" => {
                self.capture_campaign();
                self.state.overlay = Overlay::Menu;
                self.error = Some(format!(
                    "{}: storage is unavailable",
                    self.data.presentation.text("save_failed")
                ));
            }
            _ => panic!("Unknown Kestrum capture scene: {scene}"),
        }
    }

    pub fn frame(&mut self, dt: f32) {
        self.poll_storage();
        self.progress_npcs(dt);
        if self.state.overlay == Overlay::Saves {
            if let Some(error) = self.error.take() {
                self.saves.status = error;
            }
        }
        if let Some((_, time)) = &mut self.notice {
            *time -= dt;
            if *time <= 0.0 {
                self.notice = None;
            }
        }
        clear_background(Color::new(0.06, 0.10, 0.10, 1.0));
        let viewport = begin_virtual_ui_frame(WIDTH, HEIGHT);
        let pointer = self.input(&viewport, dt);
        let campaign_view = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| engine::project(campaign, campaign.player).ok());
        let ctx = ui::Context {
            data: &self.data.presentation,
            state: &self.state,
            preferences: &self.preferences,
            view: &self.view,
            navigation: &self.navigation,
            assets: &self.assets,
            pointer,
            origin: self.origin,
            save_exists: self.save_exists,
            legacy_save_exists: self.legacy_save_exists,
            import_save_exists: self.import_save_exists,
            saves: &self.saves,
            save_error: &self.save_error,
            campaign_view: campaign_view.as_ref(),
        };
        let message = self
            .error
            .as_ref()
            .or(self.notice.as_ref().map(|(message, _)| message))
            .filter(|_| !matches!(self.state.overlay, Overlay::Saves | Overlay::SaveRecovery));
        ui::prepare_dynamic_text(&ctx, message.map(String::as_str));
        let action = ui::draw(&ctx);
        let map_action = self.map_selection_action(pointer);
        let feedback_action = message.and_then(|message| ui::feedback(&ctx, message));
        end_virtual_ui_frame();
        if let Some(intent) = feedback_action.or(action).or(map_action) {
            self.apply(intent);
        }
        if !pointer.down {
            self.origin = None;
        }
        self.was_down = pointer.down;
    }

    fn input(&mut self, viewport: &VirtualUi, dt: f32) -> Pointer {
        let mut pointer = Pointer::read(|position| viewport.screen_to_ui(position));
        if self.capture {
            return pointer.suppressed();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.go_back();
        }
        let naming = self.state.overlay == Overlay::Saves
            && matches!(self.saves.mode, ui::SaveMode::Name { .. });
        for edit in macroquad_toolkit::ui::text_entry::read_text_edits(naming) {
            self.edit_save_name(macroquad_toolkit::ui::text_entry::TextEntryAction::Edit(
                edit,
            ));
        }
        // A quick click can press and release between frames. Macroquad still
        // records the press edge even though `down` is already false.
        let started = is_mouse_button_pressed(MouseButton::Left)
            || touches()
                .iter()
                .any(|touch| touch.phase == TouchPhase::Started);
        if !self.was_down && (pointer.down || started) {
            self.origin = Some(pointer.position);
            self.map_gesture = self.state.screen == Screen::Campaign
                && self.state.overlay == Overlay::None
                && !self.map_controls_block(pointer.position);
        }
        let touches = self.gesture_touches(viewport, pointer);
        let frame = self.gesture.update_with(&touches);
        let playing = self.state.screen == Screen::Campaign && self.state.overlay == Overlay::None;
        if playing {
            if self.map_gesture {
                self.view.gesture(&frame);
            }
            if !self.map_controls_block(pointer.position) && mouse_wheel().1 != 0.0 {
                self.view
                    .zoom(pointer.position, 1.12_f32.powf(mouse_wheel().1));
            }
            let mut delta = Vec2::ZERO;
            for (key, direction) in [
                (KeyCode::Left, vec2(1.0, 0.0)),
                (KeyCode::Right, vec2(-1.0, 0.0)),
                (KeyCode::Up, vec2(0.0, 1.0)),
                (KeyCode::Down, vec2(0.0, -1.0)),
            ] {
                if is_key_down(key) {
                    delta += direction * dt * 420.0;
                }
            }
            self.view.pan(delta);
        }
        if frame.claimed {
            pointer.released = false;
        }
        pointer
    }

    fn gesture_touches(&self, viewport: &VirtualUi, pointer: Pointer) -> Vec<GestureTouch> {
        let contacts = touches();
        if !contacts.is_empty() {
            let dpi = screen_dpi_scale().max(1.0);
            return contacts
                .into_iter()
                .map(|touch| {
                    GestureTouch::new(
                        touch.id,
                        viewport.screen_to_ui(touch.position / dpi),
                        touch.phase,
                    )
                })
                .collect();
        }
        let phase = if pointer.down {
            if self.was_down {
                TouchPhase::Moved
            } else {
                TouchPhase::Started
            }
        } else if pointer.released {
            TouchPhase::Ended
        } else {
            return Vec::new();
        };
        vec![GestureTouch::new(0, pointer.position, phase)]
    }

    fn apply(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => {
                if self.save_exists || self.state.campaign.is_some() {
                    self.state.overlay = Overlay::ConfirmNew;
                } else {
                    self.start_game();
                }
            }
            UiAction::ConfirmNew => self.start_game(),
            UiAction::Continue => {
                if self.state.campaign.is_some() {
                    self.state.screen = Screen::Campaign;
                } else {
                    self.load();
                }
            }
            UiAction::Open(overlay) => self.state.overlay = overlay,
            UiAction::Back => self.go_back(),
            UiAction::MainMenu => {
                self.state.main_menu();
                self.navigation.reset(&mut self.view);
            }
            UiAction::SelectMap(selection) => self.select_map(selection),
            UiAction::EnterRegion(region) => self.enter_region(region),
            UiAction::WorldMap => self.navigation.show_world(&mut self.view),
            UiAction::CloseSelection => self.navigation.clear_selection(),
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
            UiAction::EndTurn => self.apply_campaign_command(Command::EndTurn),
            UiAction::PauseNpcs(paused) => {
                self.apply_campaign_command(Command::SetNpcPaused(paused))
            }
            UiAction::StepNpc => self.apply_campaign_command(Command::StepNpc),
            UiAction::Zoom(factor) => self.view.zoom(vec2(WIDTH / 2.0, HEIGHT / 2.0), factor),
            UiAction::Recenter => self.view.reset(),
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
        }
    }

    fn save_preferences(&mut self) {
        if self.capture {
            return;
        }
        if let Err(error) = save_json_key("kestrum", "preferences", &self.preferences) {
            self.error = Some(format!(
                "{}: {error}",
                self.data.presentation.text("settings_failed")
            ));
        }
    }
}
