//! Retained event labels never resolve foreign IDs through live state.

use super::*;
use kestrum::state::history::{HistoryKind, HistoryRecord};

pub(super) fn events(ctx: &Context<'_>) -> Vec<HistoryRow> {
    ctx.history
        .result
        .as_ref()
        .map(|page| {
            page.entries
                .iter()
                .map(|event| {
                    let heading = format!(
                        "{} · {}",
                        date(ctx, event.completed_rounds),
                        kind(ctx, &event.kind)
                    );
                    let row = HistoryRow::new(heading, summary(ctx, event));
                    if let HistoryKind::Battle { battle, .. } = event.kind {
                        if ctx.campaign_view.is_some_and(|view| {
                            view.battles.iter().any(|report| report.id == battle)
                        }) {
                            return row
                                .link(UiAction::OpenRecordedBattle(battle), "history_report");
                        }
                        return HistoryRow::new(
                            row.heading,
                            format!("{} · {}", row.detail, ctx.text("history_report_expired")),
                        );
                    }
                    row
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn kind(ctx: &Context<'_>, kind: &HistoryKind) -> String {
    match kind {
        HistoryKind::Siege {
            change,
            elapsed_steps,
            ..
        } => format!(
            "{} · {} · {}: {elapsed_steps}",
            ctx.text("siege"),
            ctx.text(match change {
                kestrum::state::siege::SiegeChange::Established => "siege_established",
                kestrum::state::siege::SiegeChange::Reinforced => "siege_reinforced",
                kestrum::state::siege::SiegeChange::Progressed => "siege_progressed",
                kestrum::state::siege::SiegeChange::Lifted => "siege_lifted",
            }),
            ctx.text("siege_elapsed")
        ),
        HistoryKind::Construction { order } => format!(
            "{} · {} / {} · {}",
            super::super::settlement::kind_name(ctx, order.kind),
            order.progress,
            order.required_steps,
            super::super::settlement::order_status(ctx, order)
        ),
        HistoryKind::FocusChanged { focus } => format!(
            "{} · {}",
            ctx.text("settlement_focus"),
            ctx.text(super::super::settlement::focus_key(*focus))
        ),
        HistoryKind::Battle { .. } => ctx.text("history_battle"),
        HistoryKind::Recruited { troop } => format!(
            "{} · {}",
            ctx.text("history_recruited"),
            ctx.text(super::super::army::troop_key(*troop))
        ),
        HistoryKind::Disbanded { troop } => format!(
            "{} · {}",
            ctx.text("history_disbanded"),
            ctx.text(super::super::army::troop_key(*troop))
        ),
        HistoryKind::Moved => ctx.text("history_moved"),
        HistoryKind::FormationTransferred => ctx.text("history_formation_transferred"),
        HistoryKind::PersonTransferred => ctx.text("history_person_transferred"),
        HistoryKind::VeterancyEarned { tier, xp, .. } => format!(
            "{} · {} · {xp} {}",
            ctx.text("history_veterancy"),
            tier_text(ctx, *tier),
            ctx.text("service_xp")
        ),
    }
}

fn summary(ctx: &Context<'_>, event: &HistoryRecord) -> String {
    let mut labels = Vec::new();
    if let Some(site) = event.sites.first() {
        labels.push(site.name.clone());
    }
    if let Some(person) = event.people.first() {
        labels.push(person.name.clone());
    } else if let Some(army) = event.armies.first() {
        labels.push(army.name.clone());
    }
    if event.people.len() > 1 {
        labels.push(format!(
            "{} {}",
            event.people.len(),
            ctx.text("history_people_recorded")
        ));
    }
    if event.armies.len() > 1 {
        labels.push(format!(
            "{} {}",
            event.armies.len(),
            ctx.text("history_armies_recorded")
        ));
    }
    labels.join(" · ")
}

pub(super) fn tier_text(ctx: &Context<'_>, tier: kestrum::state::evidence::Veterancy) -> String {
    use kestrum::state::evidence::Veterancy;
    ctx.text(match tier {
        Veterancy::Ordinary => "tier_ordinary",
        Veterancy::Seasoned => "tier_seasoned",
        Veterancy::Veteran => "tier_veteran",
    })
}
