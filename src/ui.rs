//! Kestrum's map-first view layer; rendering returns explicit player intents.

mod army;
mod atlas;
mod components;
mod menus;
mod movement;
mod saves;
mod selection;
mod typography;
mod world;

use kestrum::{
    data::{
        economy::{Economy, TroopKind},
        world::SiteId,
        PresentationData,
    },
    navigation::{MapNavigation, MapSelection, MapView},
    state::{
        military::{ArmyId, FormationId},
        people::PersonId,
        GameState, Overlay, Preferences, Screen,
    },
};
use macroquad::prelude::*;
use macroquad_toolkit::{assets::AssetManager, ui::Pointer};

pub use army::{
    ArmyMode, ArmyView, TransferSubject, TransferView, PEOPLE_PAGE_SIZE, TRANSFER_PAGE_SIZE,
};
pub use atlas::map_controls_contain;
pub use movement::{
    draw_map_overlay as draw_move_map_overlay,
    map_controls_contain as movement_map_controls_contain,
};
pub use movement::{MoveStage, MoveView, MOVE_GROUP_PAGE_SIZE, ROUTE_PAGE_SIZE};
pub use saves::{SaveMode, SaveRow, SaveView};
pub use typography::prepare_dynamic_text;

#[derive(Debug, Clone, Copy)]
pub enum UiAction {
    ArmyOrders,
    ArmyPeople,
    ArmyPeoplePage(i32),
    BeginMove(ArmyId),
    ToggleMoveArmy(ArmyId),
    MoveGroupPage(i32),
    ChooseMoveDestination,
    ReviewMove,
    MoveRoutePage(i32),
    ConfirmMove,
    CancelMove,
    BeginTransferFormation(FormationId),
    BeginTransferPerson(PersonId),
    SelectTransferArmy(ArmyId),
    ClearTransferArmy,
    SelectTransferSlot(u8),
    SelectTransferFormation(FormationId),
    TransferPage(i32),
    ConfirmTransfer,
    SplitArmy(FormationId),
    OpenArmies(SiteId),
    ArmyPage(i32),
    SelectFormation(FormationId),
    BeginRecruit(Option<ArmyId>),
    SelectRecruit(TroopKind),
    ConfirmRecruit,
    AskDisband(FormationId),
    ConfirmDisband(FormationId),
    CancelArmyAction,
    HelpPage(i32),
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
    pub economy: &'a Economy,
    pub army: &'a ArmyView,
    pub movement: &'a MoveView,
    pub help_page: usize,
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
    } else if ctx.state.overlay == Overlay::Armies {
        army::draw(ctx)
    } else if matches!(ctx.state.overlay, Overlay::MoveGroup | Overlay::MoveReview) {
        movement::draw(ctx)
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
