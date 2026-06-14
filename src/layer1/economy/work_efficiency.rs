use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct WorkEfficiency {
    pub multiplier: f32,
}

impl Default for WorkEfficiency {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}
