//! Review and confirmation retain selections and recheck current authoritative eligibility.
use super::*;

impl Game {
    pub(super) fn refresh_household(&mut self) {
        self.army.household_option = None;
        let (Some(action), Some(site), Some(campaign)) = (
            self.army.household_review,
            self.army.site,
            self.state.campaign.as_ref().and_then(Campaign::strategic),
        ) else {
            return;
        };
        self.army.household_option = Some(engine::household_option(
            campaign,
            &self.data,
            campaign.player,
            action,
            engine::HouseholdSelection {
                site,
                first: self.army.household_first,
                second: self.army.household_second,
                category: self.army.legacy_category,
                link: self.army.legacy_link,
            },
        ));
    }

    pub(in crate::game) fn confirm_household(&mut self) {
        self.refresh_household();
        let Some(option) = &self.army.household_option else {
            return;
        };
        if option.blocked.is_some() {
            return;
        }
        let Some(command) = option.command.clone() else {
            return;
        };
        let result = self.state.command(&self.data, command);
        if result.is_ok() {
            self.army.household_review = None;
            self.army.household_option = None;
            self.army.status.clear();
        }
        self.handle_campaign_result(result);
        self.army_refresh_pending = true;
    }
}
