//! Building placement and types.
//!
//! Buildings are the primary structures in the colony. They provide housing,
//! production, defense, and social functions.
//!
//! # Core Systems
//!
//! *   **Placement:** Buildings are placed on the [`crate::layer1::terrain::TerrainGrid`] using [`try_place_building`].
//! *   **Cost:** Each [`BuildingType`] has a [`crate::layer1::resources::ColonyResources`] cost (see [`BuildingType::cost`]).
//! *   **Tech:** Some buildings require specific [`crate::layer1::tech::Tech`] to be unlocked (see [`BuildingType::required_tech`]).
//! *   **Obstacles:** Most buildings block movement, but some (like Farms/Stockpiles) are walkable.
//!
//! # Entities
//!
//! A built structure is an entity with:
//! *   [`Building`]: The marker component containing the [`BuildingType`].
//! *   [`crate::layer1::map::GridPosition`]: Its location on the map.
//! *   [`crate::layer1::structure::Structure`]: Health and durability.
//! *   Specific Logic Components: e.g., [`crate::layer1::housing::Housing`], [`crate::layer1::farm::Farm`], [`crate::layer1::stockpile::Stockpile`].

pub mod types;
pub mod building_components;
pub mod systems;
pub mod placement;

#[cfg(test)]
mod tests;

pub use types::*;
pub use building_components::*;
pub use systems::*;
pub use placement::*;
