use bevy_ecs::prelude::*;

/// Component granting a Pop diplomatic immunity under their faction's laws.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct DiplomaticImmunity {
    pub faction_id: u32,
}

/// Event triggered when an immune Pop is forcibly arrested or punished,
/// causing a major international incident.
#[derive(Event, Debug, Clone)]
pub struct DiplomaticIncidentEvent {
    pub faction_id: u32,
    pub reason: String,
}
