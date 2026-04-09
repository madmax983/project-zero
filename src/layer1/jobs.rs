pub use crate::layer1::economy::ResourceType;
use bevy::prelude::*;

#[derive(Component)]
pub struct CurrentTask {
    pub task_id: u32,
    pub duration: f32,
    pub base_output: ResourceType,
    pub efficiency: f32,
    pub is_randomized_output: bool,
}

impl Default for CurrentTask {
    fn default() -> Self {
        Self {
            task_id: 0,
            duration: 0.0,
            base_output: ResourceType::Metal,
            efficiency: 1.0,
            is_randomized_output: false,
        }
    }
}
