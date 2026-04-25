use bevy::prelude::*;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::architecture::building::Building;
use scale::layer1::environment::hazards::*;
use scale::layer1::haunted_assembly_lines::*;
use scale::layer1::health::Health;
use scale::layer1::pop::Pop;
use scale::layer1::structure::Structure;
use scale::layer1::skills::Skills;

#[test]
fn test_haunted_hazards_bridge_critical_accident_haunts_building() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register events
    app.add_event::<PopDiedInAccidentEvent>();
    app.add_event::<AmputationEvent>();

    // Register systems
    app.add_systems(
        Update,
        (
            haunted_building_system,
            apply_haunted_stress_system,
            check_haunted_worker_system,
        ),
    );

    // Setup factory
    let factory = app
        .world_mut()
        .spawn((
            Building {
                building_type: scale::layer1::architecture::building::BuildingType::LumberMill,
            },
            Efficiency(1.0),
            Structure {
                current_hp: 10.0, // Low HP to increase risk if we were using handle_workplace_hazards randomly, but we will call trigger_accident directly
                max_hp: 100.0,
            }
        ))
        .id();

    // Setup worker with very low health so they die
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 10.0, // Critical severity does 80.0 damage
                max: 100.0,
                has_rust_lung: false,
            },
            Stress(0.0),
            AssignedTo {
                entity: factory,
                assignment_type: AssignmentType::FarmWorker, // Doesn't matter
            },
            Skills::default(),
        ))
        .id();

    // Act
    // Trigger an accident directly simulating handle_workplace_hazards outcome
    trigger_accident(&mut app.world_mut(), pop, Some(factory), AccidentSeverity::Critical);

    // Run systems to process PopDiedInAccidentEvent and apply EchoOfTheFallen
    app.update();

    // Assert
    // Factory should gain Echo of the Fallen and 150% efficiency
    assert!(app.world().entity(factory).contains::<EchoOfTheFallen>());
    assert_eq!(app.world().get::<Efficiency>(factory).unwrap().0, 1.5);

    // Worker is dead
    let health = app.world().get::<Health>(pop).unwrap();
    assert!(health.current <= 0.0);
}
