//! Formation specialization and its prepaid course receipt.
use super::*;

pub(in crate::ui::army) fn draw(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    id: FormationId,
) -> Option<UiAction> {
    let formation = campaign
        .formations
        .iter()
        .find(|formation| formation.id == id)?;
    body(
        ctx,
        &format!(
            "{} · {} / {}",
            ctx.text("formation_specialization"),
            ctx.text(troop_key(formation.kind)),
            formation.headcount
        ),
        vec2(112.0, 180.0),
        22.0,
        CREAM,
    );
    let site = campaign
        .armies
        .iter()
        .find(|army| army.formation_ids().any(|member| member == id))
        .map(|army| army.site);
    let existing = formation
        .service
        .specialization
        .map(|specialization| ctx.text(specialization_key(specialization)))
        .unwrap_or_else(|| ctx.text("specialization_none"));
    body(
        ctx,
        &format!("{}: {existing}", ctx.text("current_specialization")),
        vec2(112.0, 212.0),
        18.0,
        MUTED,
    );
    if let Some(course) = &formation.service.course {
        let total = ctx.progression.specializations[&course.target].course_steps;
        let status = format!(
            "{} · {} {}/{} · {}",
            ctx.text("specialization_in_progress"),
            ctx.text(specialization_key(course.target)),
            course.steps_completed,
            total,
            ctx.text(if course.steps_completed == 0 {
                "course_refund_full"
            } else {
                "course_refund_none"
            })
        );
        body(
            ctx,
            &truncate_text_to_width_ex(&status, 744.0, ctx.body_font(), 18.0),
            vec2(112.0, 252.0),
            18.0,
            BRASS,
        );
        if button(
            ctx,
            Rect::new(868.0, 226.0, 300.0, 48.0),
            &ctx.text("cancel_course"),
            true,
            false,
        ) {
            return Some(UiAction::CancelFormationCourse(id));
        }
    }
    body(
        ctx,
        &ctx.text("formation_course_pause_help"),
        vec2(112.0, 279.0),
        18.0,
        MUTED,
    );
    let mut y = 302.0;
    for (kind, rule) in &ctx.progression.specializations {
        if let Some(action) = option(ctx, campaign, formation, site, *kind, rule, y) {
            return Some(action);
        }
        y += 76.0;
    }
    button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    )
    .then_some(UiAction::CancelArmyAction)
}

fn option(
    ctx: &Context<'_>,
    campaign: &VisibleCampaign,
    formation: &kestrum::state::military::Formation,
    site: Option<SiteId>,
    kind: FormationSpecialization,
    rule: &kestrum::data::progression::SpecializationRule,
    y: f32,
) -> Option<UiAction> {
    let id = formation.id;
    let price = kestrum::engine::course_gold_cost(
        rule.gold_cost,
        &ctx.progression.careers,
        site.and_then(|id| campaign.world.focus.get(&id).copied()),
    );
    let (current, required, secondary) = requirement(formation, &rule.requirement);
    let mut description = format!(
        "{} · {} {current}/{required} · {price} {} · {} {} · {}",
        ctx.text(specialization_key(kind)),
        ctx.text(specialization_requirement_key(kind)),
        ctx.text("gold"),
        rule.course_steps,
        ctx.text("course_steps"),
        ctx.text(specialization_effect_key(kind))
    );
    if let Some((actual, needed)) = secondary {
        description.push_str(&format!(
            " · {} {actual}/{needed}",
            ctx.text("retreating_victories")
        ));
    }
    body(
        ctx,
        &truncate_text_to_width_ex(&description, 760.0, ctx.body_font(), 15.0),
        vec2(112.0, y + 26.0),
        15.0,
        if current >= required && secondary.is_none_or(|(actual, needed)| actual >= needed) {
            CREAM
        } else {
            MUTED
        },
    );
    let source = rule.sources.contains(&formation.kind);
    let available_site =
        site.filter(|site| course_site(campaign, *site, Facility::TrainingGround, false));
    let resources = campaign
        .factions
        .iter()
        .find(|faction| faction.id == campaign.observer)
        .and_then(|faction| faction.resources);
    let enough_gold = resources.is_some_and(|resources| resources.gold >= price);
    let met = current >= required && secondary.is_none_or(|(actual, needed)| actual >= needed);
    let enabled = source
        && met
        && enough_gold
        && available_site.is_some()
        && formation.service.specialization.is_none()
        && formation.service.course.is_none();
    if let (Some(site), true) = (available_site, enabled) {
        if button(
            ctx,
            Rect::new(880.0, y, 288.0, 48.0),
            &ctx.text("convert_formation"),
            true,
            false,
        ) {
            return Some(UiAction::SpecializeFormation(id, kind, site));
        }
    } else {
        let key = if !source {
            "specialization_wrong_source"
        } else if !met {
            "requirements_missing"
        } else if !enough_gold {
            "course_gold_missing"
        } else {
            "course_unavailable"
        };
        button(
            ctx,
            Rect::new(880.0, y, 288.0, 48.0),
            &ctx.text(key),
            false,
            false,
        );
    }
    None
}

fn requirement(
    formation: &kestrum::state::military::Formation,
    rule: &SpecializationRequirement,
) -> (usize, usize, Option<(usize, usize)>) {
    match *rule {
        SpecializationRequirement::DefendedAnchors { occasions } => (
            formation
                .service
                .ledger
                .counts
                .get(&EvidenceKind::DefendedAnchor)
                .copied()
                .unwrap_or(0) as usize,
            occasions as usize,
            None,
        ),
        SpecializationRequirement::MeaningfulAgainst { troop, occasions } => (
            formation
                .service
                .ledger
                .meaningful_against
                .get(&troop)
                .copied()
                .unwrap_or(0) as usize,
            occasions as usize,
            None,
        ),
        SpecializationRequirement::RoutesAndRetreatingVictories { routes, victories } => (
            formation.service.ledger.traversed_routes.len(),
            routes,
            Some((
                formation
                    .service
                    .ledger
                    .counts
                    .get(&EvidenceKind::RetreatingEnemyVictory)
                    .copied()
                    .unwrap_or(0) as usize,
                victories as usize,
            )),
        ),
    }
}
