use bevy_ecs::prelude::*;
use crate::layer1::building::BuildingType;
use crate::layer1::map::GridPosition;

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
