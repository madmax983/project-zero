#![cfg(test)]

use super::*;
use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
use crate::layer1::items::{Item, ToolType, Equipment, Tool};
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, Speed, Job};
use crate::layer1::resources::{ForestryProgress, MiningProgress};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::utility_types::{PopAction, UtilityWeights, ActionType, StartPlan};
use crate::layer1::map::GridPosition;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::combat::HitStop;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::skills::{Skills, SkillType};
use bevy_ecs::system::RunSystemOnce;
use bevy_ecs::prelude::*;

fn setup_world() -> World {
    crate::setup::init_task_pools();
    let mut world = World::new();
    let tiles = vec![TerrainType::Grass; 100];
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles,
    });
    world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::shared::time::SimulationTime::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(crate::layer1::taboo::TabooState::default());
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
    world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));

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
    world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));

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

    // Pop should have Job
    let job = world.get::<Job>(pop);
    assert!(job.is_some(), "FarmWorker assignment should create Job");
    let job = job.unwrap();
    assert_eq!(job.workplace, farm);
    assert_eq!(job.job_type, AssignmentType::FarmWorker);
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

    // Housing is not a job
    assert!(world.get::<Job>(pop).is_none());
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
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    world
        .entity_mut(designation)
        .insert(crate::layer1::resources::MiningProgress {
            current: 0.0,
            max: 1000000.0,
        });

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
            Needs::default(), // Add standard components
            UtilityWeights::default(),
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
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Chop,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
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
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

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

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
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
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

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

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
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
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

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
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
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
            Equipment {
                tool: Some(tool),
                ..Default::default()
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
    // Ludwig: Organic factor (0.9-1.1) implies range 4.5 - 5.5
    // Note: 5% Critical hit chance multiplies by 5.0 -> ~25.0
    let is_crit_range = progress.current >= 22.5 && progress.current <= 27.5;
    let is_normal_range = progress.current >= 4.5 && progress.current <= 5.5;

    assert!(
        is_normal_range || is_crit_range,
        "Expected ~5.0 (or ~25.0 crit) progress, got {}",
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
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
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
            Equipment {
                tool: Some(tool),
                ..Default::default()
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
    // Ludwig: Organic factor (0.9-1.1) implies range 10.8 - 13.2
    assert!(
        progress.current >= 10.8 && progress.current <= 13.2,
        "Expected ~12.0 progress, got {}",
        progress.current
    );
}

#[test]
fn test_work_execution_skills_mining_efficiency() {
    let mut world = World::new();
    let mut tiles = vec![TerrainType::Grass; 100];
    tiles[55] = TerrainType::Rock;
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles,
    });
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    // Pop with Mining skill level 1 (100 XP) -> Efficiency 1.1
    let mut skills = Skills::default();
    skills.add_xp(SkillType::Mining, 100.0);

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            skills,
            Equipment {
                tool: Some(tool),
                ..Default::default()
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
    // Tool efficiency = 1.0
    // Morale efficiency = 1.0 (0.5 morale is neutral)
    // Skill efficiency = 1.1
    // Expected = 10.0 * 1.0 * 1.0 * 1.1 = 11.0
    // Ludwig: Organic factor (0.9-1.1) implies range 9.9 - 12.1
    // Crit (5%) -> Range 49.5 - 60.5
    let is_normal = progress.current >= 9.9 && progress.current <= 12.1;
    let is_crit = progress.current >= 49.5 && progress.current <= 60.5;
    assert!(
        is_normal || is_crit,
        "Expected ~11.0 (or ~55.0 crit) progress, got {}",
        progress.current
    );
}

#[test]
fn test_work_execution_gains_xp() {
    let mut world = World::new();
    let mut tiles = vec![TerrainType::Grass; 100];
    tiles[55] = TerrainType::Rock;
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles,
    });
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

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
            Skills::default(),
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let skills = world.get::<Skills>(pop).unwrap();
    assert!((skills.get_xp(SkillType::Mining) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_movement_with_speed_penalty() {
    let mut world = setup_world();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(1),
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Work,
            },
            Speed {
                base: 1.0,
                current: 0.5, // Move every 2 ticks
                accumulator: 0.0,
            },
        ))
        .id();

    // Tick 1: Accumulator 0.0 + 0.5 = 0.5 (< 1.0) -> No Move
    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0);

    // Tick 2: Accumulator 0.5 + 0.5 = 1.0 (>= 1.0) -> Move
    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);
}

