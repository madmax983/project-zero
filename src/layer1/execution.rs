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

use crate::layer1::actions::hunger::handle_arrival as handle_hunger_arrival;
use crate::layer1::actions::rest::handle_arrival as handle_rest_arrival;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::building::{Building, OccupiedTiles};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::farm::Farm;
use crate::layer1::health::Health;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::memory::{Memories, calculate_effective_morale};
use crate::layer1::needs::{Needs, get_morale_efficiency};
use crate::layer1::resources::{
    ColonyResources, ForestryProgress, MiningProgress, chop_tree, mine_rock,
};
use crate::layer1::social::Tavern;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::utility_ai::{ActionType, PopAction, StartPlan};
use crate::shared::log::MessageLog;
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

/// Removes pops from farms/housing when they switch to a different action.
///
/// This system runs before `process_start_plan_system` to ensure pops are
/// properly removed from their previous assignment before moving to a new one.
pub fn cleanup_previous_assignment_system(
    pops_query: Query<(Entity, &AssignedTo), With<StartPlan>>,
    mut farms: Query<&mut Farm>,
    mut housing: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    mut commands: Commands,
) {
    for (pop_entity, assigned) in &pops_query {
        let assigned_entity = assigned.entity;
        match assigned.assignment_type {
            AssignmentType::FarmWorker => {
                if let Ok(mut farm) = farms.get_mut(assigned_entity) {
                    farm.workers.retain(|&w| w != pop_entity);
                }
            }
            AssignmentType::HousingResident => {
                if let Ok(mut h) = housing.get_mut(assigned_entity) {
                    h.residents.retain(|&r| r != pop_entity);
                }
            }
            AssignmentType::TavernVisitor => {
                if let Ok(mut tavern) = taverns.get_mut(assigned_entity) {
                    tavern.visitors.retain(|&v| v != pop_entity);
                }
            }
            AssignmentType::LibraryWorker => {}
        }

        commands.entity(pop_entity).remove::<AssignedTo>();
    }
}

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
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
pub fn movement_system(
    mut pops: Query<(Entity, &mut GridPosition, &MovementTarget), Without<AtTarget>>,
    terrain: Res<TerrainGrid>,
    mut commands: Commands,
) {
    for (pop_entity, mut current_pos, mt) in &mut pops {
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

/// Handles arrival at targets: assigns pops to farms/housing.
pub fn arrival_handler_system(
    arrivals: Query<(Entity, &MovementTarget), With<AtTarget>>,
    mut farms: Query<&mut Farm>,
    mut housing_q: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    mut commands: Commands,
) {
    for (pop_entity, mt) in &arrivals {
        let target_entity = mt.target_entity;
        let action = mt.for_action;

        match action {
            ActionType::SatisfyHunger => {
                handle_hunger_arrival(pop_entity, target_entity, &mut farms, &mut commands);
                commands
                    .entity(pop_entity)
                    .remove::<MovementTarget>()
                    .remove::<AtTarget>();
            }
            ActionType::SatisfyRest => {
                handle_rest_arrival(pop_entity, target_entity, &mut housing_q, &mut commands);
                commands
                    .entity(pop_entity)
                    .remove::<MovementTarget>()
                    .remove::<AtTarget>();
            }
            ActionType::Socialize => {
                if let Ok(mut tavern) = taverns.get_mut(target_entity)
                    && tavern.visitors.len() < tavern.capacity
                {
                    tavern.visitors.push(pop_entity);
                    commands.entity(pop_entity).insert(AssignedTo {
                        entity: target_entity,
                        assignment_type: AssignmentType::TavernVisitor,
                    });
                }
                commands
                    .entity(pop_entity)
                    .remove::<MovementTarget>()
                    .remove::<AtTarget>();
            }
            ActionType::Research => {
                commands.entity(pop_entity).insert(AssignedTo {
                    entity: target_entity,
                    assignment_type: AssignmentType::LibraryWorker,
                });
                commands
                    .entity(pop_entity)
                    .remove::<MovementTarget>()
                    .remove::<AtTarget>();
            }
            ActionType::Work | ActionType::Repair | ActionType::Haul => {
                // Work/Repair/Haul is handled by their respective systems
                // Just keep the AtTarget marker for that system
            }
            _ => {
                commands
                    .entity(pop_entity)
                    .remove::<MovementTarget>()
                    .remove::<AtTarget>();
            }
        }
    }
}

fn is_walkable_terrain(terrain: &TerrainGrid, x: i32, y: i32) -> bool {
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

fn execute_demolish(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // Find building at this position
            // We collect to avoid borrow issues if we need to mutate world later
            let building_entity = world
                .query::<(Entity, &GridPosition, &Building)>()
                .iter(world)
                .find(|(_, pos, _)| pos.x == designation_pos.x && pos.y == designation_pos.y)
                .map(|(e, _, _)| e);

            if let Some(entity) = building_entity {
                world.despawn(entity);
                // Remove from OccupiedTiles
                if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                    occupied.0.remove(&(designation_pos.x, designation_pos.y));
                }
            }

            // Despawn the designation itself
            world.despawn(designation_entity);
            true
        })
}

