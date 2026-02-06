//! Execution layer bridging utility AI decisions to actual pop actions.
//!
//! This module handles:
//! - Movement toward targets
//! - Assigning pops to farms/housing
//! - Executing work at designations (mining, chopping)
//!
//! # System Flow
//!
//! ```text
//! evaluate_actions_system (existing)
//!     ↓ inserts StartPlan
//! cleanup_previous_assignment_system
//!     ↓ removes pop from old farm/housing
//! process_start_plan_system
//!     ↓ converts StartPlan → MovementTarget
//! movement_system
//!     ↓ moves pop 1 tile, inserts AtTarget on arrival
//! arrival_handler_system
//!     ↓ assigns to farm/housing
//! work_execution_system
//!     ↓ calls mine_rock/chop_tree
//! ```

use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{
    ColonyResources, ForestryProgress, MiningProgress, chop_tree, mine_rock,
};
use crate::layer1::social::Tavern;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::utility_ai::{ActionType, StartPlan};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Work amount applied per tick when a pop is working.
const WORK_PER_TICK: f32 = 10.0;

/// Chance for a tool to break per tick when used.
const TOOL_BREAK_CHANCE: f64 = 0.01;

/// Efficiency multiplier when working without tools.
const NO_TOOL_PENALTY: f32 = 0.5;

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

/// Component tracking what a pop is assigned to.
#[derive(Component, Debug)]
pub struct AssignedTo {
    /// The entity the pop is assigned to.
    pub entity: Entity,
    /// The type of assignment.
    pub assignment_type: AssignmentType,
}

/// Types of assignments a pop can have.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignmentType {
    /// Working at a farm.
    FarmWorker,
    /// Residing in housing.
    HousingResident,
    /// Socializing at a tavern.
    TavernVisitor,
    /// Working at a library.
    LibraryWorker,
}

/// Removes pops from farms/housing when they switch to a different action.
///
/// This system runs before `process_start_plan_system` to ensure pops are
/// properly removed from their previous assignment before moving to a new one.
pub fn cleanup_previous_assignment_system(world: &mut World) {
    // Find pops that have StartPlan (about to switch actions) and have AssignedTo
    let pops_to_cleanup: Vec<(Entity, Entity, AssignmentType)> = world
        .query_filtered::<(Entity, &AssignedTo), With<StartPlan>>()
        .iter(world)
        .map(|(e, assigned)| (e, assigned.entity, assigned.assignment_type))
        .collect();

    for (pop_entity, assigned_entity, assignment_type) in pops_to_cleanup {
        match assignment_type {
            AssignmentType::FarmWorker => {
                if let Some(mut farm) = world.get_mut::<Farm>(assigned_entity) {
                    farm.workers.retain(|&w| w != pop_entity);
                }
            }
            AssignmentType::HousingResident => {
                if let Some(mut housing) = world.get_mut::<Housing>(assigned_entity) {
                    housing.residents.retain(|&r| r != pop_entity);
                }
            }
            AssignmentType::TavernVisitor => {
                if let Some(mut tavern) = world.get_mut::<Tavern>(assigned_entity) {
                    tavern.visitors.retain(|&v| v != pop_entity);
                }
            }
            AssignmentType::LibraryWorker => {
                // Library component currently doesn't track workers list, so no cleanup needed on building
            }
        }

        // Remove the AssignedTo component
        world.entity_mut(pop_entity).remove::<AssignedTo>();
    }
}

/// Consumes `StartPlan` markers and creates `MovementTarget` components.
///
/// This system bridges the utility AI's decision (`StartPlan`) with the
/// movement system by creating `MovementTarget` for each pop.
pub fn process_start_plan_system(world: &mut World) {
    // Collect StartPlan data
    let start_plans: Vec<(Entity, ActionType, Option<Entity>)> = world
        .query::<(Entity, &StartPlan)>()
        .iter(world)
        .map(|(e, sp)| (e, sp.action, sp.target))
        .collect();

    for (pop_entity, action, target) in start_plans {
        // Remove the StartPlan marker
        world.entity_mut(pop_entity).remove::<StartPlan>();

        // If there's no target, skip (e.g., Idle action)
        let Some(target_entity) = target else {
            continue;
        };

        // Check if target entity still exists
        if world.get_entity(target_entity).is_err() {
            continue;
        }

        // Get the target's position
        let Some(&target_position) = world.get::<GridPosition>(target_entity) else {
            continue;
        };

        // Remove any existing MovementTarget and AtTarget
        clear_movement_components(world, pop_entity);

        // Insert new MovementTarget
        world.entity_mut(pop_entity).insert(MovementTarget {
            target_entity,
            target_position,
            for_action: action,
        });
    }
}

