//! Scale - A 4X colony simulation game.
//!
//! This library crate contains the core simulation layers and shared utilities.

/// Layer 1: Colony-level simulation (tile grid, units).
pub mod layer1;
/// Layer 2: System-level simulation (planets, orbits).
pub mod layer2;
/// Layer 3: Galaxy-level simulation (stars, civs).
pub mod layer3;
/// Shared utilities and types.
pub mod shared;
/// User Interface components.
pub mod ui;
