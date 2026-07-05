use crate::layer1::core::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Event, Debug, Clone)]
pub struct TerraformEvent {
    pub position: GridPosition,
    pub delta_temperature: f32,
}
