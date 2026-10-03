//! Shared appearance allocation for test fixtures with access to game data.
use kestrum::{
    data::portraits::{AppearanceDescriptor, PortraitCatalog},
    engine::portraits::allocate_for_person,
    state::{people::PersonId, StrategicCampaign},
};

pub(super) fn allocate(
    campaign: &mut StrategicCampaign,
    catalog: &PortraitCatalog,
    person: PersonId,
) -> AppearanceDescriptor {
    allocate_for_person(campaign, catalog, person)
        .unwrap_or_else(|error| panic!("allocate fixture appearance for {person:?}: {error}"))
}
