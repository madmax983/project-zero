//! Layer 1: Colony Simulation.
//!
//! This layer handles individual pops, buildings, and tile-based terrain
//! similar to Dwarf Fortress or `RimWorld`.

/// Terrain generation and grid management.
pub mod terrain;
pub use terrain::*;

/// Pop entity management.
pub mod pop;
pub use pop::*;
