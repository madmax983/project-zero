use crate::layer1::resources::ResourceType;
use bevy_ecs::prelude::*;

/// Component representing a Mother Lode - a rare, infinite, but dangerous resource node.
#[derive(Component, Debug, Clone)]
pub struct MotherLode {
    /// The type of resource this lode provides.
    pub resource_type: ResourceType,
    /// Multiplier for accident risk (starts at 1.0).
    pub current_hazard: f32,
    /// Heat output added to local temperature grid per tick (starts at ~10.0).
    pub heat_output: f32,
}

impl MotherLode {
    /// Increments the hazard level and heat output of the lode.
    /// Called every time the lode is mined.
    pub fn increment_hazard(&mut self) {
        self.current_hazard += 0.1;
        self.heat_output += 2.0;
    }
}
