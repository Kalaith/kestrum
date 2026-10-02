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
                    if let HistoryKind::ItemCustodyChanged { item, .. } = event.kind {
                        if ctx.campaign_view.is_some_and(|view| {
                            view.legacy_items.iter().any(|entry| entry.id == item)
                        }) {
                            return row.link(
                                UiAction::OpenHistory(HistorySubject::Item(item)),
                                "history_open",
                            );
                        }
                    }
                    if let Some(related) = event.related_events.first() {
                        return row.link(
                            UiAction::OpenRelatedHistoryEvent(*related),
                            "history_related_war",
                        );
                    }
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
        HistoryKind::Life { event, .. } => ctx.game_text.life_event_text(event),
        HistoryKind::Diplomacy { receipt } => diplomacy_label(ctx, receipt),
        HistoryKind::Development { receipt } => development_label(ctx, receipt),
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
        HistoryKind::ItemCustodyChanged { .. } => ctx.text("history_item_custody_changed"),
        HistoryKind::Anniversary { years, .. } => ctx
            .text("history_anniversary")
            .replace("{years}", &years.to_string()),
    }
}

fn summary(ctx: &Context<'_>, event: &HistoryRecord) -> String {
    match &event.kind {
        HistoryKind::Life { .. } => {
            return event
                .people
                .iter()
                .map(|entry| entry.name.as_str())
                .chain(event.sites.iter().map(|entry| entry.name.as_str()))
                .collect::<Vec<_>>()
                .join(" / ")
        }
        HistoryKind::ItemCustodyChanged { from, to, .. } => {
            let item = event
                .items
                .first()
                .map_or_else(String::new, |entry| entry.name.clone());
            return format!(
                "{item} · {} → {}",
                custody(ctx, event, *from),
                custody(ctx, event, *to)
            );
        }
        HistoryKind::Anniversary { subject, years } => {
            let name = match subject {
                kestrum::state::history::AnniversarySubject::Person(id) => event
                    .people
                    .iter()
                    .find(|person| person.id == *id)
                    .map(|person| person.name.as_str()),
                kestrum::state::history::AnniversarySubject::Site(id) => event
                    .sites
                    .iter()
                    .find(|site| site.id == *id)
                    .map(|site| site.name.as_str()),
            }
            .unwrap_or("?");
            let topic = match subject {
                kestrum::state::history::AnniversarySubject::Person(_) => {
                    ctx.text("history_service_anniversary")
                }
                kestrum::state::history::AnniversarySubject::Site(_) => {
                    ctx.text("history_foundation_anniversary")
                }
            };
            return format!(
                "{name} · {years} {} · {topic}",
                ctx.text("history_anniversary_years")
            );
        }
        _ => {}
    }
    if let HistoryKind::Diplomacy { receipt } = &event.kind {
        return diplomacy_parties(ctx, receipt);
    }
    if matches!(event.kind, HistoryKind::Development { .. }) {
        return event
            .sites
            .iter()
            .map(|site| site.name.as_str())
            .collect::<Vec<_>>()
            .join(" → ");
    }
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

fn custody(
    ctx: &Context<'_>,
    event: &HistoryRecord,
    custody: kestrum::state::legacy::LegacyItemCustody,
) -> String {
    match custody {
        kestrum::state::legacy::LegacyItemCustody::Person(id) => event
            .people
            .iter()
            .find(|person| person.id == id)
            .map(|person| person.name.clone())
            .unwrap_or_else(|| ctx.text("history_unknown_custodian")),
        kestrum::state::legacy::LegacyItemCustody::SiteEstate(id) => event
            .sites
            .iter()
            .find(|site| site.id == id)
            .map(|site| format!("{} · {}", ctx.text("legacy_estate"), site.name))
            .unwrap_or_else(|| ctx.text("legacy_estate")),
    }
}

fn diplomacy_label(
    ctx: &Context<'_>,
    receipt: &kestrum::state::diplomacy::DiplomacyReceipt,
) -> String {
    use kestrum::state::diplomacy::{DefeatResolution, DiplomacyReceipt};
    ctx.text(match receipt {
        DiplomacyReceipt::WarDeclared { .. } => "history_war_declared",
        DiplomacyReceipt::PeaceOffered { .. } => "history_peace_offered",
        DiplomacyReceipt::PeaceRejected { .. } => "history_peace_declined",
        DiplomacyReceipt::PeaceAgreed { .. } => "peace_agreed",
        DiplomacyReceipt::ArmyWithdrawn { .. } => "history_withdrawal",
        DiplomacyReceipt::DefeatPending { .. } => "kingdom_defeated_choice",
        DiplomacyReceipt::FactionResolved {
            resolution: DefeatResolution::Annex,
            ..
        } => "history_annexed",
        DiplomacyReceipt::FactionResolved {
            resolution: DefeatResolution::Submission,
            ..
        } => "history_submission",
        DiplomacyReceipt::CampaignEnded { .. } => "kingdom_milestone",
    })
}

fn diplomacy_parties(
    ctx: &Context<'_>,
    receipt: &kestrum::state::diplomacy::DiplomacyReceipt,
) -> String {
    use kestrum::state::diplomacy::DiplomacyReceipt;
    let ids: Vec<_> = match receipt {
        DiplomacyReceipt::WarDeclared { factions }
        | DiplomacyReceipt::PeaceAgreed { factions, .. } => factions.to_vec(),
        DiplomacyReceipt::PeaceOffered {
            proposer,
            recipient,
        }
        | DiplomacyReceipt::PeaceRejected {
            proposer,
            recipient,
        } => vec![*proposer, *recipient],
        DiplomacyReceipt::ArmyWithdrawn { faction, .. } => vec![*faction],
        DiplomacyReceipt::DefeatPending { faction, victor } => vec![*faction, *victor],
        DiplomacyReceipt::FactionResolved {
            faction, victor, ..
        } => std::iter::once(*faction).chain(*victor).collect(),
        DiplomacyReceipt::CampaignEnded { .. } => Vec::new(),
    };
    ids.into_iter()
        .filter_map(|id| {
            ctx.campaign_view?
                .factions
                .iter()
                .find(|f| f.id == id)
                .map(|f| f.name.clone())
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

pub(super) fn tier_text(ctx: &Context<'_>, tier: kestrum::state::evidence::Veterancy) -> String {
    use kestrum::state::evidence::Veterancy;
    ctx.text(match tier {
        Veterancy::Ordinary => "tier_ordinary",
        Veterancy::Seasoned => "tier_seasoned",
        Veterancy::Veteran => "tier_veteran",
    })
}

fn development_label(
    ctx: &Context<'_>,
    receipt: &kestrum::state::development::DevelopmentReceipt,
) -> String {
    use kestrum::state::development::DevelopmentReceipt;
    match receipt {
        DevelopmentReceipt::HabitationChanged { from, to, .. } => format!(
            "{}: {} → {}",
            ctx.text("history_development"),
            super::super::settlement::habitation(ctx, *from),
            super::super::settlement::habitation(ctx, *to)
        ),
        DevelopmentReceipt::Ruined { .. } => ctx.text("place_ruined"),
        DevelopmentReceipt::PopulationMoved {
            amount, resettled, ..
        } => format!(
            "{}: {amount}",
            ctx.text(if *resettled {
                "resettle"
            } else {
                "history_migration"
            })
        ),
        DevelopmentReceipt::SiteRenamed {
            old_name, new_name, ..
        } => format!("{}: {old_name} → {new_name}", ctx.text("rename_place")),
        DevelopmentReceipt::CapitalMoved { .. } => ctx.text("move_capital"),
        DevelopmentReceipt::HeadquartersMoved { .. } => ctx.text("relocate_hq"),
    }
}
