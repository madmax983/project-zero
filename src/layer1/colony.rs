use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Event, Debug)]
pub struct CollapseEvent {
    pub colony: Entity,
    pub survivor_count: u32,
}
