//! Kestrum's map-first view layer; rendering returns explicit player intents.

mod army;
mod atlas;
mod battle;
mod campaign_end;
mod components;
mod history;
mod kingdom;
mod menus;
mod movement;
mod saves;
mod selection;
mod settlement;
mod siege;
mod threat;
mod typography;
mod world;

use kestrum::{
    data::{
        economy::{Economy, TroopKind},
        progression::FormationSpecialization,
        world::{PersonClass, SiteId},
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
pub use battle::{BattleTab, BattleView};
pub use history::{HistoryMode, HistoryView, RecordCategory, HISTORY_ROWS_PER_SCREEN};
pub use kingdom::{KingdomIntent, KingdomView, KINGDOM_PAGE_SIZE};
pub use movement::{
    draw_map_overlay as draw_move_map_overlay,
    map_controls_contain as movement_map_controls_contain,
};
pub use movement::{MoveStage, MoveView, MOVE_GROUP_PAGE_SIZE, ROUTE_PAGE_SIZE};
pub use saves::{SaveMode, SaveRow, SaveView};
pub use settlement::{
    BuildChoice, BuilderChoice, FocusChoice, LocalAction, LocalDestination, SettlementMode,
    SettlementView, SETTLEMENT_PAGE_SIZE,
};
pub use siege::{SiegeExit, SiegeMode, SiegePanel, SIEGE_PAGE_SIZE};
pub use threat::{ThreatPanel, ThreatStage, THREAT_PAGE_SIZE};
pub use typography::prepare_dynamic_text;

#[derive(Debug, Clone, Copy)]
pub enum UiAction {
    OpenKingdom(Option<kestrum::data::world::FactionId>),
    SelectKingdom(kestrum::data::world::FactionId),
    KingdomPage(i32),
    ReviewDiplomacy(KingdomIntent),
    ConfirmDiplomacy,
    KingdomBack,
    OpenThreat(kestrum::state::threat::ThreatId),
    ArmyThreats(ArmyId),
    ToggleThreatArmy(ArmyId),
    ThreatPage(i32),
    ReviewThreat,
    ConfirmThreat,
    ThreatBack,
    OpenSiege(SiteId),
    SiegeMode(SiegeMode),
    ToggleSiegeArmy(ArmyId),
    SiegePage(i32),
    ChooseSiegeAction(kestrum::state::siege::SiegeAction),
    SiegeDestination(SiteId),
    ConfirmSiege,
    SiegeBack,
    OpenSettlement(SiteId),
    SelectLocalAction(LocalAction),
    SelectResettleDestination(SiteId),
    EditPlaceName(macroquad_toolkit::ui::text_entry::TextEntryAction),
    ConfirmLocalAction,
    SettlementTab(SettlementMode),
    SettlementPage(i32),
    SelectConstruction(
        kestrum::state::construction::ConstructionTarget,
        kestrum::state::construction::ConstructionKind,
    ),
    ChooseBuilder,
    SelectBuilder(ArmyId),
    ConfirmConstruction,
    OpenConstructionOrder(kestrum::state::construction::OrderId),
    AskCancelConstruction,
    ConfirmCancelConstruction,
    SelectFocus(kestrum::state::construction::Focus),
    ConfirmFocus,
    SettlementBack,
    OpenRecords,
    SetRecordCategory(RecordCategory),
    RecordsPage(i32),
    OpenHistory(kestrum::state::history::HistorySubject),
    SetHistoryMode(HistoryMode),
    HistoryPage(i32),
    HistoryOverviewPage(i32),
    SetHistoryKind(Option<kestrum::state::history::HistoryKindFilter>),
    ShiftHistoryFrom(i32),
    ShiftHistoryTo(i32),
    ResetHistoryFilters,
    ApplyHistoryFilters,
    HistoryBack,
    OpenRecordedBattle(kestrum::state::battle::BattleId),
    EditHistorySearch(macroquad_toolkit::ui::text_entry::TextEntryAction),
    ApplyHistorySearch,
    OpenBattleReports,
    BattleReport(i32),
    BattlePage(i32),
    SetBattleTab(BattleTab),
    ArmyOrders,
    ArmyPeople,
    ArmyPeoplePage(i32),
    OpenPersonProgression(PersonId),
    OpenFormationProgression(FormationId),
    TrainPerson(PersonId, PersonClass, SiteId),
    PracticeRiding(PersonId, SiteId),
    CancelPersonCourse(PersonId),
    SpecializeFormation(FormationId, FormationSpecialization, SiteId),
    CancelFormationCourse(FormationId),
    SetCommander(ArmyId, Option<PersonId>),
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

impl UiAction {
    pub fn is_kingdom(self) -> bool {
        matches!(
            self,
            Self::OpenKingdom(_)
                | Self::SelectKingdom(_)
                | Self::KingdomPage(_)
                | Self::ReviewDiplomacy(_)
                | Self::ConfirmDiplomacy
                | Self::KingdomBack
        )
    }
    pub fn is_threat(self) -> bool {
        matches!(
            self,
            Self::OpenThreat(_)
                | Self::ArmyThreats(_)
                | Self::ToggleThreatArmy(_)
                | Self::ThreatPage(_)
                | Self::ReviewThreat
                | Self::ConfirmThreat
                | Self::ThreatBack
        )
    }
    pub fn is_siege(self) -> bool {
        matches!(
            self,
            Self::OpenSiege(_)
                | Self::SiegeMode(_)
                | Self::ToggleSiegeArmy(_)
                | Self::SiegePage(_)
                | Self::ChooseSiegeAction(_)
                | Self::SiegeDestination(_)
                | Self::ConfirmSiege
                | Self::SiegeBack
        )
    }

    pub fn is_settlement(self) -> bool {
        matches!(
            self,
            Self::OpenSettlement(_)
                | Self::SelectLocalAction(_)
                | Self::SelectResettleDestination(_)
                | Self::EditPlaceName(_)
                | Self::ConfirmLocalAction
                | Self::SettlementTab(_)
                | Self::SettlementPage(_)
                | Self::SelectConstruction(_, _)
                | Self::ChooseBuilder
                | Self::SelectBuilder(_)
                | Self::ConfirmConstruction
                | Self::OpenConstructionOrder(_)
                | Self::AskCancelConstruction
                | Self::ConfirmCancelConstruction
                | Self::SelectFocus(_)
                | Self::ConfirmFocus
                | Self::SettlementBack
        )
    }

    pub fn is_history(self) -> bool {
        matches!(
            self,
            Self::OpenRecords
                | Self::SetRecordCategory(_)
                | Self::RecordsPage(_)
                | Self::OpenHistory(_)
                | Self::SetHistoryMode(_)
                | Self::HistoryPage(_)
                | Self::HistoryOverviewPage(_)
                | Self::SetHistoryKind(_)
                | Self::ShiftHistoryFrom(_)
                | Self::ShiftHistoryTo(_)
                | Self::ResetHistoryFilters
                | Self::ApplyHistoryFilters
                | Self::HistoryBack
                | Self::OpenRecordedBattle(_)
                | Self::EditHistorySearch(_)
                | Self::ApplyHistorySearch
        )
    }
}

pub struct Context<'a> {
    pub kingdom: &'a KingdomView,
    pub threat: &'a ThreatPanel,
    pub siege: &'a SiegePanel,
    pub settlement: &'a SettlementView,
    pub data: &'a PresentationData,
    pub economy: &'a Economy,
    pub progression: &'a kestrum::data::progression::ProgressionRules,
    pub history: &'a HistoryView,
    pub army: &'a ArmyView,
    pub movement: &'a MoveView,
    pub battle: &'a BattleView,
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
    if ctx.state.overlay == Overlay::Kingdom {
        return kingdom::draw(ctx);
    }
    if ctx.state.overlay == Overlay::CampaignEnd {
        return campaign_end::draw(ctx);
    }
    if ctx.state.overlay == Overlay::Threat {
        return threat::draw(ctx);
    }
    if ctx.state.overlay == Overlay::Siege {
        siege::draw(ctx)
    } else if ctx.state.overlay == Overlay::Settlement {
        settlement::draw(ctx)
    } else if ctx.state.overlay == Overlay::Saves {
        saves::draw(ctx)
    } else if ctx.state.overlay == Overlay::History {
        history::draw(ctx)
    } else if ctx.state.overlay == Overlay::Battle {
        battle::draw(ctx)
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
