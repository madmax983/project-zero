//! Layer 3: Galactic Level Simulation
//!
//! This layer handles galaxy-wide simulation aspects, connecting solar systems and factions.
//! This includes the galactic market, council resolutions, diplomacy between interstellar nations,
//! and large-scale fleets.
pub mod council;
pub mod events;
pub mod market;
pub mod resources;
pub mod silence;
pub use council::*;
pub mod diplomacy;
pub use diplomacy::*;
pub mod bureaucracy_of_truth;
pub mod bureaucracy_of_vanity;
pub mod digital_detritus;
pub mod diplomacy_reflection;
pub mod fleets;
pub mod ghost_ships;
pub mod integration;
pub mod linguistic_drift;
pub mod map;
pub mod physics;
pub mod pirates;
pub mod planet;
pub mod stellar_cartography;
pub mod subjective_economics;
pub mod treaty_cruisers;
pub use bureaucracy_of_vanity::*;
pub use digital_detritus::*;
pub mod bureaucracy;
pub mod economy;
pub mod intellectual_property_wars;

pub mod auditor;
pub mod guilt;
pub mod zoo_hypothesis;

pub mod galaxy;
pub mod sovereign_armada;
pub mod quarantine;
