//! Formation conversions are unlocked only by the formation's own service.

use crate::{
    data::{
        progression::{FormationSpecialization, SpecializationRequirement},
        world::{Facility, SiteId},
        GameData,
    },
    engine::RuleError,
    state::{military::FormationId, StrategicCampaign},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecializationOption {
    pub specialization: FormationSpecialization,
    pub eligible: bool,
    pub current: usize,
    pub required: usize,
    pub gold_cost: i64,
    pub secondary_current: Option<usize>,
    pub secondary_required: Option<usize>,
}

pub fn specialization_options(
    campaign: &StrategicCampaign,
    data: &GameData,
    formation: FormationId,
) -> Result<Vec<SpecializationOption>, RuleError> {
    let formation = campaign
        .formations
        .get(&formation)
        .ok_or(RuleError::UnknownFormation { formation })?;
    Ok(data
        .progression
        .specializations
        .iter()
        .map(|(kind, rule)| {
            let (current, required, secondary_current, secondary_required) = match rule.requirement
            {
                SpecializationRequirement::DefendedAnchors { occasions } => (
                    formation
                        .service
                        .ledger
                        .counts
                        .get(&crate::state::evidence::EvidenceKind::DefendedAnchor)
                        .copied()
                        .unwrap_or(0) as usize,
                    occasions as usize,
                    None,
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
                    None,
                ),
                SpecializationRequirement::RoutesAndRetreatingVictories { routes, victories } => (
                    formation.service.ledger.traversed_routes.len(),
                    routes,
                    Some(
                        formation
                            .service
                            .ledger
                            .counts
                            .get(&crate::state::evidence::EvidenceKind::RetreatingEnemyVictory)
                            .copied()
                            .unwrap_or(0) as usize,
                    ),
                    Some(victories as usize),
                ),
            };
            SpecializationOption {
                specialization: *kind,
                eligible: rule.sources.contains(&formation.kind)
                    && formation.service.specialization.is_none()
                    && formation.service.course.is_none()
                    && current >= required
                    && secondary_current
                        .zip(secondary_required)
                        .is_none_or(|(actual, expected)| actual >= expected),
                current,
                required,
                gold_cost: rule.gold_cost,
                secondary_current,
                secondary_required,
            }
        })
        .collect())
}

pub(super) fn course_site(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    site: SiteId,
    facility: Facility,
) -> bool {
    campaign.world.site(site).is_some_and(|entry| {
        entry.controller == Some(owner)
            && entry.facilities.contains(&facility)
            && campaign.world.structural_damage(site) == 0
            && campaign.supplied_sites(owner).contains(&site)
            && !campaign.sieges.contains_key(&site)
            && data
                .scenario
                .sites
                .iter()
                .any(|authored| authored.id == site)
    })
}
