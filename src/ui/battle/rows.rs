//! Text comes from the recorded encounter and public place names, never live enemies.

use super::*;
use kestrum::{
    data::world::{FactionId, FounderClass, SiteId},
    state::{
        battle::{BattleEndReason, BattleOutcome, BattleSideReport},
        people::{
            PersonAssignment, PersonCombatOutcome, PersonDeathReason, PersonStatus, WoundCause,
        },
    },
};
use std::collections::BTreeMap;

pub(super) fn build(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    match ctx.battle.tab {
        BattleTab::Outcome => outcome(ctx, report),
        BattleTab::Forces => forces(ctx, report),
        BattleTab::People => people(ctx, report),
        BattleTab::Factors => factors(ctx, report),
    }
}

fn row(heading: String, detail: String) -> ReportRow {
    ReportRow {
        heading,
        detail,
        person: None,
    }
}

fn sides<'a>(ctx: &Context<'_>, report: &'a BattleReport) -> [(String, &'a BattleSideReport); 2] {
    [
        (ctx.text("battle_attacker"), &report.attacker),
        (ctx.text("battle_defender"), &report.defender),
    ]
}

fn outcome(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let outcome = match report.outcome {
        BattleOutcome::AttackerVictory => {
            format!("{}: {}", ctx.text("battle_victory"), report.attacker.name)
        }
        BattleOutcome::DefenderVictory => {
            format!("{}: {}", ctx.text("battle_victory"), report.defender.name)
        }
        BattleOutcome::Stalemate => ctx.text("battle_stalemate"),
        BattleOutcome::MutualDestruction => ctx.text("battle_mutual_destruction"),
    };
    let reason = ctx.text(match report.reason {
        BattleEndReason::Annihilation => "battle_annihilation",
        BattleEndReason::Rout => "battle_rout",
        BattleEndReason::ExchangeLimit => "battle_exchange_limit",
    });
    let mut rows = vec![
        row(
            outcome,
            format!(
                "{reason} · {}: {}. {}",
                ctx.text("battle_exchanges"),
                report.exchanges.len(),
                ctx.text("battle_exhausted")
            ),
        ),
        row(
            format!(
                "{}: {}",
                ctx.text("local_control"),
                faction(ctx, report, report.control_after)
            ),
            format!(
                "{}: {} · {}: +{} · {}: {}%",
                ctx.text("battle_previous_control"),
                faction(ctx, report, report.control_before),
                ctx.text("battle_structural_damage"),
                report.structural_damage_added,
                ctx.text("battle_occupation"),
                report.occupation_after
            ),
        ),
    ];
    for (role, side) in sides(ctx, report) {
        for army in &side.armies {
            let start: u64 = army
                .formations
                .iter()
                .map(|formation| u64::from(formation.start))
                .sum();
            let end: u64 = army
                .formations
                .iter()
                .map(|formation| u64::from(formation.end))
                .sum();
            let destination = army
                .final_site
                .map(|site| {
                    format!(
                        "{}: {}",
                        ctx.text(if site == report.site {
                            "battle_holds_site"
                        } else {
                            "battle_withdrew"
                        }),
                        place(ctx, site)
                    )
                })
                .unwrap_or_else(|| ctx.text("battle_army_destroyed"));
            rows.push(row(
                format!("{role} · {}", army.name),
                format!(
                    "{}: {start} → {end}. {destination}",
                    ctx.text("battle_counted_elements")
                ),
            ));
        }
    }
    rows
}

fn forces(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    for (role, side) in sides(ctx, report) {
        for army in &side.armies {
            for formation in &army.formations {
                let kind = ctx.text(super::super::army::troop_key(formation.kind));
                rows.push(row(
                    format!(
                        "{role} · {} · {} {}",
                        army.name,
                        ctx.text("battle_slot"),
                        formation.slot + 1
                    ),
                    format!(
                        "{kind}: {} → {} · {}: {} · {}: {} · {}: {:.1}%{}",
                        formation.start,
                        formation.end,
                        ctx.text("battle_combat_losses"),
                        formation.combat_losses,
                        ctx.text("battle_encirclement_losses"),
                        formation.encirclement_losses,
                        ctx.text("battle_veterancy_factor"),
                        formation.veterancy_permille as f32 / 10.0,
                        if formation.end == 0 {
                            format!(" · {}", ctx.text("battle_destroyed"))
                        } else {
                            String::new()
                        }
                    ),
                ));
            }
        }
    }
    rows
}

