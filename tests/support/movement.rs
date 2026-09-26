//! Application overlay gates around the authoritative K06 command rules.

use super::*;
use kestrum::{
    engine::RuleError,
    state::{GameState, Overlay, Screen},
};

pub(super) fn assert_application_command_gates() {
    let (data, campaign) = fixture();
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(campaign)), &data)
        .unwrap();
    for overlay in [
        Overlay::Armies,
        Overlay::Menu,
        Overlay::MoveGroup,
        Overlay::Settings,
        Overlay::Help,
        Overlay::ConfirmNew,
        Overlay::Saves,
        Overlay::SaveRecovery,
    ] {
        state.overlay = overlay;
        let before = state.campaign.clone();
        assert_eq!(
            state.command(&data, order(&[1], &[1, 5])),
            Err(RuleError::PlayObstructed)
        );
        assert_eq!(state.campaign, before);
    }
    state.overlay = Overlay::MoveReview;
    state.screen = Screen::Title;
    let before = state.campaign.clone();
    assert_eq!(
        state.command(&data, order(&[1], &[1, 5])),
        Err(RuleError::PlayObstructed)
    );
    assert_eq!(state.campaign, before);
    state.screen = Screen::Campaign;
    assert_eq!(
        state.command(&data, Command::EndTurn),
        Err(RuleError::PlayObstructed)
    );
    state.command(&data, order(&[1], &[1, 5])).unwrap();
    assert_eq!(
        state.campaign.as_ref().unwrap().strategic().unwrap().armies[&ArmyId(1)].site,
        SiteId(5)
    );

    state.overlay = Overlay::Armies;
    assert_eq!(
        state.command(&data, Command::EndTurn),
        Err(RuleError::PlayObstructed)
    );
    state
        .command(
            &data,
            Command::SplitArmy {
                formation: FormationId(2),
            },
        )
        .unwrap();
    state.overlay = Overlay::None;
    state.command(&data, Command::EndTurn).unwrap();
    state.overlay = Overlay::Armies;
    let transfer = Command::TransferFormation {
        formation: FormationId(2),
        to_army: ArmyId(1),
        to_slot: 1,
    };
    let before = state.campaign.clone();
    assert_eq!(
        state.command(&data, transfer.clone()),
        Err(RuleError::TransferPauseRequired)
    );
    assert_eq!(state.campaign, before);
    state.overlay = Overlay::None;
    state.command(&data, Command::SetNpcPaused(true)).unwrap();
    state.overlay = Overlay::Armies;
    state.command(&data, transfer).unwrap();
    assert_eq!(state.overlay, Overlay::Armies);
    assert_eq!(state.advance_npc(&data), Err(RuleError::PlayObstructed));
    assert_eq!(
        state.command(&data, Command::EndTurn),
        Err(RuleError::PlayObstructed)
    );
    assert_eq!(
        state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .armies
            .len(),
        4
    );
}
