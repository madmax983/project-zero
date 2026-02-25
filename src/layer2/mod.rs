//! Layer 2: System Simulation.
//!
//! This layer handles the planetary system view, including orbital bodies,
//! fleet movement, and system-level resources.

/// Space barnacles logic.
pub mod barnacles;
/// Combat resolution logic.
pub mod combat;
/// Orbital debris mechanics.
pub mod debris;
/// Layer 2 events.
pub mod events;
/// Fleet movement and management.
pub mod fleet;
/// Procedural generation for the system.
pub mod generation;
/// Mining and resource extraction.
pub mod mining;
/// Rendering logic for the system view.
pub mod render;
/// Ship definitions and stats.
pub mod ship;
/// Orbital stations.
pub mod station;
/// Core system simulation components and resources.
pub mod system;
/// System visibility logic (Command Center).
pub mod visibility;

#[cfg(test)]
mod debris_tests;
#[cfg(test)]
mod station_tests;