/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
pub fn movement_system(world: &mut World) {
    // Collect pops with movement targets (that haven't arrived yet)
    let pops_to_move: Vec<(Entity, GridPosition, GridPosition, ActionType)> = world
        .query_filtered::<(Entity, &GridPosition, &MovementTarget), Without<AtTarget>>()
        .iter(world)
        .map(|(e, pos, mt)| (e, *pos, mt.target_position, mt.for_action))
        .collect();

    for (pop_entity, current_pos, target_pos, action) in pops_to_move {
        // For work actions, check if adjacent to an unwalkable target (rock/tree)
        // Pops work FROM adjacent tiles, not ON the target
        if action == ActionType::Work {
            let target_walkable = is_walkable(world, target_pos.x, target_pos.y);
            if !target_walkable {
                let distance =
                    (current_pos.x - target_pos.x).abs() + (current_pos.y - target_pos.y).abs();
                if distance == 1 {
                    // Adjacent to unwalkable target - can work from here
                    world.entity_mut(pop_entity).insert(AtTarget);
                    continue;
                }
            }
        }

        let Some(new_pos) = calculate_next_position(current_pos, target_pos) else {
            continue;
        };

        // Check if new position is walkable (includes bounds check)
        if !is_walkable(world, new_pos.x, new_pos.y) {
            // Wait in place (simple approach - no pathfinding)
            continue;
        }

        // Update position
        if let Some(mut pos) = world.get_mut::<GridPosition>(pop_entity) {
            pos.x = new_pos.x;
            pos.y = new_pos.y;
        }

        // Check if now at target
        if new_pos == target_pos {
            world.entity_mut(pop_entity).insert(AtTarget);
        }

        // For work actions on unwalkable targets, also check if now adjacent
        if action == ActionType::Work {
            let target_walkable = is_walkable(world, target_pos.x, target_pos.y);
            if !target_walkable {
                let distance = (new_pos.x - target_pos.x).abs() + (new_pos.y - target_pos.y).abs();
                if distance == 1 {
                    world.entity_mut(pop_entity).insert(AtTarget);
                }
            }
        }
    }
}

