use bevy_ecs::prelude::*;
use scale::layer1::psychology::needs::Needs;
use scale::layer1::psychology::teleport_psychosis::{
    handle_teleport_system, process_psychosis_system, Dissociation, TeleportEvent,
    DISSOCIATION_PHANTOM_THRESHOLD, TELEPORT_DISSOCIATION_COST,
};
use scale::layer1::psychology::traits::{Trait, Traits};
use scale::layer1::psychology::needs::decay_needs_system;

#[test]
fn test_teleport_event_increases_dissociation_and_adds_phantom_trait() {
    scale::setup::init_task_pools();
    let mut world = World::new();

    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems((handle_teleport_system, process_psychosis_system).chain());

    world.init_resource::<Events<TeleportEvent>>();

    // Spawn a pop with 0 dissociation
    let pop = world
        .spawn((
            Dissociation { level: DISSOCIATION_PHANTOM_THRESHOLD - TELEPORT_DISSOCIATION_COST },
            Traits::default(),
        ))
        .id();

    // Send a teleport event
    world.resource_mut::<Events<TeleportEvent>>().send(TeleportEvent { entity: pop });

    // Run the schedule

    world.init_resource::<scale::shared::time::SimulationTime>();
    schedule.run(&mut world);

    // Verify Dissociation increased and Phantom trait was added
    let dissoc = world.get::<Dissociation>(pop).unwrap();
    assert_eq!(dissoc.level, DISSOCIATION_PHANTOM_THRESHOLD);

    let traits = world.get::<Traits>(pop).unwrap();
    assert!(traits.has(Trait::Phantom));
}

#[test]
fn test_phantom_trait_stops_hunger_decay() {
    scale::setup::init_task_pools();
    let mut world = World::new();

    // Setup system
    let mut schedule = Schedule::default();
    schedule.add_systems(decay_needs_system);

    // Spawn a normal pop
    let normal_pop = world
        .spawn((
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            },
            Traits::default(),
        ))
        .id();

    // Spawn a phantom pop
    let mut phantom_traits = Traits::default();
    phantom_traits.add(Trait::Phantom);
    let phantom_pop = world
        .spawn((
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 1.0,
            },
            phantom_traits,
        ))
        .id();

    // Run the decay schedule

    world.init_resource::<scale::shared::time::SimulationTime>();
    schedule.run(&mut world);

    // Verify normal pop decayed
    let normal_needs = world.get::<Needs>(normal_pop).unwrap();
    assert!(normal_needs.hunger < 1.0, "Normal pop should have their hunger decayed");

    // Verify phantom pop did not decay
    let phantom_needs = world.get::<Needs>(phantom_pop).unwrap();
    assert_eq!(phantom_needs.hunger, 1.0, "Phantom pop should NOT have their hunger decayed");
}
