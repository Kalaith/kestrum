//! Ordinary threat reports use witnessed counts without invented military identities.
use super::*;
use kestrum::state::battle::BattleDefender;

pub(super) fn outcome(ctx: &Context<'_>, report: &BattleReport) -> Option<ReportRow> {
    let BattleDefender::Threat(threat) = &report.defender else {
        return None;
    };
    Some(ReportRow {
        heading: format!(
            "{} · {}",
            threat.name,
            ctx.text(if threat.end == 0 {
                "threat_cleared"
            } else {
                "threat_remains"
            })
        ),
        detail: format!(
            "{}: {} {} · {} {} · {} {}. {}",
            ctx.text("threat_reward_paid"),
            threat.payout.gold,
            ctx.text("gold"),
            threat.payout.wood,
            ctx.text("wood"),
            threat.payout.stone,
            ctx.text("stone"),
            ctx.text("threat_reward_once")
        ),
        person: None,
    })
}

pub(super) fn forces(ctx: &Context<'_>, report: &BattleReport) -> Option<ReportRow> {
    let BattleDefender::Threat(threat) = &report.defender else {
        return None;
    };
    Some(ReportRow {
        heading: format!("{} · {}", ctx.text("battle_defender"), threat.name),
        detail: format!(
            "{} → {} · {}: {} · {}: {}",
            threat.start,
            threat.end,
            ctx.text("battle_combat_losses"),
            threat.combat_losses,
            ctx.text("battle_encirclement_losses"),
            threat.encirclement_losses
        ),
        person: None,
    })
}

pub(super) fn factors(ctx: &Context<'_>, report: &BattleReport) -> Option<ReportRow> {
    let BattleDefender::Threat(threat) = &report.defender else {
        return None;
    };
    Some(ReportRow {
        heading: threat.name.clone(),
        detail: format!(
            "{}: {} · {}: {}. {}",
            ctx.text("threat_attack"),
            threat.attack,
            ctx.text("threat_resistance"),
            threat.resistance,
            ctx.text("threat_combat_rules")
        ),
        person: None,
    })
}
