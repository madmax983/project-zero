use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SecurityDoor {
    pub required_clearance: u32,
    pub max_drift_tolerance: f32,
}
