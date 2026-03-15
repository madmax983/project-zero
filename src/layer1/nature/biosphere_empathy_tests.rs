use super::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::stress::StressTracker;

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(GlobalFloraHealth { total_health: 100.0 });
    world.init_resource::<Events<FloraDamagedEvent>>();
    world
}

#[test]
fn test_empathic_pops_sync_stress() {
    let mut world = setup_world();

    let pop1 = world.spawn((
        Pop,
        Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
        StressTracker { accumulated_stress: 80.0, ..Default::default() },
    )).id();

    let pop2 = world.spawn((
        Pop,
        Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
        StressTracker { accumulated_stress: 20.0, ..Default::default() },
    )).id();

    // We need to run it multiple times since it only pulls 10% per tick
    for _ in 0..10 {
        let _ = world.run_system_once(sync_empathic_network_system);
    }

    // Stress should equalize towards the average (50)
    let s1 = world.get::<StressTracker>(pop1).unwrap().accumulated_stress;
    let s2 = world.get::<StressTracker>(pop2).unwrap().accumulated_stress;

    assert!((s1 - 50.0).abs() < 15.0);
    assert!((s2 - 50.0).abs() < 15.0);
}

#[test]
fn test_flora_damage_spikes_stress() {
    let mut world = setup_world();

    let pop = world.spawn((
        Pop,
        Traits(std::collections::HashSet::from([Trait::EmpathicLink])),
        StressTracker { accumulated_stress: 10.0, ..Default::default() },
    )).id();

    // Simulate global flora damage event
    world.send_event(FloraDamagedEvent { damage_amount: 100.0 });
    // Run an event buffer update to ensure the event is readable
    world.run_system_once(crate::layer1::systems::update_event_buffer::<FloraDamagedEvent>).unwrap();

    let _ = world.run_system_once(handle_flora_damage_empathy_system);

    let stress = world.get::<StressTracker>(pop).unwrap().accumulated_stress;
    // Stress should spike heavily
    assert!(stress > 50.0);
}