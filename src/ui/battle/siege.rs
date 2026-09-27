//! Siege outcomes retain their actual encounter context and lasting damage.

use super::*;
use kestrum::state::battle::BattleContext;

pub(super) fn position_key(
    report: &BattleReport,
    faction: kestrum::data::world::FactionId,
    site: kestrum::data::world::SiteId,
) -> &'static str {
    if site != report.site {
        return "battle_withdrew";
    }
    if report.context != BattleContext::Field {
        if report.control_after != Some(faction) {
            return "battle_outside_walls";
        }
        if report.outcome != kestrum::state::battle::BattleOutcome::AttackerVictory
            && faction == report.attacker.faction
            && matches!(
                report.context,
                BattleContext::Sortie { .. }
                    | BattleContext::Escape { .. }
                    | BattleContext::Relief { .. }
            )
        {
            return "battle_inside_walls";
        }
    }
    "battle_holds_site"
}

pub(super) fn extra_rows(report: &BattleReport) -> usize {
    usize::from(report.context != BattleContext::Field)
        + usize::from(report.fort_damage_added > 0 || report.road_damage.is_some())
}

pub(super) fn outcome(ctx: &Context<'_>, report: &BattleReport) -> Vec<ReportRow> {
    let mut rows = Vec::new();
    if report.context != BattleContext::Field {
        let key = match report.context {
            BattleContext::Field => unreachable!(),
            BattleContext::Assault { .. } => "battle_context_assault",
            BattleContext::Sortie { .. } => "battle_context_sortie",
            BattleContext::Escape { .. } => "battle_context_escape",
            BattleContext::Relief { .. } => "battle_context_relief",
            BattleContext::BesiegerClash { .. } => "battle_context_besiegers",
        };
        rows.push(ReportRow {
            heading: ctx.text(key),
            detail: ctx.text("battle_siege_positions"),
            person: None,
        });
    }
    if report.fort_damage_added > 0 || report.road_damage.is_some() {
        let mut detail = format!(
            "{}: +{}",
            ctx.text("siege_fort_damage"),
            report.fort_damage_added
        );
        if let Some(road) = &report.road_damage {
            let route = ctx
                .campaign_view
                .and_then(|view| view.world.route(road.route));
            if let Some(route) = route {
                let names: Vec<_> = [route.from, route.to]
                    .iter()
                    .filter_map(|id| {
                        ctx.campaign_view?
                            .world
                            .site(*id)
                            .map(|site| site.name.as_str())
                    })
                    .collect();
                detail.push_str(&format!(
                    " · {}: {} (+{})",
                    ctx.text("battle_road_damage"),
                    names.join(" — "),
                    road.added
                ));
            }
        }
        rows.push(ReportRow {
            heading: ctx.text("battle_lasting_damage"),
            detail,
            person: None,
        });
    }
    rows
}

pub(super) fn wall_row(ctx: &Context<'_>, report: &BattleReport) -> Option<ReportRow> {
    (report.context != BattleContext::Field).then(|| ReportRow {
        heading: ctx.text("siege_walls"),
        detail: format!(
            "{}: {:.1}%. {}",
            ctx.text("battle_initial_walls"),
            report.wall_permille as f32 / 10.0,
            ctx.text("battle_walls_help")
        ),
        person: None,
    })
}
