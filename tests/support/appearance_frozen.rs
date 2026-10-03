//! Shared appearance allocation for fixtures using the frozen legacy catalog.
use kestrum::{
    data::portraits::{AppearanceDescriptor, PortraitCatalog},
    engine::portraits::allocate_for_person,
    state::{people::PersonId, StrategicCampaign},
};

pub(super) fn allocate_frozen(
    campaign: &mut StrategicCampaign,
    person: PersonId,
) -> AppearanceDescriptor {
    let catalog = PortraitCatalog::load_frozen().expect("load frozen portrait catalog");
    allocate_for_person(campaign, &catalog, person)
        .unwrap_or_else(|error| panic!("allocate fixture appearance for {person:?}: {error}"))
}
