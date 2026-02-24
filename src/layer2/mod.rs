//! Layer 2: System Simulation.
//!
//! This layer handles the planetary system view, including orbital bodies,
//! fleet movement, and system-level resources.

/// Procedural generation for the system.
pub mod generation;
/// Rendering logic for the system view.
pub mod render;
/// Core system simulation components and resources.
pub mod system;
/// System visibility logic (Command Center).
pub mod visibility;
/// Fleet movement and management.
pub mod fleet;
