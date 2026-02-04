//! The core library for SCALE (Simulated Colony and Life Engine).
//!
//! This library exposes the three primary architectural pillars of the game:
//!
//! 1. **Layer 1 (Colony Simulation)**: The "Dwarf Fortress" layer. It handles individual
//!    pops, buildings, terrain, and needs. See [`layer1`].
//! 2. **Shared Systems**: Common utilities, state management, and input handling that
//!    bridge the gap between simulation and UI. See [`shared`].
//! 3. **UI (User Interface)**: The terminal-based rendering logic using `ratatui`.
//!    See [`ui`].
//!
//! # Architecture
//!
//! SCALE follows a strict Model-View pattern:
//! * **Model**: `layer1` contains the simulation state and systems.
//! * **View**: `ui` (and `main.rs`) renders the state to the terminal.
//! * **Controller**: `shared::input` routes user actions to state changes.

pub mod layer1;
pub mod shared;
pub mod ui;

pub use shared::selection::{Selection, SelectionTarget, inspect_entity, inspect_tile};
pub use shared::state::GameState;
