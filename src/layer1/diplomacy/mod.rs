pub mod wards;
pub use wards::*;

use crate::layer1::social::factions::FactionId;
use bevy::prelude::*;

#[derive(Component)]
pub struct DiplomaticImmunity {
    pub faction_id: FactionId,
}

#[derive(Event)]
pub struct DiplomaticIncidentEvent {
    pub faction_id: FactionId,
    pub reason: String,
}
