//! Kestrum's map-first view layer; rendering returns explicit player intents.

mod atlas;
mod components;
mod menus;
mod saves;
mod selection;
mod typography;
mod world;

use kestrum::{
    data::PresentationData,
    navigation::{MapNavigation, MapSelection, MapView},
    state::{GameState, Overlay, Preferences, Screen},
};
use macroquad::prelude::*;
use macroquad_toolkit::{assets::AssetManager, ui::Pointer};

pub use atlas::map_controls_contain;
pub use saves::{SaveMode, SaveRow, SaveView};
pub use typography::prepare_dynamic_text;

#[derive(Debug, Clone, Copy)]
pub enum UiAction {
    SelectMap(MapSelection),
    EnterRegion(kestrum::data::world::MarkerId),
    WorldMap,
    CloseSelection,
    NewGame,
    ConfirmNew,
    Continue,
    Open(Overlay),
    Back,
    MainMenu,
    Save,
    Load,
    EndTurn,
    PauseNpcs(bool),
    StepNpc,
    LoadLegacy,
    ImportCampaign,
    SelectSave(u64),
    SavePage(i32),
    NameSave(Option<u64>),
    LoadSelectedSave,
    OverwriteSave,
    AskDeleteSave,
    ConfirmDeleteSave(u64),
    CommitNamedSave,
    CancelSaveEdit,
    RetryStorage,
    RetrySave,
    ContinueUnsaved,
    EditSaveName(macroquad_toolkit::ui::text_entry::TextEntryAction),
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
    pub navigation: &'a MapNavigation,
    pub assets: &'a AssetManager,
    pub pointer: Pointer,
    pub origin: Option<Vec2>,
    pub save_exists: bool,
    pub legacy_save_exists: bool,
    pub import_save_exists: bool,
    pub saves: &'a SaveView,
    pub save_error: &'a str,
    pub campaign_view: Option<&'a kestrum::engine::VisibleCampaign>,
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
    if ctx.state.overlay == Overlay::Saves {
        saves::draw(ctx)
    } else if ctx.state.overlay == Overlay::SaveRecovery {
        saves::recovery(ctx, ctx.save_error)
    } else if ctx.state.overlay != Overlay::None {
        menus::overlay(ctx)
    } else {
        action
    }
}

pub const FEEDBACK: Rect = Rect::new(328.0, 628.0, 642.0, 82.0);

pub fn feedback(ctx: &Context<'_>, message: &str) -> Option<UiAction> {
    use components::*;
    draw_rectangle(
        FEEDBACK.x,
        FEEDBACK.y,
        FEEDBACK.w,
        FEEDBACK.h,
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
