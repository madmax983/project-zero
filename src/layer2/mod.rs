//! Layer 2: System Simulation.
//!
//! This layer handles the planetary system view, including orbital bodies,
//! fleet movement, and system-level resources.

/// Space barnacles logic.
pub mod barnacles;
/// Combat resolution logic.
pub mod combat;
/// Fleet movement and management.
pub mod fleet;
/// Procedural generation for the system.
pub mod generation;
/// Rendering logic for the system view.
pub mod render;
/// Ship definitions and stats.
pub mod ship;
/// Core system simulation components and resources.
pub mod system;
/// System visibility logic (Command Center).
pub mod visibility;
