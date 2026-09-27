//! On-demand army composition, recruitment choices, and explicit disbanding.

mod disband;
mod orders;
mod people;
mod recruit;
mod roster;
mod transfer;

use super::{components::*, Context, UiAction};
use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        world::SiteId,
    },
    engine::{RecoveryPreview, RecruitOption, VisibleCampaign},
    state::{
        military::{Army, ArmyId, Formation, FormationId},
        people::PersonId,
    },
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};
use std::collections::BTreeMap;

pub const TRANSFER_PAGE_SIZE: usize = 5;
pub const PEOPLE_PAGE_SIZE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferSubject {
    Formation(FormationId),
    Person(PersonId),
}

#[derive(Debug, Default)]
pub struct TransferView {
    pub subject: Option<TransferSubject>,
    pub army: Option<ArmyId>,
    pub slot: Option<u8>,
    pub formation: Option<FormationId>,
    pub page: usize,
    pub blocked: Option<String>,
    pub split_blocked: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ArmyMode {
    #[default]
    Roster,
    Orders,
    People,
    Transfer,
    Recruit {
        army: Option<ArmyId>,
        kind: Option<TroopKind>,
    },
    Disband(FormationId),
}

#[derive(Debug)]
pub struct ArmyView {
    pub site: Option<SiteId>,
    pub page: usize,
    pub selected: Option<FormationId>,
    pub mode: ArmyMode,
    pub options: Vec<RecruitOption>,
    pub leadership_permille: u32,
    pub status: String,
    pub transfer: TransferView,
    pub people_page: usize,
    pub remaining: u32,
    pub supplied: bool,
    pub member_remaining: BTreeMap<FormationId, u32>,
    pub person_remaining: BTreeMap<PersonId, u32>,
    pub recovery: Option<RecoveryPreview>,
}

impl Default for ArmyView {
    fn default() -> Self {
        Self {
            site: None,
            page: 0,
            selected: None,
            mode: ArmyMode::Roster,
            options: Vec::new(),
            leadership_permille: 500,
            status: String::new(),
            transfer: TransferView::default(),
            people_page: 0,
            remaining: 0,
            supplied: false,
            member_remaining: BTreeMap::new(),
            person_remaining: BTreeMap::new(),
            recovery: None,
        }
    }
}

impl ArmyView {
    pub fn local_people<'a>(
        &self,
        campaign: &'a VisibleCampaign,
    ) -> Vec<&'a kestrum::state::people::Person> {
        people::local_people(self, campaign)
    }

    pub fn armies_at_site<'a>(&self, campaign: &'a VisibleCampaign) -> Vec<&'a Army> {
        campaign
            .armies
            .iter()
            .filter(|army| Some(army.site) == self.site)
            .collect()
    }

    pub fn selected_army<'a>(&self, campaign: &'a VisibleCampaign) -> Option<&'a Army> {
        self.armies_at_site(campaign).get(self.page).copied()
    }

    pub fn page_count(&self, campaign: &VisibleCampaign) -> usize {
        self.armies_at_site(campaign).len().max(1)
    }
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let site = ctx.army.site.and_then(|id| campaign.world.site(id));
    draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.02, 0.05, 0.05, 0.78));
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    let key = match ctx.army.mode {
        ArmyMode::Roster => "armies",
        ArmyMode::Orders => "army_orders",
        ArmyMode::People => "army_people",
        ArmyMode::Transfer => "composition",
        ArmyMode::Recruit { .. } => "recruit",
        ArmyMode::Disband(_) => "disband",
    };
    let heading = format!(
        "{} / {}",
        ctx.text(key),
        site.map(|site| site.name.as_str()).unwrap_or_default()
    );
    let heading = truncate_text_to_width_ex(&heading, 1040.0, ctx.font(), 28.0);
    text(ctx, &heading, vec2(112.0, 83.0), 28.0, CREAM);
    if let Some(resources) = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer)
        .and_then(|faction| faction.resources)
    {
        body(
            ctx,
            &resources_text(ctx, resources),
            vec2(112.0, 123.0),
            18.0,
            BRASS,
        );
    }
    economy_summary(ctx, campaign);
    draw_line(112.0, 143.0, 1168.0, 143.0, 1.0, BRASS);
    let action = match ctx.army.mode {
        ArmyMode::Roster => roster::draw(ctx, campaign),
        ArmyMode::Orders => orders::draw(ctx, campaign),
        ArmyMode::People => people::draw(ctx, campaign),
        ArmyMode::Transfer => transfer::draw(ctx, campaign),
        ArmyMode::Recruit { army, kind } => recruit::draw(ctx, campaign, army, kind),
        ArmyMode::Disband(formation) => disband::draw(ctx, campaign, formation),
    };
    if !ctx.army.status.is_empty() {
        for (index, line) in wrap_text_ex(&ctx.army.status, 1040.0, ctx.body_font(), 18.0)
            .into_iter()
            .take(2)
            .enumerate()
        {
            body(
                ctx,
                &line,
                vec2(112.0, 594.0 + index as f32 * 22.0),
                18.0,
                CREAM,
            );
        }
    }
    action
}

fn economy_summary(ctx: &Context<'_>, campaign: &VisibleCampaign) {
    let Some(statement) = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer)
        .and_then(|faction| faction.last_economy.as_ref())
    else {
        return;
    };
    let income = format!(
        "{}: {}",
        ctx.text("last_income"),
        resources_text(ctx, statement.income)
    );
    let upkeep = format!(
        "{}: {} / {} {}   ·   {}: {}",
        ctx.text("upkeep_paid"),
        statement.upkeep_paid,
        statement.upkeep_due,
        ctx.text("gold"),
        ctx.text("upkeep_shortfall"),
        statement.shortfall
    );
    for (line, label) in [income, upkeep].iter().enumerate() {
        let label = truncate_text_to_width_ex(label, 464.0, ctx.body_font(), 16.0);
        body(
            ctx,
            &label,
            vec2(704.0, 106.0 + line as f32 * 22.0),
            16.0,
            if statement.shortfall > 0 {
                BRASS
            } else {
                MUTED
            },
        );
    }
}

pub fn troop_key(kind: TroopKind) -> &'static str {
    match kind {
        TroopKind::Warriors => "troop_warriors",
        TroopKind::Spearmen => "troop_spearmen",
        TroopKind::Archers => "troop_archers",
        TroopKind::Riders => "troop_riders",
        TroopKind::Medics => "troop_medics",
        TroopKind::SiegeEngines => "troop_siege_engines",
    }
}

fn resources_text(ctx: &Context<'_>, resources: Resources) -> String {
    format!(
        "{} {}   ·   {} {}   ·   {} {}",
        resources.gold,
        ctx.text("gold"),
        resources.wood,
        ctx.text("wood"),
        resources.stone,
        ctx.text("stone")
    )
}

fn formation(campaign: &VisibleCampaign, id: FormationId) -> Option<&Formation> {
    campaign
        .formations
        .iter()
        .find(|formation| formation.id == id)
}

fn block(ctx: &Context<'_>, label: &str, at: Vec2, width: f32, color: Color) -> f32 {
    let mut y = at.y;
    for line in wrap_text_ex(label, width, ctx.body_font(), 18.0) {
        body(ctx, &line, vec2(at.x, y), 18.0, color);
        y += 23.0;
    }
    y
}

fn tapped(ctx: &Context<'_>, rect: Rect) -> bool {
    ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin))
}
