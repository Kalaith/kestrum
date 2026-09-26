//! Immediate-mode UI built from macroquad-toolkit surfaces and helpers.

use crate::data::GameData;
use crate::state::GameSession;
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;
use macroquad_toolkit::grid::{FogState, TilePos};
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    is_fully_visible, Pointer, RectExt, ScrollArea, ScrollInput, VirtualUi,
};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    NewGame,
    TogglePause,
    ToggleStats,
    PanCamera(i32, i32),
    ZoomCamera(i32),
    ResetCamera,
    Save,
    Load,
    DeleteSave,
    RunAction(String),
    SelectTile(TilePos),
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a GameSession,
    pub save_exists: bool,
    pub save_slots: &'a [String],
    pub loaded_assets: usize,
    pub camera: CameraTransform,
    pub paused: bool,
    pub dt: f32,
    pub show_stats: bool,
    pub ui: &'a VirtualUi,
}

pub fn draw_game_ui(ctx: UiContext<'_>, scroll: &mut ScrollArea) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let pointer = Pointer::read(|point| ctx.ui.screen_to_ui(point));

    draw_header(ctx.data, ctx.session);
    draw_world_panel(&ctx);
    draw_control_panel(&ctx, pointer, scroll, &mut actions);
    let pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    draw_camera_controls(pointer, &mut actions);
    if pointer.released {
        if let Some(tile) = selected_tile_at(&ctx, pointer.position) {
            actions.push(UiAction::SelectTile(tile));
        }
    }
    draw_footer(&ctx, pointer, &mut actions);

    actions
}

fn draw_header(data: &GameData, session: &GameSession) {
    let rect = Rect::new(18.0, 16.0, LOGICAL_WIDTH - 36.0, 64.0);
    let style = SurfaceStyle::new(Color::new(0.08, 0.09, 0.12, 0.96))
        .with_border(1.0, dark::ACCENT)
        .with_top_highlight(2.0, Color::new(0.55, 0.72, 0.95, 0.75));
    draw_surface(rect, &style);

    draw_text_centered_in_box_ex(
        &data.config.display_name,
        rect.x + 18.0,
        rect.y + 8.0,
        rect.w - 380.0,
        48.0,
        TextStyle::new(30.0, dark::TEXT_BRIGHT),
    );

    draw_badge(
        Rect::new(rect.right() - 332.0, rect.y + 18.0, 100.0, 28.0),
        &format!("Turn {}", session.player.turn),
        Color::new(0.18, 0.24, 0.32, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(rect.right() - 218.0, rect.y + 18.0, 92.0, 28.0),
        &format_money(session.player.points),
        Color::new(0.18, 0.28, 0.20, 1.0),
        dark::TEXT,
    );
    draw_badge(
        Rect::new(rect.right() - 112.0, rect.y + 18.0, 94.0, 28.0),
        &format!("v{}", data.config.version),
        Color::new(0.22, 0.19, 0.30, 1.0),
        dark::TEXT,
    );
}

fn draw_world_panel(ctx: &UiContext<'_>) {
    let rect = world_panel_rect();
    let style = SurfaceStyle::new(Color::new(0.06, 0.065, 0.08, 0.96))
        .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.65))
        .with_inner_border(4.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.05))
        .with_header(42.0, Color::new(0.09, 0.105, 0.13, 1.0))
        .with_header_divider(1.0, Color::new(0.38, 0.45, 0.58, 0.4));
    draw_surface_with_title(
        rect,
        Some("Toolkit Grid + Camera Example"),
        &style,
        TextStyle::new(18.0, dark::TEXT),
    );

    let grid_rect = world_grid_rect();
    draw_grid_demo(ctx, grid_rect);
}

fn draw_grid_demo(ctx: &UiContext<'_>, rect: Rect) {
    let grid = &ctx.session.world.fog;
    let view = GridView::new(ctx, rect);
    let selected = ctx.session.player.selected_tile;

    for (pos, fog) in grid.iter_with_pos() {
        let tile_rect = view.tile_rect(pos);
        if !is_fully_visible(tile_rect, rect)
            || tile_rect.x < rect.x
            || tile_rect.right() > rect.right()
        {
            continue;
        }

        let base = match fog {
            FogState::Hidden => Color::new(0.08, 0.08, 0.10, 1.0),
            FogState::Revealed => Color::new(0.16, 0.17, 0.20, 1.0),
            FogState::Visible => Color::new(0.23, 0.30, 0.25, 1.0),
        };
        let fill = if ctx.session.world.reachable.contains(&pos) {
            Color::new(base.r + 0.08, base.g + 0.08, base.b + 0.05, 1.0)
        } else {
            base
        };
        draw_rectangle(tile_rect.x, tile_rect.y, tile_rect.w, tile_rect.h, fill);

        if pos == selected {
            draw_rectangle_lines(
                tile_rect.x,
                tile_rect.y,
                tile_rect.w,
                tile_rect.h,
                3.0,
                dark::ACCENT,
            );
        }
    }

    let footer = Rect::new(rect.x, rect.bottom() + 4.0, rect.w, 28.0);
    draw_text_centered_in_box(
        &format!(
            "Selected tile: {}, {} | Reachable: {}",
            selected.x,
            selected.y,
            ctx.session.world.reachable.len()
        ),
        footer.x,
        footer.y,
        footer.w,
        footer.h,
        16.0,
        dark::TEXT_DIM,
    );
}

