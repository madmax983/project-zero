//! Layer 1: Colony Simulation.
//!
//! This layer handles individual pops, buildings, and tile-based terrain
//! similar to Dwarf Fortress or `RimWorld`.

/// Pop actions logic.
pub mod actions;
/// Game balance constants.
pub mod balance;
/// Beauty and decoration system.
pub mod beauty;
/// Building placement and types.
pub mod building;
pub mod chronicle;
/// Chronicle system and historical records.
pub mod clothing;
/// Designation system for player tools.
pub mod designation;
/// Execution layer bridging utility AI to actions.
pub mod execution;
/// Farm building and food production.
pub mod farm;
/// Fire propagation and damage.
pub mod fire;
/// Hauling logic.
pub mod hauling;
#[cfg(test)]
/// Tests for hazards logic.
pub mod hazards_tests;
/// Pop health and damage.
pub mod health;
/// Housing and rest mechanics.
pub mod housing;
/// Integration systems bridging domains.
pub mod integration;
/// Spatial primitives (GridPosition).
pub mod map;
/// Pop memories and psychological effects.
pub mod memory;
/// Medical care and hospital logic.
pub mod medical;
#[cfg(test)]
/// Tests for metal industry (Spec 024).
pub mod metal_industry_tests;
/// Pop needs (hunger, rest).
pub mod needs;
/// Notification system.
pub mod notifications;
/// Pop entity and management.
pub mod pop;
/// Refining industry (Lumber Mill, Stone Mason).
pub mod refining;
/// Colony resources and mining.
pub mod resources;
/// Field science and anomalies.
pub mod science;
/// Seasonal rhythms (Spring, Summer, Autumn, Winter).
pub mod seasons;
/// Social needs and tavern.
pub mod social;
/// Spoilage and decay mechanics.
pub mod spoilage;
/// Resource storage limits and stockpile buildings.
pub mod stockpile;
/// Structure durability and repair.
pub mod structure;
/// Technology and research system.
pub mod tech;
/// Terrain generation and grid management.
pub mod terrain;
/// Emergent utility AI system.
pub mod utility_ai;
#[cfg(test)]
/// Tests for utility AI work logic.
pub mod utility_ai_work_tests;

#[cfg(test)]
mod execution_demolish_test;

/// Named locations on the map.
pub mod locations;

pub use actions::*;
pub use balance::*;
pub use beauty::*;
pub use building::*;
pub use chronicle::*;
pub use designation::*;
pub use execution::*;
pub use farm::*;
pub use fire::*;
pub use hauling::*;
pub use health::*;
pub use housing::*;
pub use integration::*;
pub use locations::*;
pub use map::*;
pub use medical::*;
pub use memory::*;
pub use needs::*;
pub use notifications::*;
pub use pop::*;
pub use refining::*;
pub use resources::*;
pub use science::*;
pub use seasons::*;
pub use social::*;
pub use spoilage::*;
pub use stockpile::*;
pub use structure::*;
pub use tech::*;
pub use terrain::*;
pub use utility_ai::*;

#[cfg(test)]
mod tool_tests;
pub use clothing::*;

#[cfg(test)]
mod waste_tests;