fn people(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    for (role, side) in sides(ctx, report) {
        for army in &side.armies {
            for person in &army.people {
                let class = match person.class {
                    FounderClass::Officer => ctx.text("class_officer"),
                    FounderClass::Medic => ctx.text("class_medic"),
                };
                let status = match person.status {
                    PersonStatus::Fit => ctx.text("person_fit"),
                    PersonStatus::Wounded {
                        remaining_steps, ..
                    } => format!(
                        "{} · {remaining_steps} {}",
                        ctx.text("person_wounded"),
                        ctx.text("wound_steps_remaining")
                    ),
                    PersonStatus::Dead { .. } => ctx.text("person_dead"),
                };
                rows.push(row(
                    format!("{role} · {}", person.name),
                    format!(
                        "{class} · {status}. {}",
                        assignment(ctx, report, person.assignment)
                    ),
                ));
                if let Some(row) = rows.last_mut() {
                    row.person = Some(person.id);
                }
            }
        }
    }
    for event in &report.person_events {
        let detail = match &event.outcome {
            PersonCombatOutcome::Died { reason } => ctx.text(match reason {
                PersonDeathReason::FormationDestroyed => "battle_person_died_wipe",
                PersonDeathReason::NoRefuge => "battle_person_no_refuge",
            }),
            PersonCombatOutcome::Wounded {
                assignment: target,
                cause,
            } => format!(
                "{}. {}",
                ctx.text(match cause {
                    WoundCause::FormationDestroyed => "battle_wound_wipe",
                    WoundCause::CommandCasualty => "battle_wound_command",
                }),
                assignment(ctx, report, *target)
            ),
            PersonCombatOutcome::AssumedCommand { army, previous } => {
                let armies = report.attacker.armies.iter().chain(&report.defender.armies);
                let name = armies
                    .clone()
                    .find(|entry| entry.id == *army)
                    .map(|entry| entry.name.as_str())
                    .unwrap_or_default();
                let predecessor = armies
                    .flat_map(|army| &army.people)
                    .find(|person| person.id == *previous)
                    .map(|person| person.name.as_str())
                    .unwrap_or_default();
                rows.push(row(
                    ctx.text("battle_previous_commander"),
                    predecessor.to_owned(),
                ));
                format!("{}: {name}", ctx.text("battle_assumed_command"))
            }
        };
        rows.push(row(
            format!("{} · {}", ctx.text("battle_person_event"), event.name),
            detail,
        ));
    }
    if rows.is_empty() {
        rows.push(row(ctx.text("no_battle_people"), String::new()));
    }
    rows
}

fn factors(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let mut rows = vec![
        row(
            ctx.text("battle_simultaneous"),
            ctx.text("battle_simultaneous_detail"),
        ),
        row(
            ctx.text("battle_terrain"),
            format!(
                "{}: {:.1}%",
                ctx.text("battle_defender_resistance"),
                report.terrain_permille as f32 / 10.0
            ),
        ),
    ];
    for (role, side) in sides(ctx, report) {
        for army in &side.armies {
            let commander = army
                .commander
                .as_ref()
                .map(|person| person.name.as_str())
                .unwrap_or_else(|| ctx.data.text("no_commander"));
            rows.push(row(
                format!("{role} · {}", army.name),
                format!(
                    "{}: {:.1}% · {}: {commander}",
                    ctx.text("battle_initial_leadership"),
                    army.leadership_permille as f32 / 10.0,
                    ctx.text("commander")
                ),
            ));
        }
    }
    for counter in &report.counters {
        rows.push(row(
            format!(
                "{} → {}",
                ctx.text(super::super::army::troop_key(counter.source)),
                ctx.text(super::super::army::troop_key(counter.target))
            ),
            format!(
                "{}: {:.1}%",
                ctx.text("battle_counter_attack"),
                counter.permille as f32 / 10.0
            ),
        ));
    }
    rows.extend(exchange_rows(ctx, report));
    rows
}

