use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::dead_hand_chronicle_bridge;
use scale::layer1::systems::dead_hand::DoomsdayTriggeredEvent;

#[test]
fn test_dead_hand_chronicle_bridge() {
    let mut world = World::new();

    // 1. Init Resources
    world.init_resource::<Events<DoomsdayTriggeredEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    // 2. Setup Schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(dead_hand_chronicle_bridge);

    // 3. Fire the DoomsdayTriggeredEvent
    let dummy_device = world.spawn_empty().id();
    world.send_event(DoomsdayTriggeredEvent { device: dummy_device });

    // 4. Run the system
    schedule.run(&mut world);

    // 5. Verify the seam is connected!
    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected exactly 1 AddChronicleEvent");
    assert!(
        events[0].text.contains("Dead Hand"),
        "Expected chronicle event text to mention Dead Hand"
    );
}