fn draw_control_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    scroll: &mut ScrollArea,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(852.0, 96.0, 410.0, 520.0);
    let style = SurfaceStyle::new(Color::new(0.08, 0.085, 0.105, 0.97))
        .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.65))
        .with_header(42.0, Color::new(0.105, 0.12, 0.15, 1.0))
        .with_header_divider(1.0, Color::new(0.38, 0.45, 0.58, 0.4));
    draw_surface_with_title(
        rect,
        Some("Toolkit UI + Persistence"),
        &style,
        TextStyle::new(18.0, dark::TEXT),
    );

    let content = rect.inset(18.0);
    let mut y = content.y + 40.0;
    meter(
        Rect::new(content.x, y, content.w, 24.0),
        ctx.session.player.energy,
        ctx.data.config.max_energy,
        dark::POSITIVE,
        Some(&format!(
            "Energy {:.0}/{:.0}",
            ctx.session.player.energy, ctx.data.config.max_energy
        )),
    );
    y += 42.0;

    draw_text_block(
        "Actions — drag to scroll",
        content.x,
        y,
        content.w,
        24.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    y += 30.0;
    let view = Rect::new(content.x, y, content.w, 176.0);
    let layout = GridLayout::new(view.x, view.y, view.w - 16.0, 8.0, 1, 80.0);
    let height = layout.content_height(ctx.data.actions.len());
    scroll.update_with(
        view,
        height,
        ScrollInput {
            pointer: pointer.position,
            down: pointer.down,
            pressed: is_mouse_button_pressed(MouseButton::Left)
                || touches()
                    .iter()
                    .any(|touch| touch.phase == TouchPhase::Started),
            wheel: mouse_wheel().1,
            dt: ctx.dt,
        },
    );
    let card_pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    for (index, action) in ctx.data.ordered_actions().into_iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, scroll.offset());
        let rect = Rect::new(x, y, w, h);
        if is_fully_visible(rect, view)
            && draw_action_card(
                rect,
                action,
                !ctx.paused && ctx.session.can_run_action(action),
                card_pointer,
            )
        {
            actions.push(UiAction::RunAction(action.id.clone()));
        }
    }
    scroll.draw_scrollbar(view, height);
    let pointer = card_pointer;
    y = view.bottom() + 16.0;
    draw_text_block(
        "Save Data",
        content.x,
        y,
        content.w,
        24.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    y += 28.0;

    let btn_w = (content.w - 10.0) / 2.0;
    if virtual_button(
        Rect::new(content.x, y, btn_w, 44.0),
        "Save",
        true,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::Save);
    }
    if virtual_button(
        Rect::new(content.x + btn_w + 10.0, y, btn_w, 44.0),
        "Load",
        ctx.save_exists,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::Load);
    }
    y += 52.0;

    if virtual_button(
        Rect::new(content.x, y, btn_w, 44.0),
        "New Game",
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::NewGame);
    }
    if virtual_button(
        Rect::new(content.x + btn_w + 10.0, y, btn_w, 44.0),
        "Delete Save",
        ctx.save_exists,
        ButtonTone::Danger,
        pointer,
    ) {
        actions.push(UiAction::DeleteSave);
    }
    y += 52.0;

    let saves = if ctx.save_slots.is_empty() {
        "No save slots found".to_owned()
    } else {
        format!("Save slots: {}", ctx.save_slots.join(", "))
    };
    draw_text_block(
        &format!(
            "{}\nManifest textures loaded: {}\nToolkit save slot: {}",
            saves, ctx.loaded_assets, ctx.data.config.save_slot
        ),
        content.x,
        y,
        content.w,
        74.0,
        15.0,
        3.0,
        dark::TEXT_DIM,
    );
}

fn draw_action_card(
    rect: Rect,
    action: &crate::data::ActionDef,
    enabled: bool,
    pointer: Pointer,
) -> bool {
    let hovered = enabled && pointer.hovering_over(rect);
    let fill = if hovered {
        Color::new(0.15, 0.18, 0.23, 1.0)
    } else {
        Color::new(0.11, 0.125, 0.16, 1.0)
    };
    let style = SurfaceStyle::new(fill)
        .with_left_accent(
            4.0,
            if enabled {
                dark::ACCENT
            } else {
                dark::TEXT_DIM
            },
        )
        .with_border(1.0, Color::new(0.5, 0.55, 0.65, 0.35));
    draw_surface(rect, &style);

    draw_text_block(
        &action.name,
        rect.x + 16.0,
        rect.y + 6.0,
        rect.w - 32.0,
        22.0,
        18.0,
        0.0,
        if enabled { dark::TEXT } else { dark::TEXT_DIM },
    );
    draw_text_block(
        &action.description,
        rect.x + 16.0,
        rect.y + 30.0,
        rect.w - 32.0,
        24.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        &format!(
            "-{:.0} energy / +{} points",
            action.energy_cost, action.points_reward
        ),
        rect.x + 16.0,
        rect.y + 56.0,
        rect.w - 32.0,
        18.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    enabled && pointer.released_on(rect)
}

// Toolkit renders the button; Pointer supplies DPI-correct touch activation.
fn virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    pointer: Pointer,
) -> bool {
    button_rect_tone_at(rect, text, enabled, tone, pointer.position);
    enabled && pointer.released_on(rect)
}

