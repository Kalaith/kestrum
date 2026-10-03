//! Campaign-owned reservations keep exact portrait signatures unavailable across pruning.

mod migration;

pub use migration::migrate_legacy;

use crate::{
    data::portraits::{AppearanceDescriptor, AppearanceSignature, PortraitCatalog},
    state::people::PersonId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const APPEARANCE_REGISTRY_SCHEMA_VERSION: u32 = 1;
const APPEARANCE_SEED_NAMESPACE: u64 = 0x4B45_5354_5255_4D50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceRegistry {
    pub schema_version: u32,
    pub appearance_salt: u64,
    pub catalog_revision: u32,
    pub allocation_revision: u32,
    pub reservations: BTreeMap<String, AppearanceReservation>,
    pub fallback_usage: AppearanceFallbackUsage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceReservation {
    /// Duplicated in the key so a malformed or altered map entry is rejected.
    pub signature: String,
    pub signature_parts: AppearanceSignature,
    pub first_person: PersonId,
    pub first_round: u32,
    pub last_person: PersonId,
    pub last_round: u32,
    pub reuse_count: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceFallbackUsage {
    pub near_duplicate_relaxed: u64,
    pub deceased_signature_reuse: u64,
    pub living_signature_reuse: u64,
}

impl AppearanceRegistry {
    pub fn for_campaign_seed(seed: u64, catalog_revision: u32, allocation_revision: u32) -> Self {
        Self {
            schema_version: APPEARANCE_REGISTRY_SCHEMA_VERSION,
            appearance_salt: derive_appearance_salt(seed),
            catalog_revision,
            allocation_revision,
            reservations: BTreeMap::new(),
            fallback_usage: AppearanceFallbackUsage::default(),
        }
    }

    pub fn validate(&self, seed: u64, catalog: &PortraitCatalog) -> Result<(), String> {
        let index = catalog.validation_index()?;
        self.validate_with_index(seed, catalog, &index)
    }

    pub(crate) fn validate_with_index(
        &self,
        seed: u64,
        catalog: &PortraitCatalog,
        index: &crate::data::portraits::PortraitCatalogIndex<'_>,
    ) -> Result<(), String> {
        if self.schema_version != APPEARANCE_REGISTRY_SCHEMA_VERSION {
            return Err("appearance_registry: unsupported schema version".into());
        }
        if self.appearance_salt != derive_appearance_salt(seed) {
            return Err("appearance_registry: salt does not match the campaign seed".into());
        }
        if !catalog
            .supported_catalog_revisions
            .contains(&self.catalog_revision)
            || !catalog
                .supported_allocation_revisions
                .contains(&self.allocation_revision)
        {
            return Err("appearance_registry: unsupported saved allocator revision".into());
        }
        let finite_space = catalog.candidate_count()?;
        self.validate_content_with_index(catalog, index, finite_space)
    }

    pub(crate) fn validate_content_with_index(
        &self,
        catalog: &PortraitCatalog,
        index: &crate::data::portraits::PortraitCatalogIndex<'_>,
        finite_space: usize,
    ) -> Result<(), String> {
        if self.schema_version != APPEARANCE_REGISTRY_SCHEMA_VERSION {
            return Err("appearance_registry: unsupported schema version".into());
        }
        if !catalog
            .supported_catalog_revisions
            .contains(&self.catalog_revision)
            || !catalog
                .supported_allocation_revisions
                .contains(&self.allocation_revision)
        {
            return Err("appearance_registry: unsupported saved allocator revision".into());
        }
        if self.reservations.len() > finite_space {
            return Err(
                "appearance_registry: reservation count exceeds finite catalog space".into(),
            );
        }
        let mut repeat_allocations = 0_u64;
        let mut total_allocations = 0_u64;
        for (key, reservation) in &self.reservations {
            if key.is_empty()
                || key != &reservation.signature
                || key != &reservation.signature_parts.canonical_key()
                || reservation.first_person.0 == 0
                || reservation.last_person.0 == 0
                || reservation.reuse_count == 0
                || reservation.first_round > reservation.last_round
            {
                return Err("appearance_registry: invalid signature reservation".into());
            }
            repeat_allocations = repeat_allocations
                .checked_add(u64::from(reservation.reuse_count - 1))
                .ok_or_else(|| "appearance_registry: reuse total overflow".to_owned())?;
            total_allocations = total_allocations
                .checked_add(u64::from(reservation.reuse_count))
                .ok_or_else(|| "appearance_registry: allocation total overflow".to_owned())?;
            index.validate_signature(&reservation.signature_parts)?;
        }
        let recorded_reuses = self
            .fallback_usage
            .deceased_signature_reuse
            .checked_add(self.fallback_usage.living_signature_reuse)
            .ok_or_else(|| "appearance_registry: fallback total overflow".to_owned())?;
        if repeat_allocations != recorded_reuses {
            return Err("appearance_registry: reuse counters do not match reservations".into());
        }
        if self.fallback_usage.near_duplicate_relaxed > total_allocations {
            return Err("appearance_registry: near-duplicate counter exceeds allocations".into());
        }
        Ok(())
    }

    /// Explicitly move future allocations to a supported catalog revision.
    /// Existing descriptors and reservation keys remain unchanged.
    pub fn transition_to_catalog(&mut self, catalog: &PortraitCatalog) -> Result<(), String> {
        if !catalog
            .supported_catalog_revisions
            .contains(&self.catalog_revision)
            || !catalog
                .supported_allocation_revisions
                .contains(&self.allocation_revision)
        {
            return Err("appearance_registry: catalog cannot preserve saved revisions".into());
        }
        let index = catalog.validation_index()?;
        for reservation in self.reservations.values() {
            index.validate_signature(&reservation.signature_parts)?;
        }
        self.catalog_revision = catalog.catalog_revision;
        self.allocation_revision = catalog.allocation_revision;
        Ok(())
    }

    pub fn has_signature(&self, signature: &str) -> bool {
        self.reservations.contains_key(signature)
    }

    pub(crate) fn record(
        &mut self,
        appearance: &AppearanceDescriptor,
        signature_parts: &AppearanceSignature,
        person: PersonId,
        round: u32,
        fallback: RegistryFallback,
    ) -> Result<(), String> {
        if person.0 == 0 || appearance.signature.is_empty() {
            return Err("appearance_registry: cannot reserve an invalid identity".into());
        }
        let signature = appearance.signature.clone();
        let next_reuse_count = match self.reservations.get(&signature) {
            Some(_) if fallback == RegistryFallback::None => {
                return Err("appearance_registry: exact signature is already reserved".into());
            }
            Some(_) if fallback == RegistryFallback::NearDuplicate => {
                return Err(
                    "appearance_registry: near-duplicate fallback requires an unused signature"
                        .into(),
                );
            }
            Some(record) => Some(
                record
                    .reuse_count
                    .checked_add(1)
                    .ok_or_else(|| "appearance_registry: reuse count overflow".to_owned())?,
            ),
            None if matches!(
                fallback,
                RegistryFallback::DeceasedReuse | RegistryFallback::LivingReuse
            ) =>
            {
                return Err(
                    "appearance_registry: reuse fallback requires a reserved signature".into(),
                );
            }
            None => None,
        };
        let mut next_fallback_usage = self.fallback_usage.clone();
        match fallback {
            RegistryFallback::None => {}
            RegistryFallback::NearDuplicate => {
                next_fallback_usage.near_duplicate_relaxed = next_fallback_usage
                    .near_duplicate_relaxed
                    .checked_add(1)
                    .ok_or_else(|| {
                        "appearance_registry: near-duplicate counter overflow".to_owned()
                    })?;
            }
            RegistryFallback::DeceasedReuse => {
                next_fallback_usage.deceased_signature_reuse = next_fallback_usage
                    .deceased_signature_reuse
                    .checked_add(1)
                    .ok_or_else(|| {
                        "appearance_registry: deceased-reuse counter overflow".to_owned()
                    })?;
            }
            RegistryFallback::LivingReuse => {
                next_fallback_usage.living_signature_reuse = next_fallback_usage
                    .living_signature_reuse
                    .checked_add(1)
                    .ok_or_else(|| {
                        "appearance_registry: living-reuse counter overflow".to_owned()
                    })?;
            }
        }
        if let Some(reuse_count) = next_reuse_count {
            let Some(record) = self.reservations.get_mut(&signature) else {
                return Err("appearance_registry: reservation disappeared during update".into());
            };
            record.last_person = person;
            record.last_round = round.max(record.last_round);
            record.reuse_count = reuse_count;
        } else {
            self.reservations.insert(
                signature.clone(),
                AppearanceReservation {
                    signature,
                    signature_parts: signature_parts.clone(),
                    first_person: person,
                    first_round: round,
                    last_person: person,
                    last_round: round,
                    reuse_count: 1,
                },
            );
        }
        self.fallback_usage = next_fallback_usage;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistryFallback {
    None,
    NearDuplicate,
    DeceasedReuse,
    LivingReuse,
}

pub fn derive_appearance_salt(seed: u64) -> u64 {
    splitmix64(seed ^ APPEARANCE_SEED_NAMESPACE)
}

pub(crate) fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
