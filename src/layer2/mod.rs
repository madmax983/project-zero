//! Layer 2: System/Planetary Simulation.
//!
//! This layer abstractly models the wider planetary and star system context surrounding the colony.
//!
//! Layer 2.
//!
//! Layer 2.
//!
pub mod barnacles;
pub mod binary_star;
pub mod combat;
pub mod debris;
#[cfg(test)]
mod debris_tests;
pub mod designation;
pub mod environment;
pub mod nebulae;
pub mod primitives;
pub use environment::*;
pub mod cartographers_curse;
pub mod cascade;
pub use cascade::*;
pub mod derelict_stations;
pub mod events;
pub mod events_new;
pub mod fleet;
pub mod generation;
pub mod governance;
pub mod integration;
pub mod mining;
#[cfg(test)]
mod mining_tests;
pub mod orphan_fleet;
pub mod phantom;
pub mod render;
pub mod shielding;
pub mod ship;
pub mod silent_mutiny;
pub mod station;
#[cfg(test)]
mod station_tests;
pub mod system;
pub mod syzygy;
pub mod thermal;
#[cfg(test)]
mod thermal_bloom_tests;
pub mod tourism;
pub mod trade;
pub mod visibility;
pub use cartographers_curse::*;
pub mod empathic_plague;
pub mod exploration;
pub mod moon_hermits;
pub mod mutiny;
pub mod navigation;
pub mod sensor_ambiguity;
pub mod skyhooks;

pub mod megastructure;
pub mod rogue_planets;
pub mod weather;
pub use weather::*;
pub mod bombardment;
pub mod orbit;
pub mod orbital_necropolis;

pub mod orbital_mirrors;
pub use orbital_mirrors::*;
pub mod culture;
pub mod sensors;
