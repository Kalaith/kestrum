//! Placement prices and receipts shared by previews, commands, UI and cleanup.

use crate::{
    data::{progression::CareerRules, world::SiteId, GameData},
    engine::RuleError,
    state::{construction::Focus, people::PersonCourse, StrategicCampaign},
};

/// Round the discounted total upward, including odd base prices.
pub fn course_gold_cost(base: i64, rules: &CareerRules, focus: Option<Focus>) -> i64 {
    if focus != Some(Focus::TroopTraining) {
        return base;
    }
    let percent = 100_u32.saturating_sub(rules.training_focus_discount_percent);
    ((i128::from(base) * i128::from(percent) + 99) / 100) as i64
}

pub(super) fn local_cost(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: SiteId,
    base: i64,
) -> i64 {
    course_gold_cost(
        base,
        &data.progression.careers,
        campaign.world.focus.get(&site).copied(),
    )
}

pub(super) fn person_refund(course: &PersonCourse) -> i64 {
    match course {
        PersonCourse::Class {
            paid_gold,
            steps_completed: 0,
            ..
        } => *paid_gold,
        _ => 0,
    }
}

/// Death and retirement release courses immediately; settle their receipt once
/// while both the before and after states of the accepted action are available.
pub(crate) fn refund_departures(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
) -> Result<(), RuleError> {
    for previous in before.people.values() {
        let departed = campaign.people.get(&previous.id).is_some_and(|person| {
            (previous.is_alive() && !person.is_alive())
                || (!previous.career.retired && person.career.retired)
        });
        if !departed {
            continue;
        }
        let amount = previous.career.course.as_ref().map_or(0, person_refund);
        let resources = &mut campaign
            .factions
            .get_mut(&previous.faction)
            .expect("course owner")
            .resources;
        resources.gold = resources
            .gold
            .checked_add(amount)
            .ok_or(RuleError::Overflow {
                field: "departed course refund",
            })?;
    }
    Ok(())
}
