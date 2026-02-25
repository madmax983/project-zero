use crate::layer1::resources::ResourceType;
use bevy_ecs::prelude::*;

/// Represents a single stack of cargo in a fleet.
#[derive(Debug, Clone, Copy)]
pub struct CargoStack {
    /// The type of resource in this stack.
    pub resource_type: ResourceType,
    /// The amount of the resource.
    pub amount: f32,
}

/// Component for fleets to store mined resources.
#[derive(Component, Debug, Clone, Default)]
pub struct FleetCargo {
    /// The list of resource stacks.
    pub contents: Vec<CargoStack>,
    /// The total capacity of the cargo hold.
    pub capacity: f32,
}
