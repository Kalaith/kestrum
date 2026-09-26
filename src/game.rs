//! Application coordination, toolkit persistence, and routed atlas gestures.

use crate::ui::{self, UiAction};
use kestrum::{
    data::GameData,
    navigation::{MapView, HEIGHT, WIDTH},
    state::{Campaign, GameState, Overlay, Preferences, Screen, SAVE_SLOT},
};
use macroquad::prelude::*;
use macroquad_toolkit::{
    assets::AssetManager,
    input::gestures::{GestureTouch, TouchGesture},
    persistence::{
        json_key_exists, load_from_slot, load_json_key, save_json_key, save_to_slot, slot_exists,
    },
    ui::{begin_virtual_ui_frame, end_virtual_ui_frame, Pointer, VirtualUi},
};

pub struct Game {
    data: GameData,
    state: GameState,
    preferences: Preferences,
    assets: AssetManager,
    view: MapView,
    gesture: TouchGesture,
    origin: Option<Vec2>,
    was_down: bool,
    map_gesture: bool,
    save_exists: bool,
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
        let mut assets = AssetManager::new();
        assets
            .load_texture_with_filter("atlas", &data.map_path, FilterMode::Linear)
            .await?;
        assets.load_font("cinzel", &data.font_path).await?;
        assets.load_font("body", &data.body_font_path).await?;
        let mut game = Self {
            save_exists: slot_exists(&data.game_id, SAVE_SLOT),
            data,
            assets,
            state: GameState::default(),
            preferences: Preferences::default(),
            view: MapView::default(),
            gesture: TouchGesture::new(),
            origin: None,
            was_down: false,
            map_gesture: false,
            capture: false,
            notice: None,
            error: None,
            #[cfg(not(target_arch = "wasm32"))]
            fullscreen: false,
            running: true,
        };
        if json_key_exists("kestrum", "preferences") {
            match load_json_key("kestrum", "preferences") {
                Ok(preferences) => game.preferences = preferences,
                Err(error) => game.error = Some(error),
            }
        }
        Ok(game)
    }

    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture = true;
        self.state = GameState::default();
        self.preferences = Preferences::default();
        self.view.reset();
        self.notice = None;
        self.error = None;
        self.save_exists = false;
        match scene.trim_end_matches("_minimum") {
            "title" => {}
            "gameplay" => self.state.new_game(),
            "zoomed" => {
                self.state.new_game();
                self.view.zoom(vec2(720.0, 330.0), 2.0);
            }
            "menu" => {
                self.state.new_game();
                self.state.overlay = Overlay::Menu;
            }
            "settings" => self.state.overlay = Overlay::Settings,
            "help" => self.state.overlay = Overlay::Help,
            "confirm_new" => self.state.overlay = Overlay::ConfirmNew,
            "save_error" => {
                self.state.new_game();
                self.state.overlay = Overlay::Menu;
                self.error = Some(format!(
                    "{}: storage is unavailable",
                    self.data.text("save_failed")
                ));
            }
            _ => panic!("Unknown Kestrum capture scene: {scene}"),
        }
    }

    pub fn frame(&mut self, dt: f32) {
        if let Some((_, time)) = &mut self.notice {
            *time -= dt;
            if *time <= 0.0 {
                self.notice = None;
            }
        }
        clear_background(Color::new(0.06, 0.10, 0.10, 1.0));
        let viewport = begin_virtual_ui_frame(WIDTH, HEIGHT);
        let pointer = self.input(&viewport, dt);
        let ctx = ui::Context {
            data: &self.data,
            state: &self.state,
            preferences: &self.preferences,
            view: &self.view,
            assets: &self.assets,
            pointer,
            origin: self.origin,
            save_exists: self.save_exists,
        };
        let action = ui::draw(&ctx);
        let message = self
            .error
            .as_ref()
            .or(self.notice.as_ref().map(|(message, _)| message));
        let feedback_action = message.and_then(|message| ui::feedback(&ctx, message));
        end_virtual_ui_frame();
        if let Some(intent) = feedback_action.or(action) {
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
            self.state.back();
        }
        if pointer.down && !self.was_down {
            self.origin = Some(pointer.position);
            self.map_gesture = self.state.screen == Screen::Campaign
                && self.state.overlay == Overlay::None
                && !ui::map_controls_contain(pointer.position);
        }
        let touches = self.gesture_touches(viewport, pointer);
        let frame = self.gesture.update_with(&touches);
        let playing = self.state.screen == Screen::Campaign && self.state.overlay == Overlay::None;
        if playing {
            if self.map_gesture {
                self.view.gesture(&frame);
            }
            if !ui::map_controls_contain(pointer.position) && mouse_wheel().1 != 0.0 {
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
            UiAction::Back => self.state.back(),
            UiAction::MainMenu => {
                self.state.main_menu();
                self.view.reset();
            }
            UiAction::Save => self.save(true),
            UiAction::Load => self.load(),
            UiAction::EndTurn => {
                if self.state.end_turn() {
                    self.save(false);
                }
            }
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

    fn start_game(&mut self) {
        self.state.new_game();
        self.view.reset();
        self.error = None;
        self.save(false);
    }

    fn save(&mut self, announce: bool) {
        if self.capture {
            return;
        }
        let Some(campaign) = &self.state.campaign else {
            return;
        };
        match save_to_slot(&self.data.game_id, SAVE_SLOT, campaign) {
            Ok(()) => {
                self.save_exists = true;
                self.error = None;
                if announce {
                    self.notice = Some((self.data.text("save_success").into(), 3.0));
                }
            }
            Err(error) => self.error = Some(format!("{}: {error}", self.data.text("save_failed"))),
        }
    }

    fn load(&mut self) {
        let loaded: Result<Campaign, String> = load_from_slot(&self.data.game_id, SAVE_SLOT);
        match loaded.and_then(|campaign| self.state.load_campaign(campaign)) {
            Ok(()) => {
                self.view.reset();
                self.error = None;
                self.notice = Some((self.data.text("load_success").into(), 3.0));
            }
            Err(error) => self.error = Some(format!("{}: {error}", self.data.text("load_failed"))),
        }
    }

    fn save_preferences(&mut self) {
        if self.capture {
            return;
        }
        if let Err(error) = save_json_key("kestrum", "preferences", &self.preferences) {
            self.error = Some(format!("{}: {error}", self.data.text("settings_failed")));
        }
    }
}
