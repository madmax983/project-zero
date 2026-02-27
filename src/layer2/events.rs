use bevy_ecs::prelude::*;

/// Event triggered when a launch occurs.
#[derive(Event, Debug, Clone)]
pub struct LaunchEvent {
    /// The planet launched from.
    pub planet: Entity,
    /// Whether the launch was successful.
    pub success: bool,
}

/// Event triggered when a ship is destroyed.
#[derive(Event, Debug, Clone)]
pub struct ShipDestroyedEvent {
    /// The planet where the destruction occurred.
    pub planet: Entity,
    /// The class of the destroyed ship.
    pub ship_class: String,
}

/// Event triggered when the colony's thermal signature is detected by hostile forces.
#[derive(Event, Debug, Clone, Default)]
pub struct DetectionEvent;
