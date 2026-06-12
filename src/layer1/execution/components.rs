use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

/// Component indicating a pop is moving toward a target.
#[derive(Component, Debug)]
pub struct MovementTarget {
    /// The entity being targeted (farm, housing, or designation).
    pub target_entity: Entity,
    /// The grid position of the target.
    pub target_position: GridPosition,
    /// The action type this movement is for.
    pub for_action: ActionType,
}

/// Marker component indicating a pop has arrived at its target.
#[derive(Component, Debug)]
pub struct AtTarget;

#[derive(Component, Debug, Clone)]
pub struct HabituatedRoute {
    pub path: Vec<GridPosition>,
    pub urgency: f32,
    pub frustration: u32,
    pub last_pos: Option<GridPosition>,
}
