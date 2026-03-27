//! The SCALE Prelude.
//!
//! This module provides easy access to commonly used types and functions in the SCALE library.
//! By adding `use scale::prelude::*;` to your code, you can avoid deep nested imports.

pub use crate::setup::{setup_world_with_config, SetupConfig};
pub use crate::shared::narrative::{NarrativeContext, NarrativeGenerator};
pub use crate::shared::time::SimulationTime;
pub use crate::simulation::run_simulation_tick;

#[cfg(feature = "nova")]
pub use crate::layer1::oral_tradition::{OralTradition, Story, StoryGenre};
