//! Layer 1: Colony Simulation.
//!
//! This is the "Dwarf Fortress" or "RimWorld" layer of the game. It simulates the daily life
//! of the colony on a tile-based grid.
//!
//! # Core Concepts
//!
//! ## The Grid
//! The world is represented by a 2D grid of tiles (see [`crate::layer1::map::GridPosition`] and [`crate::layer1::terrain::TerrainGrid`]).
//! Each tile can contain:
//! - **Terrain:** The base layer (Grass, Water, Rock).
//! - **Building:** Constructed structures (Housing, Farm, Walls). See [`crate::layer1::building`].
//! - **Entities:** Pops, Visitors, Fauna, and Items.
//!
//! ## The Agents (Pops)
//! "Pops" are the primary agents. They are not directly controlled by the player. Instead, they:
//! 1.  **Have Needs:** Hunger, Rest, Social, Leisure (see [`crate::layer1::needs`]).
//! 2.  **Make Decisions:** Utility AI scores potential actions based on needs and environment (see [`crate::layer1::utility_ai`]).
//! 3.  **Perform Actions:** Working, Eating, Sleeping, Socializing (see [`crate::layer1::actions`]).
//!
//! ## The Simulation Loop
//! The simulation advances in discrete ticks (see [`crate::simulation`]).
//! 1.  **Decision Phase:** AI systems evaluate options and assign `PopAction`s.
//! 2.  **Execution Phase:** Systems like `movement_system` and `work_execution_system` progress these actions.
//! 3.  **Economy Phase:** Resources are produced/consumed, needs decay.
//!
//! # Module Structure
//! - **Entities:** [`crate::layer1::pop`], [`crate::layer1::building`], [`crate::layer1::fauna`], [`crate::layer1::visitor`]
//! - **Systems:** [`crate::layer1::needs`], [`crate::layer1::health`], [`crate::layer1::combat`], [`crate::layer1::tech`]
//! - **Environment:** [`crate::layer1::terrain`], [`crate::layer1::map`], [`crate::layer1::weather`], [`crate::layer1::lighting`]
//! - **Economy:** [`crate::layer1::resources`], [`crate::layer1::trade`]

pub mod entities;
pub use entities::*;
pub mod architecture;
pub use architecture::*;
pub mod economy;
pub use economy::*;
pub mod access_control;
/// Pop actions logic.
pub mod actions;
pub mod mind;
pub use mind::*;
pub mod ad_screen;
/// Bureaucratic Drag system (Spec 175).
pub mod administration;
pub use administration::*;
/// Game balance constants.
pub mod balance;
/// Beauty and decoration system.
pub mod beauty;
/// Pop biography system.
pub mod biography;
/// Cultural, Religious, and Belief systems.
pub mod culture;
pub use culture::*;
/// Building placement and types.
/// Chronicle system and historical records.
pub mod clothing;
/// Door control system (Spec 134).
/// Crowding system (Spec 176).
pub mod crowding;
/// Defensive structures and logic.
pub mod defense;
/// Designation system for player tools.
/// Pop dreams system.
pub mod dreams;
/// Ecological succession system (Spec 161).
/// Colony policies and edicts.
/// Execution layer bridging utility AI to actions.
pub mod execution;
/// Historical geography naming.
pub mod geography;
/// Farm building and food production.
/// Pop health and damage.
/// Deep crust geomes system (Spec 515).
/// Fire propagation and damage.
/// Hauling logic.
/// Workplace hazards logic.

/// Tests for hazards logic.
/// Pop hobbies logic (Spec 137).
pub mod hobby;

/// Housing and rest mechanics.
/// The Inspector system (Spec 091).
/// Institutional Memory system (Spec 172).
pub mod institutional_memory;
/// Personal inventory system.
/// Item definitions (Tools, Equipment).
pub mod law;
pub use law::orphaned_edict::*;
/// Colony Mascot system (Spec 129).
/// Medical care and hospital logic.
/// Pop memories and psychological effects.
pub mod memory;

pub mod jobs;
/// Tests for metal industry (Spec 024).
pub mod metal_industry_tests;
/// Terrain generation and grid management.
/// Tests for mining logic (Spec 052).
pub mod mining_tests;
/// Pop needs (hunger, rest).
pub mod needs;
/// Notification system.
pub mod notifications;
/// Pathfinding algorithms.
pub mod pathfinding;
/// Pop entity and management.
/// Refining industry (Lumber Mill, Stone Mason).
/// Colony resources and mining.
/// Field science and anomalies.
pub mod science;
/// Seasonal rhythms (Spring, Summer, Autumn, Winter).
mod shift_integration_tests;
/// Social needs and tavern.
pub mod social;
/// Spoilage and decay mechanics.
pub mod spoilage;
/// Resource storage limits and stockpile buildings.
/// Structure durability and repair.
/// Technology and research system.
pub mod tech;

