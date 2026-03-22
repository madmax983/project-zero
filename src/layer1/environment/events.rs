use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct DebrisFallEvent {
    pub location: GridPosition,
    pub severity: f32,
}
