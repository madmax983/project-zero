use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::hologram::HologramFailureEvent;
use scale::layer1::integration::hologram_failure_chronicle_bridge;
use scale::layer1::map::GridPosition;

#[test]
fn test_hologram_failure_adds_chronicle_event() {
    let mut world = World::new();
    world.init_resource::<Events<HologramFailureEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(hologram_failure_chronicle_bridge);

    // Trigger HologramFailureEvent
    world.send_event(HologramFailureEvent {
        position: GridPosition { x: 5, y: 5 },
        radius: 5.0,
    });

    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert!(
        !chronicle_events.is_empty(),
        "HologramFailureEvent should trigger an AddChronicleEvent"
    );

    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("holographic facade failed"));
}
