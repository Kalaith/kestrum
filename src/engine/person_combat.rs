//! P13 person consequences apply after casualties/retreats, before zero cleanup.

mod healing;
pub(super) use healing::heal_wounds;

use super::RuleError;
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        military::{ArmyId, FormationId},
        people::{
            PersonAssignment, PersonCombatEvent, PersonCombatOutcome, PersonDeathReason, PersonId,
            PersonStatus, WoundCause,
        },
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonCombatSide {
    pub faction: FactionId,
    pub armies: Vec<ArmyId>,
    pub starting_headcounts: BTreeMap<FormationId, u32>,
    /// All fit appointed commanders before the exchanges. The lowest eligible
    /// person ID whose surviving formation reaches the loss threshold rolls once.
    pub commanders: Vec<PersonId>,
    /// Common retreat rules supply legal, adjacent, controlled inhabited refuges.
    pub refuges: Vec<SiteId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonCombatContext {
    pub site: SiteId,
    pub sides: Vec<PersonCombatSide>,
}

/// The caller has applied final headcounts and retreat positions, retaining zero
/// formations until this returns. Failures leave both the candidate and RNG intact.
pub fn resolve_person_combat(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    context: &PersonCombatContext,
) -> Result<Vec<PersonCombatEvent>, RuleError> {
    data.combat.validate().map_err(RuleError::InvalidState)?;
    validate_context(campaign, context)?;
    let mut candidate = campaign.clone();
    let mut events = Vec::new();
    let participants = participants(&candidate, data, context);
    // Global person-ID order, irrespective of which faction is the attacker.
    for (id, side, formation) in participants {
        let movement_allowance = candidate.person_movement_allowance(&candidate.people[&id], data);
        candidate
            .people
            .get_mut(&id)
            .expect("participant")
            .movement_spent = movement_allowance;
        if candidate.formations[&formation].headcount == 0 {
            resolve_wipe(&mut candidate, data, context, side, id, &mut events);
        }
    }
    let mut sides: Vec<_> = context.sides.iter().collect();
    sides.sort_by_key(|side| side.faction);
    for side in sides {
        resolve_commander(&mut candidate, data, side, &mut events);
    }
    clear_unfit_commands(&mut candidate, data);
    *campaign = candidate;
    Ok(events)
}

fn validate_context(
    campaign: &StrategicCampaign,
    context: &PersonCombatContext,
) -> Result<(), RuleError> {
    let invalid = || RuleError::InvalidState("Person encounter snapshot is inconsistent.".into());
    if campaign.world.site(context.site).is_none()
        || context.sides.is_empty()
        || context.sides.len() > 2
        || context
            .sides
            .iter()
            .map(|side| side.faction)
            .collect::<BTreeSet<_>>()
            .len()
            != context.sides.len()
    {
        return Err(invalid());
    }
    let mut all_armies = BTreeSet::new();
    for side in &context.sides {
        if !campaign.factions.contains_key(&side.faction) || side.armies.is_empty() {
            return Err(invalid());
        }
        let mut formations = BTreeSet::new();
        for id in &side.armies {
            let army = campaign.armies.get(id).ok_or_else(invalid)?;
            if army.faction != side.faction || !all_armies.insert(*id) {
                return Err(invalid());
            }
            formations.extend(army.formation_ids());
        }
        if formations != side.starting_headcounts.keys().copied().collect() {
            return Err(invalid());
        }
        for (id, starting) in &side.starting_headcounts {
            let formation = campaign.formations.get(id).ok_or_else(invalid)?;
            if *starting == 0
                || formation.faction != side.faction
                || formation.headcount > *starting
            {
                return Err(invalid());
            }
        }
        let mut commanders = BTreeSet::new();
        for commander in &side.commanders {
            let person = campaign.people.get(commander).ok_or_else(invalid)?;
            if person.faction != side.faction
                || !commanders.insert(*commander)
                || !matches!(person.assignment,
                PersonAssignment::Formation {formation} if formations.contains(&formation))
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

fn participants(
    campaign: &StrategicCampaign,
    data: &GameData,
    context: &PersonCombatContext,
) -> Vec<(PersonId, usize, FormationId)> {
    campaign
        .people
        .values()
        .filter_map(|person| {
            let PersonAssignment::Formation { formation } = person.assignment else {
                return None;
            };
            if !person.is_alive() || person.status != PersonStatus::Fit {
                return None;
            }
            context
                .sides
                .iter()
                .position(|side| {
                    side.faction == person.faction
                        && side.starting_headcounts.contains_key(&formation)
                        && (person.age_years(campaign.completed_rounds)
                            < data.lifecycle.elder_age_years
                            || side.armies.iter().any(|army| {
                                campaign
                                    .armies
                                    .get(army)
                                    .is_some_and(|army| army.commander == Some(person.id))
                            }))
                })
                .map(|side| (person.id, side, formation))
        })
        .collect()
}

fn resolve_wipe(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    context: &PersonCombatContext,
    side_index: usize,
    person: PersonId,
    events: &mut Vec<PersonCombatEvent>,
) {
    let side = &context.sides[side_index];
    if campaign.rng.combat.below(100) < data.combat.wipe_death_percent as usize {
        kill(
            campaign,
            person,
            context.site,
            PersonDeathReason::FormationDestroyed,
            events,
        );
        return;
    }
    let survivor = side
        .starting_headcounts
        .keys()
        .copied()
        .find(|id| campaign.formations[id].headcount > 0);
    let assignment = survivor
        .map(|formation| PersonAssignment::Formation { formation })
        .or_else(|| {
            refuge(campaign, context.site, side).map(|site| PersonAssignment::Site { site })
        });
    let Some(assignment) = assignment else {
        kill(
            campaign,
            person,
            context.site,
            PersonDeathReason::NoRefuge,
            events,
        );
        return;
    };
    campaign
        .people
        .get_mut(&person)
        .expect("wiped participant")
        .assignment = assignment;
    wound(
        campaign,
        data,
        person,
        WoundCause::FormationDestroyed,
        events,
    );
}

fn refuge(campaign: &StrategicCampaign, origin: SiteId, side: &PersonCombatSide) -> Option<SiteId> {
    let adjacent = campaign.world.adjacent_sites(origin);
    side.refuges
        .iter()
        .copied()
        .filter(|site| {
            adjacent.contains(site)
                && campaign.world.is_secure(*site, side.faction)
                && campaign
                    .world
                    .site(*site)
                    .is_some_and(|site| site.habitation != Habitation::Unsettled)
        })
        .min()
}

fn resolve_commander(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    side: &PersonCombatSide,
    events: &mut Vec<PersonCombatEvent>,
) {
    let eligible = side
        .commanders
        .iter()
        .copied()
        .filter_map(|person| {
            eligible_commander(campaign, data, side, person).map(|army| (person, army))
        })
        .min();
    let Some((id, army_id)) = eligible else {
        return;
    };
    if campaign.rng.combat.below(100) >= data.combat.commander_wound_percent as usize {
        return;
    }
    wound(campaign, data, id, WoundCause::CommandCasualty, events);
    let army = &campaign.armies[&army_id];
    let successor = campaign.people.values().find(|person|
        person.faction == side.faction
        && !person.career.retired
        && person.is_fit_for_field(campaign.completed_rounds, data.rules.leadership.field_min_age_years)
        && matches!(person.assignment, PersonAssignment::Formation { formation }
            if army.formation_ids().any(|id| id == formation) && campaign.formations[&formation].headcount > 0)
    ).map(|person| person.id);
    campaign
        .armies
        .get_mut(&army_id)
        .expect("surviving command army")
        .commander = successor;
    if let Some(person) = successor {
        push_event(
            campaign,
            person,
            PersonCombatOutcome::AssumedCommand {
                army: army_id,
                previous: id,
            },
            events,
        );
    }
}

fn eligible_commander(
    campaign: &StrategicCampaign,
    data: &GameData,
    side: &PersonCombatSide,
    id: PersonId,
) -> Option<ArmyId> {
    let person = &campaign.people[&id];
    if !person.is_fit_for_field(
        campaign.completed_rounds,
        data.rules.leadership.field_min_age_years,
    ) {
        return None;
    }
    let PersonAssignment::Formation { formation } = person.assignment else {
        return None;
    };
    let starting = side.starting_headcounts.get(&formation)?;
    let remaining = campaign.formations[&formation].headcount;
    if remaining == 0
        || u64::from(starting - remaining) * 100
            < u64::from(*starting) * u64::from(data.combat.commander_loss_percent)
    {
        return None;
    }
    side.armies
        .iter()
        .copied()
        .find(|id| campaign.armies[id].commander == Some(person.id))
}

fn wound(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: PersonId,
    cause: WoundCause,
    events: &mut Vec<PersonCombatEvent>,
) {
    let person = campaign.people.get_mut(&id).expect("wounded participant");
    person.status = PersonStatus::Wounded {
        since_round: campaign.completed_rounds,
        remaining_steps: data.combat.wound_recovery_steps,
    };
    let assignment = person.assignment;
    push_event(
        campaign,
        id,
        PersonCombatOutcome::Wounded { assignment, cause },
        events,
    );
}

fn kill(
    campaign: &mut StrategicCampaign,
    id: PersonId,
    site: SiteId,
    reason: PersonDeathReason,
    events: &mut Vec<PersonCombatEvent>,
) {
    super::lifecycle::mark_dead(campaign, id, site);
    push_event(campaign, id, PersonCombatOutcome::Died { reason }, events);
}

fn push_event(
    campaign: &StrategicCampaign,
    id: PersonId,
    outcome: PersonCombatOutcome,
    events: &mut Vec<PersonCombatEvent>,
) {
    let person = &campaign.people[&id];
    events.push(PersonCombatEvent {
        person: id,
        name: person.name.clone(),
        faction: person.faction,
        outcome,
    });
}

fn clear_unfit_commands(campaign: &mut StrategicCampaign, data: &GameData) {
    for army in campaign.armies.values_mut() {
        let valid = army.commander.and_then(|id| campaign.people.get(&id)).is_some_and(|person|
            person.faction == army.faction
                && !person.career.retired
                && person.is_fit_for_field(campaign.completed_rounds, data.rules.leadership.field_min_age_years)
                && matches!(person.assignment, PersonAssignment::Formation { formation }
                    if army.formation_ids().any(|id| id == formation) && campaign.formations[&formation].headcount > 0));
        if !valid {
            army.commander = None;
        }
    }
}
