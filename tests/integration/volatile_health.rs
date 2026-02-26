use bevy_ecs::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::health::Health;
use scale::layer1::pop::Pop;
use scale::layer1::volatile::{Volatile, volatile_decay_system, handle_explosion_system, ExplosionEvent};
use scale::shared::time::SimulationTime;

#[test]
fn test_explosion_damages_pops() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.init_resource::<Events<ExplosionEvent>>();

    // Spawn Volatile (Explosion Source)
    let center = GridPosition { x: 5, y: 5 };
    world.spawn((
        Volatile {
            stability: 0.1, // Will reach <= 0 immediately
            decay_rate: 1.0,
            explosion_power: 20.0,
            explosion_radius: 2,
            paused: false,
        },
        center,
    ));

    // Spawn Pop (Victim)
    let pop_pos = GridPosition { x: 6, y: 5 }; // Distance 1
    let pop = world.spawn((
        Pop,
        Health { current: 100.0, max: 100.0 },
        pop_pos,
    )).id();

    // Run Systems
    let mut schedule = Schedule::default();
    schedule.add_systems(volatile_decay_system);
    schedule.add_systems(handle_explosion_system.after(volatile_decay_system));

    // We need to update events between systems if we were using multiple schedules,
    // but within one schedule, events persist until cleaned up.
    // However, EventWriter writes to "current frame" events, EventReader reads "current frame".
    // Wait, Bevy events are double-buffered. Events written in a system are available to readers in the SAME frame if they run later.
    // BUT, default Events<T> resource needs maintainence.
    // Actually, `Events::update()` is needed to swap buffers if we want to clear old events.
    // But for a single run, it should be fine.

    schedule.run(&mut world);

    // Verify Damage
    let health = world.get::<Health>(pop).expect("Pop should still exist (health > 0)");
    assert!(health.current < 100.0, "Pop should take damage from explosion. Current: {}", health.current);
    assert_eq!(health.current, 80.0, "Damage should be 20.0");
}
