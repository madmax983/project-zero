
use crate::layer1::core::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisasterType {
    Fissure,
    MegaHurricane,
}

#[derive(Event, Debug, Clone)]
pub struct DisasterEvent {
    pub position: GridPosition,
    pub disaster_type: DisasterType,
}

#[derive(Event)]
pub struct MegaEvent {
    pub planet_entity: Entity,
    pub event_type: String,
    pub intensity: f32,
}
