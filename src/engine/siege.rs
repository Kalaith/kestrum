//! Persistent inside/outside rosters and one eligible weakening step per season.

mod arrival;
mod commands;
mod projection;

pub(super) use arrival::{admission, arrive};
pub(super) use commands::{execute, validate_order};
pub use projection::{siege_view, SiegeActionOption, SiegeView};

use super::{actions::record_fact, combat, retreat, ActionOutcome, RuleError};
use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        campaign::DomainFactKind,
        military::ArmyId,
        people::PersonAssignment,
        siege::{Siege, SiegeChange, SiegeId},
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub(crate) fn destinations(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: SiteId,
) -> Vec<SiteId> {
    retreat::eligible(campaign, owner, site, None)
}

pub(super) fn exhaust(campaign: &mut StrategicCampaign, data: &GameData, armies: &[ArmyId]) {
    let formations: BTreeSet<_> = armies
        .iter()
        .filter_map(|id| campaign.armies.get(id))
        .flat_map(|army| army.formation_ids())
        .collect();
    let spent = formations
        .iter()
        .filter_map(|id| {
            campaign
                .formations
                .get(id)
                .map(|formation| (*id, formation.movement_allowance(data)))
        })
        .collect::<Vec<_>>();
    for (id, allowance) in spent {
        if let Some(formation) = campaign.formations.get_mut(&id) {
            formation.movement_spent = allowance;
        }
    }
    for person in campaign.people.values_mut() {
        if matches!(person.assignment, PersonAssignment::Formation{formation} if formations.contains(&formation))
        {
            person.movement_spent = data.rules.leadership.officer_movement_allowance;
        }
    }
}

pub(super) fn occupants(
    campaign: &StrategicCampaign,
    site: SiteId,
    faction: FactionId,
) -> Vec<ArmyId> {
    campaign
        .armies
        .values()
        .filter(|army| army.site == site && army.faction == faction && !army.is_empty())
        .map(|army| army.id)
        .collect()
}

pub(super) fn establish(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    defender: FactionId,
    besieger: FactionId,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let siege = Siege {
        id: campaign.next_ids.siege,
        site,
        defender,
        besieger,
        defending: occupants(campaign, site, defender),
        besieging: occupants(campaign, site, besieger),
        started_round: campaign.completed_rounds,
        elapsed_steps: 0,
        last_progress_round: None,
    };
    campaign.next_ids.siege = SiegeId(siege.id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "siege identifiers",
    })?);
    exhaust(campaign, data, &siege.besieging);
    campaign.world.contested_sites.insert(site);
    campaign.sieges.insert(site, siege.clone());
    campaign.reconcile_region_control();
    record_fact(
        campaign,
        outcome,
        DomainFactKind::SiegeChanged {
            siege,
            change: SiegeChange::Established,
        },
    )
}

pub(super) fn reconcile(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    for old in campaign.sieges.values().cloned().collect::<Vec<_>>() {
        let mut siege = old.clone();
        siege.defending = occupants(campaign, siege.site, siege.defender);
        siege.besieging = occupants(campaign, siege.site, siege.besieger);
        if (!campaign.is_independent(siege.defender) && !siege.defending.is_empty())
            || (!campaign.is_independent(siege.besieger) && !siege.besieging.is_empty())
        {
            return Err(RuleError::Siege(
                "Resolve inactive faction military forces before reconciling its siege.".into(),
            ));
        }
        let control = campaign
            .world
            .site(siege.site)
            .and_then(|site| site.controller);
        let hostile = retreat::hostile(campaign, siege.defender, siege.besieger);
        if !hostile {
            withdraw_peace(campaign, data, &mut siege)?;
        }
        let active = !siege.defending.is_empty()
            && !siege.besieging.is_empty()
            && hostile
            && control == Some(siege.defender);
        if !active {
            campaign.sieges.remove(&siege.site);
            campaign.world.contested_sites.remove(&siege.site);
            if siege.defending.is_empty() && !siege.besieging.is_empty() && hostile {
                combat::capture(campaign, data, siege.site, siege.besieger);
            } else if siege.defending.is_empty() && siege.besieging.is_empty() {
                campaign
                    .world
                    .sites
                    .iter_mut()
                    .find(|site| site.id == siege.site)
                    .expect("siege site")
                    .controller = None;
            }
            record_fact(
                campaign,
                outcome,
                DomainFactKind::SiegeChanged {
                    siege: old,
                    change: SiegeChange::Lifted,
                },
            )?;
        } else if siege != old {
            campaign.sieges.insert(siege.site, siege.clone());
            record_fact(
                campaign,
                outcome,
                DomainFactKind::SiegeChanged {
                    siege,
                    change: SiegeChange::Reinforced,
                },
            )?;
        }
    }
    campaign.reconcile_region_control();
    Ok(())
}

pub(super) fn progress(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    for site in campaign.sieges.keys().copied().collect::<Vec<_>>() {
        let siege = campaign.sieges.get_mut(&site).expect("known siege");
        if siege.last_progress_round == Some(campaign.completed_rounds) {
            continue;
        }
        siege.elapsed_steps = siege
            .elapsed_steps
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "siege elapsed steps",
            })?;
        siege.last_progress_round = Some(campaign.completed_rounds);
        let receipt = siege.clone();
        let damage = campaign.world.fort_damage.entry(site).or_default();
        *damage = damage
            .saturating_add(data.siege.fort_damage_per_step)
            .min(100);
        record_fact(
            campaign,
            outcome,
            DomainFactKind::SiegeChanged {
                siege: receipt,
                change: SiegeChange::Progressed,
            },
        )?;
    }
    Ok(())
}

/// Reconcile an already resolved diplomacy/control candidate. The caller retains
/// the original pre-diplomacy campaign if this candidate has no legal withdrawal.
pub fn reconcile_sieges(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<ActionOutcome, RuleError> {
    let mut candidate = campaign.clone();
    candidate.accepted_sequence =
        candidate
            .accepted_sequence
            .checked_add(1)
            .ok_or(RuleError::Overflow {
                field: "accepted action sequence",
            })?;
    let mut outcome = ActionOutcome {
        battle: None,
        accepted_sequence: candidate.accepted_sequence,
        active_faction: candidate.active_faction(),
        round_completed: false,
        facts: Vec::new(),
        consumed_facts: Vec::new(),
        recruited: None,
        disbanded: None,
        movement: None,
        split_army: None,
    };
    reconcile(&mut candidate, data, &mut outcome)?;
    super::construction::reconcile(&mut candidate, data, &mut outcome)?;
    super::history::record_facts(&mut candidate, campaign, &outcome.facts)?;
    candidate.validate(data).map_err(RuleError::InvalidState)?;
    *campaign = candidate;
    Ok(outcome)
}

fn withdraw_peace(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    siege: &mut Siege,
) -> Result<(), RuleError> {
    if siege.besieging.is_empty() {
        return Ok(());
    }
    let destination = retreat::destination(campaign, siege.besieger, siege.site, None, None)
        .ok_or_else(|| {
            RuleError::Siege("Forces must withdraw first: no legal peace withdrawal exists.".into())
        })?;
    for id in &siege.besieging {
        campaign.armies.get_mut(id).expect("besieger").site = destination;
    }
    exhaust(campaign, data, &siege.besieging);
    siege.besieging.clear();
    Ok(())
}