fn draw_footer(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(18.0, 632.0, LOGICAL_WIDTH - 36.0, 70.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.06, 0.075, 0.96))
            .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.45)),
    );
    draw_text_block(
        "Tap a tile to explore. Tap an action to earn points. Drag the action list for more. Use the map controls to pan, zoom, or reset the view.",
        rect.x + 18.0,
        rect.y + 14.0,
        rect.w - 310.0,
        rect.h - 20.0,
        17.0,
        4.0,
        dark::TEXT_DIM,
    );
    for (index, label, action) in [
        (
            0,
            if ctx.paused { "Resume" } else { "Pause" },
            UiAction::TogglePause,
        ),
        (
            1,
            if ctx.show_stats {
                "Hide FPS"
            } else {
                "Show FPS"
            },
            UiAction::ToggleStats,
        ),
    ] {
        if virtual_button(
            Rect::new(
                rect.right() - 274.0 + index as f32 * 132.0,
                rect.y + 12.0,
                122.0,
                44.0,
            ),
            label,
            true,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(action);
        }
    }
}

pub fn tile_move_from_keys() -> Option<(i32, i32)> {
    if is_key_pressed(KeyCode::Up) {
        Some((0, -1))
    } else if is_key_pressed(KeyCode::Right) {
        Some((1, 0))
    } else if is_key_pressed(KeyCode::Down) {
        Some((0, 1))
    } else if is_key_pressed(KeyCode::Left) {
        Some((-1, 0))
    } else {
        None
    }
}

pub fn selected_tile_at(ctx: &UiContext<'_>, mouse: Vec2) -> Option<TilePos> {
    let rect = world_grid_rect();
    if !rect.contains_point(mouse) {
        return None;
    }

    let grid = &ctx.session.world.fog;
    let view = GridView::new(ctx, rect);
    let pos = view.tile_at(mouse);
    let tile = view.tile_rect(pos);
    (grid.is_valid(pos)
        && tile.contains(mouse)
        && is_fully_visible(tile, rect)
        && tile.x >= rect.x
        && tile.right() <= rect.right())
    .then_some(pos)
}

fn world_panel_rect() -> Rect {
    Rect::new(18.0, 96.0, 812.0, 520.0)
}

pub fn world_grid_rect() -> Rect {
    let rect = world_panel_rect();
    Rect::new(rect.x + 24.0, rect.y + 66.0, rect.w - 48.0, rect.h - 180.0)
}

fn draw_camera_controls(pointer: Pointer, actions: &mut Vec<UiAction>) {
    let labels = ["Left", "Right", "Up", "Down", "Zoom -", "Zoom +", "Reset"];
    let commands = [
        UiAction::PanCamera(-1, 0),
        UiAction::PanCamera(1, 0),
        UiAction::PanCamera(0, -1),
        UiAction::PanCamera(0, 1),
        UiAction::ZoomCamera(-1),
        UiAction::ZoomCamera(1),
        UiAction::ResetCamera,
    ];
    let layout = GridLayout::new(42.0, 552.0, 764.0, 8.0, 7, 44.0);
    for (i, (label, command)) in labels.iter().zip(commands).enumerate() {
        let (x, y, w, h) = layout.get_item_rect(i, 0.0);
        if virtual_button(
            Rect::new(x, y, w, h),
            label,
            true,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(command);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GridView {
    viewport: Rect,
    camera: CameraTransform,
    tile_size: f32,
    world_center: Vec2,
}

impl GridView {
    fn new(ctx: &UiContext<'_>, rect: Rect) -> Self {
        let grid = &ctx.session.world.fog;
        let tile_size = (rect.w / grid.width as f32)
            .min(rect.h / grid.height as f32)
            .floor();
        Self {
            viewport: rect,
            camera: ctx.camera,
            tile_size,
            world_center: vec2(grid.width as f32, grid.height as f32) * tile_size * 0.5,
        }
    }

    fn tile_rect(self, pos: TilePos) -> Rect {
        let world = vec2(pos.x as f32, pos.y as f32) * self.tile_size - self.world_center;
        let origin = self
            .camera
            .world_to_screen(self.viewport, world)
            .expect("valid map viewport");
        let size = self.tile_size * self.camera.zoom() - 2.0;
        Rect::new(origin.x, origin.y, size, size)
    }

    fn tile_at(self, point: Vec2) -> TilePos {
        let world = self
            .camera
            .screen_to_world(self.viewport, point)
            .expect("valid map viewport");
        let tile = (world + self.world_center) / self.tile_size;
        TilePos::new(tile.x.floor() as i32, tile.y.floor() as i32)
    }
}

#[cfg(test)]
mod tests;
