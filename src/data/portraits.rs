//! Stable portrait feature IDs and the semantic catalog used by allocation.

mod catalog;
mod schema;

pub(crate) use catalog::PortraitCandidateSpace;
pub use catalog::{PortraitCatalog, PortraitCatalogIndex, PORTRAIT_CATALOG_SOURCE};
pub use schema::{
    AppearanceDescriptor, AppearanceSignature, EyeFeature, EyeFitAssets, FaceFeature,
    GeometryTuple, HairFeature, HairFitAssets, MaskedArtAssets, NoseFeature, PaletteRamp,
    PortraitRig, ShadedArtAssets, SupportingArtAssets, APPEARANCE_SCHEMA_VERSION,
    BALD_HAIR_PALETTE_ID, PORTRAIT_CATALOG_SCHEMA_VERSION,
};