/// Executes work at designations when pop is at target with Work action.
pub fn work_execution_system(world: &mut World) {
    // Check tools at the start of the system
    let (has_tools, mut tool_broken) = {
        let res = world.resource::<ColonyResources>();
        (res.tools >= 1.0, false)
    };

    let tool_efficiency = if has_tools { 1.0 } else { NO_TOOL_PENALTY };

    // Find pops at their work target and capture their morale
    // Since we need to access Needs which is a component, and we need &mut World later,
    // we should collect Needs data first.
    let workers_data: Vec<(Entity, Entity, f32, ActionType)> = world
        .query_filtered::<(Entity, &MovementTarget, Option<&Needs>, Option<&Memories>), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt, _, _)| mt.for_action == ActionType::Work || mt.for_action == ActionType::Repair)
        .map(|(e, mt, needs, memories)| {
            let morale = needs.map_or(0.5, |n| {
                memories.map_or_else(
                    || n.morale(),
                    |m| calculate_effective_morale(n, m),
                )
            });
            (e, mt.target_entity, morale, mt.for_action)
        })
        .collect();

    for (pop_entity, designation_entity, morale, action_type) in workers_data {
        let morale_efficiency = get_morale_efficiency(morale);
        let work_amount = WORK_PER_TICK * tool_efficiency * morale_efficiency;

        // Check if designation still exists
        if world.get_entity(designation_entity).is_err() {
            cleanup_pop_work_state(world, pop_entity);
            continue;
        }

        // Get designation type
        let Some(designation) = world.get::<Designation>(designation_entity) else {
            continue;
        };
        let designation_type = designation.designation_type;

        let worked = match designation_type {
            DesignationType::Mine => {
                process_mining(world, designation_entity, work_amount);
                true
            }
            DesignationType::Chop => {
                process_logging(world, designation_entity, work_amount);
                true
            }
            DesignationType::Demolish => execute_demolish(world, designation_entity),
            DesignationType::Repair => {
                crate::layer1::structure::process_repair(world, designation_entity, work_amount);
                true
            }
        };

        // After work: if designation was despawned (work completed), reset pop state
        if world.get_entity(designation_entity).is_err() {
            cleanup_pop_work_state(world, pop_entity);
        }

        // Workplace Hazards
        if worked {
            handle_workplace_hazards(world, pop_entity, action_type);

            if has_tools && !tool_broken {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(TOOL_BREAK_CHANCE) {
                    tool_broken = true;
                }
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

fn cleanup_pop_work_state(world: &mut World, pop_entity: Entity) {
    world
        .entity_mut(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();
    if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 1;
    }
}

fn process_mining(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // Ensure MiningProgress exists
    if world.get::<MiningProgress>(designation_entity).is_none() {
        world
            .entity_mut(designation_entity)
            .insert(MiningProgress::default());
    }
    mine_rock(world, designation_entity, work_amount);
}

fn process_logging(world: &mut World, designation_entity: Entity, work_amount: f32) {
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
}

fn handle_workplace_hazards(world: &mut World, pop_entity: Entity, action_type: ActionType) {
    let danger = action_type.danger_level();
    let mut rng = rand::thread_rng();
    if rng.gen_bool(danger) {
        let damage = action_type.accident_damage();
        // Apply damage if pop has Health
        if let Some(mut health) = world.get_mut::<Health>(pop_entity) {
            health.take_damage(damage);

            // Log accident
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!("ACCIDENT: Worker injured! (-{damage} HP)"));
            }
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
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        crate::setup::init_task_pools();
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

        world.run_system_once(process_start_plan_system).unwrap();

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

        world.run_system_once(process_start_plan_system).unwrap();

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

        world.run_system_once(process_start_plan_system).unwrap();

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

        world.run_system_once(movement_system).unwrap();

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

        world.run_system_once(movement_system).unwrap();

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

        world.run_system_once(movement_system).unwrap();

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

        world.run_system_once(movement_system).unwrap();

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

        world.run_system_once(movement_system).unwrap();

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

        world.run_system_once(arrival_handler_system).unwrap();

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

        world.run_system_once(arrival_handler_system).unwrap();

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

        world.run_system_once(arrival_handler_system).unwrap();

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

    #[test]
    fn test_work_execution_resets_pop_on_completion() {
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
                PopAction {
                    current: ActionType::Work,
                    current_utility: 0.8,
                    ticks_committed: 5,
                },
            ))
            .id();

        work_execution_system(&mut world);

        // Designation should be despawned after mining completes
        assert!(
            world.get_entity(designation).is_err(),
            "Designation should be despawned"
        );

        // Pop should have MovementTarget and AtTarget removed
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget should be removed after work completes"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget should be removed after work completes"
        );

        // PopAction should be reset to Idle with zero utility
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle);
        assert!(
            (action.current_utility - 0.0).abs() < f32::EPSILON,
            "Utility should be reset to 0.0"
        );
    }

    #[test]
    fn test_work_execution_cleans_up_stale_target() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());

        // Pop targeting a non-existent designation entity
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(9999),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
                PopAction {
                    current: ActionType::Work,
                    current_utility: 0.8,
                    ticks_committed: 5,
                },
            ))
            .id();

        work_execution_system(&mut world);

        // Pop should have stale references cleaned up
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget should be removed for stale target"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget should be removed for stale target"
        );

        // PopAction should be reset to Idle
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle);
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

        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

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

        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

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
        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

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
        world.run_system_once(process_start_plan_system).unwrap();
        assert!(world.get::<MovementTarget>(pop).is_some());

        // Move toward farm (2 ticks)
        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);

        world.run_system_once(movement_system).unwrap();
        assert!(world.get::<AtTarget>(pop).is_some());

        // Handle arrival
        world.run_system_once(arrival_handler_system).unwrap();
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
            world.run_system_once(movement_system).unwrap();
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
        world.run_system_once(update_action_timer_system).unwrap();
        evaluate_actions_system(&mut world);

        // Should have StartPlan for work
        assert!(
            world.get::<StartPlan>(pop).is_some(),
            "Pop should have StartPlan after first evaluation"
        );

        // Process and start moving
        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();
        world.run_system_once(process_start_plan_system).unwrap();
        world.run_system_once(movement_system).unwrap();
        world.run_system_once(arrival_handler_system).unwrap();
        work_execution_system(&mut world);

        let pos1 = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos1.x, 1, "Pop should have moved to x=1");

        // Tick 2-4: Continue moving toward rock (stop adjacent at x=4)
        // Pop can't stand ON the rock, so they work from adjacent tile
        for tick in 2..=4 {
            world.run_system_once(update_action_timer_system).unwrap();
            evaluate_actions_system(&mut world);
            world
                .run_system_once(cleanup_previous_assignment_system)
                .unwrap();
            world.run_system_once(process_start_plan_system).unwrap();
            world.run_system_once(movement_system).unwrap();
            world.run_system_once(arrival_handler_system).unwrap();
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

    #[test]
    fn test_work_execution_efficiency_low_morale() {
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
            ))
            .id();

        // Spawn a pop with low morale (hunger=0.1, rest=0.1, leisure=0.1 -> morale=0.1)
        // Expected efficiency: 0.5 (penalty)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base work = 10.0
        // Tool efficiency = 1.0 (default resources has 2 tools)
        // Morale efficiency = 0.5 (low morale)
        // Expected = 10.0 * 1.0 * 0.5 = 5.0
        assert!(
            (progress.current - 5.0).abs() < f32::EPSILON,
            "Expected 5.0 progress, got {}",
            progress.current
        );
    }

    #[test]
    fn test_work_execution_efficiency_high_morale() {
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
            ))
            .id();

        // Spawn a pop with high morale (all 1.0 -> morale=1.0)
        // Expected efficiency: 1.2 (bonus)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base work = 10.0
        // Tool efficiency = 1.0 (default resources has 2 tools)
        // Morale efficiency = 1.2 (high morale)
        // Expected = 10.0 * 1.0 * 1.2 = 12.0
        assert!(
            (progress.current - 12.0).abs() < f32::EPSILON,
            "Expected 12.0 progress, got {}",
            progress.current
        );
    }
}
