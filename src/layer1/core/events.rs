//! Core building lifecycle events.
//!
//! # Context
//! This module defines the essential events emitted when buildings are constructed or destroyed
//! within the simulation grid. These events are integral to systems reacting to structural changes,
//! such as updating navigation meshes, adjusting resource outputs, or generating historical chronicles.
//!
//! # Usage
//! ```rust
//! use bevy_ecs::prelude::*;
//! use scale::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};
//! use scale::layer1::building::BuildingType;
//! use scale::layer1::map::GridPosition;
//!
//! let mut world = World::new();
//! world.init_resource::<Events<BuildingCompletedEvent>>();
//! let mut events = world.resource_mut::<Events<BuildingCompletedEvent>>();
//!
//! // Emitting a construction event
//! events.send(BuildingCompletedEvent {
//!     entity: Entity::from_raw(1),
//! });
//! ```
//!
//! # Details
//! * [`BuildingCompletedEvent`] is triggered *after* a building entity has been fully initialized and placed in the world.
//! * [`BuildingRemovedEvent`] must be emitted *before* the entity is despawned, capturing its final state and position.
//!
//! # Links
//! - [`BuildingCompletedEvent`]
//! - [`BuildingRemovedEvent`]

use crate::layer1::building::BuildingType;
use crate::layer1::core::map::GridPosition;
use bevy_ecs::prelude::*;

/// Event triggered when a building is removed (demolished or destroyed).
#[derive(Event, Debug, Clone)]
pub struct BuildingRemovedEvent {
    /// The entity ID of the removed building (before despawn).
    pub entity: Entity,
    /// The position where the building was located.
    pub position: GridPosition,
    /// The type of the building removed.
    pub building_type: BuildingType,
}

/// Event triggered when a building is successfully placed/constructed.
#[derive(Event, Debug, Clone)]
pub struct BuildingCompletedEvent {
    /// The entity ID of the new building.
    pub entity: Entity,
}

/// Event fired when a pop is consumed by a living building.
#[derive(Event, Debug, Clone)]
pub struct PopConsumedEvent {
    /// The consumed pop.
    pub pop: Entity,
    /// The building that consumed it.
    pub building: Entity,
}
