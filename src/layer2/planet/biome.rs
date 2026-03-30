use bevy_ecs::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlanetBiome {
    Ice,
    Arid,
    Lush,
    Volcanic,
    Toxic,
}