/// Tests for mother lode logic (Spec 168).
pub mod mother_lode_tests;

mod execution_demolish_test;

/// Named locations on the map.
pub mod locations;

/// Graffiti and Signage system (Spec 144).
pub mod graffiti;

pub use access_control::*;
pub use actions::*;
pub use balance::*;
pub use beauty::*;
pub use biography::*;
pub use crowding::*;
pub use defense::*;
pub use dreams::*;

pub mod biology;
pub use biology::*;
pub use execution::*;

pub use graffiti::*;
pub use hobby::*;
pub use institutional_memory::*;
pub use jobs::*;
pub use locations::*;
pub use memory::*;
pub use needs::*;
pub use notifications::*;
pub use science::*;
pub use social::*;
pub use spoilage::*;
pub use tech::*;

mod tool_tests;
pub use clothing::*;

/// Lighting system.
pub mod lighting;
/// Pop skills and experience system.
pub mod skills;

mod waste_tests;

/// Vermin infestation logic (Spec 073).
pub use lighting::*;

/// Trade system.
/// Energy system (Spec 042).
pub mod energy;
pub use energy::*;

/// Pop lifecycle and aging (Spec 062).
pub mod lifecycle;
pub mod whispering_ore;
pub use lifecycle::*;
pub use whispering_ore::*;
/// Atmospheric simulation (Spec 063).
/// Terraforming and Planetary Atmosphere (Spec 207).

/// Room quality calculation and memories (Spec 064).
/// Radio Nostalgia system (Spec 814).
pub mod radio_nostalgia;
pub use radio_nostalgia::*;

/// Ruins system (Spec 167).
/// Hostile Fauna (Spec 048).
pub mod fauna;
pub use fauna::*;
/// Procedural Fauna Generation (Spec 164).
/// Combat system and drafting logic.
pub mod combat;
/// Visitor system (Spec 074).
/// The Visitor (Mega-Fauna) system (Spec 234).
/// Day/Night cycle system (Spec 065).
pub mod day_night;
pub use day_night::*;

/// Planetary Quirks (Spec 080).
pub mod quirks;
pub use quirks::*;

/// Sleepwalking mechanics (Spec 081).
pub mod sleepwalking;
pub use sleepwalking::*;

#[cfg(test)]
mod quirks_tests;

mod sleepwalking_tests;

mod material_provenance_tests;

/// Cabin Fever mechanics (Spec 082).
pub mod cabin_fever;
pub use cabin_fever::*;

/// Stowaway system (Spec 086).
pub mod stowaway;
pub use stowaway::*;
/// Pop personality traits (Spec 084).
pub mod traits;
pub use traits::*;
/// Erosion system (Spec 093).
/// Eureka Moments system (Spec 196).
pub mod eureka;
pub use eureka::*;

/// Pheromone Gardening system (Spec 170).
pub mod pheromone;
pub use pheromone::*;

/// Heirloom tech system (Spec 070).
pub mod heirloom;
pub use heirloom::*;

mod heirloom_items_tests;

mod heirloom_tests;

mod retrograde_tests;

/// Animal Husbandry system (Spec 075).
/// Gastronomy system (Spec 166).
/// Antagonistic Flora system (Spec 092).
pub mod flora;
pub use flora::*;
/// Gut Biome system (Spec 211).
/// Private stash system for pops.
pub mod private_stash;
pub use private_stash::*;
/// Resource purity system (Spec 106).
pub mod purity;
pub use purity::*;

mod fuel_consumption_tests;

mod fuel_industry_tests;

/// Palette fatigue system (Spec 114).
pub mod palette_fatigue;

pub use palette_fatigue::*;

/// Spontaneous Architecture system (Spec 110).
/// Observatory and Overview Effect (Spec 115).
pub mod observatory;
pub use observatory::*;

/// Conveyor and Hopper Logistics (Spec 111).
pub mod logistics;
pub use logistics::*;

/// Stress and mental breakdown system (Spec 127).
pub mod stress;
pub use stress::*;
/// Turret system (Spec 135).
/// Wild Child system (Spec 124).
pub mod prototyping;

mod tech_storage_tests;
pub use prototyping::*;
/// Cybernetic augmentation system (Spec 151).
pub mod psychic;
pub use psychic::*;
pub mod psionics;
pub use psionics::*;

pub mod geology;
pub use geology::*;

/// Oral Tradition system (Nova Feature).
#[cfg(feature = "nova")]
pub mod oral_tradition;
#[cfg(feature = "nova")]
pub use oral_tradition::*;