#[test]
fn test_combat_execution_system_attacks_in_range() {
    use crate::layer1::combat::{AttackProperties, Weapon};
    use crate::layer1::health::Health;

    let mut world = setup_world();

    // Create Enemy
    let enemy = world
        .spawn((
            GridPosition { x: 1, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    // Create Weapon
    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    // Create Pop targeting enemy
    world.spawn((
        Pop,
        GridPosition { x: 0, y: 0 }, // Adjacent (dist 1)
        Equipment {
            weapon: Some(weapon),
            ..Default::default()
        },
        MovementTarget {
            target_entity: enemy,
            target_position: GridPosition { x: 1, y: 0 },
            for_action: ActionType::Fight,
        },
    ));

    combat_execution_system(&mut world);

    // Enemy should take damage
    let health = world.get::<Health>(enemy).unwrap();
    let dmg = 100.0 - health.current;
    assert!(
        (dmg - 10.0).abs() < f32::EPSILON || (dmg - 20.0).abs() < f32::EPSILON,
        "Damage should be 10.0 or 20.0 (crit), got {}",
        dmg
    );
}

#[test]
fn test_combat_execution_system_chases_out_of_range() {
    use crate::layer1::combat::{AttackProperties, Weapon};
    use crate::layer1::health::Health;

    let mut world = setup_world();

    // Create Enemy far away
    let enemy = world
        .spawn((
            GridPosition { x: 5, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0, // Short range
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    // Create Pop targeting enemy
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment {
                weapon: Some(weapon),
                ..Default::default()
            },
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Fight,
            },
            AtTarget, // Simulate arrived at previous target position?
                      // Or simply ensure AtTarget is removed if present
        ))
        .id();

    combat_execution_system(&mut world);

    // Enemy should NOT take damage
    let health = world.get::<Health>(enemy).unwrap();
    assert!((health.current - 100.0).abs() < f32::EPSILON);

    // AtTarget should be removed (to allow movement)
    assert!(world.get::<AtTarget>(pop).is_none());
}

#[test]
fn test_arrival_assigns_to_hospital() {
    let mut world = setup_world();

    // Hospital component might need to be imported or fully qualified
    // It is fully qualified in the test body I prepared.
    let hospital = world
        .spawn((
            Building {
                building_type: BuildingType::Hospital,
            },
            GridPosition { x: 5, y: 5 },
            crate::layer1::medical::Hospital::default(),
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Needs::default(),
            MovementTarget {
                target_entity: hospital,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::SeekMedicalCare,
            },
            AtTarget,
        ))
        .id();

    world.run_system_once(arrival_handler_system).unwrap();

    let assigned = world.get::<AssignedTo>(pop);
    assert!(assigned.is_some(), "Pop should be assigned to hospital");
    assert_eq!(assigned.unwrap().assignment_type, AssignmentType::Patient);

    // Hospital is not a job
    assert!(world.get::<Job>(pop).is_none());

    // MovementTarget should be removed
    assert!(world.get::<MovementTarget>(pop).is_none());
}

#[test]
fn test_arrival_assigns_to_library() {
    let mut world = setup_world();

    let library = world
        .spawn((
            Building {
                building_type: BuildingType::Library,
            },
            GridPosition { x: 5, y: 5 },
            crate::layer1::tech::Library,
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Needs::default(),
            MovementTarget {
                target_entity: library,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Research,
            },
            AtTarget,
        ))
        .id();

    world.run_system_once(arrival_handler_system).unwrap();

    let assigned = world.get::<AssignedTo>(pop);
    assert!(assigned.is_some(), "Pop should be assigned to library");
    assert_eq!(
        assigned.unwrap().assignment_type,
        AssignmentType::LibraryWorker
    );

    // Library is a job
    let job = world.get::<Job>(pop);
    assert!(job.is_some(), "LibraryWorker assignment should create Job");
    let job = job.unwrap();
    assert_eq!(job.workplace, library);
    assert_eq!(job.job_type, AssignmentType::LibraryWorker);

    // MovementTarget should be removed
    assert!(world.get::<MovementTarget>(pop).is_none());
}

#[test]
fn test_movement_system_fast_walker() {
    use crate::layer1::traits::{Trait, Traits};
    use std::collections::HashSet;

    let mut world = setup_world();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(1),
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Work,
            },
            Speed {
                base: 1.0,
                current: 1.0,
                accumulator: 0.0,
            },
            Traits(HashSet::from([Trait::FastWalker])), // +10% speed
        ))
        .id();

    // Tick 1: Acc = 0.0 + (1.0 * 1.1) = 1.1 -> Move -> Acc = 0.1
    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);
    let speed = world.get::<Speed>(pop).unwrap();
    assert!((speed.accumulator - 0.1).abs() < 0.001);
}