/// Handles arrival at targets: assigns pops to farms/housing.
pub fn arrival_handler_system(world: &mut World) {
    // Collect pops that just arrived
    let arrivals: Vec<(Entity, Entity, ActionType)> = world
        .query_filtered::<(Entity, &MovementTarget), With<AtTarget>>()
        .iter(world)
        .map(|(e, mt)| (e, mt.target_entity, mt.for_action))
        .collect();

    for (pop_entity, target_entity, action) in arrivals {
        // Check if target still exists
        if world.get_entity(target_entity).is_err() {
            // Target despawned, remove movement components
            clear_movement_components(world, pop_entity);
            continue;
        }

        match action {
            ActionType::SatisfyHunger => {
                if assign_to_farm(world, target_entity, pop_entity) {
                    world.entity_mut(pop_entity).insert(AssignedTo {
                        entity: target_entity,
                        assignment_type: AssignmentType::FarmWorker,
                    });
                }
                clear_movement_components(world, pop_entity);
            }
            ActionType::SatisfyRest => {
                if assign_to_housing(world, target_entity, pop_entity) {
                    world.entity_mut(pop_entity).insert(AssignedTo {
                        entity: target_entity,
                        assignment_type: AssignmentType::HousingResident,
                    });
                }
                clear_movement_components(world, pop_entity);
            }
            ActionType::Socialize => {
                if assign_to_tavern(world, target_entity, pop_entity) {
                    world.entity_mut(pop_entity).insert(AssignedTo {
                        entity: target_entity,
                        assignment_type: AssignmentType::TavernVisitor,
                    });
                }
                clear_movement_components(world, pop_entity);
            }
            ActionType::Research => {
                // Library has no worker limit logic yet, so always succeed
                world.entity_mut(pop_entity).insert(AssignedTo {
                    entity: target_entity,
                    assignment_type: AssignmentType::LibraryWorker,
                });
                clear_movement_components(world, pop_entity);
            }
            ActionType::Work => {
                // Work is handled by work_execution_system
                // Just keep the AtTarget marker for that system
            }
            _ => {
                // Other actions (Idle, Explore) - just clear movement
                clear_movement_components(world, pop_entity);
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn assign_to_farm(world: &mut World, farm_entity: Entity, pop_entity: Entity) -> bool {
    let Some(mut farm) = world.get_mut::<Farm>(farm_entity) else {
        return false;
    };

    if farm.workers.len() >= farm.capacity {
        return false;
    }

    farm.workers.push(pop_entity);
    true
}

fn assign_to_housing(world: &mut World, housing_entity: Entity, pop_entity: Entity) -> bool {
    let Some(mut housing) = world.get_mut::<Housing>(housing_entity) else {
        return false;
    };

    if housing.residents.len() >= housing.capacity {
        return false;
    }

    housing.residents.push(pop_entity);
    true
}

fn assign_to_tavern(world: &mut World, tavern_entity: Entity, pop_entity: Entity) -> bool {
    let Some(mut tavern) = world.get_mut::<Tavern>(tavern_entity) else {
        return false;
    };

    if tavern.visitors.len() >= tavern.capacity {
        return false;
    }

    tavern.visitors.push(pop_entity);
    true
}

fn clear_movement_components(world: &mut World, pop_entity: Entity) {
    world
        .entity_mut(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();
}

fn is_walkable(world: &World, x: i32, y: i32) -> bool {
    let terrain = world.resource::<TerrainGrid>();
    if let (Ok(x_idx), Ok(y_idx)) = (usize::try_from(x), usize::try_from(y)) {
        terrain
            .get(x_idx, y_idx)
            .is_some_and(crate::layer1::terrain::TerrainType::is_walkable)
    } else {
        false
    }
}

#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
fn calculate_next_position(current: GridPosition, target: GridPosition) -> Option<GridPosition> {
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

/// Executes work at designations when pop is at target with Work action.
pub fn work_execution_system(world: &mut World) {
    // Check tools at the start of the system
    let (has_tools, mut tool_broken) = {
        let res = world.resource::<ColonyResources>();
        (res.tools >= 1.0, false)
    };

    let efficiency = if has_tools { 1.0 } else { NO_TOOL_PENALTY };
    let work_amount = WORK_PER_TICK * efficiency;

    // Find pops at their work target
    let workers: Vec<(Entity, Entity)> = world
        .query_filtered::<(Entity, &MovementTarget), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt)| mt.for_action == ActionType::Work)
        .map(|(e, mt)| (e, mt.target_entity))
        .collect();

    for (_pop_entity, designation_entity) in workers {
        // Check if designation still exists
        if world.get_entity(designation_entity).is_err() {
            continue;
        }

        // Get designation type
        let designation_type =
            if let Some(designation) = world.get::<Designation>(designation_entity) {
                designation.designation_type
            } else {
                continue;
            };

        let worked = match designation_type {
            DesignationType::Mine => {
                // Ensure MiningProgress exists
                if world.get::<MiningProgress>(designation_entity).is_none() {
                    world
                        .entity_mut(designation_entity)
                        .insert(MiningProgress::default());
                }
                mine_rock(world, designation_entity, work_amount);
                true
            }
            DesignationType::Chop => {
                // Ensure ForestryProgress exists
                if world.get::<ForestryProgress>(designation_entity).is_none() {
                    world
                        .entity_mut(designation_entity)
                        .insert(ForestryProgress {
                            current: 0.0,
                            max: 50.0,
                        });
                }
                chop_tree(world, designation_entity, work_amount);
                true
            }
            DesignationType::Demolish => {
                // TODO: Implement demolish logic
                false
            }
        };

        if worked && has_tools && !tool_broken {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(TOOL_BREAK_CHANCE) {
                tool_broken = true;
            }
        }
    }

    if tool_broken {
        let mut res = world.resource_mut::<ColonyResources>();
        if res.tools >= 1.0 {
            res.tools -= 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::TerrainType;
    use crate::layer1::utility_ai::{PopAction, UtilityWeights};

    fn setup_world() -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world
    }

    // =========================================================================
    // process_start_plan_system tests
    // =========================================================================

    #[test]
    fn test_process_start_plan_creates_movement_target() {
        let mut world = setup_world();

        // Create a farm as target
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        // Create a pop with StartPlan
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        process_start_plan_system(&mut world);

        // Should have MovementTarget
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should have MovementTarget");
        let mt = mt.unwrap();
        assert_eq!(mt.target_entity, farm);
        assert_eq!(mt.target_position, GridPosition { x: 5, y: 5 });
        assert_eq!(mt.for_action, ActionType::SatisfyHunger);
    }

    #[test]
    fn test_process_start_plan_removes_itself() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        process_start_plan_system(&mut world);

        // StartPlan should be removed
        assert!(
            world.get::<StartPlan>(pop).is_none(),
            "StartPlan should be removed"
        );
    }

    #[test]
    fn test_process_start_plan_handles_despawned_target() {
        let mut world = setup_world();

        // Create a pop with StartPlan pointing to a despawned entity
        let fake_entity = Entity::from_raw(9999);
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(fake_entity),
                },
            ))
            .id();

        process_start_plan_system(&mut world);

        // Should not have MovementTarget
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "Should not create MovementTarget for despawned target"
        );
    }

    // =========================================================================
    // movement_system tests
    // =========================================================================

    #[test]
    fn test_movement_system_moves_toward_target() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        movement_system(&mut world);

        let pos = world.get::<GridPosition>(pop).unwrap();
        // Should have moved 1 tile toward target (horizontal first)
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_system_marks_arrival() {
        let mut world = setup_world();

        // Pop already at target position
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        movement_system(&mut world);

        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be marked as AtTarget"
        );
    }

    #[test]
    fn test_movement_system_marks_arrival_on_last_step() {
        let mut world = setup_world();

        // Pop 1 tile away from target
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 4, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        movement_system(&mut world);

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be marked as AtTarget"
        );
    }

    #[test]
    fn test_movement_blocked_by_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[1] = TerrainType::Rock; // Block position (1, 0)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        movement_system(&mut world);

        // Pop should not have moved (blocked)
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_blocked_by_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[1] = TerrainType::Water; // Block position (1, 0)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        movement_system(&mut world);

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    // =========================================================================
    // arrival_handler_system tests
    // =========================================================================

    #[test]
    fn test_arrival_assigns_to_farm() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: farm,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyHunger,
                },
                AtTarget,
            ))
            .id();

        arrival_handler_system(&mut world);

        // Pop should be in farm workers list
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(farm_comp.workers.contains(&pop));

        // Pop should have AssignedTo
        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some());
        assert_eq!(
            assigned.unwrap().assignment_type,
            AssignmentType::FarmWorker
        );
    }

    #[test]
    fn test_arrival_assigns_to_housing() {
        let mut world = setup_world();

        let housing = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Housing::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: housing,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyRest,
                },
                AtTarget,
            ))
            .id();

        arrival_handler_system(&mut world);

        // Pop should be in housing residents list
        let housing_comp = world.get::<Housing>(housing).unwrap();
        assert!(housing_comp.residents.contains(&pop));

        // Pop should have AssignedTo
        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some());
        assert_eq!(
            assigned.unwrap().assignment_type,
            AssignmentType::HousingResident
        );
    }

    #[test]
    fn test_arrival_farm_at_capacity() {
        let mut world = setup_world();

        let other_pop = world.spawn(Pop).id();
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm {
                    capacity: 1,
                    workers: vec![other_pop], // Already full
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: farm,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyHunger,
                },
                AtTarget,
            ))
            .id();

        arrival_handler_system(&mut world);

        // Pop should NOT be in farm workers
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(!farm_comp.workers.contains(&pop));

        // MovementTarget should be removed
        assert!(world.get::<MovementTarget>(pop).is_none());
    }

    // =========================================================================
    // work_execution_system tests
    // =========================================================================

    #[test]
    fn test_work_execution_calls_mine_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // MiningProgress should be added and incremented
        let progress = world.get::<MiningProgress>(designation);
        assert!(progress.is_some(), "MiningProgress should be added");
        assert!(progress.unwrap().current > 0.0, "Progress should increase");

        // Pop should still exist
        assert!(world.get_entity(pop).is_ok());
    }

    #[test]
    fn test_work_execution_calls_chop_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // ForestryProgress should be added and incremented
        let progress = world.get::<ForestryProgress>(designation);
        assert!(progress.is_some(), "ForestryProgress should be added");
        assert!(progress.unwrap().current > 0.0, "Progress should increase");
    }

    #[test]
    fn test_work_execution_completes_mining() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress {
                    current: 95.0,
                    max: 100.0,
                },
            ))
            .id();

        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // Designation should be despawned
        assert!(
            world.get_entity(designation).is_err(),
            "Designation should be despawned"
        );

        // Terrain should be Dirt
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // ResourceItem should be spawned
        let items: Vec<_> = world
            .query::<&crate::layer1::resources::ResourceItem>()
            .iter(&world)
            .collect();
        assert!(!items.is_empty(), "ResourceItem should be spawned");
        assert_eq!(
            items[0].resource_type,
            crate::layer1::resources::ResourceType::Stone
        );
    }

    // =========================================================================
    // cleanup_previous_assignment_system tests
    // =========================================================================

    #[test]
    fn test_cleanup_removes_from_farm() {
        let mut world = setup_world();

        let pop = world.spawn(Pop).id();
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm {
                    capacity: 2,
                    workers: vec![pop],
                },
            ))
            .id();

        // Pop is assigned and wants to switch
        world.entity_mut(pop).insert((
            AssignedTo {
                entity: farm,
                assignment_type: AssignmentType::FarmWorker,
            },
            StartPlan {
                action: ActionType::SatisfyRest,
                target: None,
            },
        ));

        cleanup_previous_assignment_system(&mut world);

        // Pop should be removed from farm workers
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(!farm_comp.workers.contains(&pop));

        // AssignedTo should be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    #[test]
    fn test_cleanup_removes_from_housing() {
        let mut world = setup_world();

        let pop = world.spawn(Pop).id();
        let housing = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Housing {
                    capacity: 2,
                    residents: vec![pop],
                },
            ))
            .id();

        world.entity_mut(pop).insert((
            AssignedTo {
                entity: housing,
                assignment_type: AssignmentType::HousingResident,
            },
            StartPlan {
                action: ActionType::SatisfyHunger,
                target: None,
            },
        ));

        cleanup_previous_assignment_system(&mut world);

        // Pop should be removed from housing residents
        let housing_comp = world.get::<Housing>(housing).unwrap();
        assert!(!housing_comp.residents.contains(&pop));

        // AssignedTo should be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    #[test]
    fn test_cleanup_handles_despawned_building() {
        let mut world = setup_world();

        let fake_entity = Entity::from_raw(9999);
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: fake_entity, // Doesn't exist
                    assignment_type: AssignmentType::FarmWorker,
                },
                StartPlan {
                    action: ActionType::SatisfyRest,
                    target: None,
                },
            ))
            .id();

        // Should not panic
        cleanup_previous_assignment_system(&mut world);

        // AssignedTo should still be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    #[test]
    fn test_full_execution_flow_farm() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 2, y: 0 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        // Process start plan
        process_start_plan_system(&mut world);
        assert!(world.get::<MovementTarget>(pop).is_some());

        // Move toward farm (2 ticks)
        movement_system(&mut world);
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);

        movement_system(&mut world);
        assert!(world.get::<AtTarget>(pop).is_some());

        // Handle arrival
        arrival_handler_system(&mut world);
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(farm_comp.workers.contains(&pop));
    }

    #[test]
    fn test_pop_moves_multiple_tiles_to_designation() {
        let mut world = setup_world();

        // Create a designation 5 tiles away
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        // Move 5 times - pop should reach destination
        for tick in 1..=5 {
            movement_system(&mut world);
            let pos = world.get::<GridPosition>(pop).unwrap();
            assert_eq!(pos.x, tick, "Pop should be at x={tick} after {tick} ticks",);
        }

        // Should be at target now
        assert!(world.get::<AtTarget>(pop).is_some());
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_persists_across_evaluation_cycles() {
        use crate::layer1::utility_ai::{
            UtilityConfig, evaluate_actions_system, update_action_timer_system,
        };
        use crate::shared::time::SimulationTime;

        let mut world = setup_world();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());

        // Put a rock at (5, 0) for mining
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[5] = TerrainType::Rock;
        }

        // Create a mining designation
        let _designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // Create a pop that will want to work
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Tick 1: Pop should decide to work
        update_action_timer_system(&mut world);
        evaluate_actions_system(&mut world);

        // Should have StartPlan for work
        assert!(
            world.get::<StartPlan>(pop).is_some(),
            "Pop should have StartPlan after first evaluation"
        );

        // Process and start moving
        cleanup_previous_assignment_system(&mut world);
        process_start_plan_system(&mut world);
        movement_system(&mut world);
        arrival_handler_system(&mut world);
        work_execution_system(&mut world);

        let pos1 = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos1.x, 1, "Pop should have moved to x=1");

        // Tick 2-4: Continue moving toward rock (stop adjacent at x=4)
        // Pop can't stand ON the rock, so they work from adjacent tile
        for tick in 2..=4 {
            update_action_timer_system(&mut world);
            evaluate_actions_system(&mut world);
            cleanup_previous_assignment_system(&mut world);
            process_start_plan_system(&mut world);
            movement_system(&mut world);
            arrival_handler_system(&mut world);
            work_execution_system(&mut world);

            let pos = *world.get::<GridPosition>(pop).unwrap();
            assert_eq!(pos.x, tick, "Tick {tick}: Pop should be at x={tick}");
        }

        // After tick 4, pop should be adjacent to rock (at x=4) and marked AtTarget
        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be AtTarget when adjacent to work designation"
        );
        let final_pos = *world.get::<GridPosition>(pop).unwrap();
        assert_eq!(final_pos.x, 4, "Pop should stop adjacent to rock at x=4");

        // Pop should now be at the designation
        assert!(world.get::<AtTarget>(pop).is_some());
    }
}
