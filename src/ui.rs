//! Kestrum's map-first view layer; rendering returns explicit player intents.

mod atlas;
mod components;
mod menus;

use kestrum::{
    data::PresentationData,
    navigation::MapView,
    state::{GameState, Overlay, Preferences, Screen},
};
use macroquad::prelude::*;
use macroquad_toolkit::{assets::AssetManager, ui::Pointer};

pub use atlas::map_controls_contain;

#[derive(Debug, Clone, Copy)]
pub enum UiAction {
    NewGame,
    ConfirmNew,
    Continue,
    Open(Overlay),
    Back,
    MainMenu,
    Save,
    Load,
    EndTurn,
    Zoom(f32),
    Recenter,
    ToggleLabels,
    ToggleContrast,
    DismissFeedback,
    #[cfg(not(target_arch = "wasm32"))]
    Fullscreen,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

pub struct Context<'a> {
    pub data: &'a PresentationData,
    pub state: &'a GameState,
    pub preferences: &'a Preferences,
    pub view: &'a MapView,
    pub assets: &'a AssetManager,
    pub pointer: Pointer,
    pub origin: Option<Vec2>,
    pub save_exists: bool,
}

impl Context<'_> {
    pub fn font(&self) -> Option<&Font> {
        self.assets.get_font("cinzel")
    }
    pub fn body_font(&self) -> Option<&Font> {
        self.assets.get_font("body")
    }
    pub fn text(&self, key: &str) -> String {
        self.data.text(key).to_owned()
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    atlas::draw_landscape(ctx);
    let action = if ctx.state.screen == Screen::Title {
        menus::title(ctx)
    } else {
        atlas::hud(ctx)
    };
    if ctx.state.overlay != Overlay::None {
        menus::overlay(ctx)
    } else {
        action
    }
}

pub fn feedback(ctx: &Context<'_>, message: &str) -> Option<UiAction> {
    use components::*;
    draw_rectangle(
        328.0,
        628.0,
        642.0,
        82.0,
        Color::new(0.07, 0.12, 0.12, 0.98),
    );
    paragraph(ctx, message, vec2(346.0, 653.0), 500.0);
    if button(
        ctx,
        Rect::new(852.0, 646.0, 108.0, 48.0),
        &ctx.text("close"),
        true,
        false,
    ) {
        Some(UiAction::DismissFeedback)
    } else {
        None
    }
}
