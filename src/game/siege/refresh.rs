//! Current selection is checked without resolving combat or exposing enemy rosters.

use super::*;

impl Game {
    pub(in crate::game) fn refresh_siege(&mut self) {
        let Some(site) = self.siege.site else { return };
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            self.siege = ui::SiegePanel::default();
            return;
        };
        self.siege.view = engine::siege_view(campaign, &self.data, campaign.player, site);
        self.siege.blocked = None;
        self.siege.exits.clear();
        let Some(view) = &self.siege.view else { return };
        self.siege
            .selected
            .retain(|id| view.own_armies.contains(id));
        self.siege.remaining = view
            .own_armies
            .iter()
            .filter_map(|id| {
                engine::army_remaining(campaign, &self.data, *id)
                    .ok()
                    .map(|left| (*id, left))
            })
            .collect();
        if let Some(action) = self.siege.action {
            if let Some(option) = view.actions.iter().find(|option| option.action == action) {
                for destination in &option.destinations {
                    let order = SiegeOrder {
                        site,
                        action,
                        armies: self.siege.selected.clone(),
                        destination: Some(*destination),
                    };
                    let blocked = engine::preview(
                        campaign,
                        &self.data,
                        engine::Actor::Player,
                        Command::Siege(order),
                    )
                    .err()
                    .map(|error| error.to_string());
                    let name = campaign
                        .world
                        .site(*destination)
                        .map(|site| site.name.clone())
                        .unwrap_or_default();
                    self.siege.exits.push(ui::SiegeExit {
                        site: *destination,
                        name,
                        blocked,
                    });
                }
            }
            if let Some(order) = self.siege_order() {
                self.siege.blocked = engine::preview(
                    campaign,
                    &self.data,
                    engine::Actor::Player,
                    Command::Siege(order),
                )
                .err()
                .map(|error| error.to_string());
            }
        }
        let count = if self.siege.mode == SiegeMode::Exits {
            self.siege.exits.len()
        } else {
            self.siege
                .view
                .as_ref()
                .map_or(0, |view| view.own_armies.len())
        };
        self.siege.page = self
            .siege
            .page
            .min(count.div_ceil(ui::SIEGE_PAGE_SIZE).saturating_sub(1));
    }
}
