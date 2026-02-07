use bevy_ecs::prelude::*;
use scale::layer1::health::{Health, death_system, starvation_damage_system};
use scale::layer1::memory::{Memories, MemoryType};
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::shared::log::MessageLog;
use scale::shared::time::SimulationTime;

fn setup() -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(MessageLog::default());
    world.insert_resource(SimulationTime::default());
    world
}

#[test]
fn test_death_causes_witnessed_memory() {
    let mut world = setup();

    // Spawn survivor first
    let survivor = world.spawn((
        Pop,
        Health::default(),
        Memories::default(),
    )).id();

    // Spawn victim (dead)
    let victim = world.spawn((
        Pop,
        Health { current: -1.0, max: 100.0 }, // Already dead
        Memories::default(),
    )).id();

    // Run death system
    death_system(&mut world);

    // Victim should be despawned
    assert!(world.get_entity(victim).is_err(), "Victim should be despawned");

    // Survivor should have WitnessedDeath memory
    let survivor_memories = world.get::<Memories>(survivor).expect("Survivor should have Memories");

    // Assert failure (RED phase)
    if survivor_memories.items.is_empty() {
        panic!("RED PHASE: Survivor has no memories!");
    }

    assert_eq!(survivor_memories.items[0].memory_type, MemoryType::WitnessedDeath, "Memory should be WitnessedDeath");
}

#[test]
fn test_starvation_causes_trauma_memory() {
    let mut world = setup();

    let pop = world.spawn((
        Pop,
        Health::default(),
        Needs { hunger: 0.0, rest: 1.0, leisure: 1.0 }, // Starving
        Memories::default(),
    )).id();

    // Run starvation system
    starvation_damage_system(&mut world);

    // Check health decreased
    let health = world.get::<Health>(pop).unwrap();
    assert!(health.current < 100.0, "Health should decrease");

    // Check memory added
    let memories = world.get::<Memories>(pop).unwrap();

    // Assert failure (RED phase)
    if memories.items.is_empty() {
        panic!("RED PHASE: Pop has no memories!");
    }

    assert_eq!(memories.items[0].memory_type, MemoryType::StarvationTrauma, "Memory should be StarvationTrauma");
}
