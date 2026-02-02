//! Layer 1: Colony Simulation.
//!
//! This layer handles individual pops, buildings, and tile-based terrain
//! similar to Dwarf Fortress or `RimWorld`.

/// Building placement and types.
pub mod building;
/// Farm building and food production.
pub mod farm;
/// Housing and rest mechanics.
pub mod housing;
/// Pop needs (hunger, rest).
pub mod needs;
/// Pop entity and management.
pub mod pop;
/// Terrain generation and grid management.
pub mod terrain;

pub use building::*;
pub use farm::*;
pub use housing::*;
pub use needs::*;
pub use pop::*;
pub use terrain::*;
