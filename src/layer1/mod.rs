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
/// Execution layer bridging utility AI to actions.
pub mod execution;
/// Farm building and food production.
pub mod farm;
/// Housing and rest mechanics.
pub mod housing;
/// Named locations on the map.
pub mod locations;
/// Spatial primitives (GridPosition).
pub mod map;
/// Pop needs (hunger, rest).
pub mod needs;
/// Pop entity and management.
pub mod pop;
/// Refining industry (Lumber Mill, Stone Mason).
pub mod refining;
/// Colony resources and mining.
pub mod resources;
/// Seasonal rhythms (Spring, Summer, Autumn, Winter).
pub mod seasons;
/// Social needs and tavern.
pub mod social;
/// Resource storage limits and stockpile buildings.
pub mod stockpile;
/// Terrain generation and grid management.
pub mod terrain;
/// Pop thoughts and personality.
pub mod thoughts;
/// Emergent utility AI system.
pub mod utility_ai;
#[cfg(test)]
/// Tests for utility AI work logic.
pub mod utility_ai_work_tests;

pub use building::*;
pub use chronicle::*;
pub use designation::*;
pub use execution::*;
pub use farm::*;
pub use housing::*;
pub use locations::*;
pub use map::*;
pub use needs::*;
pub use pop::*;
pub use refining::*;
pub use resources::*;
pub use seasons::*;
pub use social::*;
pub use stockpile::*;
pub use terrain::*;
pub use thoughts::*;
pub use utility_ai::*;
