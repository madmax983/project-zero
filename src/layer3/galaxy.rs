use bevy::prelude::*;

#[derive(Component)]
pub struct GalaxyNode;

#[derive(Event)]
pub struct FleetTravelEvent {
    pub fleet: Entity,
    pub destination: Entity,
}
