//! Save receipts validate historical identity separately from live order bindings.

use super::*;
use crate::data::GameData;
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub(crate) fn validate_construction(&self, data: &GameData) -> Result<(), String> {
        if self.world.population.len() != self.world.sites.len()
            || self
                .world
                .sites
                .iter()
                .any(|site| !self.world.population.contains_key(&site.id))
            || self
                .world
                .focus
                .keys()
                .any(|id| self.world.site(*id).is_none())
        {
            return Err("construction: population/focus has unknown or missing sites".into());
        }
        if self.next_ids.order.0 == 0 {
            return Err("construction: zero next order identifier".into());
        }
        let mut targets = BTreeSet::new();
        let mut builders = BTreeSet::new();
        let mut terminal = BTreeSet::new();
        for (id, order) in &self.construction {
            self.validate_construction_receipt(order, self.completed_rounds)?;
            if *id != order.id || order.created_round > self.completed_rounds {
                return Err("construction: mismatched identity/date".into());
            }
            if order.is_open() {
                if !targets.insert(order.target)
                    || !self.owns_construction_target(order.owner, order.target)
                {
                    return Err("construction: duplicate or foreign work".into());
                }
                if let Some(builder) = order.builder {
                    if !builders.insert(builder)
                        || self
                            .armies
                            .get(&builder)
                            .is_none_or(|army| army.faction != order.owner)
                    {
                        return Err("construction: duplicate or unavailable builder".into());
                    }
                } else if !matches!(order.kind, ConstructionKind::Facility(_))
                    && order.status
                        != (ConstructionStatus::Paused {
                            reason: ConstructionPause::BuilderMissing,
                        })
                {
                    return Err("construction: missing assigned builder".into());
                }
                if matches!(order.kind, ConstructionKind::Facility(_)) && order.builder.is_some() {
                    return Err("construction: facility retains a builder reservation".into());
                }
            } else if !terminal.insert(order.target) {
                return Err("construction: unbounded terminal receipts at one target".into());
            }
            let (cost, steps) = crate::engine::construction::terms(data, order.kind);
            if order.paid != cost || order.required_steps != steps {
                return Err(
                    "construction: paid cost/duration disagrees with compatible rules".into(),
                );
            }
        }
        Ok(())
    }

    pub(crate) fn validate_construction_receipt(
        &self,
        order: &ConstructionOrder,
        recorded_round: u32,
    ) -> Result<(), String> {
        order.paid.validate("construction", "paid")?;
        let valid = order.id.0 > 0
            && order.id < self.next_ids.order
            && self.factions.contains_key(&order.owner)
            && order.created_round <= recorded_round
            && order.required_steps > 0
            && order.required_steps <= 100
            && order.progress <= order.required_steps
            && order
                .builder
                .is_none_or(|id| id.0 > 0 && id < self.next_ids.army)
            && order.last_progress_round.is_none_or(|round| {
                round >= order.created_round
                    && round <= recorded_round
                    && u64::from(order.progress)
                        <= u64::from(round) - u64::from(order.created_round) + 1
            })
            && ((order.progress > 0) == order.last_progress_round.is_some());
        if !valid {
            return Err("construction receipt: invalid identity, cost, progress or date".into());
        }
        match (order.target, order.kind) {
            (
                ConstructionTarget::Site(id),
                ConstructionKind::Outpost | ConstructionKind::Fort | ConstructionKind::Facility(_),
            ) if self.world.site(id).is_some() => {}
            (
                ConstructionTarget::Route(id),
                ConstructionKind::Road | ConstructionKind::RoadRepair,
            ) if self.world.route(id).is_some() => {}
            _ => return Err("construction receipt: invalid target/kind".into()),
        }
        let terminal_date = match order.status {
            ConstructionStatus::Completed { completed_rounds }
                if order.progress == order.required_steps
                    && order.last_progress_round == Some(completed_rounds) =>
            {
                Some(completed_rounds)
            }
            ConstructionStatus::Cancelled {
                completed_rounds, ..
            } if order.progress < order.required_steps => Some(completed_rounds),
            ConstructionStatus::Active | ConstructionStatus::Paused { .. }
                if order.progress < order.required_steps =>
            {
                None
            }
            _ => return Err("construction receipt: progress disagrees with status".into()),
        };
        if terminal_date.is_some_and(|round| round < order.created_round || round > recorded_round)
        {
            return Err("construction receipt: invalid terminal date".into());
        }
        Ok(())
    }
}