/// Genius Loci system (Nova Feature).
#[cfg(feature = "nova")]
pub mod loci;
#[cfg(feature = "nova")]
pub use loci::*;

/// Constellation Mythology system (Nova Feature).
#[cfg(feature = "nova")]
pub mod constellations;
#[cfg(feature = "nova")]
pub use constellations::*;

pub mod tech_envy;

/// Drone Networks (Spec 116).
/// The Observer Effect (Nova Feature).
#[cfg(feature = "nova")]
pub mod observer;
#[cfg(feature = "nova")]
pub use observer::*;

/// Machine Consciousness (Nova Feature).
#[cfg(feature = "nova")]
pub mod machine_consciousness;
#[cfg(feature = "nova")]
pub use machine_consciousness::*;

/// The Sleep-Deprived Savant system (Nova Feature).
#[cfg(feature = "nova")]
pub mod sleep_deprived_savant;
#[cfg(feature = "nova")]
pub use sleep_deprived_savant::*;

/// System registration and sets (Facade).
pub mod somnambulism;
pub use somnambulism::*;

pub mod systems;

mod institutional_memory_tests;
pub mod memetics;
/// Orbital Crossfire system (Spec 206).
/// Scrapcode virus system (Spec 178).
pub mod scrapcode;
pub use scrapcode::*;

/// Bio-Acoustic Chorus (Spec 419).

/// Chemical regulation system (Spec 181).
pub mod chemical;
/// Cryo-Stasis system (Spec 139).
pub mod cryo;
pub mod cryo_dreams;
pub use chemical::*;
pub use cryo::*;

#[cfg(test)]
mod cryo_tests;

/// Radioactive system (Spec 191).
/// Company Scrip and Economy system (Spec 194).
/// Language and Dialect system (Spec 193).
pub mod language;
pub use language::*;

mod equipment_tests;

/// Solar cycle and power generation (Spec 213).
/// Hygiene system (Spec 220).
pub mod hygiene;
pub use hygiene::*;

/// Gene Bank system (Spec 165).
pub mod geodetic;

mod improvised_tools_tests;

mod urban_heat_tests;
pub use geodetic::*;

/// The Mother Lode system (Spec 168).
pub mod mother_lode;
pub use mother_lode::*;

/// Organic Recycling system (Spec 221).
pub mod recycling;
pub use recycling::*;
/// Direct Link (Possession) system (Spec 236).
pub mod direct_link;
/// Permit system for advanced construction.
pub use direct_link::*;

/// Void Signals (Nova Feature).
#[cfg(feature = "nova")]
pub mod void_signals;
#[cfg(feature = "nova")]
pub use void_signals::*;
/// Volatile Resources system (Spec 223).

/// The Hum system (Spec 238).
pub mod hum;
pub use hum::*;

/// Photophobic resources system (Spec 237).

/// Operational Detritus system (Spec 239).
pub mod clutter;
pub use clutter::*;

/// Biometric Security system (Spec 244).
pub mod security;
pub use security::*;

mod geodetic_tests;

/// Void Stare (Spec 242).
pub mod void_stare;
pub use void_stare::*;

/// Quantum Twins (Spec 245).
pub mod quantum_twins;
pub use quantum_twins::*;

/// Holographic Facades (Spec 249).
pub mod construction;
pub mod hologram;

mod hologram_tests;
pub use hologram::*;

pub mod memory_core;
pub use memory_core::*;

/// Olfactory map and scent system (Spec 446).
pub mod olfactory;
pub use olfactory::*;

/// The Overview Effect system (Spec 449).
pub mod overview_effect;
pub use overview_effect::*;

/// Light Pollution system (Spec 450).

/// The Spiteful Will (Spec 451).
pub mod spiteful_will;
pub use spiteful_will::*;

pub mod physics;
pub use physics::*;

pub mod nature;
pub use nature::*;
pub mod shipbreaking;
pub use shipbreaking::*;
pub mod void_weed;
pub use void_weed::*;

pub mod temporal_ghost_towns;
pub use temporal_ghost_towns::*;

/// Infrastructure system (Spec 452).
pub mod infrastructure;
pub use infrastructure::*;

/// Nanite Fabrication system (Spec 453).
pub mod nanite_fabrication;
pub use nanite_fabrication::*;
pub mod exodus;
pub use exodus::*;
pub mod environment;
pub mod spore_diplomat;
pub use spore_diplomat::*;
/// Agriculture and Food Production
pub mod agriculture;
pub mod diplomacy;
pub use agriculture::*;
pub mod haunted_assembly_lines;
pub mod local_tributes;
pub mod pop_memories;
pub mod religion;
pub use pop_memories::*;
pub mod unseen_bureaucracy;
pub use unseen_bureaucracy::*;
pub mod core;
pub use core::*;
