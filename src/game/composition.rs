//! Reassignment intents use the same transactional previews as applied orders.

use super::*;
use kestrum::state::military::FormationId;

impl Game {
    pub(super) fn begin_transfer(&mut self, subject: ui::TransferSubject) {
        self.army.transfer = ui::TransferView {
            subject: Some(subject),
            ..Default::default()
        };
        self.army.mode = ui::ArmyMode::Transfer;
        self.army.status.clear();
        self.refresh_transfer();
    }

    fn transfer_command(&self) -> Option<Command> {
        match self.army.transfer.subject? {
            ui::TransferSubject::Formation(formation) => Some(Command::TransferFormation {
                formation,
                to_army: self.army.transfer.army?,
                to_slot: usize::from(self.army.transfer.slot?),
            }),
            ui::TransferSubject::Person(person) => Some(Command::TransferPerson {
                person,
                to_formation: self.army.transfer.formation?,
            }),
        }
    }

    pub(super) fn refresh_transfer(&mut self) {
        if self.army.mode != ui::ArmyMode::Transfer {
            return;
        }
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        self.army.transfer.blocked = self
            .transfer_command()
            .map(|command| {
                engine::preview(campaign, &self.data, engine::Actor::Player, command)
                    .err()
                    .map(|error| error.to_string())
            })
            .unwrap_or_else(|| Some("Choose a destination army and formation slot.".into()));
        self.army.transfer.split_blocked = match self.army.transfer.subject {
            Some(ui::TransferSubject::Formation(formation)) => engine::preview(
                campaign,
                &self.data,
                engine::Actor::Player,
                Command::SplitArmy { formation },
            )
            .err()
            .map(|error| error.to_string()),
            _ => Some("Choose a whole formation to create a new army.".into()),
        };
    }

    pub(super) fn confirm_transfer(&mut self) {
        if let Some(command) = self.transfer_command() {
            self.apply_composition(command);
        }
    }

    pub(super) fn split_army(&mut self, formation: FormationId) {
        self.apply_composition(Command::SplitArmy { formation });
    }

    fn apply_composition(&mut self, command: Command) {
        match self.state.command(&self.data, command) {
            Ok(outcome) => {
                self.army.mode = ui::ArmyMode::Roster;
                self.army.status =
                    "Assignment changed. Movement already spent is preserved.".into();
                self.army.selected = None;
                if let Some(army) = outcome.split_army {
                    self.army.page = self
                        .local_armies()
                        .iter()
                        .position(|id| *id == army)
                        .unwrap_or(0);
                }
                self.refresh_army();
            }
            Err(error) => self.army.status = error.to_string(),
        }
    }
}
