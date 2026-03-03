use super::setup_world;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::execution::arrival::arrival_handler_system;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::farm::Farm;
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Job, Pop};
use crate::layer1::utility_types::ActionType;
use bevy_ecs::system::RunSystemOnce;

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

    let farm_comp = world.get::<Farm>(farm).unwrap();
    assert!(farm_comp.workers.contains(&pop));

    let assigned = world.get::<AssignedTo>(pop);
    assert!(assigned.is_some());
    assert_eq!(
        assigned.unwrap().assignment_type,
        AssignmentType::FarmWorker
    );

    let job = world.get::<Job>(pop);
    assert!(job.is_none(), "Eating should not assign a Job");
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

    let housing_comp = world.get::<Housing>(housing).unwrap();
    assert!(housing_comp.residents.contains(&pop));

    let assigned = world.get::<AssignedTo>(pop);
    assert!(assigned.is_some());
    assert_eq!(
        assigned.unwrap().assignment_type,
        AssignmentType::HousingResident
    );

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
                workers: vec![other_pop],
                ..Default::default()
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

    let farm_comp = world.get::<Farm>(farm).unwrap();
    assert!(!farm_comp.workers.contains(&pop));

    assert!(world.get::<MovementTarget>(pop).is_none());
}

#[test]
fn test_arrival_assigns_to_hospital() {
    let mut world = setup_world();

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
    assert!(assigned.is_some());
    assert_eq!(assigned.unwrap().assignment_type, AssignmentType::Patient);

    assert!(world.get::<Job>(pop).is_none());
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
    assert!(assigned.is_some());
    assert_eq!(
        assigned.unwrap().assignment_type,
        AssignmentType::LibraryWorker
    );

    let job = world.get::<Job>(pop);
    assert!(job.is_some());
    let job = job.unwrap();
    assert_eq!(job.workplace, library);
    assert_eq!(job.job_type, AssignmentType::LibraryWorker);

    assert!(world.get::<MovementTarget>(pop).is_none());
}
