use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Discovery {
    pub data_type: String,
    pub value: u32,
}
