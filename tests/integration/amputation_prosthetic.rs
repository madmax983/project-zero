#![cfg(test)]

use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::cybernetics::{Augmentations, Prosthetic, ProstheticType, PendingSurgery, surgery_system, get_efficiency_bonus, MissingLimb};
use scale::layer1::hazards::AmputationEvent;
use scale::layer1::memory::{Memories, MemoryType};
use scale::layer1::pop::Pop;
use scale::layer1::actions::{AssignedTo, AssignmentType};
use scale::layer1::integration::amputation_handler_system;
use scale::shared::log::MessageLog;
use scale::shared::time::SimulationTime;

#[test]
fn test_amputation_applies_component_and_memory() {
    let mut world = World::new();
    world.insert_resource(MessageLog::default());
    world.init_resource::<Events<AmputationEvent>>();
    world.init_resource::<SimulationTime>();

    let pop = world.spawn((
        Pop,
        Memories::default(),
        Augmentations::default(),
    )).id();

    // Trigger event
    world.send_event(AmputationEvent { entity: pop });

    // Run handler
    world.run_system_once(amputation_handler_system).unwrap();

    // Assert MissingLimb component
    let missing = world.get::<MissingLimb>(pop);
    assert!(missing.is_some(), "Pop should have MissingLimb component");
    assert_eq!(missing.unwrap().severity, 0.5);

    // Assert Memory
    let memories = world.get::<Memories>(pop).unwrap();
    assert!(memories.items.iter().any(|m| m.memory_type == MemoryType::LostLimb), "Pop should have LostLimb memory");
}

#[test]
fn test_missing_limb_penalty() {
    let mut world = World::new();

    let pop = world.spawn((
        Pop,
        Augmentations::default(),
        MissingLimb { severity: 0.5 },
    )).id();

    let bonus = get_efficiency_bonus(&world, pop);
    assert!((bonus - (-0.5)).abs() < f32::EPSILON, "Efficiency should be -0.5");
}

#[test]
fn test_prosthetic_surgery_removes_missing_limb() {
    let mut world = World::new();

    // Spawn Prosthetic Item
    let bionic_arm = world.spawn(Prosthetic {
        prosthetic_type: ProstheticType::BionicArm,
        efficiency_bonus: 0.2,
        social_penalty: 0.0,
        power_consumption: 0.0,
    }).id();

    let pop = world.spawn((
        Pop,
        Augmentations::default(),
        MissingLimb { severity: 0.5 },
        AssignedTo {
            assignment_type: AssignmentType::Surgery,
            entity: Entity::PLACEHOLDER,
        },
        PendingSurgery {
            item: bionic_arm,
            progress: 9.0,
            duration: 10.0,
        },
    )).id();

    // Run surgery system to complete it
    world.run_system_once(surgery_system).unwrap();

    // Assert MissingLimb removed
    assert!(world.get::<MissingLimb>(pop).is_none(), "MissingLimb should be removed after surgery");

    // Assert Prosthetic installed
    let augs = world.get::<Augmentations>(pop).unwrap();
    assert!(augs.installed.contains(&bionic_arm));

    // Check bonus (should be +0.2 now)
    let bonus = get_efficiency_bonus(&world, pop);
    assert!((bonus - 0.2).abs() < f32::EPSILON);
}
