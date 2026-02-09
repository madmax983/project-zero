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
/// Colony policies and edicts.
pub mod edicts;
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
/// Item definitions (Tools, Equipment).
pub mod items;
/// Spatial primitives (GridPosition).
pub mod map;
/// Medical care and hospital logic.
pub mod medical;
/// Pop memories and psychological effects.
pub mod memory;
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
pub use edicts::*;
pub use execution::*;
pub use farm::*;
pub use fire::*;
pub use hauling::*;
pub use health::*;
pub use housing::*;
pub use integration::*;
pub use items::*;
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

/// Lighting system.
pub mod lighting;
/// Rumor web system (Spec 055).
pub mod rumor;
/// Pop skills and experience system.
pub mod skills;
#[cfg(test)]
mod waste_tests;

pub use lighting::*;
pub use rumor::*;

/// Designated zones logic (Spec 056).
pub mod zone;
pub use zone::*;

/// Funeral rites and corpse management (Spec 057).
pub mod funeral;
pub use funeral::*;

/// Acoustic simulation (Spec 060).
pub mod acoustic;
pub use acoustic::*;

/// Trade system.
pub mod trade;
pub use trade::*;
/// Energy system (Spec 042).
pub mod energy;
pub use energy::*;

/// Pop lifecycle and aging (Spec 062).
pub mod lifecycle;
pub use lifecycle::*;
/// Atmospheric simulation (Spec 063).
pub mod atmosphere;
/// Civil unrest and mental break system.
pub mod unrest;
pub use atmosphere::*;

/// Room quality calculation and memories (Spec 064).
pub mod room_quality;
pub use room_quality::*;

/// Hostile Fauna (Spec 048).
pub mod fauna;
pub use fauna::*;
pub mod art;
pub use art::*;
