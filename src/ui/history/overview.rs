//! Current own service and dated foreign observations have separate presentation paths.

use super::*;
use kestrum::state::{
    evidence::{EvidenceKind, EvidenceLedger, FormationService, Veterancy},
    knowledge::ObservedCondition,
    people::{Person, PersonStatus},
};

pub(super) fn title(ctx: &Context<'_>) -> String {
    let Some(view) = ctx.campaign_view else {
        return ctx.text("history");
    };
    match ctx.history.subject {
        Some(HistorySubject::Person(_)) => ctx.history.person.as_ref().map(|p| p.name().to_owned()),
        Some(HistorySubject::Site(id)) => view.world.site(id).map(|s| s.name.clone()),
        Some(HistorySubject::Army(id)) => view
            .armies
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone()),
        Some(HistorySubject::Formation(id)) => {
            view.formations.iter().find(|f| f.id == id).map(|f| {
                format!(
                    "{} · {}",
                    ctx.text(super::super::army::troop_key(f.kind)),
                    ctx.text("formation_service")
                )
            })
        }
        None => None,
    }
    .unwrap_or_else(|| ctx.text("history"))
}

pub(super) fn build(ctx: &Context<'_>) -> Vec<HistoryRow> {
    let Some(view) = ctx.campaign_view else {
        return Vec::new();
    };
    let mut rows = match ctx.history.subject {
        Some(HistorySubject::Person(_)) => person(ctx),
        Some(HistorySubject::Formation(id)) => view
            .formations
            .iter()
            .find(|f| f.id == id)
            .map(|f| service(ctx, &f.service))
            .unwrap_or_default(),
        Some(HistorySubject::Site(id)) => view
            .world
            .site(id)
            .map(|site| {
                let owner = site
                    .controller
                    .and_then(|id| view.factions.iter().find(|f| f.id == id))
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| ctx.text("uncontrolled"));
                vec![HistoryRow::new(ctx.text("local_control"), owner)]
            })
            .unwrap_or_default(),
        Some(HistorySubject::Army(id)) => view
            .armies
            .iter()
            .find(|a| a.id == id)
            .map(|army| {
                let site = view
                    .world
                    .site(army.site)
                    .map(|site| site.name.clone())
                    .unwrap_or_default();
                vec![HistoryRow::new(ctx.text("history_current_site"), site)]
            })
            .unwrap_or_default(),
        None => Vec::new(),
    };
    // Notables are already audience-filtered and retain their own historical labels.
    if !matches!(
        ctx.history.person,
        Some(PersonKnowledge::LastEncountered { .. })
    ) {
        if let Some(page) = &ctx.history.result {
            for notable in &page.notables {
                rows.push(HistoryRow::new(
                    format!(
                        "{} · {}",
                        date(ctx, notable.completed_rounds),
                        super::rows::kind(ctx, &notable.kind)
                    ),
                    notable
                        .site
                        .as_ref()
                        .map(|s| s.name.clone())
                        .unwrap_or_default(),
                ));
            }
        }
    }
    rows
}

fn person(ctx: &Context<'_>) -> Vec<HistoryRow> {
    match &ctx.history.person {
        Some(PersonKnowledge::CurrentOwn(person)) => own_person(ctx, person),
        Some(PersonKnowledge::LastEncountered {
            snapshot,
            available_report,
        }) => {
            let condition = ctx.text(match snapshot.condition {
                ObservedCondition::Fit => "person_fit",
                ObservedCondition::Wounded => "person_wounded",
                ObservedCondition::Dead => "person_dead",
            });
            let mut rows = vec![
                HistoryRow::new(
                    ctx.text("last_encountered"),
                    format!(
                        "{} · {}",
                        date(ctx, snapshot.completed_rounds),
                        snapshot.site_name
                    ),
                ),
                HistoryRow::new(
                    ctx.text("history_observed_role"),
                    class_name(ctx, snapshot.class),
                ),
                HistoryRow::new(ctx.text("history_observed_condition"), condition),
                HistoryRow::new(
                    ctx.text("history_observed_army"),
                    snapshot.army_name.clone(),
                ),
            ];
            let report = HistoryRow::new(
                ctx.text("history_recorded_details"),
                ctx.text(if available_report.is_some() {
                    "history_last_known_help"
                } else {
                    "history_report_expired"
                }),
            );
            rows.push(if let Some(id) = available_report {
                report.link(UiAction::OpenRecordedBattle(*id), "history_report")
            } else {
                report
            });
            rows
        }
        None => vec![HistoryRow::new(
            ctx.text("history_unknown_person"),
            ctx.text("history_no_hidden_facts"),
        )],
    }
}

fn own_person(ctx: &Context<'_>, person: &Person) -> Vec<HistoryRow> {
    let round = ctx.campaign_view.map_or(0, |view| view.completed_rounds);
    let status = match person.status {
        PersonStatus::Fit => ctx.text("person_fit"),
        PersonStatus::Displaced {
            completed_rounds, ..
        } => format!(
            "{} · {}",
            ctx.text("kingdom_displaced"),
            date(ctx, completed_rounds)
        ),
        PersonStatus::Wounded {
            remaining_steps, ..
        } => format!(
            "{} · {remaining_steps} {}",
            ctx.text("person_wounded"),
            ctx.text("wound_steps_remaining")
        ),
        PersonStatus::Dead {
            completed_rounds, ..
        } => format!(
            "{} · {}",
            ctx.text("person_dead"),
            date(ctx, completed_rounds)
        ),
    };
    let mut rows = vec![
        HistoryRow::new(
            format!(
                "{} · {} {}",
                class_name(ctx, person.class),
                ctx.text("history_age"),
                person.age_years(round)
            ),
            status,
        ),
        HistoryRow::new(
            ctx.text("history_service_start"),
            date(ctx, person.service_start_round),
        ),
        HistoryRow::new(
            ctx.text("history_personal_service"),
            ctx.text("history_no_inherited_deeds"),
        ),
    ];
    rows.extend(evidence(ctx, &person.evidence));
    rows
}

