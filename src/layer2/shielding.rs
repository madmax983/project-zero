use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct OrbitalShield {
    pub capacity: f32,
    pub max_capacity: f32,
}
