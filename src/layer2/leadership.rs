use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Admiral {
    pub fleet: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct FleetMutinyEvent {
    pub admiral: Entity,
    pub fleet: Entity,
}
