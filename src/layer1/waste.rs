use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct IndustrialWaste {
    pub amount: u32,
}