#[test]
fn test_work_execution_hard_worker() {
    use crate::layer1::traits::{Trait, Traits};
    use std::collections::HashSet;

    let mut world = World::new();
    let mut tiles = vec![TerrainType::Grass; 100];
    tiles[55] = TerrainType::Rock;
    world.insert_resource(TerrainGrid {
        width: 10,
        height: 10,
        tiles,
    });
    world.insert_resource(crate::layer1::resources::ColonyResources::default());
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
    world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    world.spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        Equipment {
            tool: Some(tool),
            ..Default::default()
        },
        MovementTarget {
            target_entity: designation,
            target_position: GridPosition { x: 5, y: 5 },
            for_action: ActionType::Work,
        },
        AtTarget,
        Traits(HashSet::from([Trait::HardWorker])), // +20% Work Speed
    ));

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation).unwrap();
    // Base 10.0 * 1.2 = 12.0.
    // Organic factor 0.9-1.1 -> Range 10.8 - 13.2
    // Crit (5%) -> Range 54.0 - 66.0
    let is_normal = progress.current >= 10.8 && progress.current <= 13.2;
    let is_crit = progress.current >= 54.0 && progress.current <= 66.0;
    assert!(is_normal || is_crit, "Got {}", progress.current);
}

#[test]
fn test_coyote_speed_movement() {
    let mut world = setup_world();

    // 0.96 accumulator, 1.0 cost.
    // Without coyote: 0.96 < 1.0 -> No move.
    // With coyote (0.05): 0.96 >= 0.95 -> Move.
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(1),
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Work,
            },
            Speed {
                base: 1.0,
                current: 0.0, // Don't add more speed this tick to isolate accumulator check
                accumulator: 0.96,
            },
        ))
        .id();

    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1, "Should move due to Coyote Speed");

    let speed = world.get::<Speed>(pop).unwrap();
    // 0.96 - 1.0 = -0.04
    assert!((speed.accumulator - (-0.04)).abs() < 0.001);
}
#[test]
fn test_movement_system_stuck_in_greedy_corner() {
    let mut world = setup_world();

    // Map setup
    // P # .
    // . . T
    // Pop at (0,0). Target at (2,1). Wall at (1,0).

    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[1] = TerrainType::Rock; // (1,0)
    }

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(1),
                target_position: GridPosition { x: 2, y: 1 },
                for_action: ActionType::Work,
            },
        ))
        .id();

    // Run movement
    world.run_system_once(movement_system).unwrap();

    // Expectation: Pop moves to (0,1) because (1,0) is blocked
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0);
    assert_eq!(pos.y, 1, "Pop should detour to Y if X is blocked");
}

