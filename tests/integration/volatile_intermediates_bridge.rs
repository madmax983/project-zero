use bevy_ecs::prelude::*;
use scale::layer1::environment::volatile::ExplosionEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::map::GridPosition;
use scale::layer1::core::integration::volatile_explosion_chronicle_bridge;

#[test]
fn test_volatile_explosion_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<ExplosionEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(volatile_explosion_chronicle_bridge);

    // Send an ExplosionEvent
    world.resource_mut::<Events<ExplosionEvent>>().send(ExplosionEvent {
        center: GridPosition { x: 10, y: 15 },
        damage: 50.0,
        radius: 2,
    });

    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let event = reader.read(chronicle_events).next().unwrap();

    assert_eq!(event.importance, EventImportance::Major);
    assert_eq!(
        event.text,
        "A volatile explosion occurred at (10, 15), dealing 50 damage and spreading industrial waste."
    );
}
