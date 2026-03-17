use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct PsychicBackground {
    pub intensity: f32, // 0.0 (normal) to 1.0 (screaming void)
}
