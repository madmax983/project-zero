//! Movement system and components.
//!
//! Handles pathfinding, movement execution, and target tracking for pops.

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::utility_ai::{ActionType, StartPlan};

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

/// Consumes `StartPlan` markers and creates `MovementTarget` components.
///
/// This system bridges the utility AI's decision (`StartPlan`) with the
/// movement system by creating `MovementTarget` for each pop.
pub fn process_start_plan_system(
    plans: Query<(Entity, &StartPlan)>,
    positions: Query<&GridPosition>,
    mut commands: Commands,
) {
    for (pop_entity, start_plan) in &plans {
        let action = start_plan.action;
        let target = start_plan.target;

        // Remove the StartPlan marker and any existing movement components
        commands
            .entity(pop_entity)
            .remove::<StartPlan>()
            .remove::<MovementTarget>()
            .remove::<AtTarget>();

        // If there's no target, skip (e.g., Idle action)
        let Some(target_entity) = target else {
            continue;
        };

        // Get the target's position (also validates entity existence)
        let Ok(&target_position) = positions.get(target_entity) else {
            continue;
        };

        // Insert new MovementTarget
        commands.entity(pop_entity).insert(MovementTarget {
            target_entity,
            target_position,
            for_action: action,
        });
    }
}

/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
pub fn movement_system(
    mut pops: Query<
        (
            Entity,
            &mut GridPosition,
            &MovementTarget,
            Option<&mut Speed>,
        ),
        Without<AtTarget>,
    >,
    terrain: Res<TerrainGrid>,
    mut commands: Commands,
) {
    for (pop_entity, mut current_pos, mt, mut speed_opt) in &mut pops {
        // Handle variable movement speed
        if let Some(ref mut speed) = speed_opt {
            speed.accumulator += speed.current;
            if speed.accumulator < 1.0 {
                continue;
            }
            speed.accumulator -= 1.0;
        }

        let target_pos = mt.target_position;
        let action = mt.for_action;

        // For work/repair actions, check if adjacent to an unwalkable target (rock/tree/building)
        // Pops work FROM adjacent tiles, not ON the target
        if action == ActionType::Work || action == ActionType::Repair {
            let target_walkable = is_walkable_terrain(&terrain, target_pos.x, target_pos.y);
            if !target_walkable {
                let distance =
                    (current_pos.x - target_pos.x).abs() + (current_pos.y - target_pos.y).abs();
                if distance == 1 {
                    commands.entity(pop_entity).insert(AtTarget);
                    continue;
                }
            }
        }

        let Some(new_pos) = calculate_next_position(*current_pos, target_pos) else {
            continue;
        };

        if !is_walkable_terrain(&terrain, new_pos.x, new_pos.y) {
            continue;
        }

        current_pos.x = new_pos.x;
        current_pos.y = new_pos.y;

        if new_pos == target_pos {
            commands.entity(pop_entity).insert(AtTarget);
        }

        // For work/repair actions on unwalkable targets, also check if now adjacent
        if action == ActionType::Work || action == ActionType::Repair {
            let target_walkable = is_walkable_terrain(&terrain, target_pos.x, target_pos.y);
            if !target_walkable {
                let distance = (new_pos.x - target_pos.x).abs() + (new_pos.y - target_pos.y).abs();
                if distance == 1 {
                    commands.entity(pop_entity).insert(AtTarget);
                }
            }
        }
    }
}

/// Checks if a tile at (x, y) is walkable.
pub fn is_walkable_terrain(terrain: &TerrainGrid, x: i32, y: i32) -> bool {
    if let (Ok(x_idx), Ok(y_idx)) = (usize::try_from(x), usize::try_from(y)) {
        terrain
            .get(x_idx, y_idx)
            .is_some_and(TerrainType::is_walkable)
    } else {
        false
    }
}

/// Calculates the next position to move to in order to reach the target.
/// Uses simple Manhattan distance heuristic (horizontal then vertical).
#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
pub fn calculate_next_position(current: GridPosition, target: GridPosition) -> Option<GridPosition> {
    // Calculate movement direction (Manhattan)
    let dx = (target.x - current.x).signum();
    let dy = (target.y - current.y).signum();

    // Prefer horizontal movement, then vertical
    if dx != 0 {
        Some(GridPosition {
            x: current.x + dx,
            y: current.y,
        })
    } else {
        Some(GridPosition {
            x: current.x,
            y: current.y + dy,
        })
    }
}
