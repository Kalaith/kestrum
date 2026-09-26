//! High-level game loop, state transitions, and toolkit integration.

use crate::data::GameData;
use crate::state::{migrate_save_value, GameSession, SaveData};
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    delete_slot, get_save_slots, load_from_slot_with_migration, save_to_slot_with_version,
    slot_exists,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{ScrollArea, VirtualUi};

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    camera: CameraTransform,
    camera_drag: Option<Vec2>,
    events: EventBus<UiAction>,
    settings: GameSettings,
    debug: DebugOverlay,
    action_scroll: ScrollArea,
    paused: bool,
    frame_dt: f32,
    save_exists: bool,
    save_slots: Vec<String>,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Template embedded data failed to load: {}", err);
        });

        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.75, 0.2, 0.8, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let loaded_assets = assets.load_texture_configs(&data.texture_manifest).await;

        let mut notifications = NotificationManager::new();
        notifications.info(format!(
            "Template booted with macroquad-toolkit systems; {} manifest textures loaded",
            loaded_assets
        ));

        let session = GameSession::new(&data.config);
        let camera = CameraTransform::new(Vec2::ZERO, 1.0).expect("valid initial camera");

        let settings = GameSettings::load(&data.config.game_name);
        let mut debug = DebugOverlay::new();
        debug.visible = settings.show_fps;
        let mut game = Self {
            settings,
            debug,
            action_scroll: ScrollArea::new(),
            paused: false,
            frame_dt: 0.0,
            data,
            session,
            assets,
            notifications,
            camera,
            camera_drag: None,
            events: EventBus::new(),
            save_exists: false,
            save_slots: Vec::new(),
        };
        game.refresh_save_state();
        game
    }

    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.session = GameSession::new(&self.data.config);
        self.notifications.clear();
        self.events.drain().for_each(drop);
        self.action_scroll = ScrollArea::new();
        self.paused = false;
        self.debug.visible = false;
        self.save_exists = false;
        self.save_slots.clear();
        self.apply_action(UiAction::ResetCamera);
        match scene {
            "gameplay" => {}
            "paused" => self.paused = true,
            "scrolled" => self.action_scroll.set_offset(88.0),
            "zoomed" => {
                self.apply_action(UiAction::ZoomCamera(1));
                self.apply_action(UiAction::PanCamera(1, 0));
            }
            _ => panic!("Unknown template capture scene: {scene}"),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_dt = dt;
        self.debug.record_frame(dt);
        self.notifications.update(dt);

        let input = InputState::capture();
        if input.escape_pressed {
            self.events.push(UiAction::TogglePause);
        }
        if input.space_pressed {
            if let Some(action) = self.data.ordered_actions().first() {
                self.events.push(UiAction::RunAction(action.id.clone()));
            }
        }
        if is_key_pressed(KeyCode::S) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(KeyCode::L) {
            self.events.push(UiAction::Load);
        }
        if let Some((dx, dy)) = ui::tile_move_from_keys() {
            self.session.move_selection(dx, dy);
        }

        let viewport = VirtualUi::new(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let mouse = viewport.mouse_position();
        let rect = ui::world_grid_rect();
        if rect.contains(mouse) {
            if is_mouse_button_pressed(MouseButton::Right) {
                self.camera_drag = Some(mouse);
            }
            if is_mouse_button_down(MouseButton::Right) {
                if let Some(last) = self.camera_drag.replace(mouse) {
                    self.camera.pan_screen(mouse - last);
                }
            } else {
                self.camera_drag = None;
            }
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                self.camera
                    .zoom_at(rect, mouse, 1.1_f32.powf(wheel), (0.75, 1.75));
            }
        } else {
            self.camera_drag = None;
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.events.push(UiAction::ZoomCamera(1));
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.events.push(UiAction::ZoomCamera(-1));
        }

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
        self.camera.constrain(
            ui::world_grid_rect(),
            CameraBounds::new(vec2(-240.0, -160.0), vec2(240.0, 160.0)),
            CameraBoundsPolicy::TargetInside,
        );
        if !self.paused {
            self.session.update_energy(&self.data.config, dt);
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let ctx = UiContext {
            data: &self.data,
            session: &self.session,
            save_exists: self.save_exists,
            save_slots: &self.save_slots,
            loaded_assets: self.assets.len(),
            camera: self.camera,
            paused: self.paused,
            show_stats: self.debug.visible,
            dt: self.frame_dt,
            ui: &virtual_ui,
        };

        let actions = ui::draw_game_ui(ctx, &mut self.action_scroll);
        end_virtual_ui_frame();

        for action in actions {
            self.events.push(action);
        }

        self.debug.draw(&[]);
        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::TopRight,
                ..Default::default()
            });
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::TogglePause => self.paused = !self.paused,
            UiAction::ToggleStats => {
                self.debug.toggle();
                self.settings.show_fps = self.debug.visible;
                if let Err(err) = self.settings.save(&self.data.config.game_name) {
                    self.notifications
                        .warning(format!("Settings save failed: {err}"));
                }
            }
            UiAction::PanCamera(x, y) => {
                self.camera.pan_screen(vec2(x as f32, y as f32) * -48.0);
            }
            UiAction::ZoomCamera(direction) => {
                let rect = ui::world_grid_rect();
                self.camera.zoom_at(
                    rect,
                    rect.center(),
                    if direction > 0 { 1.2 } else { 1.0 / 1.2 },
                    (0.75, 1.75),
                );
            }
            UiAction::ResetCamera => {
                self.camera = CameraTransform::new(Vec2::ZERO, 1.0).expect("valid initial camera");
                self.camera_drag = None;
            }
            UiAction::NewGame => {
                self.session = GameSession::new(&self.data.config);
                self.paused = false;
                self.action_scroll.set_offset(0.0);
                self.apply_action(UiAction::ResetCamera);
                self.notifications.info("Started a fresh template session");
            }
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::DeleteSave => self.delete_save(),
            UiAction::RunAction(id) => self.run_data_action(&id),
            UiAction::SelectTile(tile) => {
                self.session.select_tile(tile);
                self.notifications
                    .info(format!("Selected tile {}, {}", tile.x, tile.y));
            }
        }
    }

    fn run_data_action(&mut self, action_id: &str) {
        if self.paused {
            return;
        }
        let Some(action) = self.data.actions.get(action_id) else {
            self.notifications
                .warning(format!("Unknown action: {}", action_id));
            return;
        };

        if self.session.apply_action(action) {
            self.notifications
                .success(format!("{} complete", action.name));
        } else {
            self.notifications.warning("Not enough energy");
        }
    }

    fn save_game(&mut self) {
        let save = self.session.to_save(&self.data.config.version);
        match save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        ) {
            Ok(()) => {
                self.notifications.success("Saved with toolkit save slots");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.danger(format!("Save failed: {}", err)),
        }
    }

    fn load_game(&mut self) {
        let loaded: Result<SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| migrate_save_value(version, value, &self.data.config),
        );

        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save);
                self.notifications
                    .success("Loaded save with migration support");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.warning(format!("Load failed: {}", err)),
        }
    }

    fn delete_save(&mut self) {
        match delete_slot(&self.data.config.game_name, &self.data.config.save_slot) {
            Ok(()) => {
                self.notifications.info("Deleted template save slot");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.danger(format!("Delete failed: {}", err)),
        }
    }

    fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
        self.save_slots = get_save_slots(&self.data.config.game_name);
    }
}
