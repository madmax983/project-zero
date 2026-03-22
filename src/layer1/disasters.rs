use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisasterType {
    ReactorMeltdown,
    MassiveEarthquake,
    ViolentUprising,
    LocalizedFire,
}

#[derive(Event, Clone)]
pub struct DisasterEvent {
    pub disaster_type: DisasterType,
    pub location: GridPosition,
    pub severity: f32,
}
