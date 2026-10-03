//! Deterministic campaign portrait assignment, independent of gameplay RNG.

mod validation;

pub use crate::state::appearance::migrate_legacy;
pub use validation::validate_campaign;

use crate::{
    data::portraits::{
        AppearanceDescriptor, PortraitCandidateSpace, PortraitCatalog, PortraitCatalogIndex,
    },
    state::{
        appearance::{derive_appearance_salt, splitmix64, AppearanceRegistry, RegistryFallback},
        campaign::StrategicCampaign,
        people::{PersonId, PersonStatus},
    },
};
use std::collections::BTreeSet;

const PREFERRED_SAMPLE_LIMIT: usize = 256;
const ALLOCATION_STEP_NAMESPACE: u64 = 0x5052_5452_414C_4C4F;

/// Assign the stable identity for a newly accepted person, without touching any game RNG stream.
pub fn allocate_for_person(
    campaign: &mut StrategicCampaign,
    catalog: &PortraitCatalog,
    id: PersonId,
) -> Result<AppearanceDescriptor, String> {
    if id.0 == 0 || campaign.people.contains_key(&id) {
        return Err(format!(
            "portrait allocation: person ID {:?} is invalid or already present",
            id
        ));
    }
    if campaign.appearance_registry.appearance_salt != derive_appearance_salt(campaign.seed) {
        return Err("portrait allocation: appearance salt does not match campaign seed".into());
    }
    let living = campaign
        .people
        .values()
        .filter(|person| !matches!(person.status, PersonStatus::Dead { .. }))
        .map(|person| person.appearance.clone())
        .collect::<Vec<_>>();
    allocate_from_registry(
        &mut campaign.appearance_registry,
        catalog,
        id,
        campaign.completed_rounds,
        &living,
    )
}

/// Allocate an ID for migration or campaign creation from explicit, authorized living identities.
/// The caller owns deterministic ID ordering and supplies living appearances only.
pub fn allocate_from_registry(
    registry: &mut AppearanceRegistry,
    catalog: &PortraitCatalog,
    id: PersonId,
    round: u32,
    living: &[AppearanceDescriptor],
) -> Result<AppearanceDescriptor, String> {
    if id.0 == 0 {
        return Err("portrait allocation: person ID zero is reserved".into());
    }
    if registry.catalog_revision != catalog.catalog_revision
        || registry.allocation_revision != catalog.allocation_revision
    {
        return Err("portrait allocation: transition saved registry before allocating".into());
    }
    let index = catalog.validation_index()?;
    let space = catalog.candidate_space()?;
    registry.validate_content_with_index(catalog, &index, space.len())?;
    let living_signatures = living
        .iter()
        .map(|entry| entry.signature.as_str())
        .collect::<BTreeSet<_>>();
    let living_groups = living
        .iter()
        .map(|entry| {
            index.validate_descriptor(entry)?;
            index.distinctiveness(entry)
        })
        .collect::<Result<BTreeSet<_>, String>>()?;
    let count = space.len();
    let start = (splitmix64(registry.appearance_salt ^ u64::from(id.0)) % count as u64) as usize;
    let step_seed =
        splitmix64(registry.appearance_salt ^ (u64::from(id.0) << 32) ^ ALLOCATION_STEP_NAMESPACE);
    let step = coprime_step((step_seed % count as u64) as usize, count);

    for offset in 0..count.min(PREFERRED_SAMPLE_LIMIT) {
        let candidate_index = permuted_index(start, step, offset, count);
        let candidate = space.candidate_at(candidate_index)?;
        if registry.has_signature(&candidate.signature)
            || !preserves_preferred_distinction(&index, &candidate, &living_groups)?
        {
            continue;
        }
        let parts = index.signature_parts(&candidate)?;
        registry.record(&candidate, &parts, id, round, RegistryFallback::None)?;
        return Ok(candidate);
    }

    // The bounded preferred sample failed. Prove exact availability by traversing every signature.
    for offset in 0..count {
        let candidate_index = permuted_index(start, step, offset, count);
        let candidate = space.candidate_at(candidate_index)?;
        if registry.has_signature(&candidate.signature) {
            continue;
        }
        let parts = index.signature_parts(&candidate)?;
        registry.record(
            &candidate,
            &parts,
            id,
            round,
            RegistryFallback::NearDuplicate,
        )?;
        return Ok(candidate);
    }

    let (signature, fallback) = registry
        .reservations
        .iter()
        .filter(|(signature, _)| !living_signatures.contains(signature.as_str()))
        .min_by_key(|(signature, record)| {
            (record.last_round, record.last_person.0, signature.as_str())
        })
        .map(|(signature, _)| (signature.clone(), RegistryFallback::DeceasedReuse))
        .or_else(|| {
            registry
                .reservations
                .iter()
                .min_by_key(|(signature, record)| (record.reuse_count, signature.as_str()))
                .map(|(signature, _)| (signature.clone(), RegistryFallback::LivingReuse))
        })
        .ok_or_else(|| {
            "portrait allocation: finite signature space is exhausted without reservations"
                .to_owned()
        })?;
    let candidate = find_candidate(&space, &signature)?;
    let parts = index.signature_parts(&candidate)?;
    registry.record(&candidate, &parts, id, round, fallback)?;
    Ok(candidate)
}

fn preserves_preferred_distinction(
    index: &PortraitCatalogIndex<'_>,
    candidate: &AppearanceDescriptor,
    living: &BTreeSet<[String; 4]>,
) -> Result<bool, String> {
    let candidate_groups = index.distinctiveness(candidate)?;
    Ok(living.iter().all(|existing| {
        let differences = candidate_groups
            .iter()
            .zip(existing)
            .filter(|(left, right)| left != right)
            .count();
        let silhouette_differs =
            candidate_groups[0] != existing[0] || candidate_groups[1] != existing[1];
        differences >= 2 && silhouette_differs
    }))
}

fn find_candidate(
    space: &PortraitCandidateSpace<'_>,
    signature: &str,
) -> Result<AppearanceDescriptor, String> {
    for index in 0..space.len() {
        let candidate = space.candidate_at(index)?;
        if candidate.signature == signature {
            return Ok(candidate);
        }
    }
    Err("portrait allocation: reserved signature is absent from the active catalog".into())
}

fn coprime_step(seed: usize, count: usize) -> usize {
    if count == 1 {
        return 1;
    }
    let mut step = seed % count;
    if step == 0 {
        step = 1;
    }
    while greatest_common_divisor(step, count) != 1 {
        step += 1;
        if step == count {
            step = 1;
        }
    }
    step
}

fn greatest_common_divisor(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn permuted_index(start: usize, step: usize, offset: usize, count: usize) -> usize {
    ((start as u128 + step as u128 * offset as u128) % count as u128) as usize
}
