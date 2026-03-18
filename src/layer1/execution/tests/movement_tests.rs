use super::setup_world;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::combat::HitStop;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::execution::movement::{
    cleanup_previous_assignment_system, movement_system, process_start_plan_system,
};
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::{ActionType, PopAction, StartPlan, UtilityWeights};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

#[test]
fn test_process_start_plan_creates_movement_target() {
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

    let mt = world.get::<MovementTarget>(pop);
    assert!(mt.is_some());
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

    assert!(world.get::<StartPlan>(pop).is_none());
}

#[test]
fn test_process_start_plan_handles_despawned_target() {
    let mut world = setup_world();
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

    assert!(world.get::<MovementTarget>(pop).is_none());
}

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
    assert_eq!(pos.x, 1);
    assert_eq!(pos.y, 0);
}

#[test]
fn test_movement_system_marks_arrival() {
    let mut world = setup_world();

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

    assert!(world.get::<AtTarget>(pop).is_some());
}

#[test]
fn test_movement_system_marks_arrival_on_last_step() {
    let mut world = setup_world();

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
    assert!(world.get::<AtTarget>(pop).is_some());
}

#[test]
fn test_movement_blocked_by_rock() {
    let mut world = World::new();
    let mut tiles = vec![TerrainType::Grass; 100];
    tiles[1] = TerrainType::Rock;
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

#[test]
fn test_movement_blocked_by_water() {
    let mut world = World::new();
    let mut tiles = vec![TerrainType::Grass; 100];
    tiles[1] = TerrainType::Water;
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
                current: 0.5,
                accumulator: 0.0,
            },
        ))
        .id();

    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0);

    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);
}

#[test]
fn test_movement_system_fast_walker() {
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
            Traits(1 << (Trait::FastWalker as u8)),
        ))
        .id();

    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);
    let speed = world.get::<Speed>(pop).unwrap();
    assert!((speed.accumulator - 0.1).abs() < 0.001);
}

#[test]
fn test_coyote_speed_movement() {
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
                current: 0.0,
                accumulator: 0.96,
            },
        ))
        .id();

    world.run_system_once(movement_system).unwrap();
    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);

    let speed = world.get::<Speed>(pop).unwrap();
    assert!((speed.accumulator - (-0.04)).abs() < 0.001);
}

#[test]
fn test_movement_system_stuck_in_greedy_corner() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[1] = TerrainType::Rock;
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

    world.run_system_once(movement_system).unwrap();

    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0);
    assert_eq!(pos.y, 1);
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

    world.run_system_once(movement_system).unwrap();

    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 0);

    world.get_mut::<HitStop>(pop).unwrap().ticks_remaining = 0;

    world.run_system_once(movement_system).unwrap();

    let pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(pos.x, 1);
}

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
                ..Default::default()
            },
        ))
        .id();

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

    let farm_comp = world.get::<Farm>(farm).unwrap();
    assert!(!farm_comp.workers.contains(&pop));

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

    let housing_comp = world.get::<Housing>(housing).unwrap();
    assert!(!housing_comp.residents.contains(&pop));

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
                entity: fake_entity,
                assignment_type: AssignmentType::FarmWorker,
            },
            StartPlan {
                action: ActionType::SatisfyRest,
                target: None,
            },
        ))
        .id();

    world
        .run_system_once(cleanup_previous_assignment_system)
        .unwrap();

    assert!(world.get::<AssignedTo>(pop).is_none());
}
