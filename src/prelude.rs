//! The SCALE Prelude.
//!
//! This module provides easy access to commonly used types and functions in the SCALE library.
//! By adding `use scale::prelude::*;` to your code, you can avoid deep nested imports.
//!
//! ## Examples
//!
//! ```
//! use scale::prelude::*;
//!
//! // The prelude provides access to core simulation types like `World` and `SimulationTime`.
//! let config = SetupConfig {
//!     headless: true,
//!     ..Default::default()
//! };
//! let mut world = setup_world_with_config(config);
//!
//! assert_eq!(world.resource::<SimulationTime>().tick, 0);
//! ```

/// Core ECS types required for headless simulation and querying.
pub use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res, ResMut, World};

pub use crate::setup::{setup_world_with_config, SetupConfig};
pub use crate::shared::narrative::{NarrativeContext, NarrativeGenerator, NarrativeSegment};
pub use crate::shared::time::SimulationTime;
pub use crate::simulation::run_simulation_tick;

pub use crate::layer1::architecture::building;
pub use crate::layer1::architecture::building::Building;
pub use crate::layer1::biology::health;
pub use crate::layer1::biology::health::Health;
pub use crate::layer1::core::chronicle::{Chronicle, EventImportance};
pub use crate::layer1::entities::pop;
pub use crate::layer1::entities::pop::Pop;
pub use crate::layer1::psychology::needs::Needs;
pub use crate::layer1::social::Tavern;
pub use crate::shared::log::MessageLog;

#[cfg(feature = "nova")]
pub use crate::layer1::oral_tradition::{
    collect_chronicles_system, storytelling_system, OralTradition, Story, StoryGenre,
};
