//! Cheap shared rule checks reject impossible candidates before cloning state.
use super::*;

impl Planner<'_> {
    pub(super) fn candidate_allowed(&self, command: &Command) -> Result<(), RuleError> {
        super::super::actions::validate_personnel_command(
            self.campaign,
            self.data,
            self.owner,
            command,
        )?;
        match command {
            Command::Recruit { site, army, kind } => super::super::recruitment::validate_recruit(
                self.campaign,
                self.data,
                *site,
                *army,
                *kind,
            ),
            Command::StartConstruction {
                target,
                kind,
                builder,
            } => super::super::construction::validate_start(
                self.campaign,
                self.data,
                self.owner,
                *target,
                *kind,
                *builder,
            ),
            Command::SetFocus { site, focus } => super::super::construction::validate_focus(
                self.campaign,
                self.data,
                self.owner,
                *site,
                *focus,
            )
            .map(|_| ()),
            Command::DevelopCity { site } => super::super::development::city_development_check(
                self.campaign,
                self.data,
                self.owner,
                *site,
            ),
            _ => Ok(()),
        }
    }
}
