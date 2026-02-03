//! The core library for SCALE.
//!
//! This library exposes the simulation layers and UI components of the game.

pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod shared;
pub mod ui;

pub use shared::selection::{Selection, SelectionTarget, inspect_entity, inspect_tile};
pub use shared::state::GameState;