fn exchange_rows(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    let leadership_changes = leadership_changes(report);
    for exchange in &report.exchanges {
        let losses = |side: &BattleSideReport| -> u64 {
            exchange
                .losses
                .iter()
                .filter(|loss| {
                    side.armies.iter().any(|army| {
                        army.formations
                            .iter()
                            .any(|formation| formation.id == loss.formation)
                    })
                })
                .map(|loss| u64::from(loss.amount))
                .sum()
        };
        rows.push(row(
            format!("{} {}", ctx.text("battle_exchange"), exchange.number),
            format!(
                "{}: {} · {}: {}",
                ctx.text("battle_attacker_losses"),
                losses(&report.attacker),
                ctx.text("battle_defender_losses"),
                losses(&report.defender)
            ),
        ));
        for (_, army, factor) in leadership_changes
            .iter()
            .filter(|(number, _, _)| *number == exchange.number)
        {
            let name = report
                .attacker
                .armies
                .iter()
                .chain(&report.defender.armies)
                .find(|entry| entry.id == *army)
                .map(|army| army.name.as_str())
                .unwrap_or_default();
            rows.push(row(
                format!(
                    "{} {} · {name}",
                    ctx.text("battle_exchange"),
                    exchange.number
                ),
                format!(
                    "{}: {:.1}%",
                    ctx.text("battle_leadership_change"),
                    *factor as f32 / 10.0
                ),
            ));
        }
    }
    rows
}

pub(super) fn leadership_changes(
    report: &BattleReport,
) -> Vec<(u32, kestrum::state::military::ArmyId, u32)> {
    let mut previous: BTreeMap<_, _> = report
        .attacker
        .armies
        .iter()
        .chain(&report.defender.armies)
        .map(|army| (army.id, army.leadership_permille))
        .collect();
    let mut changes = Vec::new();
    for exchange in &report.exchanges {
        for entry in &exchange.leadership {
            if previous
                .insert(entry.army, entry.permille)
                .is_some_and(|before| before != entry.permille)
            {
                changes.push((exchange.number, entry.army, entry.permille));
            }
        }
    }
    changes
}

fn place(ctx: &Context<'_>, site: SiteId) -> String {
    ctx.campaign_view
        .and_then(|campaign| campaign.world.site(site))
        .map(|site| site.name.clone())
        .unwrap_or_else(|| ctx.text("battle_recorded_place"))
}

fn faction(ctx: &Context<'_>, report: &BattleReport, id: Option<FactionId>) -> String {
    match id {
        Some(id) if id == report.attacker.faction => report.attacker.name.clone(),
        Some(id) if id == report.defender.faction => report.defender.name.clone(),
        Some(id) => ctx
            .campaign_view
            .and_then(|campaign| campaign.factions.iter().find(|faction| faction.id == id))
            .map(|faction| faction.name.clone())
            .unwrap_or_default(),
        None => ctx.text("uncontrolled"),
    }
}

fn assignment(ctx: &Context<'_>, report: &BattleReport, target: PersonAssignment) -> String {
    match target {
        PersonAssignment::Formation { formation } => report
            .attacker
            .armies
            .iter()
            .chain(&report.defender.armies)
            .find_map(|army| {
                army.formations
                    .iter()
                    .find(|entry| entry.id == formation)
                    .map(|entry| {
                        format!(
                            "{} · {}",
                            army.name,
                            ctx.text(super::super::army::troop_key(entry.kind))
                        )
                    })
            })
            .unwrap_or_default(),
        PersonAssignment::Site { site } => {
            format!("{}: {}", ctx.text("assigned_site"), place(ctx, site))
        }
        PersonAssignment::Dead => String::new(),
    }
}
