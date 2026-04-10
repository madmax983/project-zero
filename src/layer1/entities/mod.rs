//! The Living Colony: Entities and Inhabitants
//!
//! This module aggregates all living (and pseudo-living) entities within the simulation.
//! It defines the core data structures, behaviors, and spawning logic for the various
//! actors that populate the colony, from the standard human colonists (`Pop`) to hostile
//! incursions (`Blob`), utility machines (`Drone`), and native wildlife (`Fauna`).

pub mod blob;
pub mod drone;
#[cfg(test)]
mod drone_tests;
pub mod fauna_gen;
#[cfg(test)]
mod fauna_modular_tests;
pub mod mascot;
pub mod pop;
pub mod pop_doppelganger;
pub mod the_visitor;
pub mod vermin;
#[cfg(test)]
mod vermin_evolution_tests;
pub mod visitor;
pub mod wild_child;

pub use blob::*;
pub use drone::*;
pub use fauna_gen::*;
pub use mascot::*;
pub use pop::*;
pub use pop_doppelganger::*;
pub use the_visitor::*;
pub use vermin::*;
pub use visitor::*;
pub use wild_child::*;
