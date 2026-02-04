//! Layer 1: Colony Simulation.
//!
//! This layer handles individual pops, buildings, and tile-based terrain
//! similar to Dwarf Fortress or `RimWorld`.

/// Building placement and types.
pub mod building;
/// Chronicle system and historical records.
pub mod chronicle;
/// Designation system for player tools.
pub mod designation;
/// Farm building and food production.
pub mod farm;
/// Housing and rest mechanics.
pub mod housing;
/// Pop needs (hunger, rest).
pub mod needs;
/// Pop entity and management.
pub mod pop;
/// Colony resources and mining.
pub mod resources;
/// Resource storage limits and stockpile buildings.
pub mod stockpile;
/// Terrain generation and grid management.
pub mod terrain;

pub use building::*;
pub use chronicle::*;
pub use designation::*;
pub use farm::*;
pub use housing::*;
pub use needs::*;
pub use pop::*;
pub use resources::*;
pub use stockpile::*;
pub use terrain::*;