fn service(ctx: &Context<'_>, service: &FormationService) -> Vec<HistoryRow> {
    let rules = ctx.progression;
    let (factor, next) = match service.tier {
        Veterancy::Ordinary => (rules.ordinary_permille, Some(rules.seasoned_xp)),
        Veterancy::Seasoned => (rules.seasoned_permille, Some(rules.veteran_xp)),
        Veterancy::Veteran => (rules.veteran_permille, None),
    };
    let progress = next
        .map(|next| format!("{} / {next} {}", service.xp, ctx.text("service_xp")))
        .unwrap_or_else(|| {
            format!(
                "{} {} · {}",
                service.xp,
                ctx.text("service_xp"),
                ctx.text("service_highest_tier")
            )
        });
    let mut rows = vec![
        HistoryRow::new(
            super::rows::tier_text(ctx, service.tier),
            format!(
                "{progress} · {}: {:.1}%",
                ctx.text("service_combat_factor"),
                factor as f32 / 10.0
            ),
        ),
        HistoryRow::new(
            ctx.text("service_earning"),
            format!(
                "{}: +{} · {}: +{} · {}: +{} · {}: {}",
                ctx.text("evidence_meaningful"),
                rules.battle_xp,
                ctx.text("evidence_victory"),
                rules.victory_xp,
                ctx.text("evidence_outnumbered"),
                rules.outnumbered_xp,
                ctx.text("service_round_cap"),
                rules.round_xp_cap
            ),
        ),
        HistoryRow::new(
            ctx.text("service_preserved"),
            ctx.text("service_recovery_help"),
        ),
    ];
    rows.extend(evidence(ctx, &service.ledger));
    for season in service.recent.iter().rev() {
        rows.push(HistoryRow::new(
            format!(
                "{} · {}",
                ctx.text("service_recent"),
                date(ctx, season.completed_rounds)
            ),
            format!(
                "{} {} · {} {}",
                season.xp,
                ctx.text("service_xp"),
                season.encounters.len(),
                ctx.text("service_encounters")
            ),
        ));
    }
    rows
}

fn evidence(ctx: &Context<'_>, ledger: &EvidenceLedger) -> Vec<HistoryRow> {
    let mut rows: Vec<_> = ledger
        .counts
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(kind, count)| {
            HistoryRow::new(
                ctx.text(evidence_key(*kind)),
                format!("{}: {count}", ctx.text("service_recorded_occurrences")),
            )
        })
        .collect();
    if !ledger.encountered_troops.is_empty() {
        let kinds = ledger
            .encountered_troops
            .iter()
            .map(|kind| ctx.text(super::super::army::troop_key(*kind)))
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(HistoryRow::new(ctx.text("service_encountered"), kinds));
    }
    if rows.is_empty() {
        rows.push(HistoryRow::new(
            ctx.text("service_no_evidence"),
            ctx.text("service_no_evidence_help"),
        ));
    }
    rows
}

fn class_name(ctx: &Context<'_>, class: kestrum::data::world::FounderClass) -> String {
    ctx.text(match class {
        kestrum::data::world::FounderClass::Recruit => "class_recruit",
        kestrum::data::world::FounderClass::Infantry => "class_infantry",
        kestrum::data::world::FounderClass::Archer => "class_archer",
        kestrum::data::world::FounderClass::Scout => "class_scout",
        kestrum::data::world::FounderClass::Cavalry => "class_cavalry",
        kestrum::data::world::FounderClass::Officer => "class_officer",
        kestrum::data::world::FounderClass::Medic => "class_medic",
    })
}

fn evidence_key(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::EncounteredBandits => "evidence_bandits",
        EvidenceKind::EncounteredWildlife => "evidence_wildlife",
        EvidenceKind::ClearedThreat => "evidence_clear_threat",
        EvidenceKind::AssaultedFort => "evidence_assaulted_fort",
        EvidenceKind::DefendedFort => "evidence_defended_fort",
        EvidenceKind::Sortie => "evidence_sortie",
        EvidenceKind::EscapeAttempt => "evidence_escape_attempt",
        EvidenceKind::Relief => "evidence_relief",
        EvidenceKind::Battle => "evidence_battle",
        EvidenceKind::MeaningfulEncounter => "evidence_meaningful",
        EvidenceKind::Victory => "evidence_victory",
        EvidenceKind::RetreatingEnemyVictory => "evidence_retreating_enemy_victory",
        EvidenceKind::Defeat => "evidence_defeat",
        EvidenceKind::SurvivedOutnumbered => "evidence_outnumbered",
        EvidenceKind::DefendedAnchor => "evidence_defended_anchor",
        EvidenceKind::CapturedAnchor => "evidence_captured_anchor",
        EvidenceKind::Retreated => "evidence_retreated",
        EvidenceKind::TreatedWounded => "evidence_treated",
        EvidenceKind::CommanderWounded => "evidence_commander_wounded",
        EvidenceKind::AssumedCommand => "evidence_assumed_command",
        EvidenceKind::CommandedVictory => "evidence_commanded_victory",
    }
}
