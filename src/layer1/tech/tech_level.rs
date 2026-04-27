use bevy_ecs::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TechLevel {
    Primitive,
    Standard,
    Advanced,
}
