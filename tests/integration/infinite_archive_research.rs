use bevy_ecs::prelude::*;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::tech::Library;
use scale::layer1::resources::ColonyResources;
use scale::layer1::tech::infinite_archive::Archive;
use scale::layer1::tech::process_research_system;

#[test]
fn test_archive_efficiency_affects_research() {
    let mut world = World::new();

    // 1. Setup Resources
    world.insert_resource(ColonyResources::default());

    // Simulate 50% efficiency from Archive bloat
    world.insert_resource(Archive {
        capacity: 100.0,
        used: 50.0,
        efficiency_multiplier: 0.5,
    });

    // 2. Setup Entities
    let library_entity = world.spawn(Library).id();

    // Spawn a pop assigned to the library
    world.spawn(AssignedTo {
        entity: library_entity,
        assignment_type: AssignmentType::LibraryWorker,
    });

    // 3. Setup Schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(process_research_system);

    // 4. Run System
    schedule.run(&mut world);

    // 5. Assert Output
    let resources = world.resource::<ColonyResources>();

    // Base gain is 0.01 per worker.
    // 0.01 * 0.5 (efficiency) = 0.005
    let expected_gain = 0.01 * 0.5;

    assert!(
        (resources.knowledge - expected_gain).abs() < f32::EPSILON,
        "Knowledge should be scaled by Archive efficiency. Expected {}, got {}",
        expected_gain,
        resources.knowledge
    );
}