#[test]
fn test_movement_system_blocked_by_hit_stop() {
    let mut world = setup_world();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            MovementTarget {
                target_entity: Entity::from_raw(1),
                target_position: GridPosition { x: 5, y: 0 },
                for_action: ActionType::Work,
            },
            HitStop { ticks_remaining: 1 },
        ))
        .id();

    // Run movement
    world.run_system_once(movement_system).unwrap();

    // Should NOT move
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0, "Pop should be frozen by HitStop");

    // Decrement HitStop (manually or via system)
    world.get_mut::<HitStop>(pop).unwrap().ticks_remaining = 0;

    // Run movement again
    world.run_system_once(movement_system).unwrap();

    // Should move now (if HitStop is 0, we still check ticks_remaining)
    // Wait, if ticks_remaining is 0, we treat it as no hit stop?
    // My implementation: if hit_stop.ticks_remaining > 0 { continue }
    // So 0 is fine.
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1, "Pop should move when HitStop expires");
}

#[test]
fn test_combat_execution_blocked_by_hit_stop() {
    use crate::layer1::combat::{AttackProperties, Weapon};
    use crate::layer1::health::Health;

    let mut world = setup_world();

    let enemy = world
        .spawn((
            GridPosition { x: 1, y: 0 },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let weapon = world
        .spawn(Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 0,
                accuracy: 1.0,
            },
        })
        .id();

    world.spawn((
        Pop,
        GridPosition { x: 0, y: 0 },
        Equipment {
            weapon: Some(weapon),
            ..Default::default()
        },
        MovementTarget {
            target_entity: enemy,
            target_position: GridPosition { x: 1, y: 0 },
            for_action: ActionType::Fight,
            },
            HitStop { ticks_remaining: 1 },
        ));

        combat_execution_system(&mut world);

        // Enemy should NOT take damage
        let health = world.get::<Health>(enemy).unwrap();
        assert!(
            (health.current - 100.0).abs() < f32::EPSILON,
            "HitStop should prevent attack"
        );
    }

    #[test]
    fn test_work_execution_augmentation_bonus() {
        use crate::layer1::cybernetics::{Augmentations, Prosthetic, ProstheticType};

        let mut world = setup_world();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item::default(),
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        // Prosthetic with +50% efficiency
        let prosthetic = world
            .spawn(Prosthetic {
                prosthetic_type: ProstheticType::BionicArm,
                efficiency_bonus: 0.5,
                social_penalty: 0.0,
                power_consumption: 0.0,
            })
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            Augmentations {
                installed: vec![prosthetic],
            },
        ));

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base 10.0. Tool 1.0. Morale 1.0 (neutral 0.5).
        // Augmentation +0.5 -> Multiplier 1.5.
        // Expected: 10.0 * 1.5 = 15.0.
        // Organic: 13.5 - 16.5.
        // Crit (5%): ~75.0.

        let is_normal = progress.current >= 13.5 && progress.current <= 16.5;
        let is_crit = progress.current >= 67.5 && progress.current <= 82.5;

        assert!(
            is_normal || is_crit,
            "Expected ~15.0 (or crit), got {}",
            progress.current
        );
    }

    #[test]
    fn test_calculate_work_amount_cap() {
        use crate::layer1::cybernetics::{Augmentations, Prosthetic, ProstheticType};

        let mut world = setup_world();
        // Insert AdminStats with efficiency 1.0 (default struct is 0.0)
        world.insert_resource(crate::layer1::admin::AdminStats {
            efficiency: 1.0,
            ..Default::default()
        });

        let pop = world.spawn(Pop).id();
        let tool_entity = world
            .spawn((
                Item::default(),
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        // Massive augmentation bonus (10,000x)
        let prosthetic = world
            .spawn(Prosthetic {
                prosthetic_type: ProstheticType::BionicArm,
                efficiency_bonus: 10000.0,
                social_penalty: 0.0,
                power_consumption: 0.0,
            })
            .id();
        world.entity_mut(pop).insert(Augmentations {
            installed: vec![prosthetic],
        });

        // Call logic directly
        let work_amount = calculate_work_amount(
            &world,
            pop,
            DesignationType::Mine,
            Some(tool_entity),
            1.0, // Morale
            1.0, // Work Speed Mod
        );

        // Raw would be ~10 * 10000 = 100,000.
        // Cap is 1000.0.
        assert!(
            (work_amount - 1000.0).abs() < 0.001,
            "Work amount should be capped at 1000.0, got {}",
            work_amount
        );
    }
