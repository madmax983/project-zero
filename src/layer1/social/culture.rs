use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Peaceful,
    Hostile,
}

#[derive(Component)]
pub struct Culture {
    pub alignment: Alignment,
}
