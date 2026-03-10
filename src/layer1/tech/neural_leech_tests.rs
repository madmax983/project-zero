use super::neural_leech::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::skills::Skills;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

fn setup_world() -> World {
    let mut world = World::new();
    world.init_resource::<Events<NeuralHubDeathEvent>>();
    // Base setup...
    world
}

#[test]
fn test_neural_hub_buffs_nearby_pops() {
    let mut world = setup_world();

    let _hub = world
        .spawn((Pop, NeuralHub, GridPosition { x: 0, y: 0 }))
        .id();

    let worker = world
        .spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 5, y: 0 }, // Within radius
        ))
        .id();

    let _ = world.run_system_once(apply_neural_link_buffs_system);

    // Worker should now have the Linked buff
    assert!(world.entity(worker).contains::<NeuralLinked>());
}

#[test]
fn test_neural_hub_stress_maxes_out() {
    let mut world = setup_world();

    let hub = world
        .spawn((
            Pop,
            NeuralHub,
            crate::layer1::stress::StressTracker::default(),
        ))
        .id();

    let _ = world.run_system_once(process_neural_hub_decay_system);

    let stress = world
        .get::<crate::layer1::stress::StressTracker>(hub)
        .unwrap();
    assert!(stress.accumulated_stress >= 99.0); // Should be pegged to max
}

#[test]
fn test_hub_death_causes_cascading_breakdown() {
    let mut world = setup_world();

    let worker = world
        .spawn((
            Pop,
            NeuralLinked {
                hub_entity: Entity::PLACEHOLDER,
            }, // Will be updated manually for test
        ))
        .id();

    // Simulate hub death event
    world.send_event(NeuralHubDeathEvent {
        hub_entity: Entity::PLACEHOLDER,
    });

    let _ = world.run_system_once(handle_hub_death_system);

    // Worker should now have a severe breakdown component
    assert!(world.entity(worker).contains::<NeuralShock>());
}
