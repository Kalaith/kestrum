//! Text-entry behavior for the production campaign setup screen.

use super::Game;
use kestrum::state::Overlay;
use macroquad_toolkit::ui::text_entry::{apply_text_edit, TextEntryAction};

impl Game {
    pub(super) fn edit_setup_name(&mut self, action: TextEntryAction) {
        if self.state.overlay != Overlay::Setup || !self.setup.editing_name {
            return;
        }
        match action {
            TextEntryAction::SetPage(page) => self.setup.keyboard_page = page,
            TextEntryAction::Edit(edit) => {
                self.setup.name_error = apply_text_edit(
                    &mut self.setup.kingdom_name,
                    edit,
                    self.data.rules.kingdom_name_max_chars,
                )
                .err()
                .map(|error| error.to_string());
            }
        }
    }
}
