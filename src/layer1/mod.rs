//! Layer 1: Colony Simulation.
//!
//! This layer handles individual pops, buildings, and tile-based terrain
//! similar to Dwarf Fortress or `RimWorld`.

/// Pop entity and management.
pub mod pop;
/// Terrain generation and grid management.
pub mod terrain;

pub use pop::*;
pub use terrain::*;
