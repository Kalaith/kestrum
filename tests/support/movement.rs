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
        Overlay::Battle,
        Overlay::History,
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

/// K10 makes empty forts claimable; hidden defenders still cannot change preview strength.
pub(super) fn assert_fort_entry_and_hidden_contact(initial: &StrategicCampaign, data: &GameData) {
    let mut neutral_fort = initial.clone();
    neutral_fort.world.sites[4].military = kestrum::data::world::MilitaryLayer::Fort;
    let result = apply(&mut neutral_fort, data, Actor::Player, order(&[1], &[1, 5])).unwrap();
    assert!(result.battle.is_none());
    assert!(neutral_fort.sieges.is_empty());
    assert_eq!(
        neutral_fort.world.site(SiteId(5)).unwrap().controller,
        Some(initial.player)
    );
    assert_eq!(neutral_fort.armies[&ArmyId(1)].site, SiteId(5));
    assert_eq!(result.movement.unwrap().spent, 2);
    assert_eq!(neutral_fort.rng, initial.rng);
    for (id, formation) in &neutral_fort.formations {
        assert_eq!(formation.headcount, initial.formations[id].headcount);
    }

    let mut hidden = initial.clone();
    hidden.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(5);
    let mut expected =
        movement_preview(initial, data, initial.player, &[ArmyId(1)], SiteId(6)).unwrap();
    // K08 reveals nearby presence; route cost and hidden strength remain unchanged.
    expected.observed_hostile_sites.insert(SiteId(5));
    assert_eq!(
        movement_preview(&hidden, data, hidden.player, &[ArmyId(1)], SiteId(6)).unwrap(),
        expected
    );
    assert_eq!(
        preview(&hidden, data, Actor::Player, order(&[1], &[1, 5, 6])),
        preview(initial, data, Actor::Player, order(&[1], &[1, 5, 6]))
    );
    let contact = apply(&mut hidden, data, Actor::Player, order(&[1], &[1, 5, 6])).unwrap();
    assert!(contact.battle.is_some());
    assert_eq!(contact.movement.unwrap().path, [SiteId(1), SiteId(5)]);
    assert_eq!(hidden.battles.len(), 1);
}
