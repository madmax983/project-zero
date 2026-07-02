use bevy::prelude::*;

#[derive(Event)]
pub struct MegaEvent {
    pub planet_entity: Entity,
    pub event_type: String,
    pub intensity: f32,
}
