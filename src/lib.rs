//! The core library for SCALE.
//!
//! This library exposes the simulation layers and UI components of the game.

/// Experimental features.
pub mod experimental;
pub mod layer1;
/// Platform abstraction for native/WASM backends.
pub mod platform;
/// Shared world setup.
pub mod setup;
pub mod shared;
/// Shared simulation tick logic.
pub mod simulation;
pub mod ui;

pub use shared::selection::{Selection, SelectionTarget, inspect_entity, inspect_tile};
pub use shared::state::GameState;
