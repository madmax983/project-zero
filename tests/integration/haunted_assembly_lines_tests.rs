use bevy_app::App;

use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::entities::pop::Pop;
use scale::layer1::haunted_assembly_lines::*;

#[test]
fn test_haunted_assembly_lines_basic_behavior() {
    // Arrange
    let mut app = App::new();
    app.add_event::<PopDiedInAccidentEvent>();
    app.add_systems(
        bevy_app::Update,
        (haunted_building_system, apply_haunted_stress_system),
    );

    // Setup a factory and a worker
    let factory = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            Efficiency(1.0),
        ))
        .id();
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Stress(0.0),
            AssignedTo {
                entity: factory,
                assignment_type: AssignmentType::FarmWorker,
            },
        ))
        .id();

    // Act
    // Simulate industrial accident death
    app.world_mut().send_event(PopDiedInAccidentEvent {
        pop,
        location: factory,
    });
    app.update();

    // Assert
    // Factory should gain Echo of the Fallen and 150% efficiency
    assert!(app.world().entity(factory).contains::<EchoOfTheFallen>());
    assert_eq!(app.world().get::<Efficiency>(factory).unwrap().0, 1.5);

    // New worker assigned should gain massive stress over time
    let new_worker = app
        .world_mut()
        .spawn((
            Pop,
            Stress(0.0),
            AssignedTo {
                entity: factory,
                assignment_type: AssignmentType::FarmWorker,
            },
        ))
        .id();
    app.update(); // Tick time
    assert!(app.world().get::<Stress>(new_worker).unwrap().0 > 0.5);
}

#[test]
fn test_haunted_assembly_lines_edge_cases() {
    // Arrange
    let mut app = App::new();
    app.add_event::<PopDiedInAccidentEvent>();
    app.add_systems(
        bevy_app::Update,
        (
            haunted_building_system,
            apply_haunted_stress_system,
            check_haunted_worker_system,
        ),
    );

    // Setup an already haunted factory
    let factory = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            Efficiency(1.5),
            EchoOfTheFallen,
        ))
        .id();
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Stress(0.0),
            AssignedTo {
                entity: factory,
                assignment_type: AssignmentType::FarmWorker,
            },
        ))
        .id();

    // Act
    // Simulate another industrial accident death in the same haunted factory
    app.world_mut().send_event(PopDiedInAccidentEvent {
        pop,
        location: factory,
    });
    app.update();

    // Assert
    // Efficiency should not stack beyond 150%
    assert_eq!(app.world().get::<Efficiency>(factory).unwrap().0, 1.5);

    // Worker refusing to enter at max stress
    let stressed_worker = app
        .world_mut()
        .spawn((
            Pop,
            Stress(1.0),
            AssignedTo {
                entity: factory,
                assignment_type: AssignmentType::FarmWorker,
            },
        ))
        .id();
    app.update();
    assert!(app.world().get::<AssignedTo>(stressed_worker).is_none());
}
