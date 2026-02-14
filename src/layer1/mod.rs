//! Layer 1: Colony Simulation.
//!
//! This is the "Dwarf Fortress" or "RimWorld" layer of the game. It simulates the daily life
//! of the colony on a tile-based grid.
//!
//! # Core Concepts
//!
//! ## The Grid
//! The world is represented by a 2D grid of tiles (see [`map::GridPosition`] and [`terrain::TerrainGrid`]).
//! Each tile can contain:
//! - **Terrain:** The base layer (Grass, Water, Rock).
//! - **Building:** Constructed structures (Housing, Farm, Walls). See [`building`].
//! - **Entities:** Pops, Visitors, Fauna, and Items.
//!
//! ## The Agents (Pops)
//! "Pops" are the primary agents. They are not directly controlled by the player. Instead, they:
//! 1.  **Have Needs:** Hunger, Rest, Social, Leisure (see [`needs`]).
//! 2.  **Make Decisions:** Utility AI scores potential actions based on needs and environment (see [`utility_ai`]).
//! 3.  **Perform Actions:** Working, Eating, Sleeping, Socializing (see [`actions`]).
//!
//! ## The Simulation Loop
//! The simulation advances in discrete ticks (see [`crate::simulation`]).
//! 1.  **Decision Phase:** AI systems evaluate options and assign `PopAction`s.
//! 2.  **Execution Phase:** Systems like `movement_system` and `work_execution_system` progress these actions.
//! 3.  **Economy Phase:** Resources are produced/consumed, needs decay.
//!
//! # Module Structure
//! - **Entities:** [`pop`], [`building`], [`fauna`], [`visitor`]
//! - **Systems:** [`needs`], [`health`], [`combat`], [`tech`]
//! - **Environment:** [`terrain`], [`map`], [`weather`], [`lighting`]
//! - **Economy:** [`resources`], [`trade`], [`market`]

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
/// Defensive structures and logic.
pub mod defense;
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
/// Workplace hazards logic.
pub mod hazards;
#[cfg(test)]
/// Tests for hazards logic.
pub mod hazards_tests;
/// Pop health and damage.
pub mod health;
/// Housing and rest mechanics.
pub mod housing;
/// The Inspector system (Spec 091).
pub mod inspector;
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
/// Structural integrity (cave-ins and supports).
pub mod structural_integrity;
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
/// Shared types for utility AI.
pub mod utility_types;

#[cfg(test)]
/// Tests for mining logic (Spec 052).
pub mod mining_tests;

#[cfg(test)]
mod preservation_tests;

#[cfg(test)]
mod execution_demolish_test;

#[cfg(test)]
mod medical_triage_tests;

/// Named locations on the map.
pub mod locations;

pub use actions::*;
pub use balance::*;
pub use beauty::*;
pub use building::*;
pub use chronicle::*;
pub use defense::*;
pub use designation::*;
pub use edicts::*;
pub use execution::*;
pub use farm::*;
pub use fire::*;
pub use hauling::*;
pub use hazards::*;
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
pub use structural_integrity::*;
pub use structure::*;
pub use tech::*;
pub use terrain::*;
pub use utility_ai::*;
pub use utility_types::*;

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

/// Vermin infestation logic (Spec 073).
pub mod vermin;
pub use vermin::*;

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
pub mod combat;
/// Visitor system (Spec 074).
pub mod visitor;
pub use visitor::*;

/// Day/Night cycle system (Spec 065).
pub mod day_night;
pub use day_night::*;
pub mod justice;
pub use justice::*;

/// Planetary Quirks (Spec 080).
pub mod quirks;
pub use quirks::*;

/// Sleepwalking mechanics (Spec 081).
pub mod sleepwalking;
pub use sleepwalking::*;

#[cfg(test)]
mod quirks_tests;

#[cfg(test)]
mod sleepwalking_tests;

#[cfg(test)]
mod material_provenance_tests;

/// Pop factions system (Spec 068).
pub mod factions;
pub use factions::*;

/// Weather system (Spec 079).
pub mod weather;
pub use weather::*;

/// Cabin Fever mechanics (Spec 082).
pub mod cabin_fever;
pub use cabin_fever::*;
/// Mentorship system (Spec 069).
pub mod mentorship;
pub use mentorship::*;

/// Stowaway system (Spec 086).
pub mod stowaway;
pub use stowaway::*;
/// Pop personality traits (Spec 084).
pub mod traits;
pub use traits::*;
/// Omens & Taboos system (Spec 088).
pub mod taboo;
pub use taboo::*;
/// Erosion system (Spec 093).
pub mod erosion;
pub use erosion::*;
#[cfg(test)]
/// Tests for faction demands logic (Spec 085).
pub mod faction_demands_tests;

/// Heirloom tech system (Spec 070).
pub mod heirloom;
pub use heirloom::*;
#[cfg(test)]
mod heirloom_items_tests;
#[cfg(test)]
mod heirloom_tests;
#[cfg(test)]
mod structure_jury_rig_tests;

/// Animal Husbandry system (Spec 075).
pub mod husbandry;
pub use husbandry::*;

/// Water simulation (Spec 096).
pub mod water;
pub use water::*;

/// Antagonistic Flora system (Spec 092).
pub mod flora;
pub use flora::*;
pub mod private_stash;
pub use private_stash::*;
/// Resource purity system (Spec 106).
pub mod purity;
pub use purity::*;
/// Visual particle effects system (Juice).
pub mod particles;
pub use particles::*;

/// Emotional Contagion system (Spec 090).
pub mod contagion;
pub use contagion::*;

/// Morale system (Spec 031/090).
pub mod morale;
pub use morale::*;

/// Biocompatibility system (Spec 107).
pub mod biocompatibility;
pub use biocompatibility::*;

#[cfg(test)]
mod fuel_industry_tests;
#[cfg(test)]
mod greenhouse_tests;
mod structure_maintenance_tests;
#[cfg(test)]
mod structural_integrity_overflow_tests;
/// Palette fatigue system (Spec 114).
pub mod palette_fatigue;
pub use palette_fatigue::*;
