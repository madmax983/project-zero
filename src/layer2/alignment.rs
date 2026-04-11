use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct PlanetaryAlignment {
    pub days_until: u32,
}
